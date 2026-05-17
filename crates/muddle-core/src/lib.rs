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
}
