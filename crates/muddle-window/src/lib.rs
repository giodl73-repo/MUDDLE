use std::{
    env, fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

use muddle_cli::{render_transcript, write_play_panels, MuddleCliHostInfo};
use muddle_core::{MuddleCommand, MuddleHost, MuddleSession, MuddleSessionSave};

#[derive(Clone, Copy)]
pub struct MuddleWindowHostRegistration {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub suggested_commands: &'static str,
    pub create: fn() -> Box<dyn MuddleHost>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleWindowRunOptions {
    pub host_name: Option<String>,
    pub addr: String,
    pub open: bool,
    pub list_hosts: bool,
    pub load_path: Option<PathBuf>,
    pub save_path: Option<PathBuf>,
    pub transcript_path: Option<PathBuf>,
}

struct MuddleWindowState {
    host: Box<dyn MuddleHost>,
    session: MuddleSession,
    registration: MuddleWindowHostRegistration,
    last_response: String,
    save_path: Option<PathBuf>,
    transcript_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SaveSlotDetail {
    name: String,
    path: PathBuf,
    bytes: u64,
    modified_unix: u64,
}

impl Default for MuddleWindowRunOptions {
    fn default() -> Self {
        Self {
            host_name: None,
            addr: "127.0.0.1:4777".to_string(),
            open: false,
            list_hosts: false,
            load_path: None,
            save_path: None,
            transcript_path: None,
        }
    }
}

pub fn run_muddle_window_hosts_from_env_args(
    registrations: Vec<MuddleWindowHostRegistration>,
) -> io::Result<()> {
    let options = parse_window_run_options(env::args().skip(1))?;
    run_muddle_window_hosts(registrations, options)
}

pub fn run_muddle_window_hosts(
    registrations: Vec<MuddleWindowHostRegistration>,
    options: MuddleWindowRunOptions,
) -> io::Result<()> {
    if registrations.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "muddle-window requires at least one host registration",
        ));
    }

    if options.list_hosts {
        print_window_hosts(&registrations);
        return Ok(());
    }

    let registration = if let Some(host_name) = &options.host_name {
        find_window_host(&registrations, host_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown MUDDLE window host `{host_name}`."),
            )
        })?
    } else {
        registrations[0]
    };

    let url = format!("http://{}", options.addr);
    let mut state = MuddleWindowState::new(
        registration,
        options.load_path.clone(),
        options.save_path.clone(),
        options.transcript_path.clone(),
    )?;
    let listener = TcpListener::bind(&options.addr)?;
    println!("MUDDLE window client listening at {url}");
    println!("Host mounted: {}", state.registration.name);
    println!("Press Ctrl+C to stop.");

    if options.open {
        open_browser(&url)?;
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, &registrations, &mut state)?,
            Err(error) => eprintln!("MUDDLE window connection failed: {error}"),
        }
    }

    Ok(())
}

pub fn parse_window_run_options(
    args: impl IntoIterator<Item = String>,
) -> io::Result<MuddleWindowRunOptions> {
    let mut options = MuddleWindowRunOptions::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--open" => options.open = true,
            "--list-hosts" => options.list_hosts = true,
            "--host" => {
                options.host_name = Some(args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "`--host` requires a host name")
                })?);
            }
            "--addr" => {
                options.addr = args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "`--addr` requires an address")
                })?;
            }
            "--load" => {
                options.load_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "`--load` requires a path")
                })?));
            }
            "--save" => {
                options.save_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "`--save` requires a path")
                })?));
            }
            "--transcript" => {
                options.transcript_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "`--transcript` requires a path",
                    )
                })?));
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--host=") {
                    if value.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`--host` requires a host name",
                        ));
                    }
                    options.host_name = Some(value.to_string());
                } else if let Some(value) = arg.strip_prefix("--addr=") {
                    if value.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`--addr` requires an address",
                        ));
                    }
                    options.addr = value.to_string();
                } else if let Some(value) = arg.strip_prefix("--load=") {
                    if value.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`--load` requires a path",
                        ));
                    }
                    options.load_path = Some(PathBuf::from(value));
                } else if let Some(value) = arg.strip_prefix("--save=") {
                    if value.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`--save` requires a path",
                        ));
                    }
                    options.save_path = Some(PathBuf::from(value));
                } else if let Some(value) = arg.strip_prefix("--transcript=") {
                    if value.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`--transcript` requires a path",
                        ));
                    }
                    options.transcript_path = Some(PathBuf::from(value));
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Unknown MUDDLE window argument `{arg}`."),
                    ));
                }
            }
        }
    }

    Ok(options)
}

