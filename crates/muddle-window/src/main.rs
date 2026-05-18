use std::{
    env,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    process::Command,
};

use muddle_amaze_spike::AmazeSilverstreamHost;
use muddle_banish_spike::BanishPilgrimLossHost;
use muddle_cli::{write_play_panels, MuddleCliHostInfo};
use muddle_core::{MuddleCommand, MuddleHost, MuddleSession};
use muddle_mock_sim::MuddleMockSimHost;

const DEFAULT_HOST: &str = "mock-labyrinth";
const DEFAULT_ADDR: &str = "127.0.0.1:4777";

struct WindowHostRegistration {
    name: &'static str,
    description: &'static str,
    suggested_commands: &'static str,
    create: fn() -> Box<dyn MuddleHost>,
}

struct WindowState {
    host: Box<dyn MuddleHost>,
    session: MuddleSession,
    info: MuddleCliHostInfo,
    last_response: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowOptions {
    host_name: String,
    addr: String,
    open: bool,
    list_hosts: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            host_name: DEFAULT_HOST.to_string(),
            addr: DEFAULT_ADDR.to_string(),
            open: false,
            list_hosts: false,
        }
    }
}

fn main() -> io::Result<()> {
    let options = parse_args(env::args()).map_err(|message| {
        eprintln!("{message}");
        print_usage();
        io::Error::new(io::ErrorKind::InvalidInput, message)
    })?;

    if options.list_hosts {
        print_hosts();
        return Ok(());
    }

    let registration = find_host(&options.host_name).ok_or_else(|| {
        let message = format!("Unknown MUDDLE host `{}`.", options.host_name);
        eprintln!("{message}");
        print_hosts();
        io::Error::new(io::ErrorKind::InvalidInput, message)
    })?;

    let url = format!("http://{}", options.addr);
    let mut state = WindowState::new(registration)?;
    let listener = TcpListener::bind(&options.addr)?;
    println!("MUDDLE window client listening at {url}");
    println!("Host mounted: {}", state.info.name);
    println!("Press Ctrl+C to stop.");

    if options.open {
        open_browser(&url)?;
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, &mut state)?,
            Err(error) => eprintln!("MUDDLE window connection failed: {error}"),
        }
    }

    Ok(())
}

impl WindowState {
    fn new(registration: WindowHostRegistration) -> io::Result<Self> {
        let host = (registration.create)();
        let session = MuddleSession::for_host(host.as_ref()).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("registered host cannot start: {error:?}"),
            )
        })?;
        Ok(Self {
            host,
            session,
            info: registration.info(),
            last_response: "Window session ready.".to_string(),
        })
    }
}

fn handle_connection(mut stream: TcpStream, state: &mut WindowState) -> io::Result<()> {
    let mut buffer = [0_u8; 8192];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let (method, path) = request_line(&request);
    match (method, path) {
        ("GET", "/") => write_response(&mut stream, "200 OK", "text/html", CLIENT_HTML),
        ("GET", "/state") => write_response(
            &mut stream,
            "200 OK",
            "application/json",
            &render_state_json(state)?,
        ),
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

fn render_state_json(state: &WindowState) -> io::Result<String> {
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
        "{{\"host\":\"{}\",\"description\":\"{}\",\"suggested\":\"{}\",\"room\":\"{}\",\"turns\":{},\"panels\":\"{}\",\"room_card\":\"{}\",\"last_response\":\"{}\"}}",
        json_escape(state.info.name),
        json_escape(state.info.description),
        json_escape(state.info.suggested_commands),
        json_escape(&state.session.current_room),
        state.session.transcript.len(),
        json_escape(&panels),
        json_escape(&room_card),
        json_escape(&state.last_response)
    ))
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

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<WindowOptions, String> {
    let mut args = args.into_iter();
    let _program = args.next();
    let mut options = WindowOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--open" => options.open = true,
            "--list-hosts" => options.list_hosts = true,
            "--host" => {
                options.host_name = args
                    .next()
                    .ok_or_else(|| "`--host` requires a host name.".to_string())?;
            }
            "--addr" => {
                options.addr = args
                    .next()
                    .ok_or_else(|| "`--addr` requires an address.".to_string())?;
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--host=") {
                    if value.is_empty() {
                        return Err("`--host` requires a host name.".to_string());
                    }
                    options.host_name = value.to_string();
                } else if let Some(value) = arg.strip_prefix("--addr=") {
                    if value.is_empty() {
                        return Err("`--addr` requires an address.".to_string());
                    }
                    options.addr = value.to_string();
                } else {
                    return Err(format!("Unknown argument `{arg}`."));
                }
            }
        }
    }

    Ok(options)
}

