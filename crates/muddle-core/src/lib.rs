use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleRoom {
    pub id: String,
    pub title: String,
    pub description: String,
    pub exits: Vec<MuddleExit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleExit {
    pub command: String,
    pub target_room: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleCommand {
    pub verb: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleTurn {
    pub room_id: String,
    pub command: MuddleCommand,
    pub response: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleSession {
    pub current_room: String,
    pub transcript: Vec<MuddleTurn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleSessionSave {
    pub current_room: String,
    pub commands: Vec<String>,
    pub host_checkpoint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleClientInfo {
    pub host: String,
    pub description: String,
    pub suggested_commands: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleClientSnapshot {
    pub host: String,
    pub description: String,
    pub suggested_commands: String,
    pub room: String,
    pub turns: usize,
    pub room_card: String,
    pub last_response: String,
    pub panels: MuddleClientPanels,
    pub commands: Vec<MuddleCommandHint>,
    pub history: Vec<MuddleClientHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleClientPanels {
    pub resources: Vec<MuddleResource>,
    pub inventory: Vec<MuddleInventoryItem>,
    pub map: Option<String>,
    pub objectives: Vec<String>,
    pub recent_log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleClientHistoryEntry {
    pub turn: usize,
    pub room: String,
    pub command: String,
    pub response: String,
}

pub trait MuddleClient {
    type Error;

    fn render_snapshot(&mut self, snapshot: &MuddleClientSnapshot) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleCommandOutcome {
    pub response: String,
    pub next_room: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MuddleError {
    InvalidStartRoom {
        room_id: String,
    },
    RoomNotFound {
        room_id: String,
    },
    MissingCommandTarget {
        room_id: String,
        verb: String,
    },
    UnknownCommand {
        room_id: String,
        command: MuddleCommand,
    },
    InvalidSessionSave {
        message: String,
    },
    InvalidHostCheckpoint {
        message: String,
    },
    ResumeRoomMismatch {
        expected_room: String,
        actual_room: String,
    },
}

pub trait MuddleHost {
    fn start_room(&self) -> &str;
    fn room(&self, room_id: &str) -> Option<&MuddleRoom>;
    fn resource_panel(&self) -> Vec<MuddleResource> {
        Vec::new()
    }
    fn inventory_panel(&self) -> Vec<MuddleInventoryItem> {
        Vec::new()
    }
    fn map_panel(&self, _current_room: &str) -> Option<String> {
        None
    }
    fn objective_panel(&self, _current_room: &str) -> Vec<String> {
        Vec::new()
    }
    fn command_panel(&self, _current_room: &str) -> Vec<MuddleCommandHint> {
        Vec::new()
    }
    fn export_checkpoint(&self) -> Option<String> {
        None
    }
    fn import_checkpoint(&mut self, checkpoint: &str) -> Result<(), MuddleError> {
        Err(MuddleError::InvalidHostCheckpoint {
            message: format!("host does not support checkpoints: {checkpoint}"),
        })
    }
    fn handle_command(
        &mut self,
        room_id: &str,
        command: &MuddleCommand,
    ) -> Result<MuddleCommandOutcome, MuddleError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleResource {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleInventoryItem {
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleCommandHint {
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleStaticHost {
    start_room: String,
    rooms: HashMap<String, MuddleRoom>,
}

impl MuddleCommand {
    pub fn parse(input: &str) -> Self {
        let mut parts = input.split_whitespace();
        let verb = parts.next().unwrap_or("look").to_ascii_lowercase();
        let target = parts.collect::<Vec<_>>().join(" ");
        let target = if target.is_empty() {
            None
        } else {
            Some(target)
        };

        Self { verb, target }
    }

    pub fn normalized(&self) -> String {
        match &self.target {
            Some(target) => format!("{} {}", self.verb, target),
            None => self.verb.clone(),
        }
    }
}

impl MuddleRoom {
    pub fn ascii_card(&self) -> String {
        let exits = self
            .exits
            .iter()
            .map(|exit| format!("{} -> {}", exit.command, exit.label))
            .collect::<Vec<_>>()
            .join(" | ");

        format!(
            "+ {title}\n| {description}\n| exits: {exits}",
            title = self.title,
            description = self.description
        )
    }
}

impl MuddleSession {
    pub fn new(start_room: impl Into<String>) -> Self {
        Self {
            current_room: start_room.into(),
            transcript: Vec::new(),
        }
    }

    pub fn record_turn(
        &mut self,
        command: MuddleCommand,
        response: impl Into<String>,
    ) -> &MuddleTurn {
        self.transcript.push(MuddleTurn {
            room_id: self.current_room.clone(),
            command,
            response: response.into(),
        });
        self.transcript.last().expect("turn was just recorded")
    }

    pub fn move_to(&mut self, room_id: impl Into<String>) {
        self.current_room = room_id.into();
    }

    pub fn for_host<H: MuddleHost + ?Sized>(host: &H) -> Result<Self, MuddleError> {
        let start_room = host.start_room();
        if host.room(start_room).is_none() {
            return Err(MuddleError::InvalidStartRoom {
                room_id: start_room.to_string(),
            });
        }

        Ok(Self::new(start_room))
    }

    pub fn play_turn<H: MuddleHost + ?Sized>(
        &mut self,
        host: &mut H,
        command: MuddleCommand,
    ) -> Result<&MuddleTurn, MuddleError> {
        if host.room(&self.current_room).is_none() {
            return Err(MuddleError::RoomNotFound {
                room_id: self.current_room.clone(),
            });
        }

        let outcome = host.handle_command(&self.current_room, &command)?;

        if let Some(next_room) = &outcome.next_room {
            if host.room(next_room).is_none() {
                return Err(MuddleError::RoomNotFound {
                    room_id: next_room.clone(),
                });
            }
        }

        let next_room = outcome.next_room;
        let turn_index = self.transcript.len();
        self.transcript.push(MuddleTurn {
            room_id: self.current_room.clone(),
            command,
            response: outcome.response,
        });

        if let Some(next_room) = next_room {
            self.move_to(next_room);
        }

        Ok(&self.transcript[turn_index])
    }

    pub fn save(&self) -> MuddleSessionSave {
        MuddleSessionSave {
            current_room: self.current_room.clone(),
            commands: self
                .transcript
                .iter()
                .map(|turn| turn.command.normalized())
                .collect(),
            host_checkpoint: None,
        }
    }

    pub fn save_for_host<H: MuddleHost + ?Sized>(&self, host: &H) -> MuddleSessionSave {
        MuddleSessionSave {
            host_checkpoint: host.export_checkpoint(),
            ..self.save()
        }
    }

    pub fn resume_for_host<H: MuddleHost + ?Sized>(
        host: &mut H,
        save: &MuddleSessionSave,
    ) -> Result<Self, MuddleError> {
        let mut session = Self::for_host(host)?;
        for command in &save.commands {
            session.play_turn(host, MuddleCommand::parse(command))?;
        }

        if session.current_room != save.current_room {
            return Err(MuddleError::ResumeRoomMismatch {
                expected_room: save.current_room.clone(),
                actual_room: session.current_room,
            });
        }

        if let Some(checkpoint) = &save.host_checkpoint {
            host.import_checkpoint(checkpoint)?;
        }

        Ok(session)
    }

    pub fn client_snapshot<H: MuddleHost + ?Sized>(
        &self,
        host: &H,
        info: MuddleClientInfo,
        last_response: impl Into<String>,
    ) -> MuddleClientSnapshot {
        let commands = host.command_panel(&self.current_room);
        MuddleClientSnapshot {
            host: info.host,
            description: info.description,
            suggested_commands: info.suggested_commands,
            room: self.current_room.clone(),
            turns: self.transcript.len(),
            room_card: host
                .room(&self.current_room)
                .map(MuddleRoom::ascii_card)
                .unwrap_or_else(|| format!("Room missing: {}", self.current_room)),
            last_response: last_response.into(),
            panels: MuddleClientPanels {
                resources: host.resource_panel(),
                inventory: host.inventory_panel(),
                map: host.map_panel(&self.current_room),
                objectives: host.objective_panel(&self.current_room),
                recent_log: self.recent_log_panel(3),
            },
            commands,
            history: self
                .transcript
                .iter()
                .enumerate()
                .map(|(index, turn)| MuddleClientHistoryEntry {
                    turn: index + 1,
                    room: turn.room_id.clone(),
                    command: turn.command.normalized(),
                    response: turn.response.clone(),
                })
                .collect(),
        }
    }

    pub fn recent_log_panel(&self, limit: usize) -> Vec<String> {
        self.transcript
            .iter()
            .rev()
            .take(limit)
            .map(|turn| format!("{} -> {}", turn.command.normalized(), turn.response))
            .collect()
    }
}

impl MuddleSessionSave {
    const HEADER: &'static str = "MUDDLE_SESSION_V1";

    pub fn encode(&self) -> String {
        let mut lines = vec![
            Self::HEADER.to_string(),
            format!("current_room={}", encode_field(&self.current_room)),
        ];
        if let Some(checkpoint) = &self.host_checkpoint {
            lines.push(format!("host_checkpoint={}", encode_field(checkpoint)));
        }
        lines.extend(
            self.commands
                .iter()
                .map(|command| format!("command={}", encode_field(command))),
        );
        lines.join("\n")
    }

    pub fn decode(input: &str) -> Result<Self, MuddleError> {
        let mut lines = input.lines();
        match lines.next() {
            Some(Self::HEADER) => {}
            _ => {
                return Err(MuddleError::InvalidSessionSave {
                    message: "missing MUDDLE_SESSION_V1 header".to_string(),
                });
            }
        }

        let current_room_line = lines
            .next()
            .ok_or_else(|| MuddleError::InvalidSessionSave {
                message: "missing current_room line".to_string(),
            })?;
        let current_room = current_room_line
            .strip_prefix("current_room=")
            .ok_or_else(|| MuddleError::InvalidSessionSave {
                message: "current_room line is malformed".to_string(),
            })
            .and_then(decode_field)?;

        let mut commands = Vec::new();
        let mut host_checkpoint = None;
        for line in lines {
            if let Some(encoded) = line.strip_prefix("command=") {
                commands.push(decode_field(encoded)?);
            } else if let Some(encoded) = line.strip_prefix("host_checkpoint=") {
                if host_checkpoint.is_some() {
                    return Err(MuddleError::InvalidSessionSave {
                        message: "duplicate host_checkpoint line".to_string(),
                    });
                }
                host_checkpoint = Some(decode_field(encoded)?);
            } else {
                return Err(MuddleError::InvalidSessionSave {
                    message: format!("unexpected save line `{line}`"),
                });
            }
        }

        Ok(Self {
            current_room,
            commands,
            host_checkpoint,
        })
    }
}

impl MuddleCommandOutcome {
    pub fn stay(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            next_room: None,
        }
    }

    pub fn move_to(response: impl Into<String>, next_room: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            next_room: Some(next_room.into()),
        }
    }
}

impl MuddleStaticHost {
    pub fn try_new(
        start_room: impl Into<String>,
        rooms: impl IntoIterator<Item = MuddleRoom>,
    ) -> Result<Self, MuddleError> {
        let start_room = start_room.into();
        let rooms = rooms
            .into_iter()
            .map(|room| (room.id.clone(), room))
            .collect::<HashMap<_, _>>();

        if !rooms.contains_key(&start_room) {
            return Err(MuddleError::InvalidStartRoom {
                room_id: start_room,
            });
        }

        Ok(Self { start_room, rooms })
    }
}

fn encode_field(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\n', "%0A")
        .replace('\r', "%0D")
        .replace('=', "%3D")
}

fn decode_field(value: &str) -> Result<String, MuddleError> {
    let mut decoded = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            decoded.push(ch);
            continue;
        }

        let hi = chars
            .next()
            .ok_or_else(|| MuddleError::InvalidSessionSave {
                message: "incomplete percent escape".to_string(),
            })?;
        let lo = chars
            .next()
            .ok_or_else(|| MuddleError::InvalidSessionSave {
                message: "incomplete percent escape".to_string(),
            })?;
        match (hi, lo) {
            ('2', '5') => decoded.push('%'),
            ('0', 'A') => decoded.push('\n'),
            ('0', 'D') => decoded.push('\r'),
            ('3', 'D') => decoded.push('='),
            _ => {
                return Err(MuddleError::InvalidSessionSave {
                    message: format!("unknown percent escape %{hi}{lo}"),
                });
            }
        }
    }
    Ok(decoded)
}

impl MuddleHost for MuddleStaticHost {
    fn start_room(&self) -> &str {
        &self.start_room
    }

    fn room(&self, room_id: &str) -> Option<&MuddleRoom> {
        self.rooms.get(room_id)
    }

    fn handle_command(
        &mut self,
        room_id: &str,
        command: &MuddleCommand,
    ) -> Result<MuddleCommandOutcome, MuddleError> {
        let room = self
            .room(room_id)
            .ok_or_else(|| MuddleError::RoomNotFound {
                room_id: room_id.to_string(),
            })?;

        if command.verb == "look" {
            return Ok(MuddleCommandOutcome::stay(room.ascii_card()));
        }

        if command.verb == "go" && command.target.is_none() {
            return Err(MuddleError::MissingCommandTarget {
                room_id: room_id.to_string(),
                verb: command.verb.clone(),
            });
        }

        let normalized = command.normalized();
        if let Some(exit) = room.exits.iter().find(|exit| exit.command == normalized) {
            return Ok(MuddleCommandOutcome::move_to(
                format!("You move to {}.", exit.label),
                exit.target_room.clone(),
            ));
        }

        Err(MuddleError::UnknownCommand {
            room_id: room_id.to_string(),
            command: command.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_player_commands() {
        assert_eq!(
            MuddleCommand::parse("go north"),
            MuddleCommand {
                verb: "go".to_string(),
                target: Some("north".to_string())
            }
        );
        assert_eq!(MuddleCommand::parse("").verb, "look");
    }

    #[test]
    fn renders_ascii_room_cards() {
        let room = MuddleRoom {
            id: "campfire".to_string(),
            title: "Campfire".to_string(),
            description: "A shared starting room for playable sims.".to_string(),
            exits: vec![MuddleExit {
                command: "go road".to_string(),
                target_room: "pilgrim-road".to_string(),
                label: "Pilgrim Road".to_string(),
            }],
        };

        let card = room.ascii_card();
        assert!(card.contains("+ Campfire"));
        assert!(card.contains("go road -> Pilgrim Road"));
    }

    #[test]
    fn records_transcript_turns() {
        let mut session = MuddleSession::new("campfire");
        session.record_turn(MuddleCommand::parse("look"), "You see exits.");
        session.move_to("pilgrim-road");

        assert_eq!(session.transcript.len(), 1);
        assert_eq!(session.current_room, "pilgrim-road");
    }

    #[test]
    fn encodes_and_decodes_session_saves() {
        let save = MuddleSessionSave {
            current_room: "north=gate".to_string(),
            commands: vec!["look".to_string(), "go north%gate".to_string()],
            host_checkpoint: Some("embers=1;glyphs=true".to_string()),
        };

        assert_eq!(MuddleSessionSave::decode(&save.encode()), Ok(save));
    }

    #[test]
    fn decodes_command_replay_saves_without_host_checkpoints() {
        let save = MuddleSessionSave::decode("MUDDLE_SESSION_V1\ncurrent_room=camp\ncommand=look")
            .expect("legacy save decodes");

        assert_eq!(
            save,
            MuddleSessionSave {
                current_room: "camp".to_string(),
                commands: vec!["look".to_string()],
                host_checkpoint: None,
            }
        );
    }

    #[test]
    fn resumes_session_by_replaying_commands() {
        let mut host = MuddleStaticHost::try_new(
            "campfire",
            [
                MuddleRoom {
                    id: "campfire".to_string(),
                    title: "Campfire".to_string(),
                    description: "Start.".to_string(),
                    exits: vec![MuddleExit {
                        command: "go road".to_string(),
                        target_room: "road".to_string(),
                        label: "Road".to_string(),
                    }],
                },
                MuddleRoom {
                    id: "road".to_string(),
                    title: "Road".to_string(),
                    description: "Away.".to_string(),
                    exits: Vec::new(),
                },
            ],
        )
        .expect("host is valid");

        let mut session = MuddleSession::for_host(&host).expect("session starts");
        session
            .play_turn(&mut host, MuddleCommand::parse("go road"))
            .expect("turn plays");

        let save = session.save();
        let mut fresh_host = MuddleStaticHost::try_new(
            "campfire",
            [
                MuddleRoom {
                    id: "campfire".to_string(),
                    title: "Campfire".to_string(),
                    description: "Start.".to_string(),
                    exits: vec![MuddleExit {
                        command: "go road".to_string(),
                        target_room: "road".to_string(),
                        label: "Road".to_string(),
                    }],
                },
                MuddleRoom {
                    id: "road".to_string(),
                    title: "Road".to_string(),
                    description: "Away.".to_string(),
                    exits: Vec::new(),
                },
            ],
        )
        .expect("fresh host is valid");
        let resumed =
            MuddleSession::resume_for_host(&mut fresh_host, &save).expect("session resumes");

        assert_eq!(resumed.current_room, "road");
        assert_eq!(resumed.transcript.len(), 1);
    }

    #[test]
    fn plays_turns_against_static_hosts() {
        let mut host = fixture_host();
        let mut session = MuddleSession::for_host(&host).expect("fixture host has a start room");

        let turn = session
            .play_turn(&mut host, MuddleCommand::parse("look"))
            .expect("look is handled by static hosts");
        assert!(turn.response.contains("+ Campfire"));

        let turn = session
            .play_turn(&mut host, MuddleCommand::parse("go road"))
            .expect("exit command moves to target room");
        assert_eq!(turn.room_id, "campfire");
        assert_eq!(turn.response, "You move to Pilgrim Road.");
        assert_eq!(session.current_room, "pilgrim-road");
        assert_eq!(session.transcript.len(), 2);
    }

    #[test]
    fn surfaces_unknown_host_commands() {
        let mut host = fixture_host();
        let mut session = MuddleSession::for_host(&host).expect("fixture host has a start room");

        let error = session
            .play_turn(&mut host, MuddleCommand::parse("dance wildly"))
            .expect_err("unknown commands should be explicit");

        assert_eq!(
            error,
            MuddleError::UnknownCommand {
                room_id: "campfire".to_string(),
                command: MuddleCommand {
                    verb: "dance".to_string(),
                    target: Some("wildly".to_string())
                }
            }
        );
        assert!(session.transcript.is_empty());
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
                    description: "A road that can be owned by a host repo.".to_string(),
                    exits: Vec::new(),
                },
            ],
        )
        .expect("fixture rooms include the start room")
    }
}
