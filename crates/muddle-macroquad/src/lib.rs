use std::{fs, io, path::PathBuf};

use muddle_amaze_spike::AmazeSilverstreamHost;
use muddle_banish_spike::BanishPilgrimLossHost;
use muddle_cli::{render_transcript, MuddleCliHostInfo};
use muddle_core::{
    MuddleClientControl, MuddleClientHostRegistration, MuddleClientSnapshot, MuddleCommand,
    MuddleHost, MuddleSession, MuddleSessionSave,
};
use muddle_mock_sim::MuddleMockSimHost;

const DEFAULT_HOST: &str = "mock-labyrinth";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadRunOptions {
    pub host_name: Option<String>,
    pub list_hosts: bool,
    pub show_help: bool,
    pub load_path: Option<PathBuf>,
    pub save_path: Option<PathBuf>,
    pub transcript_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuddleMacroquadMode {
    HostChooser,
    Playing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadPlayLayout {
    pub header: Vec<String>,
    pub room: MuddleMacroquadTextRegion,
    pub panels: Vec<MuddleMacroquadTextRegion>,
    pub commands: Vec<MuddleMacroquadCommandControl>,
    pub status: MuddleMacroquadTextRegion,
    pub history: MuddleMacroquadTextRegion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadTextRegion {
    pub id: String,
    pub label: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadCommandControl {
    pub index: usize,
    pub label: String,
    pub command: String,
}

pub struct MuddleMacroquadState {
    registrations: Vec<MuddleClientHostRegistration>,
    registration: MuddleClientHostRegistration,
    host: Box<dyn MuddleHost>,
    session: MuddleSession,
    mode: MuddleMacroquadMode,
    input: String,
    host_filter: String,
    selected_host_index: usize,
    command_history: Vec<String>,
    command_history_cursor: Option<usize>,
    last_status: String,
    load_path: Option<PathBuf>,
    save_path: Option<PathBuf>,
    transcript_path: Option<PathBuf>,
}

impl Default for MuddleMacroquadRunOptions {
    fn default() -> Self {
        Self {
            host_name: None,
            list_hosts: false,
            show_help: false,
            load_path: None,
            save_path: None,
            transcript_path: None,
        }
    }
}

impl MuddleMacroquadState {
    pub fn new() -> Result<Self, String> {
        Self::with_host(default_macroquad_hosts(), DEFAULT_HOST)
    }

    pub fn chooser() -> Result<Self, String> {
        Self::with_chooser(default_macroquad_hosts())
    }

    pub fn with_chooser(registrations: Vec<MuddleClientHostRegistration>) -> Result<Self, String> {
        Self::with_chooser_and_paths(registrations, None, None, None)
    }

    pub fn with_chooser_and_paths(
        registrations: Vec<MuddleClientHostRegistration>,
        load_path: Option<PathBuf>,
        save_path: Option<PathBuf>,
        transcript_path: Option<PathBuf>,
    ) -> Result<Self, String> {
        let mut state = Self::with_host_and_paths(
            registrations,
            DEFAULT_HOST,
            load_path,
            save_path,
            transcript_path,
        )?;
        state.mode = MuddleMacroquadMode::HostChooser;
        state.last_status = "Choose a host with Up/Down and Enter. Type to filter.".to_string();
        Ok(state)
    }

    pub fn with_host(
        registrations: Vec<MuddleClientHostRegistration>,
        host_name: &str,
    ) -> Result<Self, String> {
        Self::with_host_and_paths(registrations, host_name, None, None, None)
    }

    pub fn with_host_and_paths(
        registrations: Vec<MuddleClientHostRegistration>,
        host_name: &str,
        load_path: Option<PathBuf>,
        save_path: Option<PathBuf>,
        transcript_path: Option<PathBuf>,
    ) -> Result<Self, String> {
        if registrations.is_empty() {
            return Err("muddle-macroquad requires at least one host registration".to_string());
        }

        let selected_host_index = registrations
            .iter()
            .position(|registration| registration.name == host_name)
            .ok_or_else(|| format!("Unknown MUDDLE Macroquad host `{host_name}`."))?;
        let registration = registrations[selected_host_index];
        let (host, session, last_status) = start_session(registration, load_path.as_ref())?;

        Ok(Self {
            registrations,
            registration,
            host,
            session,
            mode: MuddleMacroquadMode::Playing,
            input: String::new(),
            host_filter: String::new(),
            selected_host_index,
            command_history: Vec::new(),
            command_history_cursor: None,
            last_status,
            load_path,
            save_path,
            transcript_path,
        })
    }

    pub fn mode(&self) -> MuddleMacroquadMode {
        self.mode
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn host_filter(&self) -> &str {
        &self.host_filter
    }

    pub fn active_host_name(&self) -> &str {
        self.registration.name
    }

    pub fn save_path(&self) -> Option<&PathBuf> {
        self.save_path.as_ref()
    }

    pub fn transcript_path(&self) -> Option<&PathBuf> {
        self.transcript_path.as_ref()
    }

    pub fn turns(&self) -> usize {
        self.session.transcript.len()
    }

    pub fn push_char(&mut self, character: char) {
        if character.is_control() {
            return;
        }

        match self.mode {
            MuddleMacroquadMode::HostChooser => {
                self.host_filter.push(character);
                self.keep_selected_host_visible();
            }
            MuddleMacroquadMode::Playing => {
                self.input.push(character);
                self.command_history_cursor = None;
            }
        }
    }

    pub fn backspace(&mut self) {
        match self.mode {
            MuddleMacroquadMode::HostChooser => {
                self.host_filter.pop();
                self.keep_selected_host_visible();
            }
            MuddleMacroquadMode::Playing => {
                self.input.pop();
                self.command_history_cursor = None;
            }
        }
    }

    pub fn select_next_host(&mut self) {
        self.select_relative_host(1);
    }

    pub fn select_previous_host(&mut self) {
        self.select_relative_host(-1);
    }

    pub fn choose_selected_host(&mut self) -> Result<(), String> {
        let selected = self
            .selected_visible_host_index()
            .ok_or_else(|| "No host matches the current filter.".to_string())?;
        let name = self.registrations[selected].name;
        self.start_host(name)
    }

    pub fn change_host(&mut self) {
        self.mode = MuddleMacroquadMode::HostChooser;
        self.input.clear();
        self.command_history_cursor = None;
        self.last_status = "Choose a host with Up/Down and Enter. Type to filter.".to_string();
    }

    pub fn restart_host(&mut self) -> Result<(), String> {
        let name = self.registration.name;
        self.start_host(name)
    }

    pub fn save_now(&mut self) -> io::Result<()> {
        if self.save_path.is_none() && self.transcript_path.is_none() {
            self.last_status =
                "Start muddle-macroquad with --save or --transcript before saving.".to_string();
            return Ok(());
        }

        if let Some(path) = &self.save_path {
            fs::write(
                path,
                self.session.save_for_host(self.host.as_ref()).encode(),
            )?;
        }
        if let Some(path) = &self.transcript_path {
            fs::write(
                path,
                render_transcript(
                    MuddleCliHostInfo {
                        name: self.registration.name,
                        description: self.registration.description,
                        suggested_commands: self.registration.suggested_commands,
                    },
                    &self.session,
                ),
            )?;
        }

        self.last_status = match (&self.save_path, &self.transcript_path) {
            (Some(save_path), Some(transcript_path)) => format!(
                "Saved session to {} and transcript to {}.",
                save_path.display(),
                transcript_path.display()
            ),
            (Some(save_path), None) => format!("Saved session to {}.", save_path.display()),
            (None, Some(transcript_path)) => {
                format!("Saved transcript to {}.", transcript_path.display())
            }
            (None, None) => unreachable!("empty persistence targets are handled before saving"),
        };
        Ok(())
    }

    pub fn reload_save(&mut self) -> io::Result<()> {
        let Some(path) = &self.save_path else {
            self.last_status = "Start muddle-macroquad with --save before reloading.".to_string();
            return Ok(());
        };
        let encoded = fs::read_to_string(path)?;
        let save = MuddleSessionSave::decode(&encoded)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
        let mut host = (self.registration.create)();
        let session = MuddleSession::resume_for_host(host.as_mut(), &save)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;

        self.host = host;
        self.session = session;
        self.input.clear();
        self.command_history_cursor = None;
        self.last_status = format!(
            "Loaded MUDDLE session from {} with {} transcript turns.",
            path.display(),
            self.session.transcript.len()
        );
        Ok(())
    }

    pub fn recall_previous_command(&mut self) {
        if self.command_history.is_empty() || self.mode != MuddleMacroquadMode::Playing {
            return;
        }

        let index = self
            .command_history_cursor
            .map(|index| index.saturating_sub(1))
            .unwrap_or_else(|| self.command_history.len() - 1);
        self.command_history_cursor = Some(index);
        self.input = self.command_history[index].clone();
    }

    pub fn recall_next_command(&mut self) {
        if self.command_history.is_empty() || self.mode != MuddleMacroquadMode::Playing {
            return;
        }

        match self.command_history_cursor {
            Some(index) if index + 1 < self.command_history.len() => {
                let next = index + 1;
                self.command_history_cursor = Some(next);
                self.input = self.command_history[next].clone();
            }
            _ => {
                self.command_history_cursor = None;
                self.input.clear();
            }
        }
    }

    pub fn submit_input(&mut self) {
        if self.mode != MuddleMacroquadMode::Playing {
            return;
        }

        let command = self.input.trim().to_string();
        self.input.clear();
        self.command_history_cursor = None;
        self.submit_command(command);
    }

    pub fn submit_command_hint(&mut self, index: usize) {
        if self.mode != MuddleMacroquadMode::Playing {
            return;
        }

        if let Some(hint) = self.snapshot().commands.get(index) {
            self.submit_command(hint.command.clone());
        }
    }

    pub fn display_lines(&self) -> Vec<String> {
        match self.mode {
            MuddleMacroquadMode::HostChooser => self.host_chooser_lines(),
            MuddleMacroquadMode::Playing => snapshot_display_lines(&self.snapshot(), &self.input),
        }
    }

    pub fn play_layout(&self) -> Option<MuddleMacroquadPlayLayout> {
        (self.mode == MuddleMacroquadMode::Playing)
            .then(|| snapshot_play_layout(&self.snapshot(), &self.input))
    }

    pub fn snapshot(&self) -> MuddleClientSnapshot {
        self.session.client_snapshot(
            self.host.as_ref(),
            self.registration.client_info(),
            self.last_status.clone(),
        )
    }

    fn start_host(&mut self, host_name: &str) -> Result<(), String> {
        let selected_host_index = self
            .registrations
            .iter()
            .position(|registration| registration.name == host_name)
            .ok_or_else(|| format!("Unknown MUDDLE Macroquad host `{host_name}`."))?;
        let registration = self.registrations[selected_host_index];
        let (host, session, last_status) = start_session(registration, self.load_path.as_ref())?;

        self.registration = registration;
        self.host = host;
        self.session = session;
        self.mode = MuddleMacroquadMode::Playing;
        self.input.clear();
        self.selected_host_index = selected_host_index;
        self.command_history.clear();
        self.command_history_cursor = None;
        self.last_status = last_status;
        Ok(())
    }

    fn submit_command(&mut self, command: String) {
        let command = command.trim().to_string();
        if command.is_empty() {
            return;
        }

        self.command_history.push(command.clone());
        match self
            .session
            .play_turn(self.host.as_mut(), MuddleCommand::parse(&command))
        {
            Ok(turn) => self.last_status = turn.response.clone(),
            Err(error) => self.last_status = format!("Command failed: {error:?}"),
        }
    }

    fn host_chooser_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "MUDDLE Macroquad Host Chooser".to_string(),
            "Type to filter. Up/Down selects. Enter starts. Escape quits.".to_string(),
            format!("Filter: {}", self.host_filter),
            String::new(),
        ];

        let visible = self.filtered_host_indices();
        if visible.is_empty() {
            lines.push("No hosts match the current filter.".to_string());
            return lines;
        }

        let selected = self.selected_visible_host_index();
        let mut last_category = "";
        for index in visible {
            let registration = self.registrations[index];
            if registration.category != last_category {
                if !last_category.is_empty() {
                    lines.push(String::new());
                }
                lines.push(format!("[{}]", registration.category));
                last_category = registration.category;
            }
            let marker = if Some(index) == selected { ">" } else { " " };
            lines.push(format!(
                "{marker} {} - {}",
                registration.name, registration.description
            ));
        }

        lines
    }

    fn select_relative_host(&mut self, delta: isize) {
        let visible = self.filtered_host_indices();
        if visible.is_empty() {
            return;
        }

        let current = self
            .selected_visible_host_index()
            .and_then(|selected| visible.iter().position(|index| *index == selected))
            .unwrap_or(0);
        let next = (current as isize + delta).rem_euclid(visible.len() as isize) as usize;
        self.selected_host_index = visible[next];
    }

    fn keep_selected_host_visible(&mut self) {
        if self.selected_visible_host_index().is_none() {
            if let Some(first) = self.filtered_host_indices().first() {
                self.selected_host_index = *first;
            }
        }
    }

    fn selected_visible_host_index(&self) -> Option<usize> {
        let visible = self.filtered_host_indices();
        if visible.contains(&self.selected_host_index) {
            Some(self.selected_host_index)
        } else {
            visible.first().copied()
        }
    }

    fn filtered_host_indices(&self) -> Vec<usize> {
        let filter = self.host_filter.trim().to_ascii_lowercase();
        self.registrations
            .iter()
            .enumerate()
            .filter_map(|(index, registration)| {
                let matches = filter.is_empty()
                    || registration.name.to_ascii_lowercase().contains(&filter)
                    || registration.category.to_ascii_lowercase().contains(&filter)
                    || registration
                        .description
                        .to_ascii_lowercase()
                        .contains(&filter)
                    || registration
                        .suggested_commands
                        .to_ascii_lowercase()
                        .contains(&filter);
                matches.then_some(index)
            })
            .collect()
    }
}

pub fn default_macroquad_hosts() -> Vec<MuddleClientHostRegistration> {
    vec![
        MuddleClientHostRegistration {
            name: DEFAULT_HOST,
            category: "Fixtures",
            description: "Labyrinth mock sim: BANISH-like resource state plus AMAZE-like locks.",
            suggested_commands:
                "`look`, `gather ember`, `go antechamber`, `inspect glyphs`, `use ember`, `go vault`.",
            create: || Box::new(MuddleMockSimHost::new()),
        },
        MuddleClientHostRegistration {
            name: "banish-pilgrim-loss",
            category: "Games",
            description: "BANISH Pilgrim Loss adapter spike: launcher, campaign brief, and migration trail.",
            suggested_commands:
                "`look`, `choose resume`, `inspect plan`, `inspect manifest`, `go trail`, `resolve loss`.",
            create: || Box::new(BanishPilgrimLossHost::new()),
        },
        MuddleClientHostRegistration {
            name: "amaze-silverstream",
            category: "Games",
            description: "AMAZE Silverstream adapter spike: clue, signal, hatch, and escape path.",
            suggested_commands:
                "`look`, `go receiver`, `inspect clue`, `tune signal`, `unlock hatch`, `go hatch`.",
            create: || Box::new(AmazeSilverstreamHost::new()),
        },
    ]
}

fn start_session(
    registration: MuddleClientHostRegistration,
    load_path: Option<&PathBuf>,
) -> Result<(Box<dyn MuddleHost>, MuddleSession, String), String> {
    let mut host = (registration.create)();
    if let Some(path) = load_path {
        let encoded = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let save = MuddleSessionSave::decode(&encoded).map_err(|error| format!("{error:?}"))?;
        let session = MuddleSession::resume_for_host(host.as_mut(), &save)
            .map_err(|error| format!("{error:?}"))?;
        let last_status = format!(
            "Loaded MUDDLE session from {} with {} transcript turns.",
            path.display(),
            session.transcript.len()
        );
        Ok((host, session, last_status))
    } else {
        let session =
            MuddleSession::for_host(host.as_ref()).map_err(|error| format!("{error:?}"))?;
        Ok((
            host,
            session,
            format!(
                "Host mounted: {}. Type a command or press F2 to change host.",
                registration.name
            ),
        ))
    }
}

pub fn parse_macroquad_run_options(
    args: impl IntoIterator<Item = String>,
) -> Result<MuddleMacroquadRunOptions, String> {
    let mut options = MuddleMacroquadRunOptions::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => options.show_help = true,
            "--list-hosts" => options.list_hosts = true,
            "--host" => {
                options.host_name = Some(
                    args.next()
                        .ok_or_else(|| "`--host` requires a host name.".to_string())?,
                );
            }
            "--load" => {
                options.load_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "`--load` requires a path.".to_string())?,
                ));
            }
            "--save" => {
                options.save_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "`--save` requires a path.".to_string())?,
                ));
            }
            "--transcript" => {
                options.transcript_path =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        "`--transcript` requires a path.".to_string()
                    })?));
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--host=") {
                    if value.is_empty() {
                        return Err("`--host` requires a host name.".to_string());
                    }
                    options.host_name = Some(value.to_string());
                } else if let Some(value) = arg.strip_prefix("--load=") {
                    if value.is_empty() {
                        return Err("`--load` requires a path.".to_string());
                    }
                    options.load_path = Some(PathBuf::from(value));
                } else if let Some(value) = arg.strip_prefix("--save=") {
                    if value.is_empty() {
                        return Err("`--save` requires a path.".to_string());
                    }
                    options.save_path = Some(PathBuf::from(value));
                } else if let Some(value) = arg.strip_prefix("--transcript=") {
                    if value.is_empty() {
                        return Err("`--transcript` requires a path.".to_string());
                    }
                    options.transcript_path = Some(PathBuf::from(value));
                } else {
                    return Err(format!("Unknown argument `{arg}`."));
                }
            }
        }
    }

    Ok(options)
}

