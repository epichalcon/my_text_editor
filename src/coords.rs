use std::{
    collections::HashSet,
    hash::Hash,
    ops::{Add, Sub, RangeBounds}, fmt::{Display, Formatter},
};

use num::{CheckedAdd, CheckedSub, Integer, Num, Signed};

use crate::directions::Direction;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Coordinates<T>
where
    T: Num,
{
    x: T,
    y: T,
}

impl<T> Coordinates<T>
where
    T: Num + Copy,
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self::new(num::zero(), num::zero())
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }
}

#[allow(dead_code)]
impl<T> Coordinates<T>
where
    T: Integer + Signed + Copy,
{
    pub fn up(&self) -> Self {
        Self {
            x: self.x,
            y: self.y - num::one(),
        }
    }

    fn up_by(&self, steps: T) -> Self {
        Self {
            x: self.x,
            y: self.y - steps,
        }
    }
    pub fn down(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + num::one(),
        }
    }
    fn down_by(&self, steps: T) -> Self {
        Self {
            x: self.x,
            y: self.y + steps,
        }
    }

    pub fn left(&self) -> Self {
        Self {
            x: self.x - num::one(),
            y: self.y,
        }
    }
    fn left_by(&self, steps: T) -> Self {
        Self {
            x: self.x - steps,
            y: self.y,
        }
    }

    pub fn right(&self) -> Self {
        Self {
            x: self.x + num::one(),
            y: self.y,
        }
    }
    fn right_by(&self, steps: T) -> Self {
        Self {
            x: self.x + steps,
            y: self.y,
        }
    }
}

/// Utility methods for *signed or unsigned* **integer coordinates**, useful for checking that a coordinate won't go out of bounds of a grid.
#[allow(dead_code)]
impl<T> Coordinates<T>
where
    T: Integer + Copy + CheckedAdd + CheckedSub,
{
    pub fn try_up(&self) -> Option<Self> {
        self.y
            .checked_sub(&num::one())
            .map(|y| Self { x: self.x, y })
    }

    pub fn try_up_by(&self, n: T) -> Option<Self> {
        self.y.checked_sub(&n).map(|y| Self { x: self.x, y })
    }

    /// Increments the `y` coordinate by 1, returning `None` if it goes out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use advent_of_code::utils::coords::Coordinates;
    ///
    /// assert_eq!(Coordinates::new(0, 0).try_bounded_up_by(1, ..=2), Some(Coordinates::new(0, 1)));
    /// assert_eq!(Coordinates::new(0, 2).try_bounded_up_by(1, ..=2), None);
    /// ```
    pub fn try_bounded_up_by<R>(&self, n: T, y_range: R) -> Option<Self>
    where
        R: RangeBounds<T>,
    {
        self.y
            .checked_add(&n)
            .filter(|y| y_range.contains(y))
            .map(|y| (self.x, y).into())
    }

    pub fn try_down(&self) -> Option<Self> {
        self.y
            .checked_add(&num::one())
            .map(|y| Self { x: self.x, y })
    }

    pub fn try_down_by(&self, n: T) -> Option<Self> {
        self.y.checked_add(&n).map(|y| Self { x: self.x, y })
    }

    /// Decrements the `y` coordinate by 1, returning `None` if it goes out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use advent_of_code::utils::coords::Coordinates;
    ///
    /// assert_eq!(Coordinates::new(0, 2).try_bounded_down_by(1, 0..), Some(Coordinates::new(0, 1)));
    /// assert_eq!(Coordinates::new(0, 0).try_bounded_down_by(1, 0..), None);
    /// ```
    pub fn try_bounded_down_by<R>(&self, n: T, y_range: R) -> Option<Self>
    where
        R: RangeBounds<T>,
    {
        self.y
            .checked_sub(&n)
            .filter(|y| y_range.contains(y))
            .map(|y| (self.x, y).into())
    }

    pub fn try_right(&self) -> Option<Self> {
        self.x
            .checked_add(&num::one())
            .map(|x| Self { x, y: self.y })
    }

    pub fn try_right_by(&self, n: T) -> Option<Self> {
        self.x.checked_add(&n).map(|x| Self { x, y: self.y })
    }

    /// Increments the `x` coordinate by 1, returning `None` if it goes out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use advent_of_code::utils::coords::Coordinates;
    ///
    /// assert_eq!(Coordinates::new(0, 0).try_bounded_right_by(1, ..=2), Some(Coordinates::new(1, 0)));
    /// assert_eq!(Coordinates::new(2, 0).try_bounded_right_by(1, ..=2), None);
    /// ```
    pub fn try_bounded_right_by<R>(&self, n: T, x_range: R) -> Option<Self>
    where
        R: RangeBounds<T>,
    {
        self.x
            .checked_add(&n)
            .filter(|x| x_range.contains(x))
            .map(|x| (x, self.y).into())
    }

    pub fn try_left(&self) -> Option<Self> {
        self.x
            .checked_sub(&num::one())
            .map(|x| Self { x, y: self.y })
    }

    pub fn try_left_by(&self, n: T) -> Option<Self> {
        self.x.checked_sub(&n).map(|x| Self { x, y: self.y })
    }

    /// Decrements the `x` coordinate by 1, returning `None` if it goes out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use advent_of_code::utils::coords::Coordinates;
    ///
    /// assert_eq!(Coordinates::new(2, 0).try_bounded_left_by(1, 0..), Some(Coordinates::new(1, 0)));
    /// assert_eq!(Coordinates::new(0, 0).try_bounded_left_by(1, 0..), None);
    /// ```
    pub fn try_bounded_left_by<R>(&self, n: T, x_range: R) -> Option<Self>
    where
        R: RangeBounds<T>,
    {
        self.x
            .checked_sub(&n)
            .filter(|x| x_range.contains(x))
            .map(|x| (x, self.y).into())
    }
}

