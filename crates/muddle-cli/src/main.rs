use std::{
    env,
    io::{self, Write},
};

use muddle_amaze_spike::AmazeSilverstreamHost;
use muddle_banish_spike::BanishPilgrimLossHost;
use muddle_core::{MuddleCommand, MuddleHost, MuddleSession};
use muddle_mock_sim::MuddleMockSimHost;

const DEFAULT_HOST: &str = "mock-labyrinth";

struct HostRegistration {
    name: &'static str,
    description: &'static str,
    suggested_commands: &'static str,
    create: fn() -> Box<dyn MuddleHost>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliAction {
    Run { host_name: String },
    ListHosts,
}

fn main() -> io::Result<()> {
    let action = parse_args(env::args()).map_err(|message| {
        eprintln!("{message}");
        print_host_usage();
        io::Error::new(io::ErrorKind::InvalidInput, message)
    })?;

    match action {
        CliAction::ListHosts => {
            print_hosts();
            Ok(())
        }
        CliAction::Run { host_name } => {
            let registration = find_host(&host_name).ok_or_else(|| {
                let message = format!("Unknown MUDDLE host `{host_name}`.");
                eprintln!("{message}");
                print_hosts();
                io::Error::new(io::ErrorKind::InvalidInput, message)
            })?;
            run_host(registration)
        }
    }
}

fn run_host(registration: HostRegistration) -> io::Result<()> {
    let mut host = (registration.create)();
    let mut session =
        MuddleSession::for_host(host.as_ref()).expect("registered host must expose a start room");

    println!("MUDDLE CLI");
    println!("Host mounted: {}", registration.name);
    println!("{}", registration.description);
    println!("Try: {}", registration.suggested_commands);

    loop {
        print_play_panels(host.as_ref(), &session);
        print!("\n{}> ", session.current_room);
        io::stdout().flush()?;

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;
        if bytes_read == 0 {
            break;
        }

        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("Transcript turns: {}", session.transcript.len());
            break;
        }

        match session.play_turn(host.as_mut(), MuddleCommand::parse(input)) {
            Ok(turn) => println!("{}", turn.response),
            Err(error) => println!("Command failed: {error:?}"),
        }
    }

    Ok(())
}

fn print_play_panels(host: &dyn MuddleHost, session: &MuddleSession) {
    let resources = host.resource_panel();
    if !resources.is_empty() {
        let status = resources
            .iter()
            .map(|resource| format!("{}={}", resource.label, resource.value))
            .collect::<Vec<_>>()
            .join(" | ");
        println!("\n[status] {status}");
    }

    if let Some(map) = host.map_panel(&session.current_room) {
        println!("[map] {map}");
    }

    let objectives = host.objective_panel(&session.current_room);
    if !objectives.is_empty() {
        println!("[objectives] {}", objectives.join(" | "));
    }

    let commands = host.command_panel(&session.current_room);
    if !commands.is_empty() {
        let command_text = commands
            .iter()
            .map(|hint| format!("{} ({})", hint.command, hint.description))
            .collect::<Vec<_>>()
            .join(" | ");
        println!("[commands] {command_text}");
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliAction, String> {
    let mut args = args.into_iter();
    let _program = args.next();

    let mut host_name = DEFAULT_HOST.to_string();
    while let Some(arg) = args.next() {
        if arg == "--list-hosts" {
            return Ok(CliAction::ListHosts);
        }

        if arg == "--host" {
            host_name = args
                .next()
                .ok_or_else(|| "`--host` requires a host name.".to_string())?;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--host=") {
            if value.is_empty() {
                return Err("`--host` requires a host name.".to_string());
            }
            host_name = value.to_string();
            continue;
        }

        return Err(format!("Unknown argument `{arg}`."));
    }

    Ok(CliAction::Run { host_name })
}

fn find_host(name: &str) -> Option<HostRegistration> {
    host_registry()
        .into_iter()
        .find(|registration| registration.name == name)
}

fn host_registry() -> Vec<HostRegistration> {
    vec![
        HostRegistration {
            name: DEFAULT_HOST,
            description: "Labyrinth mock sim: BANISH-like resource state plus AMAZE-like locks.",
            suggested_commands:
                "`look`, `gather ember`, `go antechamber`, `inspect glyphs`, `use ember`, `go vault`, `quit`.",
            create: || Box::new(MuddleMockSimHost::new()),
        },
        HostRegistration {
            name: "banish-pilgrim-loss",
            description: "BANISH Pilgrim Loss adapter spike: launcher, campaign brief, and migration trail.",
            suggested_commands:
                "`look`, `choose resume`, `inspect plan`, `inspect manifest`, `go trail`, `resolve loss`, `quit`.",
            create: || Box::new(BanishPilgrimLossHost::new()),
        },
        HostRegistration {
            name: "amaze-silverstream",
            description: "AMAZE Silverstream adapter spike: clue, signal, hatch, and escape path.",
            suggested_commands:
                "`look`, `go receiver`, `inspect clue`, `tune signal`, `unlock hatch`, `go hatch`, `quit`.",
            create: || Box::new(AmazeSilverstreamHost::new()),
        },
    ]
}

fn print_host_usage() {
    eprintln!("Usage: muddle-cli [--host <name>] [--list-hosts]");
}

fn print_hosts() {
    println!("Available MUDDLE hosts:");
    for registration in host_registry() {
        println!("  {} - {}", registration.name, registration.description);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_mock_labyrinth() {
        assert_eq!(
            parse_args(["muddle-cli"].into_iter().map(String::from)),
            Ok(CliAction::Run {
                host_name: "mock-labyrinth".to_string()
            })
        );
    }

    #[test]
    fn parses_named_hosts() {
        assert_eq!(
            parse_args(
                ["muddle-cli", "--host", "banish-pilgrim-loss"]
                    .into_iter()
                    .map(String::from)
            ),
            Ok(CliAction::Run {
                host_name: "banish-pilgrim-loss".to_string()
            })
        );
    }

    #[test]
    fn registers_banish_pilgrim_loss_host() {
        let registration = find_host("banish-pilgrim-loss").expect("BANISH spike is registered");
        assert_eq!(registration.name, "banish-pilgrim-loss");
    }

    #[test]
    fn registers_amaze_silverstream_host() {
        let registration = find_host("amaze-silverstream").expect("AMAZE spike is registered");
        assert_eq!(registration.name, "amaze-silverstream");
    }

    #[test]
    fn lists_hosts() {
        assert_eq!(
            parse_args(["muddle-cli", "--list-hosts"].into_iter().map(String::from)),
            Ok(CliAction::ListHosts)
        );
    }

    #[test]
    fn rejects_missing_host_names() {
        assert_eq!(
            parse_args(["muddle-cli", "--host"].into_iter().map(String::from)),
            Err("`--host` requires a host name.".to_string())
        );
    }
}
