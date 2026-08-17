//! Explicit colour-domain values for profile-native records.

/// A public/cache colour with alpha in the most significant byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArgbColour(u32);

/// A colour stored by SA-MP native text-label records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeRgbaColour(u32);

impl ArgbColour {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

impl NativeRgbaColour {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

impl From<ArgbColour> for NativeRgbaColour {
    fn from(value: ArgbColour) -> Self {
        Self(value.0.rotate_left(8))
    }
}

impl From<NativeRgbaColour> for ArgbColour {
    fn from(value: NativeRgbaColour) -> Self {
        Self(value.0.rotate_right(8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_domains_round_trip_without_double_rotation() {
        let public = ArgbColour::new(0xFF6F_CF97);
        let native: NativeRgbaColour = public.into();
        assert_eq!(native.get(), 0x6F_CF97_FF);
        let round_trip: ArgbColour = native.into();
        assert_eq!(round_trip, public);
    }
}