pub fn macroquad_usage() -> &'static str {
    "Usage: muddle-macroquad [--host <name>] [--load <path>] [--save <path>] [--transcript <path>] [--list-hosts] [--help]"
}

pub fn macroquad_host_list(registrations: &[MuddleClientHostRegistration]) -> String {
    let mut lines = vec!["Available MUDDLE Macroquad hosts:".to_string()];
    for registration in registrations {
        lines.push(format!(
            "  {} [{}] - {}",
            registration.name, registration.category, registration.description
        ));
    }
    lines.join("\n")
}

pub fn snapshot_display_lines(snapshot: &MuddleClientSnapshot, input: &str) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("MUDDLE Macroquad Runner".to_string());
    lines.push(
        "Esc quits. F2 changes host. F5 restarts. F6 saves. F7 reloads. Up/Down recalls commands. Enter submits.".to_string(),
    );
    lines.push(format!(
        "Host: {} - {}",
        snapshot.host, snapshot.description
    ));
    lines.push(format!("Input: {input}"));
    lines.push(String::new());

    lines.push(snapshot.room_card.clone());

    lines.push(String::new());
    lines.push("Panels".to_string());
    lines.extend(
        snapshot
            .panels
            .resources
            .iter()
            .map(|resource| format!("{}: {}", resource.label, resource.value)),
    );
    lines.extend(
        snapshot
            .panels
            .inventory
            .iter()
            .map(|item| format!("{}: {}", item.label, item.detail)),
    );
    for objective in &snapshot.panels.objectives {
        lines.push(format!("Objective: {objective}"));
    }
    if let Some(map) = &snapshot.panels.map {
        lines.push(map.clone());
    }
    if !snapshot.panels.recent_log.is_empty() {
        lines.push(format!(
            "Recent log: {}",
            snapshot.panels.recent_log.join(" | ")
        ));
    }

    let commands = snapshot
        .commands
        .iter()
        .map(|hint| format!("{} ({})", hint.command, hint.description))
        .collect::<Vec<_>>()
        .join(" | ");
    if !commands.is_empty() {
        lines.push(format!("Commands: {commands}"));
    } else if !snapshot.suggested_commands.is_empty() {
        lines.push(format!("Try: {}", snapshot.suggested_commands));
    }

    lines.push(String::new());
    lines.push(format!("Status: {}", snapshot.last_response));
    lines.push(format!("Turns: {}", snapshot.turns));
    lines.push("Recent history".to_string());
    for turn in snapshot.history.iter().rev().take(8) {
        lines.push(format!(
            "{}. {} @ {} -> {}",
            turn.turn,
            turn.command,
            turn.room,
            turn.response.lines().next().unwrap_or_default()
        ));
    }
    lines
}

