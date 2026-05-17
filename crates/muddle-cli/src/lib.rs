use std::{
    env, fs,
    io::{self, BufRead, Write},
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
    let stdin = io::stdin();
    let stdout = io::stdout();
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

    Ok(())
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
    fn parses_save_and_load_options() {
        let options = parse_run_options(
            ["--load", "in.muddle", "--save", "out.muddle"]
                .into_iter()
                .map(String::from),
        )
        .expect("options parse");

        assert_eq!(options.load_path, Some(PathBuf::from("in.muddle")));
        assert_eq!(options.save_path, Some(PathBuf::from("out.muddle")));
    }

    #[test]
    fn runner_saves_and_loads_transcript_replay() {
        let save_path = temp_save_path("runner-save-load");
        let mut host = test_host();
        let mut output = Vec::new();

        let saved = run_muddle_host_with_options(
            &mut host,
            test_info(),
            MuddleCliRunOptions {
                load_path: None,
                save_path: Some(save_path.clone()),
            },
            "go north\nquit\n".as_bytes(),
            &mut output,
        )
        .expect("save run succeeds");

        assert_eq!(saved.current_room, "north");
        let encoded = fs::read_to_string(&save_path).expect("save file exists");
        assert!(encoded.contains("current_room=north"));
        assert!(encoded.contains("command=go north"));

        let mut resumed_host = test_host();
        let mut resumed_output = Vec::new();
        let resumed = run_muddle_host_with_options(
            &mut resumed_host,
            test_info(),
            MuddleCliRunOptions {
                load_path: Some(save_path.clone()),
                save_path: Some(save_path.clone()),
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
