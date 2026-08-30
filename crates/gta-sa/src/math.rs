//! Owned GTA San Andreas math values.

/// A two-dimensional vector copied from game state or supplied to an operation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A three-dimensional vector copied from game state or supplied to an operation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// An owned GTA transform without native attachment pointers or padding.
///
/// GTA names the basis vectors `right`, `up`, and `at`. The public Rust model
/// uses their semantic directions. This value is copied; it is not the native
/// `CMatrix` layout and never borrows game memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Matrix {
    pub right: Vector3,
    pub forward: Vector3,
    pub up: Vector3,
    pub position: Vector3,
}

impl Matrix {
    #[must_use]
    pub const fn new(right: Vector3, forward: Vector3, up: Vector3, position: Vector3) -> Self {
        Self {
            right,
            forward,
            up,
            position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_constructors_preserve_components() {
        assert_eq!(Vector2::new(1.0, -2.0), Vector2 { x: 1.0, y: -2.0 });
        assert_eq!(
            Vector3::new(1.0, -2.0, 3.5),
            Vector3 {
                x: 1.0,
                y: -2.0,
                z: 3.5,
            }
        );
    }

    #[test]
    fn owned_matrix_has_no_native_pointer_fields() {
        assert_eq!(core::mem::size_of::<Vector2>(), 8);
        assert_eq!(core::mem::size_of::<Vector3>(), 12);
        assert_eq!(core::mem::size_of::<Matrix>(), 48);
    }
}
