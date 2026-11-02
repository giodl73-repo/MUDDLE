//! External Rust consumer proof: public serializer and existing player agree.
use muddle_browser_player::{evaluate, snapshot_json};
use muddle_core::{MuddleClientInfo, MuddleCommand, MuddleSession};

#[test]
fn public_snapshot_transport_matches_the_existing_player() {
    let mut host = muddle_mock_sim::MuddleMockSimHost::new();
    let mut session = MuddleSession::for_host(&host).unwrap();
    let response = session
        .play_turn(&mut host, MuddleCommand::parse("look"))
        .unwrap()
        .response
        .clone();
    let snapshot = session.client_snapshot(
        &host,
        MuddleClientInfo {
            host: "mock-labyrinth".into(),
            description: "MUDDLE browser-mounted host; built-ins are in-repo demonstration adapters.".into(),
            suggested_commands: "Use host hints or type a command. Authored static rooms support look and exact exit commands.".into(),
        },
        response,
    );
    let existing: serde_json::Value = serde_json::from_str(&evaluate(
        r#"{"host":"mock-labyrinth","commands":["look"]}"#,
    ))
    .unwrap();
    assert_eq!(snapshot_json(&snapshot), existing["snapshot"]);
}