fn find_host(name: &str) -> Option<WindowHostRegistration> {
    host_registry()
        .into_iter()
        .find(|registration| registration.name == name)
}

fn host_registry() -> Vec<WindowHostRegistration> {
    vec![
        WindowHostRegistration {
            name: DEFAULT_HOST,
            description: "Labyrinth mock sim: BANISH-like resource state plus AMAZE-like locks.",
            suggested_commands:
                "`look`, `gather ember`, `go antechamber`, `inspect glyphs`, `use ember`, `go vault`.",
            create: || Box::new(MuddleMockSimHost::new()),
        },
        WindowHostRegistration {
            name: "banish-pilgrim-loss",
            description: "BANISH Pilgrim Loss adapter spike: launcher, campaign brief, and migration trail.",
            suggested_commands:
                "`look`, `choose resume`, `inspect plan`, `inspect manifest`, `go trail`, `resolve loss`.",
            create: || Box::new(BanishPilgrimLossHost::new()),
        },
        WindowHostRegistration {
            name: "amaze-silverstream",
            description: "AMAZE Silverstream adapter spike: clue, signal, hatch, and escape path.",
            suggested_commands:
                "`look`, `go receiver`, `inspect clue`, `tune signal`, `unlock hatch`, `go hatch`.",
            create: || Box::new(AmazeSilverstreamHost::new()),
        },
    ]
}

impl WindowHostRegistration {
    fn info(&self) -> MuddleCliHostInfo {
        MuddleCliHostInfo {
            name: self.name,
            description: self.description,
            suggested_commands: self.suggested_commands,
        }
    }
}

fn print_usage() {
    eprintln!("Usage: muddle-window [--host <name>] [--addr <ip:port>] [--open] [--list-hosts]");
}

fn print_hosts() {
    println!("Available MUDDLE window hosts:");
    for registration in host_registry() {
        println!("  {} - {}", registration.name, registration.description);
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

const CLIENT_HTML: &str = r#"<!doctype html>
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
    .muted { color: #9aa7b2; }
    .response { color: #d8f8b7; }
  </style>
</head>
<body>
  <main>
    <section>
      <h1>MUDDLE Window</h1>
      <p id="host" class="muted"></p>
      <p id="suggested"></p>
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
    async function refresh() {
      const state = await fetch('/state').then(r => r.json());
      document.title = `MUDDLE - ${state.host}`;
      document.getElementById('host').textContent = `${state.host}: ${state.description}`;
      document.getElementById('suggested').textContent = `Try: ${state.suggested}`;
      document.getElementById('room').textContent = `${state.room} (${state.turns} turns)`;
      document.getElementById('panels').textContent = state.panels || '(no panels)';
      document.getElementById('card').textContent = state.room_card;
      document.getElementById('response').textContent = state.last_response;
    }
    document.getElementById('command-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      const input = document.getElementById('command');
      const command = input.value.trim();
      if (!command) return;
      input.value = '';
      const state = await fetch('/command', { method: 'POST', body: command }).then(r => r.json());
      document.getElementById('room').textContent = `${state.room} (${state.turns} turns)`;
      document.getElementById('panels').textContent = state.panels || '(no panels)';
      document.getElementById('card').textContent = state.room_card;
      document.getElementById('response').textContent = state.last_response;
    });
    refresh();
  </script>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_options() {
        assert_eq!(
            parse_args(["muddle-window"].into_iter().map(String::from)),
            Ok(WindowOptions::default())
        );
    }

    #[test]
    fn parses_window_options() {
        assert_eq!(
            parse_args(
                [
                    "muddle-window",
                    "--host",
                    "amaze-silverstream",
                    "--addr=127.0.0.1:4888",
                    "--open"
                ]
                .into_iter()
                .map(String::from)
            ),
            Ok(WindowOptions {
                host_name: "amaze-silverstream".to_string(),
                addr: "127.0.0.1:4888".to_string(),
                open: true,
                list_hosts: false,
            })
        );
    }

    #[test]
    fn extracts_request_parts() {
        let request = "POST /command HTTP/1.1\r\nContent-Length: 4\r\n\r\nlook";
        assert_eq!(request_line(request), ("POST", "/command"));
        assert_eq!(request_body(request), "look");
    }

    #[test]
    fn escapes_json_strings() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
    }

    #[test]
    fn registers_mock_host() {
        assert!(find_host("mock-labyrinth").is_some());
    }
}
