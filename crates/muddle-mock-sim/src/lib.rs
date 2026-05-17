use std::collections::HashMap;

use muddle_core::{
    MuddleCommand, MuddleCommandHint, MuddleCommandOutcome, MuddleError, MuddleExit, MuddleHost,
    MuddleInventoryItem, MuddleResource, MuddleRoom,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMockSimHost {
    start_room: String,
    rooms: HashMap<String, MuddleRoom>,
    state: MuddleMockSimState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuddleMockSimState {
    pub ember_count: u8,
    pub glyphs_read: bool,
    pub gate_unlocked: bool,
}

impl Default for MuddleMockSimState {
    fn default() -> Self {
        Self {
            ember_count: 0,
            glyphs_read: false,
            gate_unlocked: false,
        }
    }
}

impl Default for MuddleMockSimHost {
    fn default() -> Self {
        Self::new()
    }
}

impl MuddleMockSimHost {
    pub fn new() -> Self {
        let rooms = [
            MuddleRoom {
                id: "labyrinth-camp".to_string(),
                title: "Labyrinth Camp".to_string(),
                description: "A tiny BANISH-like staging camp at the maze edge.".to_string(),
                exits: vec![MuddleExit {
                    command: "go antechamber".to_string(),
                    target_room: "glyph-antechamber".to_string(),
                    label: "Glyph Antechamber".to_string(),
                }],
            },
            MuddleRoom {
                id: "glyph-antechamber".to_string(),
                title: "Glyph Antechamber".to_string(),
                description: "An AMAZE-like labyrinth room with glyphs and a sealed gate."
                    .to_string(),
                exits: vec![
                    MuddleExit {
                        command: "go camp".to_string(),
                        target_room: "labyrinth-camp".to_string(),
                        label: "Labyrinth Camp".to_string(),
                    },
                    MuddleExit {
                        command: "go vault".to_string(),
                        target_room: "echo-vault".to_string(),
                        label: "Echo Vault".to_string(),
                    },
                ],
            },
            MuddleRoom {
                id: "echo-vault".to_string(),
                title: "Echo Vault".to_string(),
                description: "A success room proving stateful host-owned rules.".to_string(),
                exits: vec![MuddleExit {
                    command: "go antechamber".to_string(),
                    target_room: "glyph-antechamber".to_string(),
                    label: "Glyph Antechamber".to_string(),
                }],
            },
        ]
        .into_iter()
        .map(|room| (room.id.clone(), room))
        .collect();

        Self {
            start_room: "labyrinth-camp".to_string(),
            rooms,
            state: MuddleMockSimState::default(),
        }
    }

    pub fn state(&self) -> &MuddleMockSimState {
        &self.state
    }

    fn look(&self, room_id: &str) -> Result<MuddleCommandOutcome, MuddleError> {
        let room = self
            .room(room_id)
            .ok_or_else(|| MuddleError::RoomNotFound {
                room_id: room_id.to_string(),
            })?;

        Ok(MuddleCommandOutcome::stay(format!(
            "{}\n| sim: embers={} glyphs_read={} gate_unlocked={}",
            room.ascii_card(),
            self.state.ember_count,
            self.state.glyphs_read,
            self.state.gate_unlocked
        )))
    }

    fn unknown(&self, room_id: &str, command: &MuddleCommand) -> MuddleError {
        MuddleError::UnknownCommand {
            room_id: room_id.to_string(),
            command: command.clone(),
        }
    }
}

impl MuddleHost for MuddleMockSimHost {
    fn start_room(&self) -> &str {
        &self.start_room
    }

    fn room(&self, room_id: &str) -> Option<&MuddleRoom> {
        self.rooms.get(room_id)
    }

    fn resource_panel(&self) -> Vec<MuddleResource> {
        vec![
            MuddleResource {
                label: "embers".to_string(),
                value: self.state.ember_count.to_string(),
            },
            MuddleResource {
                label: "glyphs".to_string(),
                value: if self.state.glyphs_read {
                    "read".to_string()
                } else {
                    "unread".to_string()
                },
            },
            MuddleResource {
                label: "gate".to_string(),
                value: if self.state.gate_unlocked {
                    "open".to_string()
                } else {
                    "sealed".to_string()
                },
            },
        ]
    }

    fn inventory_panel(&self) -> Vec<MuddleInventoryItem> {
        let mut items = vec![MuddleInventoryItem {
            label: "map scrap".to_string(),
            detail: "route to the glyph gate".to_string(),
        }];

        if self.state.ember_count > 0 {
            items.push(MuddleInventoryItem {
                label: "banked ember".to_string(),
                detail: "opens one sealed gate".to_string(),
            });
        }

        if self.state.glyphs_read {
            items.push(MuddleInventoryItem {
                label: "glyph clue".to_string(),
                detail: "feed an ember to wake the gate".to_string(),
            });
        }

        items
    }

    fn map_panel(&self, current_room: &str) -> Option<String> {
        Some(format!(
            "{} Labyrinth Camp -- {} Glyph Antechamber -- {} Echo Vault",
            marker(current_room, "labyrinth-camp"),
            marker(current_room, "glyph-antechamber"),
            marker(current_room, "echo-vault")
        ))
    }

    fn objective_panel(&self, current_room: &str) -> Vec<String> {
        match current_room {
            "labyrinth-camp" if self.state.ember_count == 0 => {
                vec!["Gather an ember before entering the maze.".to_string()]
            }
            "labyrinth-camp" => vec!["Carry the ember into the glyph antechamber.".to_string()],
            "glyph-antechamber" if !self.state.glyphs_read => {
                vec!["Read the glyphs to learn how the sealed gate works.".to_string()]
            }
            "glyph-antechamber" if !self.state.gate_unlocked => {
                vec!["Use the ember to open the vault gate.".to_string()]
            }
            "glyph-antechamber" => vec!["Enter the echo vault.".to_string()],
            "echo-vault" => vec!["Labyrinth complete; review the transcript.".to_string()],
            _ => Vec::new(),
        }
    }

    fn command_panel(&self, current_room: &str) -> Vec<MuddleCommandHint> {
        let mut commands = vec![MuddleCommandHint {
            command: "look".to_string(),
            description: "Show the current room card.".to_string(),
        }];

        match current_room {
            "labyrinth-camp" => {
                commands.push(MuddleCommandHint {
                    command: "gather ember".to_string(),
                    description: "Collect the resource used to open the gate.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "go antechamber".to_string(),
                    description: "Enter the glyph antechamber.".to_string(),
                });
            }
            "glyph-antechamber" => {
                commands.push(MuddleCommandHint {
                    command: "inspect glyphs".to_string(),
                    description: "Read the lock clue.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "use ember".to_string(),
                    description: "Try to power the sealed gate.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "go vault".to_string(),
                    description: "Enter the vault if it is open.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "go camp".to_string(),
                    description: "Return to camp.".to_string(),
                });
            }
            "echo-vault" => {
                commands.push(MuddleCommandHint {
                    command: "go antechamber".to_string(),
                    description: "Return to the antechamber.".to_string(),
                });
            }
            _ => {}
        }

        commands
    }

    fn handle_command(
        &mut self,
        room_id: &str,
        command: &MuddleCommand,
    ) -> Result<MuddleCommandOutcome, MuddleError> {
        match (room_id, command.normalized().as_str()) {
            (_, "look") | (_, "status") => self.look(room_id),
            ("labyrinth-camp", "gather ember") if self.state.ember_count == 0 => {
                self.state.ember_count = 1;
                Ok(MuddleCommandOutcome::stay(
                    "You bank one ember for the sealed gate.",
                ))
            }
            ("labyrinth-camp", "gather ember") => Ok(MuddleCommandOutcome::stay(
                "The campfire is low; you already gathered the useful ember.",
            )),
            ("labyrinth-camp", "go antechamber") => Ok(MuddleCommandOutcome::move_to(
                "You carry the camp signal into the labyrinth antechamber.",
                "glyph-antechamber",
            )),
            ("glyph-antechamber", "go camp") => Ok(MuddleCommandOutcome::move_to(
                "You return to the labyrinth camp.",
                "labyrinth-camp",
            )),
            ("glyph-antechamber", "inspect glyphs") => {
                self.state.glyphs_read = true;
                Ok(MuddleCommandOutcome::stay(
                    "The glyphs say: feed an ember to wake the gate.",
                ))
            }
            ("glyph-antechamber", "use ember") if !self.state.glyphs_read => {
                Ok(MuddleCommandOutcome::stay(
                    "The gate rejects the ember; the glyph sequence is still unread.",
                ))
            }
            ("glyph-antechamber", "use ember") if self.state.ember_count == 0 => Ok(
                MuddleCommandOutcome::stay("You need an ember from camp before the gate can open."),
            ),
            ("glyph-antechamber", "use ember") => {
                self.state.ember_count -= 1;
                self.state.gate_unlocked = true;
                Ok(MuddleCommandOutcome::stay(
                    "The ember catches in the glyph seam. The vault gate opens.",
                ))
            }
            ("glyph-antechamber", "go vault") if self.state.gate_unlocked => {
                Ok(MuddleCommandOutcome::move_to(
                    "You step through the opened vault gate.",
                    "echo-vault",
                ))
            }
            ("glyph-antechamber", "go vault") => Ok(MuddleCommandOutcome::stay(
                "The vault gate is sealed. Read the glyphs and power it first.",
            )),
            ("echo-vault", "go antechamber") => Ok(MuddleCommandOutcome::move_to(
                "You leave the echo vault and return to the antechamber.",
                "glyph-antechamber",
            )),
            _ => Err(self.unknown(room_id, command)),
        }
    }
}

fn marker(current_room: &str, room_id: &str) -> &'static str {
    if current_room == room_id {
        "@"
    } else {
        "o"
    }
}

#[cfg(test)]
mod tests {
    use muddle_core::MuddleSession;

    use super::*;

    #[test]
    fn plays_stateful_mock_sim_path() {
        let mut host = MuddleMockSimHost::new();
        let mut session = MuddleSession::for_host(&host).expect("mock sim has a start room");

        session
            .play_turn(&mut host, MuddleCommand::parse("gather ember"))
            .expect("camp can gather an ember");
        session
            .play_turn(&mut host, MuddleCommand::parse("go antechamber"))
            .expect("camp exits to antechamber");
        session
            .play_turn(&mut host, MuddleCommand::parse("inspect glyphs"))
            .expect("ruins can inspect glyphs");
        session
            .play_turn(&mut host, MuddleCommand::parse("use ember"))
            .expect("ember opens the gate after glyphs are read");
        session
            .play_turn(&mut host, MuddleCommand::parse("go vault"))
            .expect("open gate allows vault movement");

        assert_eq!(session.current_room, "echo-vault");
        assert_eq!(session.transcript.len(), 5);
        assert_eq!(
            host.state(),
            &MuddleMockSimState {
                ember_count: 0,
                glyphs_read: true,
                gate_unlocked: true
            }
        );
        assert_eq!(host.resource_panel()[2].value, "open");
        assert!(host
            .map_panel(&session.current_room)
            .unwrap()
            .contains("@ Echo Vault"));
        assert_eq!(
            host.objective_panel(&session.current_room),
            vec!["Labyrinth complete; review the transcript.".to_string()]
        );
    }

    #[test]
    fn blocks_vault_until_host_state_unlocks_it() {
        let mut host = MuddleMockSimHost::new();
        let mut session = MuddleSession::for_host(&host).expect("mock sim has a start room");

        session
            .play_turn(&mut host, MuddleCommand::parse("go antechamber"))
            .expect("camp exits to antechamber");
        let response = session
            .play_turn(&mut host, MuddleCommand::parse("go vault"))
            .expect("locked gate returns a host response")
            .response
            .clone();

        assert_eq!(session.current_room, "glyph-antechamber");
        assert!(response.contains("sealed"));
    }
}