#[allow(dead_code)]
impl<T> Coordinates<T>
where
    T: Integer + Hash + CheckedAdd + CheckedSub + Copy,
{
    pub fn orthogonal_neighbors(&self) -> HashSet<Self> {
        let mut neighbors = HashSet::new();
        if let Some(up) = self.try_up() {
            neighbors.insert(up);
        }
        if let Some(down) = self.try_down() {
            neighbors.insert(down);
        }
        if let Some(left) = self.try_left() {
            neighbors.insert(left);
        }
        if let Some(right) = self.try_right() {
            neighbors.insert(right);
        }

        neighbors
    }

    pub fn diagonal_neighbors(&self) -> HashSet<Self> {
        let mut neighbors = HashSet::new();
        if let Some(up_left) = self.try_up().and_then(|up| up.try_left()) {
            neighbors.insert(up_left);
        }
        if let Some(down_left) = self.try_down().and_then(|down| down.try_left()) {
            neighbors.insert(down_left);
        }
        if let Some(up_right) = self.try_up().and_then(|up| up.try_right()) {
            neighbors.insert(up_right);
        }
        if let Some(down_right) = self.try_down().and_then(|down| down.try_right()) {
            neighbors.insert(down_right);
        }

        neighbors
    }

    pub fn all_neighbors(&self) -> HashSet<Self> {
        let mut neighbors = self.diagonal_neighbors();
        neighbors.extend(self.orthogonal_neighbors());
        neighbors
    }
}

#[allow(dead_code)]
impl<T> Coordinates<T>
where
    T: Integer + Signed + Copy,
{
    pub fn step(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => self.up(),
            Direction::Down => self.down(),
            Direction::Left => self.left(),
            Direction::Right => self.right(),
        }
    }

    pub fn step_by(&self, direction: Direction, steps: T) -> Self {
        match direction {
            Direction::Up => self.up_by(steps),
            Direction::Down => self.down_by(steps),
            Direction::Left => self.left_by(steps),
            Direction::Right => self.right_by(steps),
        }
    }
}

#[allow(dead_code)]
impl<T> Coordinates<T>
where
    T: Integer + Signed + Copy,
{
    pub fn orthogonal_distance(&self, other: Self) -> T {
        let x = if self.x <= other.x {
            num::abs_sub(other.x, self.x)
        } else {
            num::abs_sub(self.x, other.x)
        };
        let y = if self.y <= other.y {
            num::abs_sub(other.y, self.y)
        } else {
            num::abs_sub(self.y, other.y)
        };
        x + y
    }
}

impl<T> Sub for Coordinates<T>
where
    T: Integer,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Add for Coordinates<T>
where
    T: Integer,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}


impl<T> From<(T, T)> for Coordinates<T>
where
    T: Num + Copy,
{
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<(&T, T)> for Coordinates<T>
where
    T: Num + Copy,
{
    fn from((x, y): (&T, T)) -> Self {
        Self::new(*x, y)
    }
}

impl<T> From<(T, &T)> for Coordinates<T>
where
    T: Num + Copy,
{
    fn from((x, y): (T, &T)) -> Self {
        Self::new(x, *y)
    }
}

impl<T> From<(&T, &T)> for Coordinates<T>
where
    T: Num + Copy,
{
    fn from((x, y): (&T, &T)) -> Self {
        Self::new(*x, *y)
    }
}

impl<T> Into<(T, T)> for Coordinates<T>
where
    T: Num,
{
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T> Display for Coordinates<T>
where
    T: Num + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> Default for Coordinates<T> where T: Num + Default {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default() }
    }
}
