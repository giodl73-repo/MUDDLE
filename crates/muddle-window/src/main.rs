use std::{collections::HashMap, io};

use muddle_amaze_spike::AmazeSilverstreamHost;
use muddle_banish_spike::BanishPilgrimLossHost;
use muddle_core::{
    MuddleCommand, MuddleCommandHint, MuddleCommandOutcome, MuddleError, MuddleExit, MuddleHost,
    MuddleResource, MuddleRoom,
};
use muddle_mock_sim::MuddleMockSimHost;
use muddle_window::{run_muddle_window_hosts_from_env_args, MuddleWindowHostRegistration};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PortfolioShowcaseHost {
    rooms: HashMap<String, MuddleRoom>,
}

fn main() -> io::Result<()> {
    run_muddle_window_hosts_from_env_args(host_registry())
}

fn host_registry() -> Vec<MuddleWindowHostRegistration> {
    vec![
        MuddleWindowHostRegistration {
            name: "portfolio-showcase",
            description: "Portfolio showcase: browse MUDDLE-backed games and adjacent systems.",
            suggested_commands:
                "`look`, `go games`, `go knowledge`, `go design`, `go infrastructure`.",
            create: || Box::new(PortfolioShowcaseHost::new()),
        },
        MuddleWindowHostRegistration {
            name: "mock-labyrinth",
            description: "Labyrinth mock sim: BANISH-like resource state plus AMAZE-like locks.",
            suggested_commands:
                "`look`, `gather ember`, `go antechamber`, `inspect glyphs`, `use ember`, `go vault`.",
            create: || Box::new(MuddleMockSimHost::new()),
        },
        MuddleWindowHostRegistration {
            name: "banish-pilgrim-loss",
            description: "BANISH Pilgrim Loss adapter spike: launcher, campaign brief, and migration trail.",
            suggested_commands:
                "`look`, `choose resume`, `inspect plan`, `inspect manifest`, `go trail`, `resolve loss`.",
            create: || Box::new(BanishPilgrimLossHost::new()),
        },
        MuddleWindowHostRegistration {
            name: "amaze-silverstream",
            description: "AMAZE Silverstream adapter spike: clue, signal, hatch, and escape path.",
            suggested_commands:
                "`look`, `go receiver`, `inspect clue`, `tune signal`, `unlock hatch`, `go hatch`.",
            create: || Box::new(AmazeSilverstreamHost::new()),
        },
    ]
}

impl PortfolioShowcaseHost {
    fn new() -> Self {
        let rooms = showcase_rooms()
            .into_iter()
            .map(|room| (room.id.clone(), room))
            .collect();

        Self { rooms }
    }
}

