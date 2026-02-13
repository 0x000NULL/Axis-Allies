//! Phase 14: Multiplayer Foundation
//!
//! Provides the core abstractions for networked multiplayer:
//! - Player sessions and power assignments
//! - Game lobby management
//! - Action serialization for network transport
//! - Turn timer support
//! - Spectator mode

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::action::Action;
use crate::power::Power;

/// Unique identifier for a player session.
pub type PlayerId = String;

/// Unique identifier for a game lobby.
pub type LobbyId = String;

/// Player role in a game.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayerRole {
    /// Controls one or more powers.
    Player { powers: Vec<Power> },
    /// Watches but cannot act.
    Spectator,
}

/// A player in a game session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSession {
    pub id: PlayerId,
    pub name: String,
    pub role: PlayerRole,
    pub connected: bool,
    pub last_seen: u64,
}

impl PlayerSession {
    pub fn new(id: PlayerId, name: String) -> Self {
        PlayerSession {
            id,
            name,
            role: PlayerRole::Spectator,
            connected: true,
            last_seen: 0,
        }
    }

    /// Check if this player controls the given power.
    pub fn controls_power(&self, power: Power) -> bool {
        match &self.role {
            PlayerRole::Player { powers } => powers.contains(&power),
            PlayerRole::Spectator => false,
        }
    }
}

/// State of a game lobby.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LobbyState {
    /// Waiting for players to join and configure.
    WaitingForPlayers,
    /// Game is in progress.
    InProgress,
    /// Game is paused (player disconnected).
    Paused,
    /// Game is finished.
    Finished,
}

/// Configuration for a multiplayer game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    /// Turn time limit in seconds (0 = no limit).
    pub turn_timer_seconds: u32,
    /// Whether spectators are allowed.
    pub allow_spectators: bool,
    /// Whether to auto-save after each turn.
    pub auto_save: bool,
    /// AI difficulty for unassigned powers.
    pub ai_difficulty: String,
    /// Random seed (0 = random).
    pub seed: u64,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            turn_timer_seconds: 0,
            allow_spectators: true,
            auto_save: true,
            ai_difficulty: "Normal".into(),
            seed: 0,
        }
    }
}

/// A multiplayer game lobby.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameLobby {
    pub id: LobbyId,
    pub host: PlayerId,
    pub state: LobbyState,
    pub config: GameConfig,
    pub players: Vec<PlayerSession>,
    /// Power → PlayerId assignment. Unassigned powers are AI-controlled.
    pub power_assignments: HashMap<String, PlayerId>,
}

impl GameLobby {
    pub fn new(id: LobbyId, host_id: PlayerId, host_name: String) -> Self {
        let host_session = PlayerSession::new(host_id.clone(), host_name);
        GameLobby {
            id,
            host: host_id,
            state: LobbyState::WaitingForPlayers,
            config: GameConfig::default(),
            players: vec![host_session],
            power_assignments: HashMap::new(),
        }
    }

    /// Add a player to the lobby.
    pub fn add_player(&mut self, id: PlayerId, name: String) -> Result<(), String> {
        if self.state != LobbyState::WaitingForPlayers {
            return Err("Game already in progress".into());
        }
        if self.players.iter().any(|p| p.id == id) {
            return Err("Player already in lobby".into());
        }
        self.players.push(PlayerSession::new(id, name));
        Ok(())
    }

    /// Remove a player from the lobby.
    pub fn remove_player(&mut self, id: &str) -> Result<(), String> {
        if id == self.host {
            return Err("Host cannot leave (transfer host first)".into());
        }
        self.players.retain(|p| p.id != id);
        // Remove their power assignments
        self.power_assignments.retain(|_, v| v != id);
        Ok(())
    }

    /// Assign a power to a player.
    pub fn assign_power(&mut self, power: Power, player_id: &str) -> Result<(), String> {
        if !self.players.iter().any(|p| p.id == player_id) {
            return Err("Player not in lobby".into());
        }
        let key = format!("{:?}", power);
        self.power_assignments.insert(key, player_id.to_string());
        // Update player role
        self.refresh_roles();
        Ok(())
    }

    /// Unassign a power (will be AI-controlled).
    pub fn unassign_power(&mut self, power: Power) {
        let key = format!("{:?}", power);
        self.power_assignments.remove(&key);
        self.refresh_roles();
    }

    /// Get the player controlling a given power, if any.
    pub fn get_power_controller(&self, power: Power) -> Option<&PlayerId> {
        let key = format!("{:?}", power);
        self.power_assignments.get(&key)
    }

    /// Check if a power is AI-controlled.
    pub fn is_ai_controlled(&self, power: Power) -> bool {
        self.get_power_controller(power).is_none()
    }

