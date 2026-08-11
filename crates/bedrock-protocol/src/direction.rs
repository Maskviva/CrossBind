#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Serverbound,
    Clientbound,
}

impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Serverbound => Direction::Clientbound,
            Direction::Clientbound => Direction::Serverbound,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Serverbound => "serverbound",
            Direction::Clientbound => "clientbound",
        }
    }
}
