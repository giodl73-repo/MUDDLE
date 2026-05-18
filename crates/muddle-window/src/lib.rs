use std::{
    env, fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
};

use muddle_cli::{render_transcript, write_play_panels, MuddleCliHostInfo};
use muddle_core::{MuddleCommand, MuddleHost, MuddleSession, MuddleSessionSave};

#[derive(Clone, Copy)]
pub struct MuddleWindowHostRegistration {
    pub name: &'static str,
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
        ("POST", "/select-host") => {
            let host_name = request_body(&request).trim();
            if let Some(registration) = find_window_host(registrations, host_name) {
                *state = MuddleWindowState::new(
                    registration,
                    None,
                    state.save_path.clone(),
                    state.transcript_path.clone(),
                )?;
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
        fs::write(path, state.session.save().encode())?;
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
        println!("  {} - {}", registration.name, registration.description);
    }
}

fn render_hosts_json(registrations: &[MuddleWindowHostRegistration]) -> String {
    let hosts = registrations
        .iter()
        .map(|registration| {
            format!(
                "{{\"name\":\"{}\",\"description\":\"{}\",\"suggested\":\"{}\"}}",
                json_escape(registration.name),
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

    Ok(format!(
        "{{\"host\":\"{}\",\"description\":\"{}\",\"suggested\":\"{}\",\"room\":\"{}\",\"turns\":{},\"panels\":\"{}\",\"room_card\":\"{}\",\"last_response\":\"{}\",\"save_path\":\"{}\",\"transcript_path\":\"{}\"}}",
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
    button { margin-top: .75rem; padding: .65rem 1rem; background: #316dca; color: #fff; border: 0; border-radius: 8px; font: inherit; cursor: pointer; }
    button.secondary { background: #263241; color: #dbe6f2; }
    button.host-card { display: block; width: 100%; margin: .75rem 0; text-align: left; background: #1d2936; border: 1px solid #42566b; }
    button.host-card strong { display: block; color: #fff; margin-bottom: .25rem; }
    #chooser { max-width: 56rem; margin: 0 auto; padding: 1rem; }
    #client { display: none; }
    .muted { color: #9aa7b2; }
    .response { color: #d8f8b7; }
  </style>
</head>
<body>
  <main id="chooser">
    <section>
      <h1>Choose a MUDDLE host</h1>
      <p class="muted">Pick the game surface to mount in this local window. You can switch later, which starts a fresh session for that host.</p>
      <div id="host-list"></div>
    </section>
  </main>
  <main id="client">
    <section>
      <h1>MUDDLE Window</h1>
      <p id="host" class="muted"></p>
      <p id="suggested"></p>
      <button id="change-host" class="secondary" type="button">Change host</button>
      <p id="persistence" class="muted"></p>
      <h2>Panels</h2>
      <pre id="panels"></pre>
    </section>
    <section>
      <h2 id="room"></h2>
      <pre id="card"></pre>
      <h2>Last response</h2>
      <pre id="response" class="response"></pre>
      <form id="command-form">
        <input id="command" autocomplete="off" autofocus placeholder="type a command, e.g. look">
        <button type="submit">Send command</button>
      </form>
    </section>
  </main>
  <script>
    let selectedHost = null;

    async function loadHosts() {
      const hosts = await fetch('/hosts').then(r => r.json());
      const list = document.getElementById('host-list');
      list.innerHTML = '';
      for (const host of hosts) {
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
      const persistence = [];
      if (state.save_path) persistence.push(`save: ${state.save_path}`);
      if (state.transcript_path) persistence.push(`transcript: ${state.transcript_path}`);
      document.getElementById('persistence').textContent = persistence.join(' | ');
    }

    document.getElementById('change-host').addEventListener('click', showChooser);
    document.getElementById('command-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      if (!selectedHost) return;
      const input = document.getElementById('command');
      const command = input.value.trim();
      if (!command) return;
      input.value = '';
      const state = await fetch('/command', { method: 'POST', body: command }).then(r => r.json());
      renderState(state);
    });
    loadHosts();
  </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use muddle_core::{MuddleCommandOutcome, MuddleError, MuddleRoom};

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
            Err(MuddleError::UnknownCommand {
                room_id: room_id.to_string(),
                command: command.clone(),
            })
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
}
