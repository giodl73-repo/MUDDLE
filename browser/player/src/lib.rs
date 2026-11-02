//! Browser transport only. Hosts and shared session own all play/persistence rules.
use muddle_core::*;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
#[cfg(feature = "player-entry")]
use wasm_bindgen::prelude::*;

const REQUEST_BYTES: usize = 524288;
const REPORT_BYTES: usize = 2097152;
const SAVE_BYTES: usize = 131072;
const WORLD_BYTES: usize = 65536;
const ATTEMPTS: usize = 128;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    host: String,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    save_text: Option<String>,
    #[serde(default)]
    world_json: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct World {
    start_room: String,
    rooms: Vec<Room>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Room {
    id: String,
    title: String,
    description: String,
    exits: Vec<Exit>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exit {
    command: String,
    target_room: String,
    label: String,
}

fn bounded(text: &str, max: usize, name: &str) -> Result<(), String> {
    if text.len() > max {
        Err(format!(
            "Browser admission: {name} exceeds {max} UTF-8 bytes"
        ))
    } else {
        Ok(())
    }
}

fn authored(text: &str) -> Result<MuddleStaticHost, String> {
    bounded(text, WORLD_BYTES, "world")?;
    let w: World = serde_json::from_str(text).map_err(|e| format!("World JSON: {e}"))?;
    if w.rooms.is_empty() || w.rooms.len() > 32 {
        return Err("Browser admission: world needs 1..32 rooms".into());
    }
    let mut ids = HashSet::new();
    bounded(&w.start_room, 4096, "start room")?;
    let mut exits = 0;
    for r in &w.rooms {
        for s in [&r.id, &r.title, &r.description] {
            bounded(s, 4096, "room field")?;
        }
        if r.id.is_empty() || !ids.insert(&r.id) {
            return Err("Browser admission: room IDs must be nonempty and unique".into());
        }
        exits += r.exits.len();
        let mut commands = HashSet::new();
        for e in &r.exits {
            for s in [&e.command, &e.target_room, &e.label] {
                bounded(s, 4096, "exit field")?;
            }
            if e.command.is_empty()
                || !commands.insert(MuddleCommand::parse(&e.command).normalized())
            {
                return Err("Browser admission: exit commands must be nonempty and unique after native normalization".into());
            }
        }
    }
    if exits > 128
        || w.rooms
            .iter()
            .flat_map(|r| &r.exits)
            .any(|e| !ids.contains(&e.target_room))
    {
        return Err("Browser admission: at most 128 exits; all targets must exist".into());
    }
    let rooms = w.rooms.into_iter().map(|r| MuddleRoom {
        id: r.id,
        title: r.title,
        description: r.description,
        exits: r
            .exits
            .into_iter()
            .map(|e| MuddleExit {
                command: e.command,
                target_room: e.target_room,
                label: e.label,
            })
            .collect(),
    });
    MuddleStaticHost::try_new(w.start_room, rooms).map_err(|e| format!("{e:?}"))
}

fn control(c: &MuddleClientControl) -> Value {
    json!({"id":c.id,"kind":format!("{:?}",c.kind),"label":c.label,"text":c.text,"command":c.command,
        "image":c.image.as_ref().map(|i|json!({"source":i.source,"alt":i.alt})),"children":c.children.iter().map(control).collect::<Vec<_>>()})
}
fn visual(v: &MuddleVisualNode) -> Value {
    json!({"id":v.id,"kind":format!("{:?}",v.kind),"label":v.label,"text":v.text,"layer":v.layer,"x":v.x,"y":v.y,"width":v.width,"height":v.height,
        "sprite":v.sprite.as_ref().map(|s|json!({"source":s.source,"alt":s.alt,"frame":s.frame,"animation":s.animation})),"children":v.children.iter().map(visual).collect::<Vec<_>>()})
}
/// Original browser transport for the native shared client snapshot.
///
/// Product hosts may reuse this function from Rust without copying the field,
/// control or visual-node mapping. It does not validate product state, fetch
/// images or execute commands. Disable default `player-entry` when embedding
/// this crate to avoid exporting the standalone demo player's WASM entry point.
pub fn snapshot_json(s: &MuddleClientSnapshot) -> Value {
    json!({"host":s.host,"description":s.description,"suggested_commands":s.suggested_commands,"room":s.room,"turns":s.turns,"room_card":s.room_card,"last_response":s.last_response,
      "panels":{"resources":s.panels.resources.iter().map(|r|json!({"label":r.label,"value":r.value})).collect::<Vec<_>>(),
        "inventory":s.panels.inventory.iter().map(|i|json!({"label":i.label,"detail":i.detail})).collect::<Vec<_>>(),"map":s.panels.map,"objectives":s.panels.objectives,"recent_log":s.panels.recent_log},
      "commands":s.commands.iter().map(|c|json!({"command":c.command,"description":c.description})).collect::<Vec<_>>(),
      "history":s.history.iter().map(|h|json!({"turn":h.turn,"room":h.room,"command":h.command,"response":h.response})).collect::<Vec<_>>(),
      "controls":s.controls.iter().map(control).collect::<Vec<_>>(),"visual_nodes":s.visual_nodes.iter().map(visual).collect::<Vec<_>>()})
}

fn play(r: Request) -> Result<Value, (String, String)> {
    let refusal = |s| ("browser_refusal".into(), s);
    if r.commands.len() > ATTEMPTS {
        return Err(refusal("At most 128 command attempts".into()));
    }
    for c in &r.commands {
        bounded(c, 1024, "command").map_err(refusal)?;
    }
    // Explicit host registration only: never load code or follow supplied paths.
    let mut host: Box<dyn MuddleHost> = match r.host.as_str() {
        "mock-labyrinth" => Box::new(muddle_mock_sim::MuddleMockSimHost::new()),
        "banish-pilgrim-loss" => Box::new(muddle_banish_spike::BanishPilgrimLossHost::new()),
        "amaze-silverstream" => Box::new(muddle_amaze_spike::AmazeSilverstreamHost::new()),
        "authored-rooms" => Box::new(
            authored(
                r.world_json
                    .as_deref()
                    .ok_or_else(|| refusal("Authored host requires world_json".into()))?,
            )
            .map_err(refusal)?,
        ),
        _ => {
            return Err(refusal(
                "Unknown host; choose an explicit registration".into(),
            ))
        }
    };
    if r.host != "authored-rooms" && r.world_json.is_some() {
        return Err(refusal(
            "world_json belongs only to authored-rooms; no ignored world input".into(),
        ));
    }
    let native = |e| ("native_error".into(), format!("{e:?}"));
    let mut session = if let Some(text) = r.save_text {
        bounded(&text, SAVE_BYTES, "save").map_err(refusal)?;
        let save = MuddleSessionSave::decode(&text).map_err(native)?;
        if save.commands.len() + r.commands.len() > ATTEMPTS {
            return Err(refusal("Save commands plus new attempts exceed 128".into()));
        }
        for c in &save.commands {
            bounded(c, 1024, "saved command").map_err(refusal)?;
        }
        if let Some(c) = &save.host_checkpoint {
            bounded(c, 65536, "host checkpoint").map_err(refusal)?;
        }
        MuddleSession::resume_for_host(host.as_mut(), &save).map_err(native)?
    } else {
        MuddleSession::for_host(host.as_ref()).map_err(native)?
    };
    let mut attempts = Vec::new();
    let mut last = session
        .transcript
        .last()
        .map(|t| t.response.clone())
        .unwrap_or_else(|| "Ready. Choose a suggested command or type your own.".into());
    for input in r.commands {
        let command = MuddleCommand::parse(&input);
        let normalized = command.normalized();
        match session.play_turn(host.as_mut(), command) {
            Ok(t) => {
                last = t.response.clone();
                attempts.push(
                    json!({"input":input,"normalized":normalized,"kind":"turn","response":last}),
                );
            }
            Err(e) => {
                last = format!("{e:?}");
                attempts.push(json!({"input":input,"normalized":normalized,"kind":"native_error","error":last}));
            }
        }
    }
    let info=MuddleClientInfo{host:r.host.clone(),description:"MUDDLE browser-mounted host; built-ins are in-repo demonstration adapters.".into(),suggested_commands:"Use host hints or type a command. Authored static rooms support look and exact exit commands.".into()};
    let state = session.client_snapshot(host.as_ref(), info, last);
    let transcript = muddle_cli::render_transcript(
        muddle_cli::MuddleCliHostInfo {
            name: match r.host.as_str() {
                "mock-labyrinth" => "mock-labyrinth",
                "banish-pilgrim-loss" => "banish-pilgrim-loss",
                "amaze-silverstream" => "amaze-silverstream",
                _ => "authored-rooms",
            },
            description: "",
            suggested_commands: "",
        },
        &session,
    );
    Ok(
        json!({"kind":"report","snapshot":snapshot_json(&state),"attempts":attempts,"save_text":session.save_for_host(host.as_ref()).encode(),"replay_save_text":session.save().encode(),"transcript_text":transcript,
      "checkpoint_semantics":"Native resume replays commands, verifies final room, then imports host checkpoint. Saves do not encode host identity. Select the matching host/world explicitly; acceptance does not prove identity or provenance."}),
    )
}

/// Exact JSON transport shared by native control CLI and WASM.
#[cfg_attr(feature = "player-entry", wasm_bindgen)]
pub fn evaluate(input: &str) -> String {
    let hash = format!("{:x}", Sha256::digest(input.as_bytes()));
    let result = if input.len() > REQUEST_BYTES {
        Err(("browser_refusal".into(), "Request exceeds 512 KiB".into()))
    } else {
        serde_json::from_str::<Request>(input)
            .map_err(|e| ("input_error".into(), format!("{e}")))
            .and_then(play)
    };
    let mut value = match result {
        Ok(v) => v,
        Err((kind, error)) => json!({"kind":kind,"error":error}),
    };
    value["request_sha256"] = json!(hash);
    let report = serde_json::to_string(&value).expect("finite JSON values");
    if report.len() > REPORT_BYTES {
        return json!({"kind":"browser_refusal","error":"Report exceeds 2 MiB; previous session retained","request_sha256":hash}).to_string();
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(v: Value) -> Value {
        serde_json::from_str(&evaluate(&v.to_string())).unwrap()
    }
    #[test]
    fn mock_play_resume_and_native_save_bytes() {
        let r = run(
            json!({"host":"mock-labyrinth","commands":["look","gather ember","go antechamber","inspect glyphs","use ember","go vault"]}),
        );
        assert_eq!(r["kind"], "report");
        assert_eq!(r["snapshot"]["room"], "echo-vault");
        assert!(r["attempts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|a| a["kind"] == "turn"));
        let s = run(json!({"host":"mock-labyrinth","save_text":r["save_text"]}));
        assert_eq!(r["snapshot"]["room"], s["snapshot"]["room"]);
        assert_eq!(r["transcript_text"], s["transcript_text"]);
        assert_eq!(r["save_text"], s["save_text"]);
    }
    #[test]
    fn invalid_commands_are_attempts_not_transcript_turns() {
        let r = run(json!({"host":"mock-labyrinth","commands":["not-a-verb","look"]}));
        assert_eq!(r["attempts"][0]["kind"], "native_error");
        assert_eq!(r["snapshot"]["turns"], 1);
        assert_eq!(r["snapshot"]["history"][0]["command"], "look");
    }
    #[test]
    fn bad_save_is_native_rejection_and_bounds_are_separate() {
        assert_eq!(
            run(json!({"host":"mock-labyrinth","save_text":"broken"}))["kind"],
            "native_error"
        );
        assert_eq!(
            run(json!({"host":"mock-labyrinth","commands":vec!["look";129]}))["kind"],
            "browser_refusal"
        );
        assert_eq!(
            run(json!({"host":"mock-labyrinth","extra":1}))["kind"],
            "input_error"
        );
    }
    #[test]
    fn authored_graph_mounts_native_static_host_and_rejects_duplicate_ids() {
        let world=json!({"start_room":"a","rooms":[{"id":"a","title":"A","description":"Own room","exits":[{"command":"go next","target_room":"b","label":"B"}]},{"id":"b","title":"B","description":"Other room","exits":[]}]}).to_string();
        let r = run(json!({"host":"authored-rooms","world_json":world,"commands":["go next"]}));
        assert_eq!(r["snapshot"]["room"], "b");
        let mut bad: Value = serde_json::from_str(&world).unwrap();
        bad["rooms"][1]["id"] = json!("a");
        assert!(authored(&bad.to_string()).is_err());
    }
}
