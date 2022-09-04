use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Fixed<const FRACT_BITS: u32>(i32);

impl<const FRACT_BITS: u32> Fixed<FRACT_BITS> {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn ceil(self) -> Self {
        if FRACT_BITS > 0 {
            Self(self.0 + (1 << (FRACT_BITS - 1))).floor()
        } else {
            self
        }
    }

    pub fn floor(self) -> Self {
        if FRACT_BITS > 0 {
            Self(self.0 & !((1 << FRACT_BITS) - 1))
        } else {
            self
        }
    }

    // TODO: Is this the right interface/name?
    pub fn from_raw(raw: i32, source_fract_bits: u32) -> Self {
        Self(if source_fract_bits >= FRACT_BITS {
            raw >> (source_fract_bits - FRACT_BITS)
        } else {
            raw << (FRACT_BITS - source_fract_bits)
        })
    }

    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    // TODO: Is this the right interface/name?
    pub fn into_raw(self, target_fract_bits: u32) -> i32 {
        if target_fract_bits >= FRACT_BITS {
            self.0 << (target_fract_bits - FRACT_BITS)
        } else {
            self.0 >> (FRACT_BITS - target_fract_bits)
        }
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

impl<const FRACT_BITS: u32> Div for Fixed<FRACT_BITS> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self((((self.0 as i64) << FRACT_BITS) / other.0 as i64) as _)
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
