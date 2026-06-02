use std::collections::HashMap;

use rune_core::{ContractRegistration, DescriptorCollectionDocument, RuneContract};
use rune_derive::RuneContract as DeriveRuneContract;

pub const RUNE_COLLECTION_ID: &str = "muddle.client_contracts";
pub const RUNE_COLLECTION_VERSION: &str = "v0";

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.room",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.room.id.present", text = "id is not empty")
)]
pub struct MuddleRoom {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "campfire",
        stability = "stable"
    )]
    pub id: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Campfire",
        stability = "stable"
    )]
    pub title: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "A shared starting room.",
        stability = "stable"
    )]
    pub description: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub exits: Vec<MuddleExit>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.exit",
    version = "v0",
    kind = "command",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.exit.command.present", text = "command is not empty")
)]
pub struct MuddleExit {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "go road",
        stability = "stable"
    )]
    pub command: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "pilgrim-road",
        stability = "stable"
    )]
    pub target_room: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Pilgrim Road",
        stability = "stable"
    )]
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.command",
    version = "v0",
    kind = "command",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.command.verb.present", text = "verb is not empty")
)]
pub struct MuddleCommand {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "go",
        stability = "stable"
    )]
    pub verb: String,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "north",
        stability = "stable"
    )]
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.turn",
    version = "v0",
    kind = "event",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.turn.room_id.present", text = "room_id is not empty")
)]
pub struct MuddleTurn {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "campfire",
        stability = "stable"
    )]
    pub room_id: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "MuddleCommand",
        stability = "stable"
    )]
    pub command: MuddleCommand,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "You see exits.",
        stability = "stable"
    )]
    pub response: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.session",
    version = "v0",
    kind = "state",
    requirement = "RUNE-REQ-076",
    invariant(
        id = "muddle.session.current_room.present",
        text = "current_room is not empty"
    )
)]
pub struct MuddleSession {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "campfire",
        stability = "stable"
    )]
    pub current_room: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub transcript: Vec<MuddleTurn>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.session_save",
    version = "v0",
    kind = "state",
    requirement = "RUNE-REQ-076",
    invariant(
        id = "muddle.session_save.current_room.present",
        text = "current_room is not empty"
    )
)]
pub struct MuddleSessionSave {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "campfire",
        stability = "stable"
    )]
    pub current_room: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub commands: Vec<String>,
    #[rune_field(
        required = false,
        sensitivity = "private",
        example = "host-owned checkpoint",
        stability = "stable"
    )]
    pub host_checkpoint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.client_info",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.client_info.host.present", text = "host is not empty")
)]
pub struct MuddleClientInfo {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "mock-labyrinth",
        stability = "stable"
    )]
    pub host: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Shared test host",
        stability = "stable"
    )]
    pub description: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "look; go road",
        stability = "stable"
    )]
    pub suggested_commands: String,
}

