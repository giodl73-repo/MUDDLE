use std::{
    env, fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
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
    let mut buffer = [0_u8; 8192];
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
            slots.push(slot_name.to_string());
        }
    }
    slots.sort();
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

    Ok(format!(
        "{{\"host\":\"{}\",\"description\":\"{}\",\"suggested\":\"{}\",\"room\":\"{}\",\"turns\":{},\"panels\":\"{}\",\"room_card\":\"{}\",\"last_response\":\"{}\",\"save_path\":\"{}\",\"transcript_path\":\"{}\",\"save_slots\":{save_slots},\"commands\":{commands},\"history\":{history}}}",
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
    button { margin-top: .75rem; padding: .65rem 1rem; background: #316dca; color: #fff; border: 0; border-radius: 8px; font: inherit; cursor: pointer; }
    button.secondary { background: #263241; color: #dbe6f2; }
    button.host-card { display: block; width: 100%; margin: .75rem 0; text-align: left; background: #1d2936; border: 1px solid #42566b; }
    button.host-card strong { display: block; color: #fff; margin-bottom: .25rem; }
    button.command-button { margin: .35rem .35rem 0 0; background: #244b32; }
    button.command-button span { display: block; color: #c4d2c8; font-size: .8rem; margin-top: .2rem; }
    ol.history { padding-left: 1.25rem; }
    ol.history li { margin: .75rem 0; }
    ol.history pre { background: #0f1318; border: 1px solid #263241; border-radius: 8px; padding: .75rem; }
    #chooser { max-width: 56rem; margin: 0 auto; padding: 1rem; }
    #client { display: none; }
    .category-heading { margin: 1.25rem 0 .35rem; color: #d0e8ff; }
    .empty-hosts { border: 1px dashed #42566b; border-radius: 8px; padding: 1rem; }
    .muted { color: #9aa7b2; }
    .response { color: #d8f8b7; }
  </style>
</head>
<body>
  <main id="chooser">
    <section>
      <h1>Choose a MUDDLE host</h1>
      <p class="muted">Pick the game surface to mount in this local window. You can switch later, which starts a fresh session for that host.</p>
      <input id="host-filter" autocomplete="off" placeholder="filter hosts, e.g. game, banish, knowledge">
      <div id="host-list"></div>
    </section>
  </main>
  <main id="client">
    <section>
      <h1>MUDDLE Window</h1>
      <p id="host" class="muted"></p>
      <p id="suggested"></p>
      <button id="change-host" class="secondary" type="button">Change host</button>
      <button id="reset-host" class="secondary" type="button">Restart host</button>
      <button id="save-now" class="secondary" type="button">Save now</button>
      <button id="load-save" class="secondary" type="button">Reload save</button>
      <p id="persistence" class="muted"></p>
      <h2>Save slots</h2>
      <input id="save-slot-name" autocomplete="off" placeholder="slot name, e.g. before-boss">
      <button id="save-slot" class="secondary" type="button">Save slot</button>
      <select id="save-slot-list"></select>
      <button id="load-slot" class="secondary" type="button">Load slot</button>
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

    async function loadHosts() {
      availableHosts = await fetch('/hosts').then(r => r.json());
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
      const state = await fetch('/select-host', { method: 'POST', body: hostName }).then(r => r.json());
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
      document.title = `MUDDLE - ${state.host}`;
      document.getElementById('host').textContent = `${state.host}: ${state.description}`;
      document.getElementById('suggested').textContent = `Try: ${state.suggested}`;
      document.getElementById('room').textContent = `${state.room} (${state.turns} turns)`;
      document.getElementById('panels').textContent = state.panels || '(no panels)';
      document.getElementById('card').textContent = state.room_card;
      document.getElementById('response').textContent = state.last_response;
      renderCommandButtons(state.commands || []);
      renderHistory(state.history || []);
      renderSaveSlots(state.save_slots || []);
      const persistence = [];
      if (state.save_path) persistence.push(`save: ${state.save_path}`);
      if (state.transcript_path) persistence.push(`transcript: ${state.transcript_path}`);
      document.getElementById('persistence').textContent = persistence.join(' | ');
    }

    function renderSaveSlots(slots) {
      const list = document.getElementById('save-slot-list');
      const selected = list.value;
      list.innerHTML = '';
      if (!slots.length) {
        const option = document.createElement('option');
        option.value = '';
        option.textContent = 'No save slots yet';
        list.appendChild(option);
        return;
      }
      for (const slot of slots) {
        const option = document.createElement('option');
        option.value = slot;
        option.textContent = slot;
        list.appendChild(option);
      }
      if (slots.includes(selected)) list.value = selected;
    }

    function currentSlotName() {
      const input = document.getElementById('save-slot-name').value.trim();
      return input || document.getElementById('save-slot-list').value;
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
      const state = await fetch('/command', { method: 'POST', body: command }).then(r => r.json());
      renderState(state);
      document.getElementById('command').focus();
    }

    document.getElementById('change-host').addEventListener('click', showChooser);
    document.getElementById('host-filter').addEventListener('input', renderHosts);
    document.getElementById('reset-host').addEventListener('click', async () => {
      const state = await fetch('/reset', { method: 'POST' }).then(r => r.json());
      renderState(state);
      document.getElementById('command').focus();
    });
    document.getElementById('save-now').addEventListener('click', async () => {
      const state = await fetch('/save', { method: 'POST' }).then(r => r.json());
      renderState(state);
      document.getElementById('command').focus();
    });
    document.getElementById('load-save').addEventListener('click', async () => {
      const state = await fetch('/load-save', { method: 'POST' }).then(r => r.json());
      renderState(state);
      document.getElementById('command').focus();
    });
    document.getElementById('save-slot').addEventListener('click', async () => {
      const state = await fetch('/save-slot', { method: 'POST', body: currentSlotName() }).then(r => r.json());
      renderState(state);
      document.getElementById('command').focus();
    });
    document.getElementById('load-slot').addEventListener('click', async () => {
      const state = await fetch('/load-slot', { method: 'POST', body: currentSlotName() }).then(r => r.json());
      renderState(state);
      document.getElementById('command').focus();
    });
    document.getElementById('command-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      if (!selectedHost) return;
      const input = document.getElementById('command');
      const command = input.value.trim();
      if (!command) return;
      input.value = '';
      await sendCommand(command);
    });
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
