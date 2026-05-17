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
}

pub trait MuddleHost {
    fn start_room(&self) -> &str;
    fn room(&self, room_id: &str) -> Option<&MuddleRoom>;
    fn handle_command(
        &mut self,
        room_id: &str,
        command: &MuddleCommand,
    ) -> Result<MuddleCommandOutcome, MuddleError>;
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

    pub fn for_host(host: &impl MuddleHost) -> Result<Self, MuddleError> {
        let start_room = host.start_room();
        if host.room(start_room).is_none() {
            return Err(MuddleError::InvalidStartRoom {
                room_id: start_room.to_string(),
            });
        }

        Ok(Self::new(start_room))
    }

    pub fn play_turn(
        &mut self,
        host: &mut impl MuddleHost,
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