fn handle_connection(
    mut stream: TcpStream,
    registrations: &[MuddleWindowHostRegistration],
    state: &mut MuddleWindowState,
) -> io::Result<()> {
    let mut buffer = [0_u8; 65_536];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let (method, path) = request_line(&request);
    match (method, path) {
        ("GET", "/") => write_response(&mut stream, "200 OK", "text/html", WINDOW_HTML),
        ("GET", "/hosts") => write_response(
            &mut stream,
            "200 OK",
            "application/json",
            &render_hosts_json(registrations),
        ),
        ("GET", "/state") => write_response(
            &mut stream,
            "200 OK",
            "application/json",
            &render_state_json(state)?,
        ),
        ("GET", "/transcript") => write_response(
            &mut stream,
            "200 OK",
            "text/plain",
            &render_window_transcript(state),
        ),
        ("GET", "/save-slots") => write_response(
            &mut stream,
            "200 OK",
            "application/json",
            &render_save_slots_json(state)?,
        ),
        ("GET", "/save-slot-details") => write_response(
            &mut stream,
            "200 OK",
            "application/json",
            &render_save_slot_details_json(state)?,
        ),
        ("GET", "/export-save") => write_response(
            &mut stream,
            "200 OK",
            "text/plain",
            &render_save_export(state),
        ),
        ("POST", "/export-slot") => {
            let slot_name = request_body(&request).trim();
            match export_save_slot_text(state, slot_name)? {
                Ok(exported_save) => {
                    write_response(&mut stream, "200 OK", "text/plain", &exported_save)
                }
                Err(message) => {
                    write_response(&mut stream, "400 Bad Request", "text/plain", &message)
                }
            }
        }
        ("POST", "/select-host") => {
            let host_name = request_body(&request).trim();
            if let Some(registration) = find_window_host(registrations, host_name) {
                reset_state_for_registration(state, registration)?;
                persist_state(state)?;
                write_response(
                    &mut stream,
                    "200 OK",
                    "application/json",
                    &render_state_json(state)?,
                )
            } else {
                write_response(
                    &mut stream,
                    "400 Bad Request",
                    "application/json",
                    &format!(
                        "{{\"error\":\"Unknown MUDDLE window host `{}`.\"}}",
                        json_escape(host_name)
                    ),
                )
            }
        }
        ("POST", "/reset") => {
            let registration = state.registration;
            reset_state_for_registration(state, registration)?;
            state.last_response =
                format!("Restarted MUDDLE window host {}.", state.registration.name);
            persist_state(state)?;
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("POST", "/save") => {
            save_state_now(state)?;
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("POST", "/load-save") => {
            reload_state_from_save_path(state)?;
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("POST", "/save-slot") => {
            let slot_name = request_body(&request).trim();
            save_state_to_slot(state, slot_name)?;
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("POST", "/load-slot") => {
            let slot_name = request_body(&request).trim();
            load_state_from_slot(state, slot_name)?;
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("POST", "/delete-slot") => {
            let slot_name = request_body(&request).trim();
            delete_save_slot(state, slot_name)?;
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("POST", "/import-save") => {
            let encoded_save = request_body(&request);
            import_state_from_text(state, encoded_save)?;
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("POST", "/command") => {
            let command_text = request_body(&request).trim();
            if command_text.is_empty() {
                state.last_response = "Enter a command before sending.".to_string();
            } else {
                match state
                    .session
                    .play_turn(state.host.as_mut(), MuddleCommand::parse(command_text))
                {
                    Ok(turn) => state.last_response = turn.response.clone(),
                    Err(error) => state.last_response = format!("Command failed: {error:?}"),
                }
                persist_state(state)?;
            }
            write_response(
                &mut stream,
                "200 OK",
                "application/json",
                &render_state_json(state)?,
            )
        }
        ("GET", "/favicon.ico") => write_response(&mut stream, "204 No Content", "text/plain", ""),
        _ => write_response(&mut stream, "404 Not Found", "text/plain", "not found"),
    }
}

fn reset_state_for_registration(
    state: &mut MuddleWindowState,
    registration: MuddleWindowHostRegistration,
) -> io::Result<()> {
    *state = MuddleWindowState::new(
        registration,
        None,
        state.save_path.clone(),
        state.transcript_path.clone(),
    )?;
    Ok(())
}

fn save_state_now(state: &mut MuddleWindowState) -> io::Result<()> {
    if state.save_path.is_none() && state.transcript_path.is_none() {
        state.last_response =
            "Start muddle-window with --save or --transcript before using Save now.".to_string();
        return Ok(());
    }

    persist_state(state)?;
    state.last_response = match (&state.save_path, &state.transcript_path) {
        (Some(save_path), Some(transcript_path)) => format!(
            "Saved session to {} and transcript to {}.",
            save_path.display(),
            transcript_path.display()
        ),
        (Some(save_path), None) => format!("Saved session to {}.", save_path.display()),
        (None, Some(transcript_path)) => {
            format!("Saved transcript to {}.", transcript_path.display())
        }
        (None, None) => unreachable!("checked above"),
    };
    Ok(())
}

fn reload_state_from_save_path(state: &mut MuddleWindowState) -> io::Result<()> {
    let Some(save_path) = state.save_path.clone() else {
        state.last_response =
            "Start muddle-window with --save before using Reload save.".to_string();
        return Ok(());
    };
    if !save_path.exists() {
        state.last_response = format!("No save file found at {}.", save_path.display());
        return Ok(());
    }

    *state = MuddleWindowState::new(
        state.registration,
        Some(save_path.clone()),
        Some(save_path.clone()),
        state.transcript_path.clone(),
    )?;
    state.last_response = format!("Reloaded save from {}.", save_path.display());
    persist_state(state)?;
    Ok(())
}

fn save_state_to_slot(state: &mut MuddleWindowState, slot_name: &str) -> io::Result<()> {
    let Some((slot_name, slot_path)) =
        save_slot_path(&state.save_path, slot_name, &mut state.last_response)
    else {
        return Ok(());
    };

    fs::write(
        &slot_path,
        state.session.save_for_host(state.host.as_ref()).encode(),
    )?;
    state.last_response = format!(
        "Saved session slot `{slot_name}` to {}.",
        slot_path.display()
    );
    Ok(())
}

fn load_state_from_slot(state: &mut MuddleWindowState, slot_name: &str) -> io::Result<()> {
    let Some((slot_name, slot_path)) =
        save_slot_path(&state.save_path, slot_name, &mut state.last_response)
    else {
        return Ok(());
    };
    if !slot_path.exists() {
        state.last_response = format!("No save slot found at {}.", slot_path.display());
        return Ok(());
    }

    *state = MuddleWindowState::new(
        state.registration,
        Some(slot_path.clone()),
        state.save_path.clone(),
        state.transcript_path.clone(),
    )?;
    state.last_response = format!(
        "Loaded save slot `{slot_name}` from {}.",
        slot_path.display()
    );
    persist_state(state)?;
    Ok(())
}

fn delete_save_slot(state: &mut MuddleWindowState, slot_name: &str) -> io::Result<()> {
    let Some((slot_name, slot_path)) =
        save_slot_path(&state.save_path, slot_name, &mut state.last_response)
    else {
        return Ok(());
    };
    if !slot_path.exists() {
        state.last_response = format!("No save slot found at {}.", slot_path.display());
        return Ok(());
    }

    fs::remove_file(&slot_path)?;
    state.last_response = format!(
        "Deleted save slot `{slot_name}` from {}.",
        slot_path.display()
    );
    Ok(())
}

fn render_save_export(state: &MuddleWindowState) -> String {
    state.session.save_for_host(state.host.as_ref()).encode()
}

fn export_save_slot_text(
    state: &mut MuddleWindowState,
    slot_name: &str,
) -> io::Result<Result<String, String>> {
    let Some((slot_name, slot_path)) =
        save_slot_path(&state.save_path, slot_name, &mut state.last_response)
    else {
        return Ok(Err(state.last_response.clone()));
    };
    if !slot_path.exists() {
        state.last_response = format!("No save slot found at {}.", slot_path.display());
        return Ok(Err(state.last_response.clone()));
    }

    let exported_save = fs::read_to_string(&slot_path)?;
    state.last_response = format!(
        "Exported save slot `{slot_name}` from {}.",
        slot_path.display()
    );
    Ok(Ok(exported_save))
}

fn import_state_from_text(state: &mut MuddleWindowState, encoded_save: &str) -> io::Result<()> {
    let encoded_save = encoded_save.trim();
    if encoded_save.is_empty() {
        state.last_response = "Paste exported save text before importing.".to_string();
        return Ok(());
    }

    let save = match MuddleSessionSave::decode(encoded_save) {
        Ok(save) => save,
        Err(error) => {
            state.last_response = format!("Import failed: {error:?}");
            return Ok(());
        }
    };
    let mut host = (state.registration.create)();
    let session = match MuddleSession::resume_for_host(host.as_mut(), &save) {
        Ok(session) => session,
        Err(error) => {
            state.last_response = format!("Import failed: {error:?}");
            return Ok(());
        }
    };

    state.host = host;
    state.session = session;
    state.last_response = "Imported save text into the current host.".to_string();
    persist_state(state)?;
    Ok(())
}

impl MuddleWindowState {
    fn new(
        registration: MuddleWindowHostRegistration,
        load_path: Option<PathBuf>,
        save_path: Option<PathBuf>,
        transcript_path: Option<PathBuf>,
    ) -> io::Result<Self> {
        let mut host = (registration.create)();
        let (session, last_response) = if let Some(path) = load_path {
            let encoded = fs::read_to_string(&path)?;
            let save = MuddleSessionSave::decode(&encoded).map_err(|error| {
                io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}"))
            })?;
            let session =
                MuddleSession::resume_for_host(host.as_mut(), &save).map_err(|error| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}"))
                })?;
            (
                session,
                format!("Loaded MUDDLE window session from {}.", path.display()),
            )
        } else {
            let session = MuddleSession::for_host(host.as_ref()).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("registered window host cannot start: {error:?}"),
                )
            })?;
            (session, "Window session ready.".to_string())
        };

        Ok(Self {
            host,
            session,
            registration,
            last_response,
            save_path,
            transcript_path,
        })
    }
}

fn persist_state(state: &MuddleWindowState) -> io::Result<()> {
    if let Some(path) = &state.save_path {
        fs::write(
            path,
            state.session.save_for_host(state.host.as_ref()).encode(),
        )?;
    }
    if let Some(path) = &state.transcript_path {
        fs::write(
            path,
            render_transcript(
                MuddleCliHostInfo {
                    name: state.registration.name,
                    description: state.registration.description,
                    suggested_commands: state.registration.suggested_commands,
                },
                &state.session,
            ),
        )?;
    }
    Ok(())
}

fn save_slot_path(
    save_path: &Option<PathBuf>,
    slot_name: &str,
    last_response: &mut String,
) -> Option<(String, PathBuf)> {
    let Some(save_path) = save_path else {
        *last_response = "Start muddle-window with --save before using save slots.".to_string();
        return None;
    };
    let slot_name = match normalize_save_slot_name(slot_name) {
        Ok(slot_name) => slot_name,
        Err(message) => {
            *last_response = message;
            return None;
        }
    };

    let parent = save_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = save_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("muddle-window");
    let extension = save_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    let slot_path = parent.join(format!("{stem}.slot-{slot_name}{extension}"));
    Some((slot_name, slot_path))
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

fn list_save_slots(state: &MuddleWindowState) -> io::Result<Vec<String>> {
    Ok(list_save_slot_details(state)?
        .into_iter()
        .map(|slot| slot.name)
        .collect())
}

fn list_save_slot_details(state: &MuddleWindowState) -> io::Result<Vec<SaveSlotDetail>> {
    let Some(save_path) = &state.save_path else {
        return Ok(Vec::new());
    };
    let parent = save_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = save_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("muddle-window");
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
            slots.push(SaveSlotDetail {
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

fn find_window_host(
    registrations: &[MuddleWindowHostRegistration],
    name: &str,
) -> Option<MuddleWindowHostRegistration> {
    registrations
        .iter()
        .copied()
        .find(|registration| registration.name == name)
}

fn print_window_hosts(registrations: &[MuddleWindowHostRegistration]) {
    println!("Available MUDDLE window hosts:");
    for registration in registrations {
        println!(
            "  {} [{}] - {}",
            registration.name, registration.category, registration.description
        );
    }
}

fn render_hosts_json(registrations: &[MuddleWindowHostRegistration]) -> String {
    let hosts = registrations
        .iter()
        .map(|registration| {
            format!(
                "{{\"name\":\"{}\",\"category\":\"{}\",\"description\":\"{}\",\"suggested\":\"{}\"}}",
                json_escape(registration.name),
                json_escape(registration.category),
                json_escape(registration.description),
                json_escape(registration.suggested_commands)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{hosts}]")
}

fn render_state_json(state: &MuddleWindowState) -> io::Result<String> {
    let mut panels = Vec::new();
    write_play_panels(&mut panels, state.host.as_ref(), &state.session)?;
    let panels = String::from_utf8(panels)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let room_card = state
        .host
        .room(&state.session.current_room)
        .map(|room| room.ascii_card())
        .unwrap_or_else(|| format!("Room missing: {}", state.session.current_room));
    let commands = render_commands_json(state);
    let history = render_history_json(state);
    let save_slots = render_save_slots_json(state)?;
    let save_slot_details = render_save_slot_details_json(state)?;

    Ok(format!(
        "{{\"host\":\"{}\",\"description\":\"{}\",\"suggested\":\"{}\",\"room\":\"{}\",\"turns\":{},\"panels\":\"{}\",\"room_card\":\"{}\",\"last_response\":\"{}\",\"save_path\":\"{}\",\"transcript_path\":\"{}\",\"save_slots\":{save_slots},\"save_slot_details\":{save_slot_details},\"commands\":{commands},\"history\":{history}}}",
        json_escape(state.registration.name),
        json_escape(state.registration.description),
        json_escape(state.registration.suggested_commands),
        json_escape(&state.session.current_room),
        state.session.transcript.len(),
        json_escape(&panels),
        json_escape(&room_card),
        json_escape(&state.last_response),
        json_escape(&display_path(&state.save_path)),
        json_escape(&display_path(&state.transcript_path))
    ))
}

fn render_save_slots_json(state: &MuddleWindowState) -> io::Result<String> {
    let slots = list_save_slots(state)?
        .iter()
        .map(|slot| format!("\"{}\"", json_escape(slot)))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("[{slots}]"))
}

fn render_save_slot_details_json(state: &MuddleWindowState) -> io::Result<String> {
    let slots = list_save_slot_details(state)?
        .iter()
        .map(|slot| {
            format!(
                "{{\"name\":\"{}\",\"path\":\"{}\",\"bytes\":{},\"modified_unix\":{}}}",
                json_escape(&slot.name),
                json_escape(&slot.path.display().to_string()),
                slot.bytes,
                slot.modified_unix
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("[{slots}]"))
}

fn render_commands_json(state: &MuddleWindowState) -> String {
    let commands = state
        .host
        .command_panel(&state.session.current_room)
        .iter()
        .map(|hint| {
            format!(
                "{{\"command\":\"{}\",\"description\":\"{}\"}}",
                json_escape(&hint.command),
                json_escape(&hint.description)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{commands}]")
}

fn render_history_json(state: &MuddleWindowState) -> String {
    let turns = state
        .session
        .transcript
        .iter()
        .enumerate()
        .map(|(index, turn)| {
            format!(
                "{{\"turn\":{},\"room\":\"{}\",\"command\":\"{}\",\"response\":\"{}\"}}",
                index + 1,
                json_escape(&turn.room_id),
                json_escape(&turn.command.normalized()),
                json_escape(&turn.response)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{turns}]")
}

fn render_window_transcript(state: &MuddleWindowState) -> String {
    render_transcript(
        MuddleCliHostInfo {
            name: state.registration.name,
            description: state.registration.description,
            suggested_commands: state.registration.suggested_commands,
        },
        &state.session,
    )
}

fn display_path(path: &Option<PathBuf>) -> String {
    path.as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_default()
}

fn request_line(request: &str) -> (&str, &str) {
    let mut parts = request
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();
    (
        parts.next().unwrap_or_default(),
        parts.next().unwrap_or_default(),
    )
}

fn request_body(request: &str) -> &str {
    request.split("\r\n\r\n").nth(1).unwrap_or_default()
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn open_browser(url: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", "", url]).spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn()?;
        return Ok(());
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

const WINDOW_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>MUDDLE Window</title>
  <style>
    body { margin: 0; background: #101418; color: #e8edf2; font-family: Consolas, "Cascadia Mono", monospace; }
    main { display: grid; grid-template-columns: minmax(18rem, 28rem) 1fr; gap: 1rem; padding: 1rem; min-height: 100vh; box-sizing: border-box; }
    section { background: #171d24; border: 1px solid #2d3742; border-radius: 10px; padding: 1rem; }
    h1, h2 { margin-top: 0; color: #a7d3ff; }
    pre { white-space: pre-wrap; line-height: 1.35; }
    input { width: 100%; box-sizing: border-box; padding: .75rem; background: #0f1318; color: #fff; border: 1px solid #415061; border-radius: 8px; font: inherit; }
    select { width: 100%; box-sizing: border-box; margin-top: .75rem; padding: .75rem; background: #0f1318; color: #fff; border: 1px solid #415061; border-radius: 8px; font: inherit; }
    textarea { width: 100%; box-sizing: border-box; padding: .75rem; background: #0f1318; color: #fff; border: 1px solid #415061; border-radius: 8px; font: inherit; }
    button { margin-top: .75rem; padding: .65rem 1rem; background: #316dca; color: #fff; border: 0; border-radius: 8px; font: inherit; cursor: pointer; }
    button.secondary { background: #263241; color: #dbe6f2; }
    button.host-card { display: block; width: 100%; margin: .75rem 0; text-align: left; background: #1d2936; border: 1px solid #42566b; }
    button.host-card strong { display: block; color: #fff; margin-bottom: .25rem; }
    button.command-button { margin: .35rem .35rem 0 0; background: #244b32; }
    button.command-button span { display: block; color: #c4d2c8; font-size: .8rem; margin-top: .2rem; }
    ol.history { padding-left: 1.25rem; }
    ol.history li { margin: .75rem 0; }
    ol.history pre { background: #0f1318; border: 1px solid #263241; border-radius: 8px; padding: .75rem; }
    #command-form { position: sticky; bottom: 1rem; background: #171d24; border: 1px solid #2d3742; border-radius: 10px; margin-top: 1rem; padding: .75rem; }
    #command-form button { width: 100%; }
    #chooser { max-width: 56rem; margin: 0 auto; padding: 1rem; }
    #client { display: none; }
    .category-heading { margin: 1.25rem 0 .35rem; color: #d0e8ff; }
    .empty-hosts { border: 1px dashed #42566b; border-radius: 8px; padding: 1rem; }
    .slot-details { list-style: none; padding-left: 0; }
    .slot-details li { background: #0f1318; border: 1px solid #263241; border-radius: 8px; margin: .5rem 0; padding: .5rem; }
    .slot-copy { margin-top: .5rem; padding: .4rem .65rem; background: #1d3f5c; font-size: .85rem; }
    .path-copy { margin-right: .5rem; padding: .4rem .65rem; background: #1d3f5c; font-size: .85rem; }
    .window-status { display: none; background: #3b2630; border: 1px solid #7f4b5a; border-radius: 8px; color: #ffdce5; padding: .75rem; }
    .muted { color: #9aa7b2; }
    .response { color: #d8f8b7; }
    @media (max-width: 900px) {
      main { grid-template-columns: 1fr; padding: .5rem; }
      section { padding: .75rem; }
      button { width: 100%; }
      button.command-button { width: 100%; }
      #chooser { padding: .5rem; }
    }
  </style>
</head>
<body>
  <main id="chooser">
    <section>
      <h1>Choose a MUDDLE host</h1>
      <p class="muted">Pick the game surface to mount in this local window. You can switch later, which starts a fresh session for that host.</p>
      <p id="chooser-status" class="window-status"></p>
      <input id="host-filter" autocomplete="off" placeholder="filter hosts, e.g. game, banish, knowledge">
      <div id="host-list"></div>
    </section>
  </main>
  <main id="client">
    <section>
      <h1>MUDDLE Window</h1>
      <p id="window-status" class="window-status"></p>
      <p id="host" class="muted"></p>
      <p id="suggested"></p>
      <button id="change-host" class="secondary" type="button">Change host</button>
      <button id="reset-host" class="secondary" type="button">Restart host</button>
      <button id="save-now" class="secondary" type="button">Save now</button>
      <button id="load-save" class="secondary" type="button">Reload save</button>
      <p id="persistence" class="muted"></p>
      <div id="persistence-actions"></div>
      <p class="muted">Shortcuts: Ctrl+S save, Ctrl+R reload, Ctrl+E export, Ctrl+I import.</p>
      <h2>Save slots</h2>
      <input id="save-slot-name" autocomplete="off" placeholder="slot name, e.g. before-boss">
      <button id="save-slot" class="secondary" type="button">Save slot</button>
      <input id="save-slot-filter" autocomplete="off" placeholder="filter saved slots by name or path">
      <p id="slot-filter-summary" class="muted"></p>
      <select id="save-slot-list"></select>
      <button id="load-slot" class="secondary" type="button">Load slot</button>
      <button id="export-slot" class="secondary" type="button">Export slot text</button>
      <button id="delete-slot" class="secondary" type="button">Delete slot</button>
      <p id="slot-selection" class="muted">Select a slot to load/export/delete, or type a new name to save.</p>
      <ul id="save-slot-details" class="slot-details"></ul>
      <h2>Import / export</h2>
      <textarea id="save-export" rows="8" placeholder="exported save text"></textarea>
      <button id="export-save" class="secondary" type="button">Export save text</button>
      <button id="import-save" class="secondary" type="button">Import save text</button>
      <h2>Panels</h2>
      <pre id="panels"></pre>
    </section>
    <section>
      <h2 id="room"></h2>
      <pre id="card"></pre>
      <h2>Actions</h2>
      <div id="command-buttons"></div>
      <h2>Last response</h2>
      <pre id="response" class="response"></pre>
      <h2>History</h2>
      <p><a id="transcript-link" class="muted" href="/transcript" target="_blank" rel="noreferrer">Open full transcript</a></p>
      <ol id="history" class="history"></ol>
      <form id="command-form">
        <input id="command" autocomplete="off" autofocus placeholder="type a command, e.g. look">
        <button type="submit">Send command</button>
      </form>
    </section>
  </main>
  <script>
    let selectedHost = null;
    let availableHosts = [];
    let commandRecall = [];
    let commandRecallIndex = 0;
    let commandDraft = '';
    let currentState = null;
    let currentSlotDetails = [];

    async function requestJson(path, options = {}) {
      try {
        const response = await fetch(path, options);
        if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
        const body = await response.json();
        showWindowStatus('');
        return body;
      } catch (error) {
        showWindowStatus(`Request failed: ${error.message}`);
        throw error;
      }
    }

    async function requestText(path, options = {}) {
      try {
        const response = await fetch(path, options);
        if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
        const body = await response.text();
        showWindowStatus('');
        return body;
      } catch (error) {
        showWindowStatus(`Request failed: ${error.message}`);
        throw error;
      }
    }

    function showWindowStatus(message) {
      for (const id of ['chooser-status', 'window-status']) {
        const element = document.getElementById(id);
        if (!element) continue;
        element.textContent = message;
        element.style.display = message ? 'block' : 'none';
      }
    }

    async function loadHosts() {
      availableHosts = await requestJson('/hosts');
      renderHosts();
    }

    function renderHosts() {
      const filter = document.getElementById('host-filter').value.trim().toLowerCase();
      const list = document.getElementById('host-list');
      list.innerHTML = '';
      const hosts = availableHosts.filter(host => {
        const haystack = `${host.name} ${host.category} ${host.description} ${host.suggested}`.toLowerCase();
        return !filter || haystack.includes(filter);
      });
      const groups = new Map();
      for (const host of hosts) {
        if (!groups.has(host.category)) groups.set(host.category, []);
        groups.get(host.category).push(host);
      }
      if (!hosts.length) {
        const empty = document.createElement('p');
        empty.className = 'muted empty-hosts';
        empty.textContent = 'No hosts match that filter.';
        list.appendChild(empty);
        return;
      }
      for (const [category, categoryHosts] of groups) {
        const heading = document.createElement('h2');
        heading.className = 'category-heading';
        heading.textContent = category;
        list.appendChild(heading);
        for (const host of categoryHosts) {
          const button = document.createElement('button');
          button.className = 'host-card';
          button.type = 'button';
          const name = document.createElement('strong');
          name.textContent = host.name;
          const description = document.createElement('span');
          description.textContent = host.description;
          const suggested = document.createElement('span');
          suggested.className = 'muted';
          suggested.textContent = `Try: ${host.suggested}`;
          button.append(name, description, document.createElement('br'), suggested);
          button.addEventListener('click', () => selectHost(host.name));
          list.appendChild(button);
        }
      }
    }

    async function selectHost(hostName) {
      selectedHost = hostName;
      const state = await requestJson('/select-host', { method: 'POST', body: hostName });
      document.getElementById('chooser').style.display = 'none';
      document.getElementById('client').style.display = 'grid';
      renderState(state);
      document.getElementById('command').focus();
    }

    function showChooser() {
      selectedHost = null;
      document.getElementById('client').style.display = 'none';
      document.getElementById('chooser').style.display = 'block';
    }

    function renderState(state) {
      currentState = state;
      document.title = `MUDDLE - ${state.host}`;
      document.getElementById('host').textContent = `${state.host}: ${state.description}`;
      document.getElementById('suggested').textContent = `Try: ${state.suggested}`;
      document.getElementById('room').textContent = `${state.room} (${state.turns} turns)`;
      document.getElementById('panels').textContent = state.panels || '(no panels)';
      document.getElementById('card').textContent = state.room_card;
      document.getElementById('response').textContent = state.last_response;
      renderCommandButtons(state.commands || []);
      renderHistory(state.history || []);
      currentSlotDetails = state.save_slot_details || [];
      renderSaveSlots(currentSlotDetails);
      renderPersistenceActions(state);
      updatePersistenceControlState(state);
      const persistence = [];
      if (state.save_path) persistence.push(`save: ${state.save_path}`);
      if (state.transcript_path) persistence.push(`transcript: ${state.transcript_path}`);
      document.getElementById('persistence').textContent = persistence.join(' | ');
    }

    function renderPersistenceActions(state) {
      const actions = document.getElementById('persistence-actions');
      actions.innerHTML = '';
      if (state.save_path) {
        actions.appendChild(pathCopyButton('Copy save path', state.save_path, 'active save'));
      }
      if (state.transcript_path) {
        actions.appendChild(pathCopyButton('Copy transcript path', state.transcript_path, 'transcript'));
      }
    }

    function pathCopyButton(label, path, name) {
      const button = document.createElement('button');
      button.className = 'path-copy';
      button.type = 'button';
      button.textContent = label;
      button.addEventListener('click', () => copyText(path, `${name} path`));
      return button;
    }

    function renderSaveSlots(slotDetails) {
      const list = document.getElementById('save-slot-list');
      const details = document.getElementById('save-slot-details');
      const input = document.getElementById('save-slot-name');
      const filter = document.getElementById('save-slot-filter').value.trim().toLowerCase();
      const filteredSlotDetails = filter
        ? slotDetails.filter(slot => slotMatchesFilter(slot, filter))
        : slotDetails;
      const selected = list.value || input.value.trim();
      list.innerHTML = '';
      details.innerHTML = '';
      updateSlotFilterSummary(filteredSlotDetails.length, slotDetails.length, filter);
      if (!slotDetails.length) {
        const option = document.createElement('option');
        option.value = '';
        option.textContent = 'No save slots yet';
        list.appendChild(option);
        const item = document.createElement('li');
        item.className = 'muted';
        item.textContent = 'No save slots yet.';
        details.appendChild(item);
        updateSlotSelectionStatus('');
        updatePersistenceControlState();
        return;
      }
      if (!filteredSlotDetails.length) {
        const option = document.createElement('option');
        option.value = '';
        option.textContent = 'No matching save slots';
        list.appendChild(option);
        const item = document.createElement('li');
        item.className = 'muted';
        item.textContent = 'No save slots match the current filter.';
        details.appendChild(item);
        if (input.value.trim()) {
          updateDraftSlotStatus();
        } else {
          updateSlotSelectionStatus('');
        }
        updatePersistenceControlState();
        return;
      }
      for (const slot of filteredSlotDetails) {
        const option = document.createElement('option');
        option.value = slot.name;
        option.textContent = slot.name;
        list.appendChild(option);

        const item = document.createElement('li');
        const title = document.createElement('strong');
        title.textContent = slot.name;
        const meta = document.createElement('div');
        meta.className = 'muted';
        const modified = slot.modified_unix ? new Date(slot.modified_unix * 1000).toLocaleString() : 'unknown time';
        meta.textContent = `${slot.bytes} bytes | ${modified} | ${slot.path}`;
        const useSlot = document.createElement('button');
        useSlot.className = 'slot-copy';
        useSlot.type = 'button';
        useSlot.textContent = 'Use slot';
        useSlot.addEventListener('click', () => selectSaveSlot(slot.name));
        const copyPath = document.createElement('button');
        copyPath.className = 'slot-copy';
        copyPath.type = 'button';
        copyPath.textContent = 'Copy path';
        copyPath.addEventListener('click', () => copySlotPath(slot));
        item.append(title, meta, useSlot, copyPath);
        details.appendChild(item);
      }
      if (filteredSlotDetails.some(slot => slot.name === selected)) {
        list.value = selected;
        selectSaveSlot(list.value);
      } else if (input.value.trim()) {
        updateDraftSlotStatus();
      } else {
        selectSaveSlot(list.value);
      }
    }

    function slotMatchesFilter(slot, filter) {
      return slot.name.toLowerCase().includes(filter) || slot.path.toLowerCase().includes(filter);
    }

    function updateSlotFilterSummary(showing, total, filter) {
      document.getElementById('slot-filter-summary').textContent = filter
        ? `Showing ${showing} of ${total} save slots.`
        : `${total} save slots.`;
    }

    function refreshSlotFilter() {
      renderSaveSlots(currentSlotDetails);
    }

    function syncSelectedSlotName() {
      selectSaveSlot(document.getElementById('save-slot-list').value);
    }

    function updateDraftSlotStatus() {
      const slotName = document.getElementById('save-slot-name').value.trim();
      document.getElementById('slot-selection').textContent = slotName
        ? `Typed slot: ${slotName}. Save slot will create or overwrite this name.`
        : 'Select a slot to load/export/delete, or type a new name to save.';
      updatePersistenceControlState();
    }

    function selectSaveSlot(slotName) {
      const list = document.getElementById('save-slot-list');
      if ([...list.options].some(option => option.value === slotName)) {
        list.value = slotName;
      }
      document.getElementById('save-slot-name').value = slotName;
      updateSlotSelectionStatus(slotName);
      updatePersistenceControlState();
    }

    function updateSlotSelectionStatus(slotName) {
      if (currentState && !currentState.save_path) {
        document.getElementById('slot-selection').textContent = 'Save slots require a configured --save path.';
        return;
      }
      document.getElementById('slot-selection').textContent = slotName
        ? `Selected slot: ${slotName}. Load, export, or delete will use this slot.`
        : 'Select a slot to load/export/delete, or type a new name to save.';
    }

    function updatePersistenceControlState(state = currentState) {
      if (!state) return;
      const hasSavePath = Boolean(state.save_path);
      const hasAnyPersistencePath = hasSavePath || Boolean(state.transcript_path);
      const slotName = currentSlotName();
      const hasSlotTarget = Boolean(slotName);
      const hasExistingSlotTarget = [...document.getElementById('save-slot-list').options]
        .some(option => option.value && option.value === slotName);
      setButtonDisabled('save-now', !hasAnyPersistencePath, 'Start with --save or --transcript to enable Save now.');
      setButtonDisabled('load-save', !hasSavePath, 'Start with --save to enable Reload save.');
      document.getElementById('save-slot-name').disabled = !hasSavePath;
      document.getElementById('save-slot-filter').disabled = !hasSavePath;
      document.getElementById('save-slot-list').disabled = !hasSavePath;
      setButtonDisabled('save-slot', !(hasSavePath && hasSlotTarget), 'Start with --save and enter a slot name.');
      setButtonDisabled('load-slot', !(hasSavePath && hasExistingSlotTarget), 'Select an existing save slot to load.');
      setButtonDisabled('export-slot', !(hasSavePath && hasExistingSlotTarget), 'Select an existing save slot to export.');
      setButtonDisabled('delete-slot', !(hasSavePath && hasExistingSlotTarget), 'Select an existing save slot to delete.');
    }

    function setButtonDisabled(id, disabled, title) {
      const button = document.getElementById(id);
      button.disabled = disabled;
      button.title = disabled ? title : '';
    }

    async function copySlotPath(slot) {
      await copyText(slot.path, `save-slot path for ${slot.name}`);
    }

    async function copyText(text, label) {
      try {
        await navigator.clipboard.writeText(text);
        showWindowStatus(`Copied ${label}.`);
      } catch (error) {
        showWindowStatus(`Copy failed: ${error.message}`);
      }
    }

    function currentSlotName() {
      const input = document.getElementById('save-slot-name').value.trim();
      return input || document.getElementById('save-slot-list').value;
    }

    function persistenceTargetSummary(state) {
      const targets = [];
      if (state.save_path) targets.push(`save ${state.save_path}`);
      if (state.transcript_path) targets.push(`transcript ${state.transcript_path}`);
      return targets.length ? targets.join(' and ') : 'configured persistence outputs';
    }

    function renderCommandButtons(commands) {
      const container = document.getElementById('command-buttons');
      container.innerHTML = '';
      for (const hint of commands) {
        const button = document.createElement('button');
        button.className = 'command-button';
        button.type = 'button';
        const command = document.createElement('strong');
        command.textContent = hint.command;
        const description = document.createElement('span');
        description.textContent = hint.description;
        button.append(command, description);
        button.addEventListener('click', () => sendCommand(hint.command));
        container.appendChild(button);
      }
    }

    function renderHistory(history) {
      const container = document.getElementById('history');
      container.innerHTML = '';
      for (const turn of history.slice().reverse()) {
        const item = document.createElement('li');
        const title = document.createElement('strong');
        title.textContent = `Turn ${turn.turn} · ${turn.room} · ${turn.command}`;
        const response = document.createElement('pre');
        response.textContent = turn.response;
        item.append(title, response);
        container.appendChild(item);
      }
      if (!history.length) {
        const item = document.createElement('li');
        item.className = 'muted';
        item.textContent = 'No commands yet.';
        container.appendChild(item);
      }
    }

    async function sendCommand(command) {
      rememberCommand(command);
      const state = await requestJson('/command', { method: 'POST', body: command });
      renderState(state);
      document.getElementById('command').focus();
    }

    function rememberCommand(command) {
      if (!command) return;
      if (commandRecall[commandRecall.length - 1] !== command) commandRecall.push(command);
      commandRecallIndex = commandRecall.length;
      commandDraft = '';
    }

    function recallCommand(event) {
      if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') return;
      if (!commandRecall.length) return;
      event.preventDefault();
      const input = event.currentTarget;
      if (event.key === 'ArrowUp') {
        if (commandRecallIndex === commandRecall.length) commandDraft = input.value;
        commandRecallIndex = Math.max(0, commandRecallIndex - 1);
        input.value = commandRecall[commandRecallIndex];
      } else {
        commandRecallIndex = Math.min(commandRecall.length, commandRecallIndex + 1);
        input.value = commandRecallIndex === commandRecall.length
          ? commandDraft
          : commandRecall[commandRecallIndex];
      }
      input.setSelectionRange(input.value.length, input.value.length);
    }

    async function resetHost() {
      const state = await requestJson('/reset', { method: 'POST' });
      renderState(state);
      document.getElementById('command').focus();
    }

    async function saveNow() {
      if (document.getElementById('save-now').disabled) return;
      const state = await requestJson('/save', { method: 'POST' });
      renderState(state);
      showWindowStatus(`Saved ${persistenceTargetSummary(state)}.`);
      document.getElementById('command').focus();
    }

    async function loadSave() {
      if (document.getElementById('load-save').disabled) return;
      const state = await requestJson('/load-save', { method: 'POST' });
      renderState(state);
      showWindowStatus(`Reloaded save ${state.save_path || 'from configured save path'}.`);
      document.getElementById('command').focus();
    }

    async function saveSlot() {
      if (document.getElementById('save-slot').disabled) return;
      const slotName = currentSlotName();
      const state = await requestJson('/save-slot', { method: 'POST', body: slotName });
      renderState(state);
      showWindowStatus(`Saved slot ${slotName}.`);
      document.getElementById('command').focus();
    }

    async function loadSlot() {
      if (document.getElementById('load-slot').disabled) return;
      const slotName = currentSlotName();
      const state = await requestJson('/load-slot', { method: 'POST', body: slotName });
      renderState(state);
      showWindowStatus(`Loaded slot ${slotName}.`);
      document.getElementById('command').focus();
    }

    async function deleteSlot() {
      if (document.getElementById('delete-slot').disabled) return;
      const slotName = currentSlotName();
      const state = await requestJson('/delete-slot', { method: 'POST', body: slotName });
      renderState(state);
      showWindowStatus(`Deleted slot ${slotName}.`);
      document.getElementById('command').focus();
    }

    async function exportSlotText() {
      if (document.getElementById('export-slot').disabled) return;
      const slotName = currentSlotName();
      const exported = await requestText('/export-slot', { method: 'POST', body: slotName });
      document.getElementById('save-export').value = exported;
      showWindowStatus(`Exported save text for slot ${slotName}.`);
      document.getElementById('save-export').focus();
    }

    async function exportSaveText() {
      const exported = await requestText('/export-save');
      document.getElementById('save-export').value = exported;
      showWindowStatus(`Exported current save text (${exported.length} bytes).`);
      document.getElementById('save-export').focus();
    }

    async function importSaveText() {
      const body = document.getElementById('save-export').value;
      const state = await requestJson('/import-save', { method: 'POST', body });
      renderState(state);
      showWindowStatus(`Imported save text (${body.length} bytes).`);
      document.getElementById('command').focus();
    }

    function handleWindowShortcut(event) {
      if (!(event.ctrlKey || event.metaKey) || event.altKey || event.shiftKey) return;
      if (!selectedHost) return;
      const key = event.key.toLowerCase();
      const shortcuts = {
        s: saveNow,
        r: loadSave,
        e: exportSaveText,
        i: importSaveText,
      };
      const action = shortcuts[key];
      if (!action) return;
      event.preventDefault();
      action();
    }

    document.getElementById('change-host').addEventListener('click', showChooser);
    document.getElementById('host-filter').addEventListener('input', renderHosts);
    document.getElementById('reset-host').addEventListener('click', resetHost);
    document.getElementById('save-now').addEventListener('click', saveNow);
    document.getElementById('load-save').addEventListener('click', loadSave);
    document.getElementById('save-slot').addEventListener('click', saveSlot);
    document.getElementById('save-slot-name').addEventListener('input', updateDraftSlotStatus);
    document.getElementById('save-slot-filter').addEventListener('input', refreshSlotFilter);
    document.getElementById('save-slot-list').addEventListener('change', syncSelectedSlotName);
    document.getElementById('load-slot').addEventListener('click', loadSlot);
    document.getElementById('export-slot').addEventListener('click', exportSlotText);
    document.getElementById('delete-slot').addEventListener('click', deleteSlot);
    document.getElementById('export-save').addEventListener('click', exportSaveText);
    document.getElementById('import-save').addEventListener('click', importSaveText);
    document.getElementById('command-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      if (!selectedHost) return;
      const input = document.getElementById('command');
      const command = input.value.trim();
      if (!command) return;
      input.value = '';
      await sendCommand(command);
    });
    document.getElementById('command').addEventListener('keydown', recallCommand);
    document.addEventListener('keydown', handleWindowShortcut);
    loadHosts();
  </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use muddle_core::{MuddleCommandHint, MuddleCommandOutcome, MuddleError, MuddleRoom};

    struct EmptyHost;

    impl MuddleHost for EmptyHost {
        fn start_room(&self) -> &str {
            "entry"
        }

        fn room(&self, room_id: &str) -> Option<&MuddleRoom> {
            match room_id {
                "entry" => Some(&ENTRY_ROOM),
                _ => None,
            }
        }

        fn handle_command(
            &mut self,
            room_id: &str,
            command: &MuddleCommand,
        ) -> Result<MuddleCommandOutcome, MuddleError> {
            if command.normalized() == "look" {
                return Ok(MuddleCommandOutcome::stay("The empty room is still here."));
            }
            Err(MuddleError::UnknownCommand {
                room_id: room_id.to_string(),
                command: command.clone(),
            })
        }

        fn command_panel(&self, _current_room: &str) -> Vec<MuddleCommandHint> {
            vec![MuddleCommandHint {
                command: "look".to_string(),
                description: "Show the empty room.".to_string(),
            }]
        }
    }

    static ENTRY_ROOM: MuddleRoom = MuddleRoom {
        id: String::new(),
        title: String::new(),
        description: String::new(),
        exits: Vec::new(),
    };

    fn registration() -> MuddleWindowHostRegistration {
        MuddleWindowHostRegistration {
            name: "empty",
            category: "Test",
            description: "Empty host",
            suggested_commands: "`look`.",
            create: || Box::new(EmptyHost),
        }
    }

    #[test]
    fn parses_window_options() {
        assert_eq!(
            parse_window_run_options(
                [
                    "--host",
                    "empty",
                    "--addr=127.0.0.1:4888",
                    "--open",
                    "--save",
                    "save.muddle",
                    "--transcript=transcript.txt"
                ]
                .into_iter()
                .map(String::from)
            )
            .expect("options parse"),
            MuddleWindowRunOptions {
                host_name: Some("empty".to_string()),
                addr: "127.0.0.1:4888".to_string(),
                open: true,
                list_hosts: false,
                load_path: None,
                save_path: Some("save.muddle".into()),
                transcript_path: Some("transcript.txt".into()),
            }
        );
    }

    #[test]
    fn renders_registered_hosts_json() {
        let hosts = render_hosts_json(&[registration()]);
        assert!(hosts.contains("\"name\":\"empty\""));
        assert!(hosts.contains("\"category\":\"Test\""));
    }

    #[test]
    fn renders_command_hints_in_state_json() {
        let state = MuddleWindowState::new(registration(), None, None, None).expect("state starts");
        let rendered = render_state_json(&state).expect("state renders");
        assert!(rendered.contains("\"commands\":["));
        assert!(rendered.contains("\"command\":\"look\""));
        assert!(rendered.contains("\"description\":\"Show the empty room.\""));
        assert!(rendered.contains("\"history\":[]"));
    }

    #[test]
    fn renders_history_and_transcript() {
        let mut state =
            MuddleWindowState::new(registration(), None, None, None).expect("state starts");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "First line.\nSecond line.");

        let rendered = render_state_json(&state).expect("state renders");
        assert!(rendered.contains("\"history\":["));
        assert!(rendered.contains("\"turn\":1"));
        assert!(rendered.contains("\"command\":\"look\""));
        assert!(rendered.contains("First line.\\nSecond line."));

        let transcript = render_window_transcript(&state);
        assert!(transcript.contains("MUDDLE_TRANSCRIPT_V1"));
        assert!(transcript.contains("## Turn 1"));
        assert!(transcript.contains("command: look"));
    }

    #[test]
    fn window_html_supports_command_recall_keys() {
        assert!(WINDOW_HTML.contains("ArrowUp"));
        assert!(WINDOW_HTML.contains("ArrowDown"));
        assert!(WINDOW_HTML.contains("recallCommand"));
    }

    #[test]
    fn window_html_supports_responsive_sticky_command_form() {
        assert!(WINDOW_HTML.contains("@media (max-width: 900px)"));
        assert!(WINDOW_HTML.contains("grid-template-columns: 1fr"));
        assert!(WINDOW_HTML.contains("#command-form { position: sticky"));
    }

    #[test]
    fn window_html_reports_request_errors() {
        assert!(WINDOW_HTML.contains("window-status"));
        assert!(WINDOW_HTML.contains("chooser-status"));
        assert!(WINDOW_HTML.contains("Request failed:"));
        assert!(WINDOW_HTML.contains("requestJson"));
    }

    #[test]
    fn window_html_supports_persistence_shortcuts() {
        assert!(WINDOW_HTML.contains("Ctrl+S save"));
        assert!(WINDOW_HTML.contains("handleWindowShortcut"));
        assert!(WINDOW_HTML.contains("s: saveNow"));
        assert!(WINDOW_HTML.contains("r: loadSave"));
        assert!(WINDOW_HTML.contains("e: exportSaveText"));
        assert!(WINDOW_HTML.contains("i: importSaveText"));
    }

    #[test]
    fn window_html_supports_save_slot_path_copy() {
        assert!(WINDOW_HTML.contains("Copy path"));
        assert!(WINDOW_HTML.contains("copySlotPath"));
        assert!(WINDOW_HTML.contains("navigator.clipboard.writeText(text)"));
    }

    #[test]
    fn window_html_supports_active_persistence_path_copy() {
        assert!(WINDOW_HTML.contains("persistence-actions"));
        assert!(WINDOW_HTML.contains("Copy save path"));
        assert!(WINDOW_HTML.contains("Copy transcript path"));
        assert!(WINDOW_HTML.contains("renderPersistenceActions"));
    }

    #[test]
    fn window_html_reports_persistence_action_success() {
        assert!(WINDOW_HTML.contains("persistenceTargetSummary"));
        assert!(WINDOW_HTML.contains("Saved ${persistenceTargetSummary(state)}."));
        assert!(WINDOW_HTML.contains("Reloaded save ${state.save_path"));
        assert!(WINDOW_HTML.contains("Saved slot ${slotName}."));
        assert!(WINDOW_HTML.contains("Loaded slot ${slotName}."));
        assert!(WINDOW_HTML.contains("Deleted slot ${slotName}."));
        assert!(WINDOW_HTML.contains("Exported current save text (${exported.length} bytes)."));
        assert!(WINDOW_HTML.contains("Imported save text (${body.length} bytes)."));
    }

    #[test]
    fn window_html_supports_selected_slot_export() {
        assert!(WINDOW_HTML.contains("Export slot text"));
        assert!(WINDOW_HTML.contains("exportSlotText"));
        assert!(WINDOW_HTML.contains("/export-slot"));
    }

    #[test]
    fn window_html_syncs_selected_save_slot_to_input() {
        assert!(WINDOW_HTML.contains("slot-selection"));
        assert!(WINDOW_HTML.contains("Use slot"));
        assert!(WINDOW_HTML.contains("syncSelectedSlotName"));
        assert!(WINDOW_HTML.contains("updateDraftSlotStatus"));
        assert!(WINDOW_HTML.contains("selectSaveSlot(list.value)"));
        assert!(WINDOW_HTML.contains("Load, export, or delete will use this slot."));
        assert!(WINDOW_HTML.contains("Save slot will create or overwrite this name."));
    }

    #[test]
    fn window_html_disables_unavailable_persistence_controls() {
        assert!(WINDOW_HTML.contains("updatePersistenceControlState"));
        assert!(WINDOW_HTML.contains("Start with --save to enable Reload save."));
        assert!(WINDOW_HTML.contains("Save slots require a configured --save path."));
        assert!(WINDOW_HTML.contains("Select an existing save slot to load."));
        assert!(WINDOW_HTML.contains("setButtonDisabled"));
    }

    #[test]
    fn window_html_filters_save_slots() {
        assert!(WINDOW_HTML.contains("save-slot-filter"));
        assert!(WINDOW_HTML.contains("slot-filter-summary"));
        assert!(WINDOW_HTML.contains("slotMatchesFilter"));
        assert!(WINDOW_HTML.contains("Showing ${showing} of ${total} save slots."));
        assert!(WINDOW_HTML.contains("No save slots match the current filter."));
    }

    #[test]
    fn reset_preserves_persistence_paths() {
        let mut state = MuddleWindowState::new(
            registration(),
            None,
            Some("save.muddle".into()),
            Some("transcript.txt".into()),
        )
        .expect("state starts");
        state.last_response = "changed".to_string();

        reset_state_for_registration(&mut state, registration()).expect("state resets");

        assert_eq!(state.session.transcript.len(), 0);
        assert_eq!(state.last_response, "Window session ready.");
        assert_eq!(state.save_path, Some("save.muddle".into()));
        assert_eq!(state.transcript_path, Some("transcript.txt".into()));
    }

    #[test]
    fn save_now_writes_configured_paths() {
        let save_path = temp_file_path("save-now.muddle");
        let transcript_path = temp_file_path("save-now.txt");
        let mut state = MuddleWindowState::new(
            registration(),
            None,
            Some(save_path.clone()),
            Some(transcript_path.clone()),
        )
        .expect("state starts");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Saved.");

        save_state_now(&mut state).expect("state saves");

        let saved = fs::read_to_string(&save_path).expect("save written");
        let transcript = fs::read_to_string(&transcript_path).expect("transcript written");
        assert!(saved.contains("command=look"));
        assert!(transcript.contains("MUDDLE_TRANSCRIPT_V1"));
        assert!(state.last_response.contains("Saved session"));

        let _ = fs::remove_file(save_path);
        let _ = fs::remove_file(transcript_path);
    }

    #[test]
    fn reload_save_uses_configured_save_path() {
        let save_path = temp_file_path("reload-save.muddle");
        let transcript_path = temp_file_path("reload-save.txt");
        let mut state = MuddleWindowState::new(
            registration(),
            None,
            Some(save_path.clone()),
            Some(transcript_path.clone()),
        )
        .expect("state starts");
        save_state_now(&mut state).expect("initial save writes");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Unsaved.");

        reload_state_from_save_path(&mut state).expect("state reloads");

        assert_eq!(state.session.transcript.len(), 0);
        assert_eq!(state.save_path, Some(save_path.clone()));
        assert_eq!(state.transcript_path, Some(transcript_path.clone()));
        assert!(state.last_response.contains("Reloaded save"));

        let _ = fs::remove_file(save_path);
        let _ = fs::remove_file(transcript_path);
    }

    #[test]
    fn save_slots_round_trip_from_configured_save_path() {
        let save_path = temp_file_path("slot-active.muddle");
        let slot_path = temp_file_path("slot-active.slot-before_gate.muddle");
        let transcript_path = temp_file_path("slot-active.txt");
        let mut state = MuddleWindowState::new(
            registration(),
            None,
            Some(save_path.clone()),
            Some(transcript_path.clone()),
        )
        .expect("state starts");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Slot saved.");

        save_state_to_slot(&mut state, "before_gate").expect("slot saves");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Unsaved.");
        load_state_from_slot(&mut state, "before_gate").expect("slot loads");

        assert_eq!(state.session.transcript.len(), 1);
        assert_eq!(
            list_save_slots(&state).expect("slots list"),
            vec!["before_gate"]
        );
        assert!(render_state_json(&state)
            .expect("state renders")
            .contains("\"save_slots\":[\"before_gate\"]"));
        assert!(render_state_json(&state)
            .expect("state renders")
            .contains("\"save_slot_details\":["));
        assert!(render_save_slot_details_json(&state)
            .expect("slot details render")
            .contains("\"bytes\":"));
        assert!(state.last_response.contains("Loaded save slot"));

        let _ = fs::remove_file(save_path);
        let _ = fs::remove_file(slot_path);
        let _ = fs::remove_file(transcript_path);
    }

    #[test]
    fn rejects_unsafe_save_slot_names() {
        assert!(normalize_save_slot_name("before_gate").is_ok());
        assert!(normalize_save_slot_name("../outside").is_err());
        assert!(normalize_save_slot_name("two words").is_err());
        assert!(normalize_save_slot_name("").is_err());
    }

    #[test]
    fn delete_save_slot_removes_slot_file() {
        let save_path = temp_file_path("delete-slot.muddle");
        let slot_path = temp_file_path("delete-slot.slot-before_gate.muddle");
        let mut state = MuddleWindowState::new(registration(), None, Some(save_path.clone()), None)
            .expect("state starts");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Slot saved.");
        save_state_to_slot(&mut state, "before_gate").expect("slot saves");

        delete_save_slot(&mut state, "before_gate").expect("slot deletes");

        assert!(!slot_path.exists());
        assert!(list_save_slots(&state).expect("slots list").is_empty());
        assert!(state.last_response.contains("Deleted save slot"));

        let _ = fs::remove_file(save_path);
    }

    #[test]
    fn export_save_slot_text_reads_slot_without_loading_it() {
        let save_path = temp_file_path("export-slot.muddle");
        let slot_path = temp_file_path("export-slot.slot-before_gate.muddle");
        let mut state = MuddleWindowState::new(registration(), None, Some(save_path.clone()), None)
            .expect("state starts");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Slot saved.");
        save_state_to_slot(&mut state, "before_gate").expect("slot saves");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Still active.");

        let exported = export_save_slot_text(&mut state, "before_gate")
            .expect("slot exports")
            .expect("slot text available");

        assert!(exported.contains("command=look"));
        assert_eq!(state.session.transcript.len(), 2);
        assert!(state.last_response.contains("Exported save slot"));

        let _ = fs::remove_file(save_path);
        let _ = fs::remove_file(slot_path);
    }

    #[test]
    fn export_and_import_save_text_round_trip() {
        let save_path = temp_file_path("import-export.muddle");
        let mut state = MuddleWindowState::new(registration(), None, Some(save_path.clone()), None)
            .expect("state starts");
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Exported.");
        let exported = render_save_export(&state);
        state
            .session
            .record_turn(MuddleCommand::parse("look"), "Unsaved.");

        import_state_from_text(&mut state, &exported).expect("save imports");

        assert_eq!(state.session.transcript.len(), 1);
        assert!(state.last_response.contains("Imported save text"));
        let saved = fs::read_to_string(&save_path).expect("active save updated");
        assert_eq!(saved, exported);

        let _ = fs::remove_file(save_path);
    }

    #[test]
    fn invalid_save_import_is_user_visible() {
        let mut state =
            MuddleWindowState::new(registration(), None, None, None).expect("state starts");

        import_state_from_text(&mut state, "not a save").expect("invalid import handled");

        assert!(state.last_response.contains("Import failed"));
    }

    #[test]
    fn rejects_empty_registrations() {
        assert!(run_muddle_window_hosts(
            Vec::new(),
            MuddleWindowRunOptions {
                list_hosts: true,
                ..MuddleWindowRunOptions::default()
            }
        )
        .is_err());
    }

    fn temp_file_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!("muddle-window-test-{}-{name}", std::process::id()))
    }
}
