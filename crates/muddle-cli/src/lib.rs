use std::{
    env, fs,
    io::{self, BufRead, Cursor, Write},
    path::PathBuf,
};

use muddle_core::{MuddleCommand, MuddleHost, MuddleSession, MuddleSessionSave};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MuddleCliHostInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub suggested_commands: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MuddleCliRunOptions {
    pub load_path: Option<PathBuf>,
    pub save_path: Option<PathBuf>,
    pub transcript_path: Option<PathBuf>,
    pub script_path: Option<PathBuf>,
}

pub fn run_muddle_host(
    host: &mut dyn MuddleHost,
    info: MuddleCliHostInfo,
) -> io::Result<MuddleSession> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_muddle_host_with_options(
        host,
        info,
        MuddleCliRunOptions::default(),
        stdin.lock(),
        stdout.lock(),
    )
}

pub fn run_muddle_host_from_env_args(
    host: &mut dyn MuddleHost,
    info: MuddleCliHostInfo,
) -> io::Result<MuddleSession> {
    let options = parse_run_options(env::args().skip(1))?;
    run_muddle_host_with_stdio(host, info, options)
}

pub fn run_muddle_host_with_stdio(
    host: &mut dyn MuddleHost,
    info: MuddleCliHostInfo,
    options: MuddleCliRunOptions,
) -> io::Result<MuddleSession> {
    let stdout = io::stdout();
    if let Some(path) = &options.script_path {
        let script = fs::read_to_string(path)?;
        return run_muddle_host_with_options(
            host,
            info,
            options,
            Cursor::new(script),
            stdout.lock(),
        );
    }

    let stdin = io::stdin();
    run_muddle_host_with_options(host, info, options, stdin.lock(), stdout.lock())
}

pub fn run_muddle_host_with_io<R, W>(
    host: &mut dyn MuddleHost,
    info: MuddleCliHostInfo,
    input: R,
    output: W,
) -> io::Result<MuddleSession>
where
    R: BufRead,
    W: Write,
{
    run_muddle_host_with_options(host, info, MuddleCliRunOptions::default(), input, output)
}

pub fn run_muddle_host_with_options<R, W>(
    host: &mut dyn MuddleHost,
    info: MuddleCliHostInfo,
    options: MuddleCliRunOptions,
    mut input: R,
    mut output: W,
) -> io::Result<MuddleSession>
where
    R: BufRead,
    W: Write,
{
    let mut session = if let Some(path) = &options.load_path {
        let encoded = fs::read_to_string(path)?;
        let save = MuddleSessionSave::decode(&encoded)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
        let session = MuddleSession::resume_for_host(host, &save)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
        writeln!(
            output,
            "Loaded MUDDLE session from {} with {} transcript turns.",
            path.display(),
            session.transcript.len()
        )?;
        session
    } else {
        MuddleSession::for_host(host).expect("registered host must expose a start room")
    };

    writeln!(output, "MUDDLE CLI")?;
    writeln!(output, "Host mounted: {}", info.name)?;
    writeln!(output, "{}", info.description)?;
    writeln!(output, "Try: {}", info.suggested_commands)?;

    loop {
        write_play_panels(&mut output, host, &session)?;
        write!(output, "\n{}> ", session.current_room)?;
        output.flush()?;

        let mut command_text = String::new();
        let bytes_read = input.read_line(&mut command_text)?;
        if bytes_read == 0 {
            break;
        }

        let command_text = command_text.trim();
        if command_text.eq_ignore_ascii_case("quit") || command_text.eq_ignore_ascii_case("exit") {
            writeln!(output, "Transcript turns: {}", session.transcript.len())?;
            break;
        }

        match session.play_turn(host, MuddleCommand::parse(command_text)) {
            Ok(turn) => writeln!(output, "{}", turn.response)?,
            Err(error) => writeln!(output, "Command failed: {error:?}")?,
        }
    }

    write_save_if_requested(&mut output, &session, &options)?;
    write_transcript_if_requested(&mut output, info, &session, &options)?;
    Ok(session)
}

pub fn parse_run_options(
    args: impl IntoIterator<Item = String>,
) -> io::Result<MuddleCliRunOptions> {
    let mut options = MuddleCliRunOptions::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
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
            "--script" => {
                options.script_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "`--script` requires a path")
                })?));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown MUDDLE runner argument `{arg}`."),
                ));
            }
        }
    }

    Ok(options)
}

fn write_save_if_requested<W: Write>(
    output: &mut W,
    session: &MuddleSession,
    options: &MuddleCliRunOptions,
) -> io::Result<()> {
    if let Some(path) = &options.save_path {
        fs::write(path, session.save().encode())?;
        writeln!(output, "Saved MUDDLE session to {}.", path.display())?;
    }
    Ok(())
}

