use crate::fixed;

use core::ops::{Add, AddAssign, Sub};

#[derive(Clone, Copy)]
pub struct Iv4 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: i32,
}

impl Iv4 {
    pub fn new(x: i32, y: i32, z: i32, w: i32) -> Iv4 {
        Iv4 {
            x,
            y,
            z,
            w,
        }
    }

    pub fn splat(value: i32) -> Iv4 {
        Iv4 {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    pub fn zero() -> Iv4 {
        Iv4 {
            x: 0,
            y: 0,
            z: 0,
            w: 0,
        }
    }

    pub fn dot(self, other: Iv4, shift: u32) -> i32 {
        fixed::mul(self.x, other.x, shift) +
        fixed::mul(self.y, other.y, shift) +
        fixed::mul(self.z, other.z, shift) +
        fixed::mul(self.w, other.w, shift)
    }

    pub fn min(self, other: Iv4) -> Iv4 {
        Iv4 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
            w: self.w.min(other.w),
        }
    }

    pub fn max(self, other: Iv4) -> Iv4 {
        Iv4 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
            w: self.w.max(other.w),
        }
    }

    pub fn mul_iv4(self, other: Iv4, shift: u32) -> Iv4 {
        Iv4 {
            x: fixed::mul(self.x, other.x, shift),
            y: fixed::mul(self.y, other.y, shift),
            z: fixed::mul(self.z, other.z, shift),
            w: fixed::mul(self.w, other.w, shift),
        }
    }

    pub fn mul_s(self, other: i32, shift: u32) -> Iv4 {
        Iv4 {
            x: fixed::mul(self.x, other, shift),
            y: fixed::mul(self.y, other, shift),
            z: fixed::mul(self.z, other, shift),
            w: fixed::mul(self.w, other, shift),
        }
    }
}

impl Add for Iv4 {
    type Output = Iv4;

    fn add(self, other: Iv4) -> Iv4 {
        Iv4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl AddAssign for Iv4 {
    fn add_assign(&mut self, other: Iv4) {
        *self = *self + other
    }
}

impl Sub for Iv4 {
    type Output = Iv4;

    fn sub(self, other: Iv4) -> Iv4 {
        Iv4 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}
