use core::ops::{Add, AddAssign, Mul, Neg, Sub};

#[derive(Clone, Copy)]
pub struct Fixed<const FRACT_BITS: u32>(i32);

impl<const FRACT_BITS: u32> Fixed<FRACT_BITS> {
    pub fn min(&self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn max(&self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl<const FRACT_BITS: u32> Add for Fixed<FRACT_BITS> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl<const FRACT_BITS: u32> AddAssign for Fixed<FRACT_BITS> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl<const FRACT_BITS: u32> From<f32> for Fixed<FRACT_BITS> {
    fn from(value: f32) -> Self {
        Self((value * (1 << FRACT_BITS) as f32) as _)
    }
}

impl<const FRACT_BITS: u32> Mul for Fixed<FRACT_BITS> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(((self.0 as i64 * other.0 as i64) >> FRACT_BITS) as _)
    }
}

impl<const FRACT_BITS: u32> Neg for Fixed<FRACT_BITS> {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl<const FRACT_BITS: u32> Sub for Fixed<FRACT_BITS> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl<const FRACT_BITS: u32> From<Fixed<FRACT_BITS>> for f32 {
    fn from(value: Fixed<FRACT_BITS>) -> Self {
        value.0 as f32 / (1 << FRACT_BITS) as f32
    }
}