pub fn snapshot_play_layout(
    snapshot: &MuddleClientSnapshot,
    input: &str,
) -> MuddleMacroquadPlayLayout {
    let controls = &snapshot.controls;
    let panels = find_control(controls, "panels")
        .map(|panel_group| {
            panel_group
                .children
                .iter()
                .map(control_text_region)
                .collect::<Vec<_>>()
        })
        .filter(|panels| !panels.is_empty())
        .unwrap_or_else(|| fallback_panel_regions(snapshot));

    MuddleMacroquadPlayLayout {
        header: control_header_lines(controls, snapshot, input),
        room: find_control(controls, "room-card")
            .map(control_text_region)
            .unwrap_or_else(|| MuddleMacroquadTextRegion {
                id: "room".to_string(),
                label: "Room".to_string(),
                lines: snapshot
                    .room_card
                    .lines()
                    .map(ToString::to_string)
                    .collect(),
            }),
        panels,
        commands: control_command_buttons(controls, snapshot),
        status: find_control(controls, "status")
            .map(|control| {
                let mut region = control_text_region(control);
                region.lines.push(format!("Turns: {}", snapshot.turns));
                region
            })
            .unwrap_or_else(|| MuddleMacroquadTextRegion {
                id: "status".to_string(),
                label: "Status".to_string(),
                lines: vec![
                    snapshot.last_response.clone(),
                    format!("Turns: {}", snapshot.turns),
                ],
            }),
        history: find_control(controls, "history")
            .map(control_text_region)
            .unwrap_or_else(|| fallback_history_region(snapshot)),
    }
}

