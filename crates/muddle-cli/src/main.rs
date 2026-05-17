use std::io::{self, Write};

use muddle_core::{MuddleCommand, MuddleSession};
use muddle_mock_sim::MuddleMockSimHost;

fn main() -> io::Result<()> {
    let mut host = MuddleMockSimHost::new();
    let mut session = MuddleSession::for_host(&host).expect("fixture host must have a start room");

    println!("MUDDLE CLI");
    println!("Labyrinth mock sim mounted.");
    println!("Try: `look`, `gather ember`, `go antechamber`, `inspect glyphs`, `use ember`, `go vault`, `quit`.");

    loop {
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

        match session.play_turn(&mut host, MuddleCommand::parse(input)) {
            Ok(turn) => println!("{}", turn.response),
            Err(error) => println!("Command failed: {error:?}"),
        }
    }

    Ok(())
}
