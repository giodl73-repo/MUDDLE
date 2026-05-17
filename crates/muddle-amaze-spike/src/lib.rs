use std::collections::HashMap;

use muddle_core::{
    MuddleCommand, MuddleCommandHint, MuddleCommandOutcome, MuddleError, MuddleExit, MuddleHost,
    MuddleResource, MuddleRoom,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmazeSilverstreamHost {
    start_room: String,
    rooms: HashMap<String, MuddleRoom>,
    state: AmazeSilverstreamState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmazeSilverstreamState {
    pub clue_found: bool,
    pub signal_aligned: bool,
    pub hatch_unlocked: bool,
    pub hints_used: u8,
}

impl Default for AmazeSilverstreamHost {
    fn default() -> Self {
        Self::new()
    }
}

impl AmazeSilverstreamHost {
    pub fn new() -> Self {
        let rooms = [
            MuddleRoom {
                id: "silverstream-entry".to_string(),
                title: "Silverstream Entry".to_string(),
                description: "A trailer-scale AMAZE entry scene with a receiver wall.".to_string(),
                exits: vec![MuddleExit {
                    command: "go receiver".to_string(),
                    target_room: "receiver-wall".to_string(),
                    label: "Receiver Wall".to_string(),
                }],
            },
            MuddleRoom {
                id: "receiver-wall".to_string(),
                title: "Receiver Wall".to_string(),
                description: "A puzzle wall with a hidden clue, signal dial, and locked hatch."
                    .to_string(),
                exits: vec![
                    MuddleExit {
                        command: "go entry".to_string(),
                        target_room: "silverstream-entry".to_string(),
                        label: "Silverstream Entry".to_string(),
                    },
                    MuddleExit {
                        command: "go hatch".to_string(),
                        target_room: "hatch-exit".to_string(),
                        label: "Hatch Exit".to_string(),
                    },
                ],
            },
            MuddleRoom {
                id: "hatch-exit".to_string(),
                title: "Hatch Exit".to_string(),
                description: "The solved escape-room exit surface.".to_string(),
                exits: vec![MuddleExit {
                    command: "go receiver".to_string(),
                    target_room: "receiver-wall".to_string(),
                    label: "Receiver Wall".to_string(),
                }],
            },
        ]
        .into_iter()
        .map(|room| (room.id.clone(), room))
        .collect();

        Self {
            start_room: "silverstream-entry".to_string(),
            rooms,
            state: AmazeSilverstreamState {
                clue_found: false,
                signal_aligned: false,
                hatch_unlocked: false,
                hints_used: 0,
            },
        }
    }

    pub fn state(&self) -> &AmazeSilverstreamState {
        &self.state
    }

    fn look(&self, room_id: &str) -> Result<MuddleCommandOutcome, MuddleError> {
        let room = self
            .room(room_id)
            .ok_or_else(|| MuddleError::RoomNotFound {
                room_id: room_id.to_string(),
            })?;
        Ok(MuddleCommandOutcome::stay(format!(
            "{}\n| amaze: clue_found={} signal_aligned={} hatch_unlocked={} hints_used={}",
            room.ascii_card(),
            self.state.clue_found,
            self.state.signal_aligned,
            self.state.hatch_unlocked,
            self.state.hints_used
        )))
    }

    fn unknown(&self, room_id: &str, command: &MuddleCommand) -> MuddleError {
        MuddleError::UnknownCommand {
            room_id: room_id.to_string(),
            command: command.clone(),
        }
    }
}

impl MuddleHost for AmazeSilverstreamHost {
    fn start_room(&self) -> &str {
        &self.start_room
    }

    fn room(&self, room_id: &str) -> Option<&MuddleRoom> {
        self.rooms.get(room_id)
    }

    fn resource_panel(&self) -> Vec<MuddleResource> {
        vec![
            MuddleResource {
                label: "clue".to_string(),
                value: if self.state.clue_found {
                    "found".to_string()
                } else {
                    "hidden".to_string()
                },
            },
            MuddleResource {
                label: "signal".to_string(),
                value: if self.state.signal_aligned {
                    "aligned".to_string()
                } else {
                    "drifting".to_string()
                },
            },
            MuddleResource {
                label: "hatch".to_string(),
                value: if self.state.hatch_unlocked {
                    "unlocked".to_string()
                } else {
                    "locked".to_string()
                },
            },
            MuddleResource {
                label: "hints".to_string(),
                value: self.state.hints_used.to_string(),
            },
        ]
    }

    fn map_panel(&self, current_room: &str) -> Option<String> {
        Some(format!(
            "{} Entry -- {} Receiver Wall -- {} Hatch Exit",
            marker(current_room, "silverstream-entry"),
            marker(current_room, "receiver-wall"),
            marker(current_room, "hatch-exit")
        ))
    }

    fn objective_panel(&self, current_room: &str) -> Vec<String> {
        match current_room {
            "silverstream-entry" => vec!["Move to the receiver wall.".to_string()],
            "receiver-wall" if !self.state.clue_found => {
                vec!["Find the hidden clue before tuning the signal.".to_string()]
            }
            "receiver-wall" if !self.state.signal_aligned => {
                vec!["Tune the signal using the discovered clue.".to_string()]
            }
            "receiver-wall" if !self.state.hatch_unlocked => {
                vec!["Unlock the hatch after aligning the signal.".to_string()]
            }
            "receiver-wall" => vec!["Exit through the hatch.".to_string()],
            "hatch-exit" => {
                vec!["Escape complete; review the transcript and reset path.".to_string()]
            }
            _ => Vec::new(),
        }
    }

    fn command_panel(&self, current_room: &str) -> Vec<MuddleCommandHint> {
        let mut commands = vec![MuddleCommandHint {
            command: "look".to_string(),
            description: "Show the current room card.".to_string(),
        }];

        match current_room {
            "silverstream-entry" => {
                commands.push(MuddleCommandHint {
                    command: "go receiver".to_string(),
                    description: "Move to the puzzle wall.".to_string(),
                });
            }
            "receiver-wall" => {
                commands.push(MuddleCommandHint {
                    command: "inspect clue".to_string(),
                    description: "Search for the hidden clue.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "tune signal".to_string(),
                    description: "Align the receiver signal.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "unlock hatch".to_string(),
                    description: "Open the hatch after solving the wall.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "request hint".to_string(),
                    description: "Use an operator hint.".to_string(),
                });
                commands.push(MuddleCommandHint {
                    command: "go hatch".to_string(),
                    description: "Exit if the hatch is unlocked.".to_string(),
                });
            }
            "hatch-exit" => {
                commands.push(MuddleCommandHint {
                    command: "go receiver".to_string(),
                    description: "Return to the receiver wall.".to_string(),
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
            ("silverstream-entry", "go receiver") => Ok(MuddleCommandOutcome::move_to(
                "You step up to the receiver wall.",
                "receiver-wall",
            )),
            ("receiver-wall", "go entry") => Ok(MuddleCommandOutcome::move_to(
                "You step back to the entry scene.",
                "silverstream-entry",
            )),
            ("receiver-wall", "inspect clue") => {
                self.state.clue_found = true;
                Ok(MuddleCommandOutcome::stay(
                    "You find a silver frequency mark under the receiver bezel.",
                ))
            }
            ("receiver-wall", "tune signal") if !self.state.clue_found => {
                Ok(MuddleCommandOutcome::stay(
                    "The receiver drifts. You need the hidden frequency clue first.",
                ))
            }
            ("receiver-wall", "tune signal") => {
                self.state.signal_aligned = true;
                Ok(MuddleCommandOutcome::stay(
                    "The signal locks to the silver mark. The hatch relay clicks.",
                ))
            }
            ("receiver-wall", "unlock hatch") if !self.state.signal_aligned => Ok(
                MuddleCommandOutcome::stay("The hatch stays locked until the signal is aligned."),
            ),
            ("receiver-wall", "unlock hatch") => {
                self.state.hatch_unlocked = true;
                Ok(MuddleCommandOutcome::stay(
                    "The hatch unlocks with a clean resettable latch motion.",
                ))
            }
            ("receiver-wall", "request hint") => {
                self.state.hints_used += 1;
                Ok(MuddleCommandOutcome::stay(
                    "Operator hint: inspect the receiver bezel before tuning.",
                ))
            }
            ("receiver-wall", "go hatch") if self.state.hatch_unlocked => {
                Ok(MuddleCommandOutcome::move_to(
                    "You open the hatch and exit the Silverstream room.",
                    "hatch-exit",
                ))
            }
            ("receiver-wall", "go hatch") => Ok(MuddleCommandOutcome::stay(
                "The hatch is still locked. Solve the receiver sequence first.",
            )),
            ("hatch-exit", "go receiver") => Ok(MuddleCommandOutcome::move_to(
                "You return to the receiver wall for reset review.",
                "receiver-wall",
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
    fn plays_silverstream_escape_path() {
        let mut host = AmazeSilverstreamHost::new();
        let mut session = MuddleSession::for_host(&host).expect("AMAZE spike has a start room");

        session
            .play_turn(&mut host, MuddleCommand::parse("go receiver"))
            .expect("entry moves to receiver");
        session
            .play_turn(&mut host, MuddleCommand::parse("inspect clue"))
            .expect("receiver exposes clue");
        session
            .play_turn(&mut host, MuddleCommand::parse("tune signal"))
            .expect("receiver tunes after clue");
        session
            .play_turn(&mut host, MuddleCommand::parse("unlock hatch"))
            .expect("hatch unlocks after signal");
        session
            .play_turn(&mut host, MuddleCommand::parse("go hatch"))
            .expect("unlocked hatch exits");

        assert_eq!(session.current_room, "hatch-exit");
        assert_eq!(session.transcript.len(), 5);
        assert!(host.state().clue_found);
        assert!(host.state().signal_aligned);
        assert!(host.state().hatch_unlocked);
        assert_eq!(host.resource_panel()[2].value, "unlocked");
        assert!(host
            .map_panel(&session.current_room)
            .unwrap()
            .contains("@ Hatch Exit"));
        assert_eq!(
            host.objective_panel(&session.current_room),
            vec!["Escape complete; review the transcript and reset path.".to_string()]
        );
    }

    #[test]
    fn blocks_hatch_until_signal_sequence_is_solved() {
        let mut host = AmazeSilverstreamHost::new();
        let mut session = MuddleSession::for_host(&host).expect("AMAZE spike has a start room");

        session
            .play_turn(&mut host, MuddleCommand::parse("go receiver"))
            .expect("entry moves to receiver");
        let response = session
            .play_turn(&mut host, MuddleCommand::parse("go hatch"))
            .expect("locked hatch returns a host response")
            .response
            .clone();

        assert_eq!(session.current_room, "receiver-wall");
        assert!(response.contains("locked"));
    }
}