fn showcase_rooms() -> Vec<MuddleRoom> {
    vec![
        showcase_room(
            "portfolio-hub",
            "Portfolio Hub",
            "A local front room for the playable surfaces and systems now visible through MUDDLE.",
            [
                ("go games", "games-hub", "Games"),
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go design", "design-hub", "Design Labs"),
                ("go infrastructure", "infrastructure-hub", "Infrastructure"),
            ],
        ),
        showcase_room(
            "games-hub",
            "Games Hub",
            "Playable and soon-to-be-playable game systems connected by MUDDLE, CLI launchers, and adapter contracts.",
            [
                ("go hub", "portfolio-hub", "Portfolio Hub"),
                ("go banish", "banish-room", "BANISH"),
                ("go amaze", "amaze-room", "AMAZE"),
                ("go tigris", "tigris-room", "TIGRIS"),
                ("go quest", "quest-room", "QUEST"),
                ("go rally", "rally-room", "RALLY"),
            ],
        ),
        showcase_room(
            "knowledge-hub",
            "Knowledge Systems Hub",
            "Reusable scenario and context layers that can feed games without owning product rules.",
            [
                ("go hub", "portfolio-hub", "Portfolio Hub"),
                ("go genes", "genes-room", "GENES"),
                ("go storm", "storm-room", "STORM"),
                ("go canon", "canon-room", "CANON"),
                ("go fauna", "fauna-room", "FAUNA"),
                ("go flora", "flora-room", "FLORA"),
                ("go rite", "rite-room", "RITE"),
                ("go society", "society-room", "CERES/LUCIA/MAXIM/PORTO"),
            ],
        ),
        showcase_room(
            "design-hub",
            "Design Labs Hub",
            "Creative-review labs that inspired the panel-loop method used to evolve systems and game packs.",
            [
                ("go hub", "portfolio-hub", "Portfolio Hub"),
                ("go score", "score-room", "SCORE"),
                ("go scene", "scene-room", "SCENE"),
                ("go prose", "prose-room", "PROSE"),
                ("go reel", "reel-room", "REEL"),
            ],
        ),
        showcase_room(
            "infrastructure-hub",
            "Infrastructure Hub",
            "Shared substrate for deterministic play, repo coordination, and window-compatible host contracts.",
            [
                ("go hub", "portfolio-hub", "Portfolio Hub"),
                ("go muddle", "muddle-room", "MUDDLE"),
                ("go rally", "rally-room", "RALLY"),
                ("go tracker", "tracker-room", "TRACKER"),
            ],
        ),
        showcase_room(
            "muddle-room",
            "MUDDLE",
            "Shared room-command UX, transcript, save/resume, CLI runner, reusable window runner, host chooser, and portfolio showcase.",
            [
                ("go infrastructure", "infrastructure-hub", "Infrastructure"),
                ("go games", "games-hub", "Games"),
            ],
        ),
        showcase_room(
            "banish-room",
            "BANISH",
            "Moving-settlement and world-building simulator work. The window can launch Pilgrim Loss from this repo and product-owned window launchers can now reuse the same runner.",
            [
                ("go games", "games-hub", "Games"),
                ("go amaze", "amaze-room", "AMAZE"),
            ],
        ),
        showcase_room(
            "amaze-room",
            "AMAZE",
            "Trailer-scale escape-room design and simulation work. The window can launch Silverstream from this repo and product-owned window launchers can now reuse the same runner.",
            [
                ("go games", "games-hub", "Games"),
                ("go banish", "banish-room", "BANISH"),
            ],
        ),
        showcase_room(
            "tigris-room",
            "TIGRIS",
            "Board-game factory work with a Parliament AI MUDDLE host. Next surface: product-owned `tigris-muddle-window` using the reusable window runner.",
            [
                ("go games", "games-hub", "Games"),
                ("go quest", "quest-room", "QUEST"),
            ],
        ),
        showcase_room(
            "quest-room",
            "QUEST",
            "D&D workshop work with reusable dice and AI-DM table play. Next surface: product-owned `quest-muddle-window` using the reusable window runner.",
            [
                ("go games", "games-hub", "Games"),
                ("go tigris", "tigris-room", "TIGRIS"),
            ],
        ),
        showcase_room(
            "rally-room",
            "RALLY",
            "Deterministic playtest, simulation, metrics, comparison, and replay substrate for proving behavior before richer UI polish.",
            [
                ("go games", "games-hub", "Games"),
                ("go infrastructure", "infrastructure-hub", "Infrastructure"),
            ],
        ),
        showcase_room(
            "genes-room",
            "GENES",
            "Family trees, kinship, lineage, household continuity, contested parentage, and multiple inheritance systems.",
            [
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go storm", "storm-room", "STORM"),
            ],
        ),
        showcase_room(
            "storm-room",
            "STORM",
            "Weather, seasonal hazards, disasters, preparedness, exposure, downstream handoffs, and recovery windows.",
            [
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go genes", "genes-room", "GENES"),
            ],
        ),
        showcase_room(
            "canon-room",
            "CANON",
            "Continuity, authority, contradiction, canon tiers, and setting-fact governance for games and fiction.",
            [
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go rite", "rite-room", "RITE"),
            ],
        ),
        showcase_room(
            "fauna-room",
            "FAUNA",
            "Animals, herds, pests, predators, livestock, migration, habitat, and disease-vector context.",
            [
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go flora", "flora-room", "FLORA"),
            ],
        ),
        showcase_room(
            "flora-room",
            "FLORA",
            "Plants, crops, forests, orchards, soil ecology, succession, blight, fuel, medicine, and fiber.",
            [
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go fauna", "fauna-room", "FAUNA"),
            ],
        ),
        showcase_room(
            "rite-room",
            "RITE",
            "Ritual, taboo, sacred place, calendar, mourning, oath, legitimacy, and cultural obligation.",
            [
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go canon", "canon-room", "CANON"),
            ],
        ),
        showcase_room(
            "society-room",
            "CERES/LUCIA/MAXIM/PORTO",
            "Agriculture, logistics, settlement principles, and facility/port alignment systems already used by game packs.",
            [
                ("go knowledge", "knowledge-hub", "Knowledge Systems"),
                ("go games", "games-hub", "Games"),
            ],
        ),
        showcase_room(
            "score-room",
            "SCORE",
            "Music design lab and one inspiration for the review-loop rubric approach.",
            [
                ("go design", "design-hub", "Design Labs"),
                ("go scene", "scene-room", "SCENE"),
            ],
        ),
        showcase_room(
            "scene-room",
            "SCENE",
            "Visualization design lab and ASPECT rubric work.",
            [
                ("go design", "design-hub", "Design Labs"),
                ("go score", "score-room", "SCORE"),
            ],
        ),
        showcase_room(
            "prose-room",
            "PROSE",
            "Writing design lab for narrative and prose-oriented review workflows.",
            [
                ("go design", "design-hub", "Design Labs"),
                ("go reel", "reel-room", "REEL"),
            ],
        ),
        showcase_room(
            "reel-room",
            "REEL",
            "Film/video design lab work tracked separately; dirty local REEL pointer remains unrelated to this MUDDLE task.",
            [
                ("go design", "design-hub", "Design Labs"),
                ("go prose", "prose-room", "PROSE"),
            ],
        ),
        showcase_room(
            "tracker-room",
            "TRACKER",
            "The portfolio coordination repo that tracks submodules, dependency systems, usage gates, and cross-repo adoption.",
            [
                ("go infrastructure", "infrastructure-hub", "Infrastructure"),
                ("go muddle", "muddle-room", "MUDDLE"),
            ],
        ),
    ]
}

