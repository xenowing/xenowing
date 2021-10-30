use crate::fixed::*;

use core::ops::{Add, AddAssign, Mul, Sub};

#[derive(Clone, Copy)]
pub struct Iv4<const FRACT_BITS: u32> {
    pub x: Fixed<FRACT_BITS>,
    pub y: Fixed<FRACT_BITS>,
    pub z: Fixed<FRACT_BITS>,
    pub w: Fixed<FRACT_BITS>,
}

impl<const FRACT_BITS: u32> Iv4<FRACT_BITS> {
    pub fn new(x: Fixed<FRACT_BITS>, y: Fixed<FRACT_BITS>, z: Fixed<FRACT_BITS>, w: Fixed<FRACT_BITS>) -> Self {
        Self {
            x,
            y,
            z,
            w,
        }
    }

    pub fn splat(value: Fixed<FRACT_BITS>) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    pub fn zero() -> Self {
        Self {
            x: Fixed::zero(),
            y: Fixed::zero(),
            z: Fixed::zero(),
            w: Fixed::zero(),
        }
    }

    pub fn dot(self, other: Self) -> Fixed<FRACT_BITS> {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
            w: self.w.min(other.w),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
            w: self.w.max(other.w),
        }
    }
}

impl<const FRACT_BITS: u32> Add for Iv4<FRACT_BITS> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl<const FRACT_BITS: u32> AddAssign for Iv4<FRACT_BITS> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl<const FRACT_BITS: u32> Mul for Iv4<FRACT_BITS> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

impl<const FRACT_BITS: u32> Mul<Fixed<FRACT_BITS>> for Iv4<FRACT_BITS> {
    type Output = Self;

    fn mul(self, other: Fixed<FRACT_BITS>) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
            w: self.w * other,
        }
    }
}

impl<const FRACT_BITS: u32> Sub for Iv4<FRACT_BITS> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}
