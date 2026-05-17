use std::collections::HashMap;

use muddle_core::{
    MuddleCommand, MuddleCommandOutcome, MuddleError, MuddleExit, MuddleHost, MuddleRoom,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanishPilgrimLossHost {
    start_room: String,
    rooms: HashMap<String, MuddleRoom>,
    state: BanishPilgrimLossState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanishPilgrimLossState {
    pub seed: String,
    pub priority: String,
    pub selected_choice: Option<String>,
    pub manifest_ready: bool,
    pub session_started: bool,
}

impl Default for BanishPilgrimLossHost {
    fn default() -> Self {
        Self::new()
    }
}

impl BanishPilgrimLossHost {
    pub fn new() -> Self {
        let rooms = [
            MuddleRoom {
                id: "pilgrim-launcher".to_string(),
                title: "Pilgrim Loss Launcher".to_string(),
                description:
                    "A BANISH-shaped launcher surface for resuming a Pilgrim Loss campaign."
                        .to_string(),
                exits: vec![MuddleExit {
                    command: "go brief".to_string(),
                    target_room: "campaign-brief".to_string(),
                    label: "Campaign Brief".to_string(),
                }],
            },
            MuddleRoom {
                id: "campaign-brief".to_string(),
                title: "Campaign Brief".to_string(),
                description:
                    "A playable campaign card with pack, route, anchor, and continuity choices."
                        .to_string(),
                exits: vec![
                    MuddleExit {
                        command: "go launcher".to_string(),
                        target_room: "pilgrim-launcher".to_string(),
                        label: "Pilgrim Loss Launcher".to_string(),
                    },
                    MuddleExit {
                        command: "go trail".to_string(),
                        target_room: "migration-trail".to_string(),
                        label: "Migration Trail".to_string(),
                    },
                ],
            },
            MuddleRoom {
                id: "migration-trail".to_string(),
                title: "Migration Trail".to_string(),
                description:
                    "A first playable slice where packing, anchoring, and loss become legible."
                        .to_string(),
                exits: vec![MuddleExit {
                    command: "go brief".to_string(),
                    target_room: "campaign-brief".to_string(),
                    label: "Campaign Brief".to_string(),
                }],
            },
        ]
        .into_iter()
        .map(|room| (room.id.clone(), room))
        .collect();

        Self {
            start_room: "pilgrim-launcher".to_string(),
            rooms,
            state: BanishPilgrimLossState {
                seed: "slice-smoke".to_string(),
                priority: "continuity".to_string(),
                selected_choice: None,
                manifest_ready: false,
                session_started: false,
            },
        }
    }

    pub fn state(&self) -> &BanishPilgrimLossState {
        &self.state
    }

    fn look(&self, room_id: &str) -> Result<MuddleCommandOutcome, MuddleError> {
        let room = self
            .room(room_id)
            .ok_or_else(|| MuddleError::RoomNotFound {
                room_id: room_id.to_string(),
            })?;
        Ok(MuddleCommandOutcome::stay(format!(
            "{}\n| banish: seed={} priority={} choice={} manifest_ready={} session_started={}",
            room.ascii_card(),
            self.state.seed,
            self.state.priority,
            self.state.selected_choice.as_deref().unwrap_or("none"),
            self.state.manifest_ready,
            self.state.session_started
        )))
    }

    fn unknown(&self, room_id: &str, command: &MuddleCommand) -> MuddleError {
        MuddleError::UnknownCommand {
            room_id: room_id.to_string(),
            command: command.clone(),
        }
    }
}

impl MuddleHost for BanishPilgrimLossHost {
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
        match (room_id, command.normalized().as_str()) {
            (_, "look") | (_, "status") => self.look(room_id),
            ("pilgrim-launcher", "start") | ("pilgrim-launcher", "choose resume") => {
                self.state.selected_choice = Some("resume".to_string());
                self.state.session_started = true;
                Ok(MuddleCommandOutcome::move_to(
                    "You select resume. BANISH would run the launcher-session receipt next.",
                    "campaign-brief",
                ))
            }
            ("pilgrim-launcher", "go brief") => Ok(MuddleCommandOutcome::move_to(
                "You open the Pilgrim Loss campaign brief.",
                "campaign-brief",
            )),
            ("campaign-brief", "go launcher") => Ok(MuddleCommandOutcome::move_to(
                "You return to the launcher.",
                "pilgrim-launcher",
            )),
            ("campaign-brief", "inspect plan") => Ok(MuddleCommandOutcome::stay(
                "Plan: pack hearth-kit, route sheltered-valley, anchor memory-stone.",
            )),
            ("campaign-brief", "inspect manifest") => {
                self.state.manifest_ready = true;
                Ok(MuddleCommandOutcome::stay(
                    "Manifest ready: start_run, resume_run, replay_contract, inspect_targets.",
                ))
            }
            ("campaign-brief", "go trail") if self.state.session_started => {
                Ok(MuddleCommandOutcome::move_to(
                    "You enter the first migration trail slice.",
                    "migration-trail",
                ))
            }
            ("campaign-brief", "go trail") => Ok(MuddleCommandOutcome::stay(
                "Start or choose resume before entering the migration trail.",
            )),
            ("migration-trail", "resolve loss") => Ok(MuddleCommandOutcome::stay(
                "Visible loss resolved: packed structures survive, abandoned structures become memory.",
            )),
            ("migration-trail", "go brief") => Ok(MuddleCommandOutcome::move_to(
                "You return to the campaign brief with the trail transcript.",
                "campaign-brief",
            )),
            _ => Err(self.unknown(room_id, command)),
        }
    }
}

#[cfg(test)]
mod tests {
    use muddle_core::MuddleSession;

    use super::*;

    #[test]
    fn plays_pilgrim_loss_launcher_path() {
        let mut host = BanishPilgrimLossHost::new();
        let mut session = MuddleSession::for_host(&host).expect("BANISH spike has a start room");

        session
            .play_turn(&mut host, MuddleCommand::parse("choose resume"))
            .expect("launcher accepts resume choice");
        session
            .play_turn(&mut host, MuddleCommand::parse("inspect plan"))
            .expect("brief exposes plan");
        session
            .play_turn(&mut host, MuddleCommand::parse("inspect manifest"))
            .expect("brief exposes manifest");
        session
            .play_turn(&mut host, MuddleCommand::parse("go trail"))
            .expect("started session can enter trail");
        session
            .play_turn(&mut host, MuddleCommand::parse("resolve loss"))
            .expect("trail can resolve a loss beat");

        assert_eq!(session.current_room, "migration-trail");
        assert_eq!(session.transcript.len(), 5);
        assert_eq!(host.state().selected_choice.as_deref(), Some("resume"));
        assert!(host.state().manifest_ready);
        assert!(host.state().session_started);
    }

    #[test]
    fn blocks_trail_until_launcher_starts_session() {
        let mut host = BanishPilgrimLossHost::new();
        let mut session = MuddleSession::for_host(&host).expect("BANISH spike has a start room");

        session
            .play_turn(&mut host, MuddleCommand::parse("go brief"))
            .expect("launcher can open brief");
        let response = session
            .play_turn(&mut host, MuddleCommand::parse("go trail"))
            .expect("host returns a blocked trail response")
            .response
            .clone();

        assert_eq!(session.current_room, "campaign-brief");
        assert!(response.contains("Start or choose resume"));
    }
}