#[derive(Clone, Copy)]
pub struct MuddleClientHostRegistration {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub suggested_commands: &'static str,
    pub create: fn() -> Box<dyn MuddleHost>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.client_snapshot",
    version = "v0",
    kind = "state",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.client_snapshot.host.present", text = "host is not empty"),
    extension(
        namespace = "muddle.client_snapshot",
        name = "adoption_lane",
        value = "third_games_spike"
    )
)]
pub struct MuddleClientSnapshot {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "mock-labyrinth",
        stability = "stable"
    )]
    pub host: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Labyrinth host",
        stability = "stable"
    )]
    pub description: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "look; go road",
        stability = "stable"
    )]
    pub suggested_commands: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "campfire",
        stability = "stable"
    )]
    pub room: String,
    #[rune_field(
        required = true,
        unit = "turn",
        min = "0",
        sensitivity = "public",
        example = "0",
        stability = "stable"
    )]
    pub turns: usize,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "+ Campfire",
        stability = "stable"
    )]
    pub room_card: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Ready.",
        stability = "stable"
    )]
    pub last_response: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "MuddleClientPanels",
        stability = "stable"
    )]
    pub panels: MuddleClientPanels,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub commands: Vec<MuddleCommandHint>,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub history: Vec<MuddleClientHistoryEntry>,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub visual_nodes: Vec<MuddleVisualNode>,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub controls: Vec<MuddleClientControl>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.client_control",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.client_control.id.present", text = "id is not empty")
)]
pub struct MuddleClientControl {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "room-card",
        stability = "stable"
    )]
    pub id: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Text",
        stability = "stable"
    )]
    pub kind: MuddleClientControlKind,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Room",
        stability = "stable"
    )]
    pub label: String,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "Room text",
        stability = "stable"
    )]
    pub text: Option<String>,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "MuddleClientImage",
        stability = "stable"
    )]
    pub image: Option<MuddleClientImage>,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "look",
        stability = "stable"
    )]
    pub command: Option<String>,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub children: Vec<MuddleClientControl>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuddleClientControlKind {
    Text,
    Image,
    Button,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.client_image",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076",
    invariant(
        id = "muddle.client_image.source.present",
        text = "source is not empty"
    )
)]
pub struct MuddleClientImage {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "sprites/player.png",
        stability = "stable"
    )]
    pub source: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Player token",
        stability = "stable"
    )]
    pub alt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.visual_node",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.visual_node.id.present", text = "id is not empty")
)]
pub struct MuddleVisualNode {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "player-token",
        stability = "stable"
    )]
    pub id: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Sprite",
        stability = "stable"
    )]
    pub kind: MuddleVisualNodeKind,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Player",
        stability = "stable"
    )]
    pub label: String,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "MuddleSpriteRef",
        stability = "stable"
    )]
    pub sprite: Option<MuddleSpriteRef>,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "Camp",
        stability = "stable"
    )]
    pub text: Option<String>,
    #[rune_field(
        required = true,
        unit = "z-index",
        sensitivity = "public",
        example = "10",
        stability = "stable"
    )]
    pub layer: i32,
    #[rune_field(
        required = true,
        unit = "grid-cell",
        sensitivity = "public",
        example = "0",
        stability = "stable"
    )]
    pub x: i32,
    #[rune_field(
        required = true,
        unit = "grid-cell",
        sensitivity = "public",
        example = "0",
        stability = "stable"
    )]
    pub y: i32,
    #[rune_field(
        required = true,
        unit = "grid-cell",
        min = "0",
        sensitivity = "public",
        example = "1",
        stability = "stable"
    )]
    pub width: i32,
    #[rune_field(
        required = true,
        unit = "grid-cell",
        min = "0",
        sensitivity = "public",
        example = "1",
        stability = "stable"
    )]
    pub height: i32,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub children: Vec<MuddleVisualNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuddleVisualNodeKind {
    Sprite,
    Text,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.sprite_ref",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076",
    invariant(id = "muddle.sprite_ref.source.present", text = "source is not empty")
)]
pub struct MuddleSpriteRef {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "sprites/player.png",
        stability = "stable"
    )]
    pub source: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Player token",
        stability = "stable"
    )]
    pub alt: String,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "idle",
        stability = "stable"
    )]
    pub frame: Option<String>,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "breathing",
        stability = "stable"
    )]
    pub animation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.client_panels",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076"
)]
pub struct MuddleClientPanels {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub resources: Vec<MuddleResource>,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub inventory: Vec<MuddleInventoryItem>,
    #[rune_field(
        required = false,
        sensitivity = "public",
        example = "ASCII map",
        stability = "stable"
    )]
    pub map: Option<String>,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub objectives: Vec<String>,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "[]",
        stability = "stable"
    )]
    pub recent_log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.client_history_entry",
    version = "v0",
    kind = "event",
    requirement = "RUNE-REQ-076"
)]
pub struct MuddleClientHistoryEntry {
    #[rune_field(
        required = true,
        unit = "turn",
        min = "1",
        sensitivity = "public",
        example = "1",
        stability = "stable"
    )]
    pub turn: usize,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "campfire",
        stability = "stable"
    )]
    pub room: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "look",
        stability = "stable"
    )]
    pub command: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "You see exits.",
        stability = "stable"
    )]
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
    fn visual_nodes(&self, _current_room: &str) -> Vec<MuddleVisualNode> {
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

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.resource",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076"
)]
pub struct MuddleResource {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Embers",
        stability = "stable"
    )]
    pub label: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "3",
        stability = "stable"
    )]
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.inventory_item",
    version = "v0",
    kind = "entity",
    requirement = "RUNE-REQ-076"
)]
pub struct MuddleInventoryItem {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Lantern",
        stability = "stable"
    )]
    pub label: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Still lit",
        stability = "stable"
    )]
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DeriveRuneContract)]
#[rune(
    id = "muddle.command_hint",
    version = "v0",
    kind = "command",
    requirement = "RUNE-REQ-076",
    invariant(
        id = "muddle.command_hint.command.present",
        text = "command is not empty"
    )
)]
pub struct MuddleCommandHint {
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "look",
        stability = "stable"
    )]
    pub command: String,
    #[rune_field(
        required = true,
        sensitivity = "public",
        example = "Inspect the room",
        stability = "stable"
    )]
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