fn showcase_room(
    id: &str,
    title: &str,
    description: &str,
    exits: impl IntoIterator<Item = (&'static str, &'static str, &'static str)>,
) -> MuddleRoom {
    MuddleRoom {
        id: id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        exits: exits
            .into_iter()
            .map(|(command, target_room, label)| MuddleExit {
                command: command.to_string(),
                target_room: target_room.to_string(),
                label: label.to_string(),
            })
            .collect(),
    }
}

impl MuddleHost for PortfolioShowcaseHost {
    fn start_room(&self) -> &str {
        "portfolio-hub"
    }

    fn room(&self, room_id: &str) -> Option<&MuddleRoom> {
        self.rooms.get(room_id)
    }

    fn resource_panel(&self) -> Vec<MuddleResource> {
        vec![
            MuddleResource {
                label: "window-hosts".to_string(),
                value: (host_registry().len() - 1).to_string(),
            },
            MuddleResource {
                label: "catalog-rooms".to_string(),
                value: self.rooms.len().to_string(),
            },
        ]
    }

    fn map_panel(&self, current_room: &str) -> Option<String> {
        Some(format!(
            "{} Hub -- {} Games -- {} Knowledge -- {} Design -- {} Infrastructure",
            marker(current_room, "portfolio-hub"),
            marker(current_room, "games-hub"),
            marker(current_room, "knowledge-hub"),
            marker(current_room, "design-hub"),
            marker(current_room, "infrastructure-hub"),
        ))
    }

    fn objective_panel(&self, current_room: &str) -> Vec<String> {
        match current_room {
            "portfolio-hub" => vec![
                "Browse games, knowledge systems, design labs, and infrastructure.".to_string(),
                "Use Change host to launch direct playable adapter demos.".to_string(),
            ],
            "banish-room" | "amaze-room" => {
                vec![
                    "Return to the chooser to launch this direct playable adapter demo."
                        .to_string(),
                ]
            }
            "tigris-room" | "quest-room" => vec![
                "Add the product-owned window launcher to make this directly playable.".to_string(),
            ],
            _ => vec!["Browse related systems or return to a category hub.".to_string()],
        }
    }

    fn command_panel(&self, current_room: &str) -> Vec<MuddleCommandHint> {
        let mut commands = vec![MuddleCommandHint {
            command: "look".to_string(),
            description: "Show the current showcase card.".to_string(),
        }];
        if let Some(room) = self.room(current_room) {
            commands.extend(room.exits.iter().map(|exit| MuddleCommandHint {
                command: exit.command.clone(),
                description: format!("Open {}.", exit.label),
            }));
        }
        commands
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

        let normalized = command.normalized();
        if let Some(exit) = room.exits.iter().find(|exit| exit.command == normalized) {
            return Ok(MuddleCommandOutcome::move_to(
                format!("Opening {}.", exit.label),
                exit.target_room.clone(),
            ));
        }

        Err(MuddleError::UnknownCommand {
            room_id: room_id.to_string(),
            command: command.clone(),
        })
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
    use super::*;
    use muddle_core::MuddleSession;

    #[test]
    fn registers_portfolio_showcase_first() {
        let hosts = host_registry();
        assert_eq!(hosts[0].name, "portfolio-showcase");
        assert!(hosts.iter().any(|host| host.name == "banish-pilgrim-loss"));
        assert!(hosts.iter().any(|host| host.name == "amaze-silverstream"));
    }

    #[test]
    fn showcase_catalog_has_expected_rooms() {
        let host = PortfolioShowcaseHost::new();
        for room in [
            "portfolio-hub",
            "games-hub",
            "knowledge-hub",
            "design-hub",
            "infrastructure-hub",
            "genes-room",
            "storm-room",
            "tracker-room",
        ] {
            assert!(host.room(room).is_some(), "{room} should exist");
        }
    }

    #[test]
    fn showcase_host_browses_catalog_rooms() {
        let mut host = PortfolioShowcaseHost::new();
        let mut session = MuddleSession::for_host(&host).expect("showcase starts");
        session
            .play_turn(&mut host, MuddleCommand::parse("go knowledge"))
            .expect("hub links to knowledge systems");
        assert_eq!(session.current_room, "knowledge-hub");
        session
            .play_turn(&mut host, MuddleCommand::parse("go storm"))
            .expect("knowledge hub links to STORM");
        assert_eq!(session.current_room, "storm-room");
    }
}
