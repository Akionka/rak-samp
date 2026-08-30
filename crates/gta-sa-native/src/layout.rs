//! Fixture-backed GTA SA 1.0 US native layouts.
//!
//! These types are host-internal copied representations. They never cross the
//! stable plugin ABI and do not create references into game memory.

/// Native GTA `CVector2D` layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RawVector2 {
    pub x: f32,
    pub y: f32,
}

/// Native GTA `CVector` layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RawVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Native GTA `CMatrix` layout for the verified x86 target.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RawMatrix {
    pub right: RawVector3,
    pub flags: u32,
    pub forward: RawVector3,
    pub pad1: u32,
    pub up: RawVector3,
    pub pad2: u32,
    pub position: RawVector3,
    pub pad3: u32,
    pub attached_matrix: u32,
    pub owns_attached_matrix: u8,
    pub tail_padding: [u8; 3],
}

const _: () = {
    assert!(core::mem::size_of::<RawVector2>() == 0x08);
    assert!(core::mem::size_of::<RawVector3>() == 0x0C);
    assert!(core::mem::size_of::<RawMatrix>() == 0x48);
    assert!(core::mem::offset_of!(RawMatrix, right) == 0x00);
    assert!(core::mem::offset_of!(RawMatrix, forward) == 0x10);
    assert!(core::mem::offset_of!(RawMatrix, up) == 0x20);
    assert!(core::mem::offset_of!(RawMatrix, position) == 0x30);
    assert!(core::mem::offset_of!(RawMatrix, attached_matrix) == 0x40);
    assert!(core::mem::offset_of!(RawMatrix, owns_attached_matrix) == 0x44);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_matrix_matches_the_verified_x86_layout() {
        assert_eq!(core::mem::size_of::<RawMatrix>(), 0x48);
        assert_eq!(core::mem::offset_of!(RawMatrix, position), 0x30);
        assert_eq!(core::mem::offset_of!(RawMatrix, attached_matrix), 0x40);
    }
}