fn find_control<'a>(
    controls: &'a [MuddleClientControl],
    id: &str,
) -> Option<&'a MuddleClientControl> {
    controls.iter().find_map(|control| {
        if control.id == id {
            Some(control)
        } else {
            find_control(&control.children, id)
        }
    })
}

fn control_header_lines(
    controls: &[MuddleClientControl],
    snapshot: &MuddleClientSnapshot,
    input: &str,
) -> Vec<String> {
    let mut lines = find_control(controls, "header")
        .map(|control| {
            control
                .children
                .iter()
                .filter_map(|child| {
                    child
                        .text
                        .as_ref()
                        .map(|text| format!("{}: {}", child.label, text))
                })
                .collect::<Vec<_>>()
        })
        .filter(|lines| !lines.is_empty())
        .unwrap_or_else(|| vec![format!("{} - {}", snapshot.host, snapshot.description)]);
    lines.push("F2 host | F5 restart | F6 save | F7 reload | Esc quit".to_string());
    lines.push(format!("Input: {input}"));
    lines
}

fn control_text_region(control: &MuddleClientControl) -> MuddleMacroquadTextRegion {
    let lines = if !control.children.is_empty() {
        control
            .children
            .iter()
            .flat_map(|child| {
                if let Some(text) = &child.text {
                    vec![format!("{}: {}", child.label, text)]
                } else if let Some(command) = &child.command {
                    vec![format!("{} -> {}", child.label, command)]
                } else {
                    vec![child.label.clone()]
                }
            })
            .collect()
    } else if let Some(text) = &control.text {
        text.lines().map(ToString::to_string).collect()
    } else if let Some(command) = &control.command {
        vec![command.clone()]
    } else {
        vec![control.label.clone()]
    };

    MuddleMacroquadTextRegion {
        id: control.id.clone(),
        label: control.label.clone(),
        lines,
    }
}