fn write_transcript_if_requested<W: Write>(
    output: &mut W,
    info: MuddleCliHostInfo,
    session: &MuddleSession,
    options: &MuddleCliRunOptions,
) -> io::Result<()> {
    if let Some(path) = &options.transcript_path {
        fs::write(path, render_transcript(info, session))?;
        writeln!(output, "Exported MUDDLE transcript to {}.", path.display())?;
    }
    Ok(())
}

pub fn render_transcript(info: MuddleCliHostInfo, session: &MuddleSession) -> String {
    let mut lines = vec![
        "MUDDLE_TRANSCRIPT_V1".to_string(),
        format!("host={}", info.name),
        format!("current_room={}", session.current_room),
        format!("turns={}", session.transcript.len()),
        String::new(),
    ];

    for (index, turn) in session.transcript.iter().enumerate() {
        lines.push(format!("## Turn {}", index + 1));
        lines.push(format!("room: {}", turn.room_id));
        lines.push(format!("command: {}", turn.command.normalized()));
        lines.push("response:".to_string());
        lines.extend(turn.response.lines().map(|line| format!("  {line}")));
        lines.push(String::new());
    }

    lines.join("\n")
}

pub fn write_play_panels<W: Write>(
    output: &mut W,
    host: &dyn MuddleHost,
    session: &MuddleSession,
) -> io::Result<()> {
    let resources = host.resource_panel();
    if !resources.is_empty() {
        let status = resources
            .iter()
            .map(|resource| format!("{}={}", resource.label, resource.value))
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(output, "\n[status] {status}")?;
    }

    let inventory = host.inventory_panel();
    if !inventory.is_empty() {
        let inventory_text = inventory
            .iter()
            .map(|item| {
                if item.detail.is_empty() {
                    item.label.clone()
                } else {
                    format!("{} ({})", item.label, item.detail)
                }
            })
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(output, "[inventory] {inventory_text}")?;
    }

    if let Some(map) = host.map_panel(&session.current_room) {
        writeln!(output, "[map] {map}")?;
    }

    let objectives = host.objective_panel(&session.current_room);
    if !objectives.is_empty() {
        writeln!(output, "[objectives] {}", objectives.join(" | "))?;
    }

    let commands = host.command_panel(&session.current_room);
    if !commands.is_empty() {
        let command_text = commands
            .iter()
            .map(|hint| format!("{} ({})", hint.command, hint.description))
            .collect::<Vec<_>>()
            .join(" | ");
        writeln!(output, "[commands] {command_text}")?;
    }

    let recent_log = recent_log_panel(session, 3);
    if !recent_log.is_empty() {
        writeln!(output, "[recent] {}", recent_log.join(" | "))?;
    }

    Ok(())
}

pub fn recent_log_panel(session: &MuddleSession, limit: usize) -> Vec<String> {
    let skip_count = session.transcript.len().saturating_sub(limit);
    session
        .transcript
        .iter()
        .skip(skip_count)
        .map(|turn| {
            format!(
                "{}: {}",
                turn.command.normalized(),
                compact_response(&turn.response)
            )
        })
        .collect()
}