impl MuddleClientHostRegistration {
    pub fn client_info(self) -> MuddleClientInfo {
        MuddleClientInfo {
            host: self.name.to_string(),
            description: self.description.to_string(),
            suggested_commands: self.suggested_commands.to_string(),
        }
    }
}

impl MuddleClientControl {
    pub fn text(id: impl Into<String>, label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: MuddleClientControlKind::Text,
            label: label.into(),
            text: Some(text.into()),
            image: None,
            command: None,
            children: Vec::new(),
        }
    }

    pub fn image(
        id: impl Into<String>,
        label: impl Into<String>,
        source: impl Into<String>,
        alt: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: MuddleClientControlKind::Image,
            label: label.into(),
            text: None,
            image: Some(MuddleClientImage {
                source: source.into(),
                alt: alt.into(),
            }),
            command: None,
            children: Vec::new(),
        }
    }

    pub fn button(
        id: impl Into<String>,
        label: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: MuddleClientControlKind::Button,
            label: label.into(),
            text: None,
            image: None,
            command: Some(command.into()),
            children: Vec::new(),
        }
    }

    pub fn group(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<MuddleClientControl>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: MuddleClientControlKind::Group,
            label: label.into(),
            text: None,
            image: None,
            command: None,
            children,
        }
    }
}