fn control_command_buttons(
    controls: &[MuddleClientControl],
    snapshot: &MuddleClientSnapshot,
) -> Vec<MuddleMacroquadCommandControl> {
    find_control(controls, "commands")
        .map(|control| {
            control
                .children
                .iter()
                .filter_map(|child| {
                    child
                        .command
                        .as_ref()
                        .map(|command| (child.label.clone(), command.clone()))
                })
                .enumerate()
                .map(|(index, (label, command))| MuddleMacroquadCommandControl {
                    index,
                    label,
                    command,
                })
                .collect::<Vec<_>>()
        })
        .filter(|commands| !commands.is_empty())
        .unwrap_or_else(|| {
            snapshot
                .commands
                .iter()
                .enumerate()
                .map(|(index, hint)| MuddleMacroquadCommandControl {
                    index,
                    label: format!("{} - {}", hint.command, hint.description),
                    command: hint.command.clone(),
                })
                .collect()
        })
}

fn fallback_panel_regions(snapshot: &MuddleClientSnapshot) -> Vec<MuddleMacroquadTextRegion> {
    let mut panels = Vec::new();
    if !snapshot.panels.resources.is_empty() {
        panels.push(MuddleMacroquadTextRegion {
            id: "resources".to_string(),
            label: "Resources".to_string(),
            lines: snapshot
                .panels
                .resources
                .iter()
                .map(|resource| format!("{}: {}", resource.label, resource.value))
                .collect(),
        });
    }
    if !snapshot.panels.inventory.is_empty() {
        panels.push(MuddleMacroquadTextRegion {
            id: "inventory".to_string(),
            label: "Inventory".to_string(),
            lines: snapshot
                .panels
                .inventory
                .iter()
                .map(|item| format!("{}: {}", item.label, item.detail))
                .collect(),
        });
    }
    if !snapshot.panels.objectives.is_empty() {
        panels.push(MuddleMacroquadTextRegion {
            id: "objectives".to_string(),
            label: "Objectives".to_string(),
            lines: snapshot.panels.objectives.clone(),
        });
    }
    if let Some(map) = &snapshot.panels.map {
        panels.push(MuddleMacroquadTextRegion {
            id: "map".to_string(),
            label: "Map".to_string(),
            lines: map.lines().map(ToString::to_string).collect(),
        });
    }
    if !snapshot.panels.recent_log.is_empty() {
        panels.push(MuddleMacroquadTextRegion {
            id: "recent-log".to_string(),
            label: "Recent log".to_string(),
            lines: snapshot.panels.recent_log.clone(),
        });
    }
    panels
}