fn compact_response(response: &str) -> String {
    response
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use muddle_core::{MuddleExit, MuddleRoom, MuddleStaticHost};

    #[test]
    fn scripted_runner_returns_transcript() {
        let mut host = MuddleStaticHost::try_new(
            "entry",
            [
                MuddleRoom {
                    id: "entry".to_string(),
                    title: "Entry".to_string(),
                    description: "A test entry.".to_string(),
                    exits: vec![MuddleExit {
                        command: "go north".to_string(),
                        target_room: "north".to_string(),
                        label: "North".to_string(),
                    }],
                },
                MuddleRoom {
                    id: "north".to_string(),
                    title: "North".to_string(),
                    description: "A test north room.".to_string(),
                    exits: Vec::new(),
                },
            ],
        )
        .expect("static host is valid");

        let mut output = Vec::new();
        let session = run_muddle_host_with_io(
            &mut host,
            MuddleCliHostInfo {
                name: "test-host",
                description: "Test host.",
                suggested_commands: "`look`, `go north`, `quit`.",
            },
            "look\ngo north\nquit\n".as_bytes(),
            &mut output,
        )
        .expect("runner succeeds");

        let rendered = String::from_utf8(output).expect("runner writes utf8");
        assert_eq!(session.current_room, "north");
        assert_eq!(session.transcript.len(), 2);
        assert!(rendered.contains("Host mounted: test-host"));
        assert!(rendered.contains("Transcript turns: 2"));
    }

    #[test]
    fn writes_recent_log_panel_from_transcript() {
        let mut host = test_host();
        let mut session = MuddleSession::for_host(&host).expect("session starts");
        session
            .play_turn(&mut host, MuddleCommand::parse("look"))
            .expect("look turn plays");
        session
            .play_turn(&mut host, MuddleCommand::parse("go north"))
            .expect("move turn plays");

        let mut output = Vec::new();
        write_play_panels(&mut output, &host, &session).expect("panels render");
        let rendered = String::from_utf8(output).expect("panels are utf8");

        assert!(rendered.contains("[recent] look: + Entry | go north: You move to North."));
    }

    #[test]
    fn writes_inventory_panel_when_host_provides_items() {
        let host = muddle_mock_sim::MuddleMockSimHost::new();
        let session = MuddleSession::for_host(&host).expect("session starts");
        let mut output = Vec::new();

        write_play_panels(&mut output, &host, &session).expect("panels render");
        let rendered = String::from_utf8(output).expect("panels are utf8");

        assert!(rendered.contains("[inventory] map scrap (route to the glyph gate)"));
    }

    #[test]
    fn recent_log_panel_limits_to_latest_entries() {
        let mut session = MuddleSession::new("entry");
        session.record_turn(MuddleCommand::parse("look"), "one");
        session.record_turn(MuddleCommand::parse("inspect"), "two");
        session.record_turn(MuddleCommand::parse("wait"), "three");

        assert_eq!(
            recent_log_panel(&session, 2),
            vec!["inspect: two".to_string(), "wait: three".to_string()]
        );
    }

    #[test]
    fn parses_save_and_load_options() {
        let options = parse_run_options(
            [
                "--load",
                "in.muddle",
                "--save",
                "out.muddle",
                "--transcript",
                "play.txt",
                "--script",
                "commands.txt",
            ]
            .into_iter()
            .map(String::from),
        )
        .expect("options parse");

        assert_eq!(options.load_path, Some(PathBuf::from("in.muddle")));
        assert_eq!(options.save_path, Some(PathBuf::from("out.muddle")));
        assert_eq!(options.transcript_path, Some(PathBuf::from("play.txt")));
        assert_eq!(options.script_path, Some(PathBuf::from("commands.txt")));
    }

    #[test]
    fn runner_saves_and_loads_transcript_replay() {
        let save_path = temp_save_path("runner-save-load");
        let transcript_path = temp_save_path("runner-transcript-export");
        let mut host = test_host();
        let mut output = Vec::new();

        let saved = run_muddle_host_with_options(
            &mut host,
            test_info(),
            MuddleCliRunOptions {
                load_path: None,
                save_path: Some(save_path.clone()),
                transcript_path: Some(transcript_path.clone()),
                script_path: None,
            },
            "go north\nquit\n".as_bytes(),
            &mut output,
        )
        .expect("save run succeeds");

        assert_eq!(saved.current_room, "north");
        let encoded = fs::read_to_string(&save_path).expect("save file exists");
        assert!(encoded.contains("current_room=north"));
        assert!(encoded.contains("command=go north"));
        let transcript = fs::read_to_string(&transcript_path).expect("transcript file exists");
        assert!(transcript.contains("MUDDLE_TRANSCRIPT_V1"));
        assert!(transcript.contains("command: go north"));

        let mut resumed_host = test_host();
        let mut resumed_output = Vec::new();
        let resumed = run_muddle_host_with_options(
            &mut resumed_host,
            test_info(),
            MuddleCliRunOptions {
                load_path: Some(save_path.clone()),
                save_path: Some(save_path.clone()),
                transcript_path: Some(transcript_path.clone()),
                script_path: None,
            },
            "look\nquit\n".as_bytes(),
            &mut resumed_output,
        )
        .expect("load run succeeds");

        let rendered = String::from_utf8(resumed_output).expect("runner writes utf8");
        assert_eq!(resumed.current_room, "north");
        assert_eq!(resumed.transcript.len(), 2);
        assert!(rendered.contains("Loaded MUDDLE session"));

        fs::remove_file(save_path).expect("save file can be removed");
        fs::remove_file(transcript_path).expect("transcript file can be removed");
    }

    #[test]
    fn renders_readable_transcript_export() {
        let mut session = MuddleSession::new("entry");
        session.record_turn(MuddleCommand::parse("look"), "+ Entry\n| exits: go north");

        let transcript = render_transcript(test_info(), &session);

        assert!(transcript.contains("host=test-host"));
        assert!(transcript.contains("turns=1"));
        assert!(transcript.contains("command: look"));
        assert!(transcript.contains("  + Entry"));
    }

    fn test_info() -> MuddleCliHostInfo {
        MuddleCliHostInfo {
            name: "test-host",
            description: "Test host.",
            suggested_commands: "`look`, `go north`, `quit`.",
        }
    }

    fn test_host() -> MuddleStaticHost {
        MuddleStaticHost::try_new(
            "entry",
            [
                MuddleRoom {
                    id: "entry".to_string(),
                    title: "Entry".to_string(),
                    description: "A test entry.".to_string(),
                    exits: vec![MuddleExit {
                        command: "go north".to_string(),
                        target_room: "north".to_string(),
                        label: "North".to_string(),
                    }],
                },
                MuddleRoom {
                    id: "north".to_string(),
                    title: "North".to_string(),
                    description: "A test north room.".to_string(),
                    exits: Vec::new(),
                },
            ],
        )
        .expect("static host is valid")
    }

    fn temp_save_path(name: &str) -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("muddle-{name}-{}.muddle", std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }
}
