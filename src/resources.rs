use crate::map::Position;

pub struct PlayerInfo {
    pub position: Position,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        PlayerInfo { position: (-1, -1) }
    }
}