fn fallback_history_region(snapshot: &MuddleClientSnapshot) -> MuddleMacroquadTextRegion {
    MuddleMacroquadTextRegion {
        id: "history".to_string(),
        label: "History".to_string(),
        lines: snapshot
            .history
            .iter()
            .rev()
            .take(8)
            .map(|turn| {
                format!(
                    "{}. {} @ {} -> {}",
                    turn.turn,
                    turn.command,
                    turn.room,
                    turn.response.lines().next().unwrap_or_default()
                )
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macroquad_state_starts_with_mock_room() {
        let state = MuddleMacroquadState::new().expect("state starts");
        let lines = state.display_lines();
        assert_eq!(state.mode(), MuddleMacroquadMode::Playing);
        assert!(lines.iter().any(|line| line.contains("Labyrinth Camp")));
        assert!(lines.iter().any(|line| line.contains("Commands:")));
    }

    #[test]
    fn macroquad_state_submits_commands() {
        let mut state = MuddleMacroquadState::new().expect("state starts");
        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        assert!(state.input().is_empty());
        assert_eq!(state.turns(), 1);
        assert!(state
            .display_lines()
            .iter()
            .any(|line| line.contains("Recent history")));
    }

    #[test]
    fn macroquad_play_layout_exposes_regions_and_commands() {
        let state = MuddleMacroquadState::new().expect("state starts");
        let layout = state.play_layout().expect("playing state has layout");
        assert_eq!(layout.room.id, "room-card");
        assert!(layout
            .panels
            .iter()
            .any(|panel| panel.id == "resources" || panel.id == "map"));
        assert!(layout
            .commands
            .iter()
            .any(|command| command.command == "look"));
    }

    #[test]
    fn macroquad_chooser_filters_and_starts_hosts() {
        let mut state = MuddleMacroquadState::chooser().expect("state starts");
        assert_eq!(state.mode(), MuddleMacroquadMode::HostChooser);
        for character in "silverstream".chars() {
            state.push_char(character);
        }
        assert_eq!(state.host_filter(), "silverstream");
        state
            .choose_selected_host()
            .expect("filtered host can be chosen");
        assert_eq!(state.mode(), MuddleMacroquadMode::Playing);
        assert_eq!(state.active_host_name(), "amaze-silverstream");
    }

    #[test]
    fn macroquad_state_recalls_commands() {
        let mut state = MuddleMacroquadState::new().expect("state starts");
        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        state.recall_previous_command();
        assert_eq!(state.input(), "look");
        state.recall_next_command();
        assert_eq!(state.input(), "");
    }

    #[test]
    fn macroquad_state_restarts_current_host() {
        let mut state = MuddleMacroquadState::new().expect("state starts");
        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        assert_eq!(state.turns(), 1);
        state.restart_host().expect("host restarts");
        assert_eq!(state.turns(), 0);
        assert_eq!(state.active_host_name(), DEFAULT_HOST);
    }

    #[test]
    fn macroquad_args_parse_host_and_list() {
        let options = parse_macroquad_run_options([
            "--host".to_string(),
            "banish-pilgrim-loss".to_string(),
            "--save".to_string(),
            "play.muddle".to_string(),
            "--list-hosts".to_string(),
        ])
        .expect("args parse");
        assert_eq!(options.host_name.as_deref(), Some("banish-pilgrim-loss"));
        assert_eq!(options.save_path, Some(PathBuf::from("play.muddle")));
        assert!(options.list_hosts);
    }

    #[test]
    fn macroquad_state_saves_transcript_and_reloads() {
        let unique = format!(
            "muddle-macroquad-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time is after epoch")
                .as_nanos()
        );
        let save_path = std::env::temp_dir().join(format!("{unique}.muddle"));
        let transcript_path = std::env::temp_dir().join(format!("{unique}.txt"));

        let mut state = MuddleMacroquadState::with_host_and_paths(
            default_macroquad_hosts(),
            DEFAULT_HOST,
            None,
            Some(save_path.clone()),
            Some(transcript_path.clone()),
        )
        .expect("state starts");
        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        state.save_now().expect("state saves");
        assert!(save_path.exists());
        assert!(transcript_path.exists());

        state.restart_host().expect("host restarts");
        assert_eq!(state.turns(), 0);
        state.reload_save().expect("state reloads");
        assert_eq!(state.turns(), 1);

        let _ = fs::remove_file(save_path);
        let _ = fs::remove_file(transcript_path);
    }
}
