use std::io::{self, Write};

use muddle_core::{MuddleCommand, MuddleExit, MuddleRoom, MuddleSession, MuddleStaticHost};

fn main() -> io::Result<()> {
    let mut host = fixture_host();
    let mut session = MuddleSession::for_host(&host).expect("fixture host must have a start room");

    println!("MUDDLE CLI");
    println!("Type `look`, `go road`, `go camp`, or `quit`.");

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

fn fixture_host() -> MuddleStaticHost {
    MuddleStaticHost::try_new(
        "campfire",
        vec![
            MuddleRoom {
                id: "campfire".to_string(),
                title: "Campfire".to_string(),
                description: "A shared starting room for playable sims.".to_string(),
                exits: vec![MuddleExit {
                    command: "go road".to_string(),
                    target_room: "pilgrim-road".to_string(),
                    label: "Pilgrim Road".to_string(),
                }],
            },
            MuddleRoom {
                id: "pilgrim-road".to_string(),
                title: "Pilgrim Road".to_string(),
                description: "A road owned by a host repo in real integrations.".to_string(),
                exits: vec![MuddleExit {
                    command: "go camp".to_string(),
                    target_room: "campfire".to_string(),
                    label: "Campfire".to_string(),
                }],
            },
        ],
    )
    .expect("fixture rooms include the start room")
}
