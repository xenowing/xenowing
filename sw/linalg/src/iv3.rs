use crate::fixed::*;

use core::ops::{Add, AddAssign, Div, Mul, Sub};

#[derive(Clone, Copy)]
pub struct Iv3<const FRACT_BITS: u32> {
    pub x: Fixed<FRACT_BITS>,
    pub y: Fixed<FRACT_BITS>,
    pub z: Fixed<FRACT_BITS>,
}

impl<const FRACT_BITS: u32> Iv3<FRACT_BITS> {
    pub fn new(
        x: impl Into<Fixed<FRACT_BITS>>,
        y: impl Into<Fixed<FRACT_BITS>>,
        z: impl Into<Fixed<FRACT_BITS>>,
    ) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    pub fn splat(value: impl Into<Fixed<FRACT_BITS>>) -> Self {
        let value = value.into();
        Self {
            x: value,
            y: value,
            z: value,
        }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0.into(),
            y: 0.0.into(),
            z: 0.0.into(),
        }
    }

    pub fn dot(self, other: Self) -> Fixed<FRACT_BITS> {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }
}

impl<const FRACT_BITS: u32> Add for Iv3<FRACT_BITS> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<const FRACT_BITS: u32> AddAssign for Iv3<FRACT_BITS> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl<const FRACT_BITS: u32> Div<Fixed<FRACT_BITS>> for Iv3<FRACT_BITS> {
    type Output = Self;

    fn div(self, other: Fixed<FRACT_BITS>) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl<const FRACT_BITS: u32> Mul for Iv3<FRACT_BITS> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl<const FRACT_BITS: u32> Mul<Fixed<FRACT_BITS>> for Iv3<FRACT_BITS> {
    type Output = Self;

    fn mul(self, other: Fixed<FRACT_BITS>) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl<const FRACT_BITS: u32> Sub for Iv3<FRACT_BITS> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
