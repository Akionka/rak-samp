//! Neutral Protocol value types.

/// A two-dimensional Protocol vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector2 {
    /// The first component.
    pub x: f32,
    /// The second component.
    pub y: f32,
}

/// A three-dimensional Protocol vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    /// The first component.
    pub x: f32,
    /// The second component.
    pub y: f32,
    /// The third component.
    pub z: f32,
}