    /// Refresh player roles based on power assignments.
    fn refresh_roles(&mut self) {
        for player in &mut self.players {
            let assigned: Vec<Power> = self
                .power_assignments
                .iter()
                .filter(|(_, pid)| **pid == player.id)
                .filter_map(|(k, _)| match k.as_str() {
                    "Germany" => Some(Power::Germany),
                    "SovietUnion" => Some(Power::SovietUnion),
                    "Japan" => Some(Power::Japan),
                    "UnitedStates" => Some(Power::UnitedStates),
                    "China" => Some(Power::China),
                    "UnitedKingdom" => Some(Power::UnitedKingdom),
                    "Italy" => Some(Power::Italy),
                    "ANZAC" => Some(Power::ANZAC),
                    "France" => Some(Power::France),
                    _ => None,
                })
                .collect();

            player.role = if assigned.is_empty() {
                PlayerRole::Spectator
            } else {
                PlayerRole::Player { powers: assigned }
            };
        }
    }

    /// Check if the lobby is ready to start (at least one human player with a power).
    pub fn can_start(&self) -> bool {
        self.state == LobbyState::WaitingForPlayers && !self.power_assignments.is_empty()
    }

    /// Start the game.
    pub fn start(&mut self) -> Result<(), String> {
        if !self.can_start() {
            return Err("Not ready to start".into());
        }
        self.state = LobbyState::InProgress;
        Ok(())
    }

    /// Get the number of connected players.
    pub fn connected_count(&self) -> usize {
        self.players.iter().filter(|p| p.connected).count()
    }
}

/// A network message for multiplayer communication.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameMessage {
    /// Player submits an action.
    SubmitAction {
        player_id: PlayerId,
        action: Action,
    },
    /// Server broadcasts the result of an action.
    ActionApplied {
        action: Action,
        events: Vec<crate::action::GameEvent>,
    },
    /// Server broadcasts a state sync (full state for reconnection).
    StateSync {
        state_json: String,
    },
    /// Chat message.
    Chat {
        player_id: PlayerId,
        message: String,
    },
    /// Player connected/disconnected.
    PlayerStatus {
        player_id: PlayerId,
        connected: bool,
    },
    /// Turn timer update.
    TimerUpdate {
        remaining_seconds: u32,
    },
    /// Game over.
    GameOver {
        winner: crate::power::Team,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_lobby() {
        let lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        assert_eq!(lobby.state, LobbyState::WaitingForPlayers);
        assert_eq!(lobby.players.len(), 1);
        assert_eq!(lobby.players[0].name, "Alice");
    }

    #[test]
    fn test_add_remove_players() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        lobby.add_player("p2".into(), "Bob".into()).unwrap();
        lobby.add_player("p3".into(), "Carol".into()).unwrap();
        assert_eq!(lobby.players.len(), 3);

        lobby.remove_player("p2").unwrap();
        assert_eq!(lobby.players.len(), 2);
    }

    #[test]
    fn test_cannot_remove_host() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        assert!(lobby.remove_player("host1").is_err());
    }

    #[test]
    fn test_assign_powers() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        lobby.add_player("p2".into(), "Bob".into()).unwrap();

        lobby.assign_power(Power::Germany, "host1").unwrap();
        lobby.assign_power(Power::Japan, "host1").unwrap();
        lobby.assign_power(Power::UnitedKingdom, "p2").unwrap();

        assert!(lobby.players[0].controls_power(Power::Germany));
        assert!(lobby.players[0].controls_power(Power::Japan));
        assert!(!lobby.players[0].controls_power(Power::UnitedKingdom));
        assert!(lobby.players[1].controls_power(Power::UnitedKingdom));
    }

    #[test]
    fn test_ai_controlled() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        lobby.assign_power(Power::Germany, "host1").unwrap();

        assert!(!lobby.is_ai_controlled(Power::Germany));
        assert!(lobby.is_ai_controlled(Power::Japan));
        assert!(lobby.is_ai_controlled(Power::SovietUnion));
    }

    #[test]
    fn test_can_start() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        assert!(!lobby.can_start()); // No powers assigned

        lobby.assign_power(Power::Germany, "host1").unwrap();
        assert!(lobby.can_start());

        lobby.start().unwrap();
        assert_eq!(lobby.state, LobbyState::InProgress);
    }

    #[test]
    fn test_cannot_add_player_during_game() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        lobby.assign_power(Power::Germany, "host1").unwrap();
        lobby.start().unwrap();

        assert!(lobby.add_player("p2".into(), "Bob".into()).is_err());
    }

    #[test]
    fn test_duplicate_player_rejected() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        assert!(lobby.add_player("host1".into(), "Alice2".into()).is_err());
    }

    #[test]
    fn test_game_message_serialization() {
        let msg = GameMessage::SubmitAction {
            player_id: "p1".into(),
            action: Action::ConfirmPurchases,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: GameMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            GameMessage::SubmitAction { player_id, .. } => {
                assert_eq!(player_id, "p1");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_unassign_power() {
        let mut lobby = GameLobby::new("game1".into(), "host1".into(), "Alice".into());
        lobby.assign_power(Power::Germany, "host1").unwrap();
        assert!(!lobby.is_ai_controlled(Power::Germany));

        lobby.unassign_power(Power::Germany);
        assert!(lobby.is_ai_controlled(Power::Germany));
    }
}