impl MuddleVisualNode {
    pub fn sprite(
        id: impl Into<String>,
        label: impl Into<String>,
        source: impl Into<String>,
        alt: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: MuddleVisualNodeKind::Sprite,
            label: label.into(),
            sprite: Some(MuddleSpriteRef {
                source: source.into(),
                alt: alt.into(),
                frame: None,
                animation: None,
            }),
            text: None,
            layer: 0,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            children: Vec::new(),
        }
    }

    pub fn text(id: impl Into<String>, label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: MuddleVisualNodeKind::Text,
            label: label.into(),
            sprite: None,
            text: Some(text.into()),
            layer: 0,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            children: Vec::new(),
        }
    }

    pub fn group(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<MuddleVisualNode>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: MuddleVisualNodeKind::Group,
            label: label.into(),
            sprite: None,
            text: None,
            layer: 0,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            children,
        }
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn with_rect(mut self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_frame(mut self, frame: impl Into<String>) -> Self {
        if let Some(sprite) = &mut self.sprite {
            sprite.frame = Some(frame.into());
        }
        self
    }

    pub fn with_animation(mut self, animation: impl Into<String>) -> Self {
        if let Some(sprite) = &mut self.sprite {
            sprite.animation = Some(animation.into());
        }
        self
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
        let host_name = info.host;
        let description = info.description;
        let suggested_commands = info.suggested_commands;
        let room = self.current_room.clone();
        let turns = self.transcript.len();
        let room_card = host
            .room(&self.current_room)
            .map(MuddleRoom::ascii_card)
            .unwrap_or_else(|| format!("Room missing: {}", self.current_room));
        let last_response = last_response.into();
        let panels = MuddleClientPanels {
            resources: host.resource_panel(),
            inventory: host.inventory_panel(),
            map: host.map_panel(&self.current_room),
            objectives: host.objective_panel(&self.current_room),
            recent_log: self.recent_log_panel(3),
        };
        let commands = host.command_panel(&self.current_room);
        let visual_nodes = host.visual_nodes(&self.current_room);
        let history = self
            .transcript
            .iter()
            .enumerate()
            .map(|(index, turn)| MuddleClientHistoryEntry {
                turn: index + 1,
                room: turn.room_id.clone(),
                command: turn.command.normalized(),
                response: turn.response.clone(),
            })
            .collect::<Vec<_>>();
        let controls = build_client_controls(
            &host_name,
            &description,
            &room_card,
            &last_response,
            &panels,
            &commands,
            &visual_nodes,
            &history,
        );
        MuddleClientSnapshot {
            host: host_name,
            description,
            suggested_commands,
            room,
            turns,
            room_card,
            last_response,
            panels,
            commands,
            history,
            visual_nodes,
            controls,
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

fn build_client_controls(
    host_name: &str,
    description: &str,
    room_card: &str,
    last_response: &str,
    panels: &MuddleClientPanels,
    commands: &[MuddleCommandHint],
    visual_nodes: &[MuddleVisualNode],
    history: &[MuddleClientHistoryEntry],
) -> Vec<MuddleClientControl> {
    let mut controls = vec![
        MuddleClientControl::group(
            "header",
            "Header",
            vec![
                MuddleClientControl::text("host", "Host", host_name),
                MuddleClientControl::text("description", "Description", description),
            ],
        ),
        MuddleClientControl::text("room-card", "Room", room_card),
    ];

    let mut panel_controls = Vec::new();
    if !panels.resources.is_empty() {
        panel_controls.push(MuddleClientControl::group(
            "resources",
            "Resources",
            panels
                .resources
                .iter()
                .map(|resource| {
                    MuddleClientControl::text(
                        format!("resource-{}", control_id_fragment(&resource.label)),
                        &resource.label,
                        &resource.value,
                    )
                })
                .collect(),
        ));
    }
    if !panels.inventory.is_empty() {
        panel_controls.push(MuddleClientControl::group(
            "inventory",
            "Inventory",
            panels
                .inventory
                .iter()
                .map(|item| {
                    MuddleClientControl::text(
                        format!("inventory-{}", control_id_fragment(&item.label)),
                        &item.label,
                        &item.detail,
                    )
                })
                .collect(),
        ));
    }
    if !panels.objectives.is_empty() {
        panel_controls.push(MuddleClientControl::group(
            "objectives",
            "Objectives",
            panels
                .objectives
                .iter()
                .enumerate()
                .map(|(index, objective)| {
                    MuddleClientControl::text(
                        format!("objective-{}", index + 1),
                        "Objective",
                        objective,
                    )
                })
                .collect(),
        ));
    }
    if let Some(map) = &panels.map {
        panel_controls.push(MuddleClientControl::text("map", "Map", map));
    }
    if !panels.recent_log.is_empty() {
        panel_controls.push(MuddleClientControl::group(
            "recent-log",
            "Recent log",
            panels
                .recent_log
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    MuddleClientControl::text(format!("recent-log-{}", index + 1), "Log", entry)
                })
                .collect(),
        ));
    }
    if !panel_controls.is_empty() {
        controls.push(MuddleClientControl::group(
            "panels",
            "Panels",
            panel_controls,
        ));
    }

    if !commands.is_empty() {
        controls.push(MuddleClientControl::group(
            "commands",
            "Commands",
            commands
                .iter()
                .enumerate()
                .map(|(index, hint)| {
                    MuddleClientControl::button(
                        format!("command-{}", index + 1),
                        format!("{} - {}", hint.command, hint.description),
                        &hint.command,
                    )
                })
                .collect(),
        ));
    }

    if !visual_nodes.is_empty() {
        controls.push(MuddleClientControl::group(
            "visuals",
            "Visuals",
            visual_nodes
                .iter()
                .map(visual_node_control)
                .collect::<Vec<_>>(),
        ));
    }

    controls.push(MuddleClientControl::text("status", "Status", last_response));

    if !history.is_empty() {
        controls.push(MuddleClientControl::group(
            "history",
            "History",
            history
                .iter()
                .rev()
                .take(8)
                .map(|turn| {
                    MuddleClientControl::text(
                        format!("history-{}", turn.turn),
                        format!("{}. {}", turn.turn, turn.command),
                        format!(
                            "{} @ {} -> {}",
                            turn.command,
                            turn.room,
                            turn.response.lines().next().unwrap_or_default()
                        ),
                    )
                })
                .collect(),
        ));
    }

    controls
}

fn visual_node_control(node: &MuddleVisualNode) -> MuddleClientControl {
    match node.kind {
        MuddleVisualNodeKind::Sprite => {
            let sprite = node.sprite.as_ref();
            let source = sprite
                .map(|sprite| sprite.source.as_str())
                .unwrap_or_default();
            let alt = sprite
                .map(|sprite| sprite.alt.as_str())
                .unwrap_or(&node.label);
            MuddleClientControl::image(&node.id, &node.label, source, alt)
        }
        MuddleVisualNodeKind::Text => {
            MuddleClientControl::text(&node.id, &node.label, node.text.as_deref().unwrap_or(""))
        }
        MuddleVisualNodeKind::Group => MuddleClientControl::group(
            &node.id,
            &node.label,
            node.children
                .iter()
                .map(visual_node_control)
                .collect::<Vec<_>>(),
        ),
    }
}

fn control_id_fragment(value: &str) -> String {
    let fragment = value
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if character.is_whitespace() || character == '-' || character == '_' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if fragment.is_empty() {
        "item".to_string()
    } else {
        fragment
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

pub const RUNE_CONTRACTS: &[ContractRegistration] = &[
    ContractRegistration {
        name: "MuddleRoom",
        descriptor: MuddleRoom::descriptor,
    },
    ContractRegistration {
        name: "MuddleExit",
        descriptor: MuddleExit::descriptor,
    },
    ContractRegistration {
        name: "MuddleCommand",
        descriptor: MuddleCommand::descriptor,
    },
    ContractRegistration {
        name: "MuddleTurn",
        descriptor: MuddleTurn::descriptor,
    },
    ContractRegistration {
        name: "MuddleSession",
        descriptor: MuddleSession::descriptor,
    },
    ContractRegistration {
        name: "MuddleSessionSave",
        descriptor: MuddleSessionSave::descriptor,
    },
    ContractRegistration {
        name: "MuddleClientInfo",
        descriptor: MuddleClientInfo::descriptor,
    },
    ContractRegistration {
        name: "MuddleClientSnapshot",
        descriptor: MuddleClientSnapshot::descriptor,
    },
    ContractRegistration {
        name: "MuddleClientControl",
        descriptor: MuddleClientControl::descriptor,
    },
    ContractRegistration {
        name: "MuddleClientImage",
        descriptor: MuddleClientImage::descriptor,
    },
    ContractRegistration {
        name: "MuddleVisualNode",
        descriptor: MuddleVisualNode::descriptor,
    },
    ContractRegistration {
        name: "MuddleSpriteRef",
        descriptor: MuddleSpriteRef::descriptor,
    },
    ContractRegistration {
        name: "MuddleClientPanels",
        descriptor: MuddleClientPanels::descriptor,
    },
    ContractRegistration {
        name: "MuddleClientHistoryEntry",
        descriptor: MuddleClientHistoryEntry::descriptor,
    },
    ContractRegistration {
        name: "MuddleResource",
        descriptor: MuddleResource::descriptor,
    },
    ContractRegistration {
        name: "MuddleInventoryItem",
        descriptor: MuddleInventoryItem::descriptor,
    },
    ContractRegistration {
        name: "MuddleCommandHint",
        descriptor: MuddleCommandHint::descriptor,
    },
];

pub fn rune_descriptor_collection() -> Result<DescriptorCollectionDocument, String> {
    DescriptorCollectionDocument::from_registrations(
        RUNE_COLLECTION_ID,
        RUNE_COLLECTION_VERSION,
        RUNE_CONTRACTS,
        "MUDDLE-RUNE-001",
    )
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
    fn client_snapshot_includes_reusable_controls() {
        let host = MuddleStaticHost::try_new(
            "camp",
            vec![MuddleRoom {
                id: "camp".to_string(),
                title: "Camp".to_string(),
                description: "A reusable control test room.".to_string(),
                exits: vec![MuddleExit {
                    command: "go trail".to_string(),
                    target_room: "trail".to_string(),
                    label: "Trail".to_string(),
                }],
            }],
        )
        .expect("host builds");
        let session = MuddleSession::for_host(&host).expect("session starts");
        let snapshot = session.client_snapshot(
            &host,
            MuddleClientInfo {
                host: "test-host".to_string(),
                description: "Control host".to_string(),
                suggested_commands: "look".to_string(),
            },
            "Ready.",
        );

        assert!(snapshot
            .controls
            .iter()
            .any(|control| control.id == "room-card"
                && control.kind == MuddleClientControlKind::Text));
        assert!(snapshot
            .controls
            .iter()
            .any(|control| control.id == "status" && control.text.as_deref() == Some("Ready.")));
    }

    #[test]
    fn client_snapshot_includes_reusable_visual_nodes() {
        struct VisualHost {
            base: MuddleStaticHost,
        }

        impl MuddleHost for VisualHost {
            fn start_room(&self) -> &str {
                self.base.start_room()
            }

            fn room(&self, room_id: &str) -> Option<&MuddleRoom> {
                self.base.room(room_id)
            }

            fn visual_nodes(&self, _current_room: &str) -> Vec<MuddleVisualNode> {
                vec![MuddleVisualNode::group(
                    "scene",
                    "Scene",
                    vec![
                        MuddleVisualNode::sprite(
                            "player-token",
                            "Player",
                            "sprites/player.png",
                            "Player token",
                        )
                        .with_layer(10)
                        .with_rect(4, 2, 1, 1)
                        .with_frame("idle")
                        .with_animation("breathing"),
                        MuddleVisualNode::text("room-label", "Room label", "Camp")
                            .with_layer(20)
                            .with_rect(0, 0, 8, 1),
                    ],
                )]
            }

            fn handle_command(
                &mut self,
                room_id: &str,
                command: &MuddleCommand,
            ) -> Result<MuddleCommandOutcome, MuddleError> {
                self.base.handle_command(room_id, command)
            }
        }

        let host = VisualHost {
            base: MuddleStaticHost::try_new(
                "camp",
                vec![MuddleRoom {
                    id: "camp".to_string(),
                    title: "Camp".to_string(),
                    description: "A visual contract test room.".to_string(),
                    exits: Vec::new(),
                }],
            )
            .expect("host builds"),
        };
        let session = MuddleSession::for_host(&host).expect("session starts");
        let snapshot = session.client_snapshot(
            &host,
            MuddleClientInfo {
                host: "visual-host".to_string(),
                description: "Visual host".to_string(),
                suggested_commands: "look".to_string(),
            },
            "Ready.",
        );

        assert_eq!(snapshot.visual_nodes[0].id, "scene");
        assert_eq!(snapshot.visual_nodes[0].children[0].layer, 10);
        assert!(snapshot
            .controls
            .iter()
            .any(|control| control.id == "visuals"));
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

    #[test]
    fn rune_contract_registry_preserves_client_snapshot_metadata() {
        let collection = rune_descriptor_collection().expect("rune descriptor collection");

        assert_eq!(collection.collection_id, RUNE_COLLECTION_ID);
        assert_eq!(collection.descriptors[7].id, "muddle.client_snapshot");
        assert_eq!(
            collection.descriptors[7].fields[0].metadata.required,
            Some(true)
        );
        assert_eq!(
            collection.descriptors[7].fields[4].metadata.unit,
            Some("turn".to_owned())
        );
        assert_eq!(
            collection.descriptors[10].fields[7].metadata.unit,
            Some("grid-cell".to_owned())
        );
        assert_eq!(
            collection.descriptors[5].fields[2].metadata.sensitivity,
            Some("private".to_owned())
        );
    }

    #[test]
    fn rune_contract_registry_matches_retained_fixture() {
        let collection = rune_descriptor_collection().expect("rune descriptor collection");
        let actual = serde_json::to_string_pretty(&collection).expect("serialize rune collection");
        let expected = include_str!("../../../docs/rune/client_contracts.json");

        assert_eq!(normalize_newlines(&actual), normalize_newlines(expected));
    }

    fn normalize_newlines(value: &str) -> String {
        value.replace("\r\n", "\n").trim_end().to_owned()
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
