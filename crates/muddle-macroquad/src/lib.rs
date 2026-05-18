use muddle_core::{MuddleClientInfo, MuddleClientSnapshot, MuddleCommand, MuddleSession};
use muddle_mock_sim::MuddleMockSimHost;

pub struct MuddleMacroquadState {
    host: MuddleMockSimHost,
    session: MuddleSession,
    input: String,
    last_status: String,
}

impl MuddleMacroquadState {
    pub fn new() -> Result<Self, String> {
        let host = MuddleMockSimHost::new();
        let session = MuddleSession::for_host(&host).map_err(|error| format!("{error:?}"))?;
        Ok(Self {
            host,
            session,
            input: String::new(),
            last_status: "Type a command and press Enter. Try: look".to_string(),
        })
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn push_char(&mut self, character: char) {
        if !character.is_control() {
            self.input.push(character);
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn submit_input(&mut self) {
        let command = self.input.trim().to_string();
        self.input.clear();
        if command.is_empty() {
            return;
        }

        match self
            .session
            .play_turn(&mut self.host, MuddleCommand::parse(&command))
        {
            Ok(turn) => self.last_status = turn.response.clone(),
            Err(error) => self.last_status = format!("Command failed: {error:?}"),
        }
    }

    pub fn display_lines(&self) -> Vec<String> {
        snapshot_display_lines(&self.snapshot(), &self.input)
    }

    pub fn snapshot(&self) -> MuddleClientSnapshot {
        self.session.client_snapshot(
            &self.host,
            MuddleClientInfo {
                host: "mock-labyrinth".to_string(),
                description: "Macroquad mock-labyrinth engine spike".to_string(),
                suggested_commands: "look, gather ember, read glyphs, feed ember, go antechamber"
                    .to_string(),
            },
            self.last_status.clone(),
        )
    }
}

pub fn snapshot_display_lines(snapshot: &MuddleClientSnapshot, input: &str) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("MUDDLE Macroquad Runner".to_string());
    lines.push("Esc quits. Enter submits. Backspace edits.".to_string());
    lines.push(format!("Host: {}", snapshot.host));
    lines.push(format!("Input: {input}"));
    lines.push(String::new());

    lines.push(snapshot.room_card.clone());

    lines.push(String::new());
    lines.push("Panels".to_string());
    lines.extend(
        snapshot
            .panels
            .resources
            .iter()
            .map(|resource| format!("{}: {}", resource.label, resource.value)),
    );
    if let Some(map) = &snapshot.panels.map {
        lines.push(map.clone());
    }
    let commands = snapshot
        .commands
        .iter()
        .map(|hint| format!("{} ({})", hint.command, hint.description))
        .collect::<Vec<_>>()
        .join(" | ");
    if !commands.is_empty() {
        lines.push(format!("Commands: {commands}"));
    }

    lines.push(String::new());
    lines.push(format!("Status: {}", snapshot.last_response));
    lines.push("Recent history".to_string());
    for turn in snapshot.history.iter().rev().take(5) {
        lines.push(format!(
            "{}. {} @ {} -> {}",
            turn.turn,
            turn.command,
            turn.room,
            turn.response.lines().next().unwrap_or_default()
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macroquad_state_starts_with_mock_room() {
        let state = MuddleMacroquadState::new().expect("state starts");
        let lines = state.display_lines();
        assert!(lines.iter().any(|line| line.contains("Labyrinth Camp")));
        assert!(lines.iter().any(|line| line.contains("Commands:")));
    }

    #[test]
    fn macroquad_state_submits_commands() {
        let mut state = MuddleMacroquadState::new().expect("state starts");
        for character in "look".chars() {
            state.push_char(character);
        }
        state.submit_input();
        assert!(state.input().is_empty());
        assert_eq!(state.session.transcript.len(), 1);
        assert!(state
            .display_lines()
            .iter()
            .any(|line| line.contains("Recent history")));
    }
}
