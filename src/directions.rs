use std::fmt::{Display, Formatter};

use num::{Integer, Signed};

use super::coords::Coordinates;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[allow(dead_code)]
impl Direction {
    pub fn get_directions() -> Vec<Direction> {
        return vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
    }

    pub fn reverse_direction(&self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        };
        write!(f, "{s}")
    }
}

impl<T> Into<Coordinates<T>> for Direction
where
    T: Integer + Copy + Signed,
{
    fn into(self) -> Coordinates<T> {
        match self {
            Direction::Up => Coordinates::origin().up(),
            Direction::Down => Coordinates::origin().down(),
            Direction::Left => Coordinates::origin().left(),
            Direction::Right => Coordinates::origin().right(),
        }
    }
}
