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

    pub fn add_mixed<const RHS_FRACT_BITS: u32, const OUTPUT_FRACT_BITS: u32>(self, rhs: Fixed<RHS_FRACT_BITS>) -> Fixed<OUTPUT_FRACT_BITS> {
        let inter_fract_bits = FRACT_BITS.max(RHS_FRACT_BITS);
        let lhs = self.into_raw(inter_fract_bits);
        let rhs = rhs.into_raw(inter_fract_bits);
        Fixed::<OUTPUT_FRACT_BITS>::from_raw(lhs + rhs, inter_fract_bits)
    }

    pub fn mul_mixed<const OTHER_FRACT_BITS: u32, const OUTPUT_FRACT_BITS: u32>(self, other: Fixed<OTHER_FRACT_BITS>) -> Fixed<OUTPUT_FRACT_BITS> {
        let inter_fract_bits = FRACT_BITS + OTHER_FRACT_BITS;
        let lhs = self.0 as i64;
        let rhs = other.0 as i64;
        let res = lhs * rhs;
        Fixed(if inter_fract_bits > OUTPUT_FRACT_BITS {
            res >> (inter_fract_bits - OUTPUT_FRACT_BITS)
        } else {
            res << (OUTPUT_FRACT_BITS - inter_fract_bits)
        } as _)
    }

    pub fn div_mixed<const OTHER_FRACT_BITS: u32, const OUTPUT_FRACT_BITS: u32>(self, other: Fixed<OTHER_FRACT_BITS>) -> Fixed<OUTPUT_FRACT_BITS> {
        let lhs = self.0 as i64;
        let temp = OTHER_FRACT_BITS + OUTPUT_FRACT_BITS;
        let lhs = if temp > FRACT_BITS {
            lhs << (temp - FRACT_BITS)
        } else {
            lhs >> (FRACT_BITS - temp)
        };
        let rhs = other.0 as i64;
        let res = lhs / rhs;
        Fixed(res as _)
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
