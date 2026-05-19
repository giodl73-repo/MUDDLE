use std::{
    fs, io,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use macroquad::prelude::*;
use muddle_amaze_spike::AmazeSilverstreamHost;
use muddle_banish_spike::BanishPilgrimLossHost;
use muddle_cli::{render_transcript, MuddleCliHostInfo};
use muddle_core::{
    MuddleClientControl, MuddleClientHostRegistration, MuddleClientSnapshot, MuddleCommand,
    MuddleHost, MuddleSession, MuddleSessionSave, MuddleVisualNode, MuddleVisualNodeKind,
};
use muddle_mock_sim::MuddleMockSimHost;

const DEFAULT_HOST: &str = "mock-labyrinth";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadRunOptions {
    pub host_name: Option<String>,
    pub list_hosts: bool,
    pub visual_smoke: bool,
    pub require_visuals: bool,
    pub require_visual_frames: bool,
    pub visual_smoke_room: Option<String>,
    pub visual_smoke_commands: Vec<String>,
    pub show_help: bool,
    pub load_path: Option<PathBuf>,
    pub save_path: Option<PathBuf>,
    pub transcript_path: Option<PathBuf>,
    pub import_path: Option<PathBuf>,
    pub export_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuddleMacroquadMode {
    HostChooser,
    Playing,
    SaveSlots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuddleMacroquadSaveSlotSort {
    Name,
    Newest,
    Oldest,
    Largest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadPlayLayout {
    pub header: Vec<String>,
    pub room: MuddleMacroquadTextRegion,
    pub visual_nodes: Vec<MuddleMacroquadVisualNode>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadVisualNode {
    pub id: String,
    pub kind: MuddleVisualNodeKind,
    pub label: String,
    pub text: Option<String>,
    pub sprite_source: Option<String>,
    pub sprite_frame: Option<String>,
    pub sprite_animation: Option<String>,
    pub layer: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadSaveSlotDetail {
    pub name: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub modified_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMacroquadRunConfig {
    pub screen_title: String,
}

pub struct MuddleMacroquadState {
    registrations: Vec<MuddleClientHostRegistration>,
    registration: MuddleClientHostRegistration,
    host: Box<dyn MuddleHost>,
    session: MuddleSession,
    mode: MuddleMacroquadMode,
    input: String,
    host_filter: String,
    slot_filter: String,
    slot_sort: MuddleMacroquadSaveSlotSort,
    selected_host_index: usize,
    selected_slot_index: usize,
    command_history: Vec<String>,
    command_history_cursor: Option<usize>,
    last_status: String,
    load_path: Option<PathBuf>,
    save_path: Option<PathBuf>,
    transcript_path: Option<PathBuf>,
    import_path: Option<PathBuf>,
    export_path: Option<PathBuf>,
}

impl Default for MuddleMacroquadRunOptions {
    fn default() -> Self {
        Self {
            host_name: None,
            list_hosts: false,
            visual_smoke: false,
            require_visuals: false,
            require_visual_frames: false,
            visual_smoke_room: None,
            visual_smoke_commands: Vec::new(),
            show_help: false,
            load_path: None,
            save_path: None,
            transcript_path: None,
            import_path: None,
            export_path: None,
        }
    }
}

impl Default for MuddleMacroquadRunConfig {
    fn default() -> Self {
        Self {
            screen_title: "MUDDLE Macroquad".to_string(),
        }
    }
}

impl MuddleMacroquadSaveSlotSort {
    fn label(self) -> &'static str {
        match self {
            MuddleMacroquadSaveSlotSort::Name => "name",
            MuddleMacroquadSaveSlotSort::Newest => "newest",
            MuddleMacroquadSaveSlotSort::Oldest => "oldest",
            MuddleMacroquadSaveSlotSort::Largest => "largest",
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
        Self::with_chooser_and_paths(registrations, None, None, None, None, None)
    }

    pub fn with_chooser_and_paths(
        registrations: Vec<MuddleClientHostRegistration>,
        load_path: Option<PathBuf>,
        save_path: Option<PathBuf>,
        transcript_path: Option<PathBuf>,
        import_path: Option<PathBuf>,
        export_path: Option<PathBuf>,
    ) -> Result<Self, String> {
        let mut state = Self::with_host_and_paths(
            registrations,
            DEFAULT_HOST,
            load_path,
            save_path,
            transcript_path,
            import_path,
            export_path,
        )?;
        state.mode = MuddleMacroquadMode::HostChooser;
        state.last_status = "Choose a host with Up/Down and Enter. Type to filter.".to_string();
        Ok(state)
    }

    pub fn with_host(
        registrations: Vec<MuddleClientHostRegistration>,
        host_name: &str,
    ) -> Result<Self, String> {
        Self::with_host_and_paths(registrations, host_name, None, None, None, None, None)
    }

    pub fn with_host_and_paths(
        registrations: Vec<MuddleClientHostRegistration>,
        host_name: &str,
        load_path: Option<PathBuf>,
        save_path: Option<PathBuf>,
        transcript_path: Option<PathBuf>,
        import_path: Option<PathBuf>,
        export_path: Option<PathBuf>,
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
            slot_filter: String::new(),
            slot_sort: MuddleMacroquadSaveSlotSort::Name,
            selected_host_index,
            selected_slot_index: 0,
            command_history: Vec::new(),
            command_history_cursor: None,
            last_status,
            load_path,
            save_path,
            transcript_path,
            import_path,
            export_path,
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

    pub fn slot_filter(&self) -> &str {
        &self.slot_filter
    }

    pub fn slot_sort(&self) -> MuddleMacroquadSaveSlotSort {
        self.slot_sort
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

    pub fn import_path(&self) -> Option<&PathBuf> {
        self.import_path.as_ref()
    }

    pub fn export_path(&self) -> Option<&PathBuf> {
        self.export_path.as_ref()
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
            MuddleMacroquadMode::SaveSlots => {
                self.slot_filter.push(character);
                self.keep_selected_slot_visible();
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
            MuddleMacroquadMode::SaveSlots => {
                self.slot_filter.pop();
                self.keep_selected_slot_visible();
            }
        }
    }

    pub fn select_next_host(&mut self) {
        self.select_relative_host(1);
    }

    pub fn select_previous_host(&mut self) {
        self.select_relative_host(-1);
    }

    pub fn select_next_slot(&mut self) {
        self.select_relative_slot(1);
    }

    pub fn select_previous_slot(&mut self) {
        self.select_relative_slot(-1);
    }

    pub fn cycle_slot_sort(&mut self) {
        self.slot_sort = match self.slot_sort {
            MuddleMacroquadSaveSlotSort::Name => MuddleMacroquadSaveSlotSort::Newest,
            MuddleMacroquadSaveSlotSort::Newest => MuddleMacroquadSaveSlotSort::Oldest,
            MuddleMacroquadSaveSlotSort::Oldest => MuddleMacroquadSaveSlotSort::Largest,
            MuddleMacroquadSaveSlotSort::Largest => MuddleMacroquadSaveSlotSort::Name,
        };
        self.selected_slot_index = 0;
        self.last_status = format!("Save slots sorted by {}.", self.slot_sort.label());
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

    pub fn open_save_slots(&mut self) {
        self.mode = MuddleMacroquadMode::SaveSlots;
        self.input.clear();
        self.command_history_cursor = None;
        self.keep_selected_slot_visible();
        self.last_status = "Save slots: type to filter or name a new slot.".to_string();
    }

    pub fn close_save_slots(&mut self) {
        self.mode = MuddleMacroquadMode::Playing;
        self.slot_filter.clear();
        self.last_status = "Returned to play.".to_string();
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

    pub fn import_save_text_now(&mut self) -> io::Result<()> {
        let Some(path) = self.import_path.clone() else {
            self.last_status =
                "Start muddle-macroquad with --import before importing save text.".to_string();
            return Ok(());
        };
        let encoded_save = fs::read_to_string(&path)?;
        if self.import_save_text(&encoded_save)? {
            self.last_status = format!("Imported save text from {}.", path.display());
        }
        Ok(())
    }

    pub fn export_save_text_now(&mut self) -> io::Result<()> {
        let exported_save = self.session.save_for_host(self.host.as_ref()).encode();
        self.write_export_text("current save", &exported_save)
    }

    pub fn save_selected_slot(&mut self) -> io::Result<()> {
        let Some((slot_name, slot_path)) = self.selected_or_typed_slot_path()? else {
            return Ok(());
        };

        fs::write(
            &slot_path,
            self.session.save_for_host(self.host.as_ref()).encode(),
        )?;
        self.last_status = format!(
            "Saved session slot `{slot_name}` to {}.",
            slot_path.display()
        );
        self.keep_selected_slot_visible();
        Ok(())
    }

    pub fn load_selected_slot(&mut self) -> io::Result<()> {
        let Some((slot_name, slot_path)) = self.selected_or_typed_slot_path()? else {
            return Ok(());
        };
        if !slot_path.exists() {
            self.last_status = format!("No save slot found at {}.", slot_path.display());
            return Ok(());
        }

        let encoded = fs::read_to_string(&slot_path)?;
        let save = MuddleSessionSave::decode(&encoded)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
        let mut host = (self.registration.create)();
        let session = MuddleSession::resume_for_host(host.as_mut(), &save)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;

        self.host = host;
        self.session = session;
        self.input.clear();
        self.command_history_cursor = None;
        self.mode = MuddleMacroquadMode::Playing;
        self.last_status = format!(
            "Loaded save slot `{slot_name}` from {}.",
            slot_path.display()
        );
        Ok(())
    }

    pub fn delete_selected_slot(&mut self) -> io::Result<()> {
        let Some((slot_name, slot_path)) = self.selected_or_typed_slot_path()? else {
            return Ok(());
        };
        if !slot_path.exists() {
            self.last_status = format!("No save slot found at {}.", slot_path.display());
            return Ok(());
        }

        fs::remove_file(&slot_path)?;
        self.last_status = format!(
            "Deleted save slot `{slot_name}` from {}.",
            slot_path.display()
        );
        self.keep_selected_slot_visible();
        Ok(())
    }

    pub fn export_selected_slot_text(&mut self) -> io::Result<Option<String>> {
        let Some((slot_name, slot_path)) = self.selected_or_typed_slot_path()? else {
            return Ok(None);
        };
        if !slot_path.exists() {
            self.last_status = format!("No save slot found at {}.", slot_path.display());
            return Ok(None);
        }

        let exported_save = fs::read_to_string(&slot_path)?;
        self.last_status = format!(
            "Exported save slot `{slot_name}` from {} ({} bytes).",
            slot_path.display(),
            exported_save.len()
        );
        Ok(Some(exported_save))
    }

    pub fn export_selected_slot_text_now(&mut self) -> io::Result<()> {
        if let Some(exported_save) = self.export_selected_slot_text()? {
            self.write_export_text("selected slot", &exported_save)?;
        }
        Ok(())
    }

    pub fn save_slot_details(&self) -> io::Result<Vec<MuddleMacroquadSaveSlotDetail>> {
        list_save_slot_details(&self.save_path)
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
            MuddleMacroquadMode::SaveSlots => self.save_slot_lines(),
        }
    }

    pub fn play_layout(&self) -> Option<MuddleMacroquadPlayLayout> {
        (self.mode == MuddleMacroquadMode::Playing).then(|| {
            let mut layout = snapshot_play_layout(&self.snapshot(), &self.input);
            layout.panels.push(self.persistence_region());
            layout
        })
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
        self.slot_filter.clear();
        self.selected_host_index = selected_host_index;
        self.selected_slot_index = 0;
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

    fn import_save_text(&mut self, encoded_save: &str) -> io::Result<bool> {
        let encoded_save = encoded_save.trim();
        if encoded_save.is_empty() {
            self.last_status = "Import path contained no save text.".to_string();
            return Ok(false);
        }

        let save = match MuddleSessionSave::decode(encoded_save) {
            Ok(save) => save,
            Err(error) => {
                self.last_status = format!("Import failed: {error:?}");
                return Ok(false);
            }
        };
        let mut host = (self.registration.create)();
        let session = match MuddleSession::resume_for_host(host.as_mut(), &save) {
            Ok(session) => session,
            Err(error) => {
                self.last_status = format!("Import failed: {error:?}");
                return Ok(false);
            }
        };

        self.host = host;
        self.session = session;
        self.input.clear();
        self.command_history_cursor = None;
        Ok(true)
    }

    fn write_export_text(&mut self, label: &str, exported_save: &str) -> io::Result<()> {
        let Some(path) = &self.export_path else {
            self.last_status =
                "Start muddle-macroquad with --export before exporting save text.".to_string();
            return Ok(());
        };
        fs::write(path, exported_save)?;
        self.last_status = format!(
            "Exported {label} save text to {} ({} bytes).",
            path.display(),
            exported_save.len()
        );
        Ok(())
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

    fn save_slot_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "MUDDLE Macroquad Save Slots".to_string(),
            "Type to filter or name a new slot. Up/Down selects. F6 saves. F9 sorts. Enter/F10 loads. Delete removes. F11 exports. Esc returns.".to_string(),
            format!(
                "Base save: {}",
                self.save_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<not configured>".to_string())
            ),
            format!("Filter/new slot: {}", self.slot_filter),
            format!("Sort: {}", self.slot_sort.label()),
            String::new(),
        ];

        match self.filtered_save_slot_details() {
            Ok(slots) if slots.is_empty() => {
                if self.save_path.is_some() {
                    lines.push(
                        "No matching save slots. Type a valid slot name and press F6 to create it."
                            .to_string(),
                    );
                } else {
                    lines.push(
                        "Start muddle-macroquad with --save before using save slots.".to_string(),
                    );
                }
            }
            Ok(slots) => {
                let selected = self.selected_visible_slot_index(&slots);
                for (index, slot) in slots.iter().enumerate() {
                    let marker = if Some(index) == selected { ">" } else { " " };
                    lines.push(format!(
                        "{marker} {} - {} bytes - modified {} - {}",
                        slot.name,
                        slot.bytes,
                        slot.modified_unix,
                        slot.path.display()
                    ));
                }
            }
            Err(error) => lines.push(format!("Could not read save slots: {error}")),
        }
        lines
    }

    fn persistence_region(&self) -> MuddleMacroquadTextRegion {
        let mut lines = Vec::new();
        lines.push(format!(
            "F6 save: {}",
            if self.save_path.is_some() || self.transcript_path.is_some() {
                "available"
            } else {
                "needs --save or --transcript"
            }
        ));
        lines.push(format!(
            "F7 reload: {}",
            if self.save_path.is_some() {
                "available"
            } else {
                "needs --save"
            }
        ));
        lines.push(format!(
            "F11 export: {}",
            if self.export_path.is_some() {
                "available"
            } else {
                "needs --export"
            }
        ));
        lines.push(format!(
            "F12 import: {}",
            if self.import_path.is_some() {
                "available"
            } else {
                "needs --import"
            }
        ));
        lines.push("F8 save slots".to_string());
        lines.push(format!(
            "Save: {}",
            self.save_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<not configured>".to_string())
        ));
        lines.push(format!(
            "Transcript: {}",
            self.transcript_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<not configured>".to_string())
        ));
        lines.push(format!(
            "Import: {}",
            self.import_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<not configured>".to_string())
        ));
        lines.push(format!(
            "Export: {}",
            self.export_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<not configured>".to_string())
        ));
        match self.sorted_save_slot_details() {
            Ok(slots) if slots.is_empty() => lines.push("Slots: none".to_string()),
            Ok(slots) => {
                lines.push(format!(
                    "Slots: {} sorted by {}",
                    slots.len(),
                    self.slot_sort.label()
                ));
                for slot in slots.iter().take(3) {
                    lines.push(format!(
                        "{} ({} bytes, modified {})",
                        slot.name, slot.bytes, slot.modified_unix
                    ));
                }
            }
            Err(error) => lines.push(format!("Slots unavailable: {error}")),
        }

        MuddleMacroquadTextRegion {
            id: "persistence".to_string(),
            label: "Persistence".to_string(),
            lines,
        }
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

    fn keep_selected_slot_visible(&mut self) {
        match self.filtered_save_slot_details() {
            Ok(slots) if self.selected_visible_slot_index(&slots).is_none() => {
                self.selected_slot_index = 0;
            }
            _ => {}
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

    fn select_relative_slot(&mut self, delta: isize) {
        let Ok(slots) = self.filtered_save_slot_details() else {
            return;
        };
        if slots.is_empty() {
            return;
        }

        let current = self.selected_visible_slot_index(&slots).unwrap_or(0);
        let next = (current as isize + delta).rem_euclid(slots.len() as isize) as usize;
        self.selected_slot_index = next;
    }

    fn selected_visible_slot_index(
        &self,
        slots: &[MuddleMacroquadSaveSlotDetail],
    ) -> Option<usize> {
        if slots.is_empty() {
            None
        } else {
            Some(self.selected_slot_index.min(slots.len() - 1))
        }
    }

    fn filtered_save_slot_details(&self) -> io::Result<Vec<MuddleMacroquadSaveSlotDetail>> {
        let filter = self.slot_filter.trim().to_ascii_lowercase();
        let mut slots = self
            .sorted_save_slot_details()?
            .into_iter()
            .filter(|slot| {
                filter.is_empty()
                    || slot.name.to_ascii_lowercase().contains(&filter)
                    || slot
                        .path
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .contains(&filter)
            })
            .collect::<Vec<_>>();
        sort_save_slots(&mut slots, self.slot_sort);
        Ok(slots)
    }

    fn sorted_save_slot_details(&self) -> io::Result<Vec<MuddleMacroquadSaveSlotDetail>> {
        let mut slots = self.save_slot_details()?;
        sort_save_slots(&mut slots, self.slot_sort);
        Ok(slots)
    }

    fn selected_or_typed_slot_path(&mut self) -> io::Result<Option<(String, PathBuf)>> {
        let Some(save_path) = &self.save_path else {
            self.last_status =
                "Start muddle-macroquad with --save before using save slots.".to_string();
            return Ok(None);
        };

        let slot_name = match self.filtered_save_slot_details()? {
            slots if !slots.is_empty() => {
                let selected = self.selected_visible_slot_index(&slots).unwrap_or(0);
                slots[selected].name.clone()
            }
            _ => {
                if self.slot_filter.trim().is_empty() {
                    "quick".to_string()
                } else {
                    self.slot_filter.trim().to_string()
                }
            }
        };

        match save_slot_path(save_path, &slot_name) {
            Ok(slot_path) => Ok(Some((slot_name, slot_path))),
            Err(message) => {
                self.last_status = message;
                Ok(None)
            }
        }
    }
}

pub fn macroquad_window_conf(title: &str) -> Conf {
    Conf {
        window_title: title.to_string(),
        window_width: 960,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

pub async fn run_muddle_macroquad_hosts(
    registrations: Vec<MuddleClientHostRegistration>,
    options: MuddleMacroquadRunOptions,
    config: MuddleMacroquadRunConfig,
) -> Result<(), String> {
    if options.show_help {
        println!("{}", macroquad_usage());
        println!("{}", macroquad_host_list(&registrations));
        return Ok(());
    }
    if options.list_hosts {
        println!("{}", macroquad_host_list(&registrations));
        return Ok(());
    }
    if options.visual_smoke {
        println!(
            "{}",
            macroquad_visual_smoke(
                &registrations,
                options.require_visuals,
                options.require_visual_frames,
                options.visual_smoke_room.as_deref(),
                &options.visual_smoke_commands,
            )?
        );
        return Ok(());
    }

    let mut state = match options.host_name {
        Some(host_name) => MuddleMacroquadState::with_host_and_paths(
            registrations,
            &host_name,
            options.load_path,
            options.save_path,
            options.transcript_path,
            options.import_path,
            options.export_path,
        ),
        None => MuddleMacroquadState::with_chooser_and_paths(
            registrations,
            options.load_path,
            options.save_path,
            options.transcript_path,
            options.import_path,
            options.export_path,
        ),
    }?;
    let mut command_buttons: Vec<(Rect, usize)> = Vec::new();

    loop {
        clear_background(Color::from_rgba(18, 23, 30, 255));

        while let Some(character) = get_char_pressed() {
            state.push_char(character);
        }
        if is_key_pressed(KeyCode::Backspace) {
            state.backspace();
        }
        if is_key_pressed(KeyCode::Up) {
            match state.mode() {
                MuddleMacroquadMode::HostChooser => state.select_previous_host(),
                MuddleMacroquadMode::Playing => state.recall_previous_command(),
                MuddleMacroquadMode::SaveSlots => state.select_previous_slot(),
            }
        }
        if is_key_pressed(KeyCode::Down) {
            match state.mode() {
                MuddleMacroquadMode::HostChooser => state.select_next_host(),
                MuddleMacroquadMode::Playing => state.recall_next_command(),
                MuddleMacroquadMode::SaveSlots => state.select_next_slot(),
            }
        }
        if is_key_pressed(KeyCode::Enter) {
            match state.mode() {
                MuddleMacroquadMode::HostChooser => {
                    if let Err(error) = state.choose_selected_host() {
                        eprintln!("{error}");
                    }
                }
                MuddleMacroquadMode::Playing => state.submit_input(),
                MuddleMacroquadMode::SaveSlots => {
                    if let Err(error) = state.load_selected_slot() {
                        eprintln!("{error}");
                    }
                }
            }
        }
        if is_key_pressed(KeyCode::F2) {
            state.change_host();
        }
        if is_key_pressed(KeyCode::F5) {
            if let Err(error) = state.restart_host() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F6) {
            let result = match state.mode() {
                MuddleMacroquadMode::SaveSlots => state.save_selected_slot(),
                _ => state.save_now(),
            };
            if let Err(error) = result {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F7) {
            if let Err(error) = state.reload_save() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F8) {
            state.open_save_slots();
        }
        if is_key_pressed(KeyCode::F9) && state.mode() == MuddleMacroquadMode::SaveSlots {
            state.cycle_slot_sort();
        }
        if is_key_pressed(KeyCode::F10) {
            if let Err(error) = state.load_selected_slot() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F11) {
            let result = match state.mode() {
                MuddleMacroquadMode::SaveSlots => state.export_selected_slot_text_now(),
                _ => state.export_save_text_now(),
            };
            if let Err(error) = result {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::F12) {
            if let Err(error) = state.import_save_text_now() {
                eprintln!("{error}");
            }
        }
        if is_key_pressed(KeyCode::Delete) {
            if let Err(error) = state.delete_selected_slot() {
                eprintln!("{error}");
            }
        }
        if is_mouse_button_pressed(MouseButton::Left)
            && state.mode() == MuddleMacroquadMode::Playing
        {
            let (x, y) = mouse_position();
            let mouse = Vec2::new(x, y);
            if let Some((_, index)) = command_buttons
                .iter()
                .find(|(rect, _)| rect.contains(mouse))
            {
                state.submit_command_hint(*index);
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            match state.mode() {
                MuddleMacroquadMode::SaveSlots => state.close_save_slots(),
                _ => break,
            }
        }

        command_buttons.clear();
        match state.mode() {
            MuddleMacroquadMode::HostChooser | MuddleMacroquadMode::SaveSlots => {
                draw_lines(&state.display_lines(), 24.0, 28.0)
            }
            MuddleMacroquadMode::Playing => {
                if let Some(layout) = state.play_layout() {
                    draw_play_layout(&config.screen_title, &layout, &mut command_buttons);
                }
            }
        }

        next_frame().await;
    }

    Ok(())
}

fn save_slot_path(save_path: &Path, slot_name: &str) -> Result<PathBuf, String> {
    let slot_name = normalize_save_slot_name(slot_name)?;
    let parent = save_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = save_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("muddle-macroquad");
    let extension = save_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    Ok(parent.join(format!("{stem}.slot-{slot_name}{extension}")))
}

fn normalize_save_slot_name(slot_name: &str) -> Result<String, String> {
    let slot_name = slot_name.trim();
    if slot_name.is_empty() {
        return Err("Enter a save slot name before using save slots.".to_string());
    }
    if slot_name.len() > 48 {
        return Err("Save slot names must be 48 characters or fewer.".to_string());
    }
    if !slot_name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(
            "Save slot names can only use letters, numbers, dash, and underscore.".to_string(),
        );
    }
    Ok(slot_name.to_string())
}

fn list_save_slot_details(
    save_path: &Option<PathBuf>,
) -> io::Result<Vec<MuddleMacroquadSaveSlotDetail>> {
    let Some(save_path) = save_path else {
        return Ok(Vec::new());
    };
    let parent = save_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = save_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("muddle-macroquad");
    let extension = save_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    let prefix = format!("{stem}.slot-");

    let entries = match fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut slots = Vec::new();
    for entry in entries {
        let entry = entry?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !file_name.starts_with(&prefix) || !file_name.ends_with(&extension) {
            continue;
        }

        let without_prefix = &file_name[prefix.len()..];
        let slot_name = if extension.is_empty() {
            without_prefix
        } else {
            &without_prefix[..without_prefix.len() - extension.len()]
        };
        if normalize_save_slot_name(slot_name).is_ok() {
            let metadata = entry.metadata()?;
            let modified_unix = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or_default();
            slots.push(MuddleMacroquadSaveSlotDetail {
                name: slot_name.to_string(),
                path: entry.path(),
                bytes: metadata.len(),
                modified_unix,
            });
        }
    }
    slots.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(slots)
}

fn sort_save_slots(slots: &mut [MuddleMacroquadSaveSlotDetail], sort: MuddleMacroquadSaveSlotSort) {
    match sort {
        MuddleMacroquadSaveSlotSort::Name => {
            slots.sort_by(|left, right| left.name.cmp(&right.name));
        }
        MuddleMacroquadSaveSlotSort::Newest => {
            slots.sort_by(|left, right| {
                right
                    .modified_unix
                    .cmp(&left.modified_unix)
                    .then_with(|| left.name.cmp(&right.name))
            });
        }
        MuddleMacroquadSaveSlotSort::Oldest => {
            slots.sort_by(|left, right| {
                left.modified_unix
                    .cmp(&right.modified_unix)
                    .then_with(|| left.name.cmp(&right.name))
            });
        }
        MuddleMacroquadSaveSlotSort::Largest => {
            slots.sort_by(|left, right| {
                right
                    .bytes
                    .cmp(&left.bytes)
                    .then_with(|| left.name.cmp(&right.name))
            });
        }
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
            "--visual-smoke" => options.visual_smoke = true,
            "--require-visuals" => options.require_visuals = true,
            "--require-visual-frames" => options.require_visual_frames = true,
            "--visual-smoke-room" => {
                options.visual_smoke_room = Some(
                    args.next()
                        .ok_or_else(|| "`--visual-smoke-room` requires a room id.".to_string())?,
                );
            }
            "--visual-smoke-command" => {
                options.visual_smoke_commands.push(
                    args.next().ok_or_else(|| {
                        "`--visual-smoke-command` requires a command.".to_string()
                    })?,
                );
            }
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
            "--import" => {
                options.import_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "`--import` requires a path.".to_string())?,
                ));
            }
            "--export" => {
                options.export_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "`--export` requires a path.".to_string())?,
                ));
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
                } else if let Some(value) = arg.strip_prefix("--import=") {
                    if value.is_empty() {
                        return Err("`--import` requires a path.".to_string());
                    }
                    options.import_path = Some(PathBuf::from(value));
                } else if let Some(value) = arg.strip_prefix("--export=") {
                    if value.is_empty() {
                        return Err("`--export` requires a path.".to_string());
                    }
                    options.export_path = Some(PathBuf::from(value));
                } else if let Some(value) = arg.strip_prefix("--visual-smoke-command=") {
                    if value.is_empty() {
                        return Err("`--visual-smoke-command` requires a command.".to_string());
                    }
                    options.visual_smoke_commands.push(value.to_string());
                } else if let Some(value) = arg.strip_prefix("--visual-smoke-room=") {
                    if value.is_empty() {
                        return Err("`--visual-smoke-room` requires a room id.".to_string());
                    }
                    options.visual_smoke_room = Some(value.to_string());
                } else {
                    return Err(format!("Unknown argument `{arg}`."));
                }
            }
        }
    }

    Ok(options)
}

pub fn macroquad_usage() -> &'static str {
    "Usage: muddle-macroquad [--host <name>] [--load <path>] [--save <path>] [--transcript <path>] [--import <path>] [--export <path>] [--list-hosts] [--visual-smoke] [--require-visuals] [--require-visual-frames] [--visual-smoke-room <room-id>] [--visual-smoke-command <command>] [--help]"
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

pub fn macroquad_visual_smoke(
    registrations: &[MuddleClientHostRegistration],
    require_visuals: bool,
    require_visual_frames: bool,
    expected_room: Option<&str>,
    commands: &[String],
) -> Result<String, String> {
    let mut lines = vec!["MUDDLE Macroquad visual smoke:".to_string()];
    let mut missing_visual_hosts = Vec::new();
    let mut missing_frame_hosts = Vec::new();
    let mut room_mismatch_hosts = Vec::new();
    for registration in registrations {
        let mut host = (registration.create)();
        let mut session = MuddleSession::for_host(host.as_ref()).map_err(|error| {
            format!(
                "{} failed to start for visual smoke: {error:?}",
                registration.name
            )
        })?;
        for command in commands {
            session
                .play_turn(host.as_mut(), MuddleCommand::parse(command))
                .map_err(|error| {
                    format!(
                        "{} visual smoke command `{command}` failed: {error:?}",
                        registration.name
                    )
                })?;
        }
        let snapshot = session.client_snapshot(
            host.as_ref(),
            registration.client_info(),
            "Visual smoke snapshot.",
        );
        let layout = snapshot_play_layout(&snapshot, "");
        let sprite_count = layout
            .visual_nodes
            .iter()
            .filter(|node| node.kind == MuddleVisualNodeKind::Sprite)
            .count();
        let text_count = layout
            .visual_nodes
            .iter()
            .filter(|node| node.kind == MuddleVisualNodeKind::Text)
            .count();
        let group_count = layout
            .visual_nodes
            .iter()
            .filter(|node| node.kind == MuddleVisualNodeKind::Group)
            .count();
        let frame_count = layout
            .visual_nodes
            .iter()
            .filter(|node| node.sprite_frame.is_some())
            .count();
        let animated_count = layout
            .visual_nodes
            .iter()
            .filter(|node| node.sprite_animation.is_some())
            .count();
        let status = if layout.visual_nodes.is_empty() {
            missing_visual_hosts.push(registration.name);
            "no visual nodes"
        } else if require_visual_frames && frame_count == 0 {
            missing_frame_hosts.push(registration.name);
            "no visual frames"
        } else if expected_room.is_some_and(|room| snapshot.room != room) {
            room_mismatch_hosts.push(format!(
                "{} expected {} got {}",
                registration.name,
                expected_room.unwrap_or_default(),
                snapshot.room
            ));
            "unexpected room"
        } else {
            "visual nodes ready"
        };
        lines.push(format!(
            "  {}: {} total (room={}, sprites={}, text={}, groups={}, frames={}, animated={}, turns={}) - {}",
            registration.name,
            layout.visual_nodes.len(),
            snapshot.room,
            sprite_count,
            text_count,
            group_count,
            frame_count,
            animated_count,
            session.transcript.len(),
            status
        ));
    }
    if require_visuals && !missing_visual_hosts.is_empty() {
        return Err(format!(
            "visual smoke failed: hosts without visual nodes: {}",
            missing_visual_hosts.join(", ")
        ));
    }
    if require_visual_frames && !missing_frame_hosts.is_empty() {
        return Err(format!(
            "visual smoke failed: hosts without visual frames: {}",
            missing_frame_hosts.join(", ")
        ));
    }
    if !room_mismatch_hosts.is_empty() {
        return Err(format!(
            "visual smoke failed: hosts in unexpected rooms: {}",
            room_mismatch_hosts.join(", ")
        ));
    }
    Ok(lines.join("\n"))
}

pub fn apply_default_macroquad_paths(
    options: &mut MuddleMacroquadRunOptions,
    save_path: impl Into<PathBuf>,
    transcript_path: impl Into<PathBuf>,
    import_path: impl Into<PathBuf>,
    export_path: impl Into<PathBuf>,
) {
    if options.save_path.is_none() {
        options.save_path = Some(save_path.into());
    }
    if options.transcript_path.is_none() {
        options.transcript_path = Some(transcript_path.into());
    }
    if options.import_path.is_none() {
        options.import_path = Some(import_path.into());
    }
    if options.export_path.is_none() {
        options.export_path = Some(export_path.into());
    }
}

fn draw_play_layout(
    title: &str,
    layout: &MuddleMacroquadPlayLayout,
    command_buttons: &mut Vec<(Rect, usize)>,
) {
    let margin = 18.0;
    let width = screen_width();
    let height = screen_height();
    draw_region(
        Rect::new(margin, margin, width - margin * 2.0, 92.0),
        title,
        &layout.header,
        Color::from_rgba(28, 36, 48, 255),
    );

    let body_top = 122.0;
    let command_height = 94.0;
    let status_height = 92.0;
    let body_height = height - body_top - command_height - status_height - margin * 2.0;
    let left_width = (width - margin * 3.0) * 0.62;
    let right_width = width - margin * 3.0 - left_width;
    let left = Rect::new(margin, body_top, left_width, body_height);
    if layout.visual_nodes.is_empty() {
        draw_region(
            left,
            &layout.room.label,
            &layout.room.lines,
            Color::from_rgba(20, 27, 36, 255),
        );
    } else {
        let scene_height = (body_height * 0.46).max(140.0).min(body_height - 120.0);
        draw_visual_scene(
            Rect::new(left.x, left.y, left.w, scene_height),
            &layout.visual_nodes,
        );
        draw_region(
            Rect::new(
                left.x,
                left.y + scene_height + 8.0,
                left.w,
                left.h - scene_height - 8.0,
            ),
            &layout.room.label,
            &layout.room.lines,
            Color::from_rgba(20, 27, 36, 255),
        );
    }

    let right = Rect::new(
        margin * 2.0 + left_width,
        body_top,
        right_width,
        body_height,
    );
    draw_panel_stack(right, &layout.panels);

    let command_rect = Rect::new(
        margin,
        height - command_height - status_height - margin,
        width - margin * 2.0,
        command_height,
    );
    draw_command_buttons(command_rect, &layout.commands, command_buttons);

    let status_rect = Rect::new(
        margin,
        height - status_height,
        (width - margin * 3.0) * 0.5,
        status_height - margin,
    );
    draw_region(
        status_rect,
        &layout.status.label,
        &layout.status.lines,
        Color::from_rgba(28, 36, 48, 255),
    );
    let history_rect = Rect::new(
        status_rect.x + status_rect.w + margin,
        status_rect.y,
        width - status_rect.w - margin * 3.0,
        status_rect.h,
    );
    draw_region(
        history_rect,
        &layout.history.label,
        &layout.history.lines,
        Color::from_rgba(20, 27, 36, 255),
    );
}

fn draw_panel_stack(rect: Rect, panels: &[MuddleMacroquadTextRegion]) {
    if panels.is_empty() {
        draw_region(
            rect,
            "Panels",
            &["No host panels available.".to_string()],
            Color::from_rgba(20, 27, 36, 255),
        );
        return;
    }

    let gap = 8.0;
    let panel_height =
        ((rect.h - gap * (panels.len().saturating_sub(1) as f32)) / panels.len() as f32).max(60.0);
    let mut y = rect.y;
    for panel in panels {
        draw_region(
            Rect::new(rect.x, y, rect.w, panel_height),
            &panel.label,
            &panel.lines,
            Color::from_rgba(20, 27, 36, 255),
        );
        y += panel_height + gap;
        if y > rect.y + rect.h {
            break;
        }
    }
}

fn draw_visual_scene(rect: Rect, nodes: &[MuddleMacroquadVisualNode]) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(12, 18, 28, 255),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        Color::from_rgba(112, 175, 220, 255),
    );
    draw_text("Scene", rect.x + 12.0, rect.y + 24.0, 20.0, WHITE);

    let max_x = nodes
        .iter()
        .map(|node| node.x + node.width)
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let max_y = nodes
        .iter()
        .map(|node| node.y + node.height)
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let content = Rect::new(rect.x + 14.0, rect.y + 38.0, rect.w - 28.0, rect.h - 52.0);

    for node in nodes {
        let x = content.x + (node.x.max(0) as f32 / max_x) * content.w;
        let y = content.y + (node.y.max(0) as f32 / max_y) * content.h;
        let w = ((node.width.max(1) as f32 / max_x) * content.w).max(48.0);
        let h = ((node.height.max(1) as f32 / max_y) * content.h).max(28.0);
        let fill_color = visual_node_fill_color(node);
        let border_color = visual_node_border_color(node);
        match node.kind {
            MuddleVisualNodeKind::Sprite => {
                draw_rectangle(x, y, w.min(content.w), h.min(content.h), fill_color);
                draw_rectangle_lines(x, y, w.min(content.w), h.min(content.h), 1.5, border_color);
                draw_text(&node.label, x + 6.0, y + 18.0, 16.0, WHITE);
                if let Some(frame) = &node.sprite_frame {
                    draw_text(frame, x + 6.0, y + 36.0, 14.0, LIGHTGRAY);
                }
                if let Some(source) = &node.sprite_source {
                    draw_text(
                        &visual_source_name(source),
                        x + 6.0,
                        y + h.min(content.h) - 8.0,
                        12.0,
                        LIGHTGRAY,
                    );
                }
            }
            MuddleVisualNodeKind::Text => {
                let text = node.text.as_deref().unwrap_or(&node.label);
                draw_text(text, x, y + 18.0, 18.0, fill_color);
            }
            MuddleVisualNodeKind::Group => {
                draw_rectangle_lines(x, y, w.min(content.w), h.min(content.h), 1.0, border_color);
                draw_text(&node.label, x + 6.0, y + 18.0, 16.0, fill_color);
            }
        }
    }
}

fn visual_node_fill_color(node: &MuddleMacroquadVisualNode) -> Color {
    if node.sprite_animation.is_some() {
        return Color::from_rgba(206, 156, 72, 255);
    }
    match node.kind {
        MuddleVisualNodeKind::Sprite => match node.sprite_frame.as_deref() {
            Some("active") => Color::from_rgba(48, 136, 94, 235),
            Some("idle") => Color::from_rgba(54, 104, 153, 210),
            Some("armed") => Color::from_rgba(172, 67, 67, 235),
            Some("building") => Color::from_rgba(168, 111, 48, 225),
            Some("claimed") | Some("closed") | Some("lit") | Some("broadcast") => {
                Color::from_rgba(188, 143, 58, 235)
            }
            Some("ready") | Some("ordered") | Some("set") | Some("scouted") | Some("resolved")
            | Some("scored") | Some("frequency") | Some("bearing") => {
                Color::from_rgba(75, 142, 126, 230)
            }
            _ => Color::from_rgba(54, 104, 153, 230),
        },
        MuddleVisualNodeKind::Text => SKYBLUE,
        MuddleVisualNodeKind::Group => Color::from_rgba(126, 143, 168, 230),
    }
}

fn visual_node_border_color(node: &MuddleMacroquadVisualNode) -> Color {
    if node.sprite_animation.is_some() {
        return GOLD;
    }
    match node.sprite_frame.as_deref() {
        Some("active") => LIME,
        Some("armed") => RED,
        Some("claimed") | Some("closed") | Some("lit") | Some("broadcast") => GOLD,
        Some("ready") | Some("ordered") | Some("set") | Some("scouted") | Some("resolved")
        | Some("scored") | Some("frequency") | Some("bearing") => GREEN,
        _ => WHITE,
    }
}

fn visual_source_name(source: &str) -> String {
    Path::new(source)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.to_string())
}

fn draw_command_buttons(
    rect: Rect,
    commands: &[MuddleMacroquadCommandControl],
    command_buttons: &mut Vec<(Rect, usize)>,
) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(28, 36, 48, 255),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::from_rgba(82, 100, 128, 255),
    );
    draw_text("Commands", rect.x + 12.0, rect.y + 24.0, 20.0, WHITE);
    if commands.is_empty() {
        draw_text(
            "No command hints from this host.",
            rect.x + 12.0,
            rect.y + 54.0,
            18.0,
            GRAY,
        );
        return;
    }

    let mut x = rect.x + 12.0;
    let mut y = rect.y + 38.0;
    for command in commands {
        let label = command.command.as_str();
        let text_size = measure_text(label, None, 18, 1.0);
        let button_width = (text_size.width + 26.0).min(rect.w - 24.0);
        if x + button_width > rect.x + rect.w - 12.0 {
            x = rect.x + 12.0;
            y += 32.0;
        }
        if y + 28.0 > rect.y + rect.h - 8.0 {
            break;
        }
        let button = Rect::new(x, y, button_width, 26.0);
        draw_rectangle(
            button.x,
            button.y,
            button.w,
            button.h,
            Color::from_rgba(56, 75, 108, 255),
        );
        draw_rectangle_lines(button.x, button.y, button.w, button.h, 1.0, SKYBLUE);
        draw_text(label, button.x + 10.0, button.y + 18.0, 18.0, WHITE);
        command_buttons.push((button, command.index));
        x += button_width + 8.0;
    }
}

fn draw_region(rect: Rect, title: &str, lines: &[String], background: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::from_rgba(82, 100, 128, 255),
    );
    draw_text(title, rect.x + 12.0, rect.y + 24.0, 20.0, WHITE);
    draw_lines(lines, rect.x + 12.0, rect.y + 50.0);
}

fn draw_lines(lines: &[String], x: f32, mut y: f32) {
    for line in lines {
        for wrapped in wrap_line(line, 92) {
            draw_text(&wrapped, x, y, 18.0, LIGHTGRAY);
            y += 22.0;
            if y > screen_height() - 12.0 {
                return;
            }
        }
        if line.is_empty() {
            y += 8.0;
        }
    }
}

fn wrap_line(line: &str, max_chars: usize) -> Vec<String> {
    if line.len() <= max_chars {
        return vec![line.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        if !current.is_empty() && current.len() + word.len() + 1 > max_chars {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

pub fn snapshot_display_lines(snapshot: &MuddleClientSnapshot, input: &str) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("MUDDLE Macroquad Runner".to_string());
    lines.push(
        "Esc quits. F2 changes host. F5 restarts. F6 saves. F7 reloads. F8 opens save slots. F12 imports save text. Up/Down recalls commands. Enter submits.".to_string(),
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
    let panels = if let Some(visuals) = find_control(controls, "visuals") {
        let mut panels = panels;
        panels.push(control_text_region(visuals));
        panels
    } else {
        panels
    };

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
        visual_nodes: flatten_visual_nodes(&snapshot.visual_nodes),
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

fn flatten_visual_nodes(nodes: &[MuddleVisualNode]) -> Vec<MuddleMacroquadVisualNode> {
    let mut flattened = Vec::new();
    for node in nodes {
        push_visual_node(node, &mut flattened);
    }
    flattened.sort_by_key(|node| (node.layer, node.id.clone()));
    flattened
}

fn push_visual_node(node: &MuddleVisualNode, flattened: &mut Vec<MuddleMacroquadVisualNode>) {
    flattened.push(MuddleMacroquadVisualNode {
        id: node.id.clone(),
        kind: node.kind,
        label: node.label.clone(),
        text: node.text.clone(),
        sprite_source: node.sprite.as_ref().map(|sprite| sprite.source.clone()),
        sprite_frame: node.sprite.as_ref().and_then(|sprite| sprite.frame.clone()),
        sprite_animation: node
            .sprite
            .as_ref()
            .and_then(|sprite| sprite.animation.clone()),
        layer: node.layer,
        x: node.x,
        y: node.y,
        width: node.width,
        height: node.height,
    });
    for child in &node.children {
        push_visual_node(child, flattened);
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
    lines.push(
        "F2 host | F5 restart | F6 save | F7 reload | F8 slots | F12 import | Esc quit".to_string(),
    );
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
    fn macroquad_play_layout_flattens_visual_nodes_for_scene_rendering() {
        let snapshot = MuddleClientSnapshot {
            host: "visual-host".to_string(),
            description: "Visual host".to_string(),
            suggested_commands: "look".to_string(),
            room: "scene".to_string(),
            turns: 0,
            room_card: "Scene room".to_string(),
            last_response: "Ready.".to_string(),
            panels: muddle_core::MuddleClientPanels {
                resources: Vec::new(),
                inventory: Vec::new(),
                map: None,
                objectives: Vec::new(),
                recent_log: Vec::new(),
            },
            commands: Vec::new(),
            history: Vec::new(),
            visual_nodes: vec![MuddleVisualNode::group(
                "scene",
                "Scene",
                vec![
                    MuddleVisualNode::sprite("hero", "Hero", "sprites/hero.png", "Hero")
                        .with_layer(10)
                        .with_rect(2, 3, 1, 1)
                        .with_frame("idle"),
                    MuddleVisualNode::text("caption", "Caption", "Ready")
                        .with_layer(20)
                        .with_rect(0, 0, 4, 1),
                ],
            )],
            controls: vec![MuddleClientControl::text("room-card", "Room", "Scene room")],
        };

        let layout = snapshot_play_layout(&snapshot, "");

        assert!(layout.visual_nodes.iter().any(|node| node.id == "hero"
            && node.sprite_source.as_deref() == Some("sprites/hero.png")
            && node.sprite_frame.as_deref() == Some("idle")));
        assert!(layout
            .visual_nodes
            .windows(2)
            .all(|nodes| nodes[0].layer <= nodes[1].layer));
    }

    #[test]
    fn macroquad_visual_node_colors_reflect_frames() {
        let active = MuddleMacroquadVisualNode {
            id: "active".to_string(),
            kind: MuddleVisualNodeKind::Sprite,
            label: "Active".to_string(),
            text: None,
            sprite_source: Some("sprites/active.png".to_string()),
            sprite_frame: Some("active".to_string()),
            sprite_animation: None,
            layer: 0,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        let armed = MuddleMacroquadVisualNode {
            sprite_frame: Some("armed".to_string()),
            ..active.clone()
        };
        let pulse = MuddleMacroquadVisualNode {
            sprite_animation: Some("pulse".to_string()),
            ..active.clone()
        };

        assert_eq!(visual_node_border_color(&active), LIME);
        assert_eq!(visual_node_border_color(&armed), RED);
        assert_eq!(visual_node_border_color(&pulse), GOLD);
        assert_eq!(
            visual_source_name("sprites/banish/winter-hearth.png"),
            "winter-hearth.png"
        );
        assert_ne!(
            visual_node_fill_color(&active),
            visual_node_fill_color(&armed)
        );
    }

    #[test]
    fn macroquad_play_layout_exposes_persistence_availability() {
        let save_path = PathBuf::from("play.muddle");
        let transcript_path = PathBuf::from("play.txt");
        let import_path = PathBuf::from("import.muddle");
        let export_path = PathBuf::from("export.muddle");
        let state = MuddleMacroquadState::with_host_and_paths(
            default_macroquad_hosts(),
            DEFAULT_HOST,
            None,
            Some(save_path.clone()),
            Some(transcript_path.clone()),
            Some(import_path.clone()),
            Some(export_path.clone()),
        )
        .expect("state starts");
        let layout = state.play_layout().expect("playing state has layout");
        let persistence = layout
            .panels
            .iter()
            .find(|panel| panel.id == "persistence")
            .expect("persistence panel is present");

        assert_eq!(persistence.label, "Persistence");
        assert!(persistence
            .lines
            .iter()
            .any(|line| line == "F6 save: available"));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line == "F7 reload: available"));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line == "F11 export: available"));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line == "F12 import: available"));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line.contains("play.muddle")));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line.contains("play.txt")));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line.contains("import.muddle")));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line.contains("export.muddle")));

        let state = MuddleMacroquadState::new().expect("state starts");
        let layout = state.play_layout().expect("playing state has layout");
        let persistence = layout
            .panels
            .iter()
            .find(|panel| panel.id == "persistence")
            .expect("persistence panel is present");
        assert!(persistence
            .lines
            .iter()
            .any(|line| line == "F6 save: needs --save or --transcript"));
        assert!(persistence
            .lines
            .iter()
            .any(|line| line == "F7 reload: needs --save"));
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
            "--import=portable.muddle".to_string(),
            "--export".to_string(),
            "exported.muddle".to_string(),
            "--list-hosts".to_string(),
            "--visual-smoke".to_string(),
            "--require-visuals".to_string(),
            "--require-visual-frames".to_string(),
            "--visual-smoke-room".to_string(),
            "scene".to_string(),
            "--visual-smoke-command".to_string(),
            "look".to_string(),
            "--visual-smoke-command=wait".to_string(),
        ])
        .expect("args parse");
        assert_eq!(options.host_name.as_deref(), Some("banish-pilgrim-loss"));
        assert_eq!(options.save_path, Some(PathBuf::from("play.muddle")));
        assert_eq!(options.import_path, Some(PathBuf::from("portable.muddle")));
        assert_eq!(options.export_path, Some(PathBuf::from("exported.muddle")));
        assert!(options.list_hosts);
        assert!(options.visual_smoke);
        assert!(options.require_visuals);
        assert!(options.require_visual_frames);
        assert_eq!(options.visual_smoke_room.as_deref(), Some("scene"));
        assert_eq!(options.visual_smoke_commands, ["look", "wait"]);
    }

    #[test]
    fn macroquad_visual_smoke_reports_registered_visual_nodes() {
        let output = macroquad_visual_smoke(
            &[MuddleClientHostRegistration {
                name: "visual-host",
                category: "Tests",
                description: "Visual smoke test host.",
                suggested_commands: "`look`.",
                create: || Box::new(VisualSmokeTestHost::new()),
            }],
            true,
            false,
            None,
            &[],
        )
        .expect("visual smoke succeeds");

        assert!(output.contains("visual-host: 2 total"));
        assert!(output.contains("sprites=1"));
        assert!(output.contains("text=1"));
    }

    #[test]
    fn macroquad_visual_smoke_replays_commands_before_snapshot() {
        let output = macroquad_visual_smoke(
            &[MuddleClientHostRegistration {
                name: "visual-host",
                category: "Tests",
                description: "Visual smoke test host.",
                suggested_commands: "`look`.",
                create: || Box::new(VisualSmokeTestHost::new()),
            }],
            true,
            true,
            Some("scene"),
            &["advance".to_string()],
        )
        .expect("visual smoke succeeds");

        assert!(output.contains("turns=1"));
        assert!(output.contains("frames=1"));
    }

    #[test]
    fn macroquad_visual_smoke_can_require_final_room() {
        let error = macroquad_visual_smoke(
            &[MuddleClientHostRegistration {
                name: "visual-host",
                category: "Tests",
                description: "Visual smoke test host.",
                suggested_commands: "`look`.",
                create: || Box::new(VisualSmokeTestHost::new()),
            }],
            true,
            true,
            Some("elsewhere"),
            &["advance".to_string()],
        )
        .expect_err("visual smoke fails in unexpected room");

        assert!(error.contains("visual-host expected elsewhere got scene"));
    }

    #[test]
    fn macroquad_visual_smoke_can_require_visual_frames() {
        let error = macroquad_visual_smoke(
            &[MuddleClientHostRegistration {
                name: "visual-host",
                category: "Tests",
                description: "Visual smoke test host.",
                suggested_commands: "`look`.",
                create: || Box::new(VisualSmokeTestHost::new()),
            }],
            true,
            true,
            None,
            &[],
        )
        .expect_err("strict frame visual smoke fails without frames");

        assert!(error.contains("visual-host"));
        assert!(error.contains("visual frames"));
    }

    #[test]
    fn macroquad_visual_smoke_can_require_visual_nodes() {
        let error = macroquad_visual_smoke(
            &[MuddleClientHostRegistration {
                name: "plain-host",
                category: "Tests",
                description: "Plain smoke test host.",
                suggested_commands: "`look`.",
                create: || Box::new(MuddleMockSimHost::new()),
            }],
            true,
            false,
            None,
            &[],
        )
        .expect_err("strict visual smoke fails without nodes");

        assert!(error.contains("plain-host"));
    }

    struct VisualSmokeTestHost {
        room: muddle_core::MuddleRoom,
        advanced: bool,
    }

    impl VisualSmokeTestHost {
        fn new() -> Self {
            Self {
                room: muddle_core::MuddleRoom {
                    id: "scene".to_string(),
                    title: "Scene".to_string(),
                    description: "A visual smoke scene.".to_string(),
                    exits: Vec::new(),
                },
                advanced: false,
            }
        }
    }

    impl MuddleHost for VisualSmokeTestHost {
        fn start_room(&self) -> &str {
            "scene"
        }

        fn room(&self, room_id: &str) -> Option<&muddle_core::MuddleRoom> {
            if room_id == self.room.id {
                Some(&self.room)
            } else {
                None
            }
        }

        fn visual_nodes(&self, _current_room: &str) -> Vec<MuddleVisualNode> {
            let hero = MuddleVisualNode::sprite("hero", "Hero", "sprites/hero.png", "Hero")
                .with_layer(10)
                .with_rect(1, 1, 1, 1);
            let hero = if self.advanced {
                hero.with_frame("ready")
            } else {
                hero
            };
            vec![
                hero,
                MuddleVisualNode::text("caption", "Caption", "Ready")
                    .with_layer(20)
                    .with_rect(1, 2, 2, 1),
            ]
        }

        fn handle_command(
            &mut self,
            _room_id: &str,
            command: &MuddleCommand,
        ) -> Result<muddle_core::MuddleCommandOutcome, muddle_core::MuddleError> {
            if command.verb == "advance" {
                self.advanced = true;
            }
            Ok(muddle_core::MuddleCommandOutcome::stay("Ready."))
        }
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
            None,
            None,
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

    #[test]
    fn macroquad_state_imports_save_text_from_path() {
        let unique = format!(
            "muddle-macroquad-import-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time is after epoch")
                .as_nanos()
        );
        let import_path = std::env::temp_dir().join(format!("{unique}.muddle"));

        let mut source = MuddleMacroquadState::new().expect("source starts");
        for character in "look".chars() {
            source.push_char(character);
        }
        source.submit_input();
        fs::write(
            &import_path,
            source.session.save_for_host(source.host.as_ref()).encode(),
        )
        .expect("import save writes");

        let mut state = MuddleMacroquadState::with_host_and_paths(
            default_macroquad_hosts(),
            DEFAULT_HOST,
            None,
            None,
            None,
            Some(import_path.clone()),
            None,
        )
        .expect("state starts");
        assert_eq!(state.turns(), 0);
        state.import_save_text_now().expect("state imports");
        assert_eq!(state.turns(), 1);
        assert_eq!(state.import_path(), Some(&import_path));
        assert!(state.display_lines().iter().any(|line| {
            line.contains("Imported save text from") || line.contains("Recent history")
        }));

        fs::write(&import_path, "not a save").expect("invalid import save writes");
        state
            .import_save_text_now()
            .expect("invalid import handled");
        assert!(state
            .display_lines()
            .iter()
            .any(|line| line.contains("Import failed")));

        let _ = fs::remove_file(import_path);
    }

    #[test]
    fn macroquad_state_exports_save_text_to_path() {
        let unique = format!(
            "muddle-macroquad-export-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time is after epoch")
                .as_nanos()
        );
        let save_path = std::env::temp_dir().join(format!("{unique}.muddle"));
        let export_path = std::env::temp_dir().join(format!("{unique}.export.muddle"));

        let mut state = MuddleMacroquadState::with_host_and_paths(
            default_macroquad_hosts(),
            DEFAULT_HOST,
            None,
            Some(save_path.clone()),
            None,
            None,
            Some(export_path.clone()),
        )
        .expect("state starts");
        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        state.export_save_text_now().expect("current save exports");
        let exported_current = fs::read_to_string(&export_path).expect("export reads");
        assert!(exported_current.contains("MUDDLE_SESSION_V1"));
        assert_eq!(state.export_path(), Some(&export_path));

        state.open_save_slots();
        for character in "camp".chars() {
            state.push_char(character);
        }
        state.save_selected_slot().expect("slot saves");
        state
            .export_selected_slot_text_now()
            .expect("selected slot exports");
        let exported_slot = fs::read_to_string(&export_path).expect("slot export reads");
        assert!(exported_slot.contains("MUDDLE_SESSION_V1"));
        assert!(state.last_status.contains("Exported selected slot"));

        for slot in state.save_slot_details().expect("slots list") {
            let _ = fs::remove_file(slot.path);
        }
        let _ = fs::remove_file(save_path);
        let _ = fs::remove_file(export_path);
    }

    #[test]
    fn macroquad_state_manages_save_slots() {
        let unique = format!(
            "muddle-macroquad-slot-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time is after epoch")
                .as_nanos()
        );
        let save_path = std::env::temp_dir().join(format!("{unique}.muddle"));

        let mut state = MuddleMacroquadState::with_host_and_paths(
            default_macroquad_hosts(),
            DEFAULT_HOST,
            None,
            Some(save_path.clone()),
            None,
            None,
            None,
        )
        .expect("state starts");
        state.open_save_slots();
        for character in "camp".chars() {
            state.push_char(character);
        }
        state.save_selected_slot().expect("slot saves");

        let slots = state.save_slot_details().expect("slots list");
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].name, "camp");
        assert!(slots[0].path.exists());

        state.close_save_slots();
        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        assert_eq!(state.turns(), 1);

        state.open_save_slots();
        state.load_selected_slot().expect("slot loads");
        assert_eq!(state.mode(), MuddleMacroquadMode::Playing);
        assert_eq!(state.turns(), 0);

        state.open_save_slots();
        let exported = state
            .export_selected_slot_text()
            .expect("slot exports")
            .expect("slot exists");
        assert!(exported.contains("MUDDLE_SESSION_V1"));
        state.delete_selected_slot().expect("slot deletes");
        assert!(state.save_slot_details().expect("slots list").is_empty());

        let _ = fs::remove_file(save_path);
    }

    #[test]
    fn macroquad_save_slots_sort_and_show_details() {
        let unique = format!(
            "muddle-macroquad-slot-sort-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time is after epoch")
                .as_nanos()
        );
        let save_path = std::env::temp_dir().join(format!("{unique}.muddle"));

        let mut state = MuddleMacroquadState::with_host_and_paths(
            default_macroquad_hosts(),
            DEFAULT_HOST,
            None,
            Some(save_path.clone()),
            None,
            None,
            None,
        )
        .expect("state starts");

        state.open_save_slots();
        for character in "small".chars() {
            state.push_char(character);
        }
        state.save_selected_slot().expect("small slot saves");
        state.close_save_slots();

        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        state.open_save_slots();
        for character in "large".chars() {
            state.push_char(character);
        }
        state.save_selected_slot().expect("large slot saves");
        state.slot_filter.clear();

        let by_name = state.filtered_save_slot_details().expect("slots sort");
        assert_eq!(
            by_name
                .iter()
                .map(|slot| slot.name.as_str())
                .collect::<Vec<_>>(),
            vec!["large", "small"]
        );
        let lines = state.save_slot_lines();
        assert!(lines.iter().any(|line| line.contains("Sort: name")));
        assert!(lines.iter().any(|line| line.contains("modified")));

        state.cycle_slot_sort();
        state.cycle_slot_sort();
        state.cycle_slot_sort();
        assert_eq!(state.slot_sort(), MuddleMacroquadSaveSlotSort::Largest);
        let by_size = state.filtered_save_slot_details().expect("slots sort");
        assert_eq!(
            by_size.first().map(|slot| slot.name.as_str()),
            Some("large")
        );

        for slot in state.save_slot_details().expect("slots list") {
            let _ = fs::remove_file(slot.path);
        }
        let _ = fs::remove_file(save_path);
    }
}
