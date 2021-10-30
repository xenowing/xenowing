use core::intrinsics;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub};

#[derive(Clone, Copy)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl V3 {
    pub fn new(x: f32, y: f32, z: f32) -> V3 {
        V3 {
            x,
            y,
            z,
        }
    }

    pub fn splat(value: f32) -> V3 {
        V3 {
            x: value,
            y: value,
            z: value,
        }
    }

    pub fn zero() -> V3 {
        V3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn len(self) -> f32 {
        unsafe { intrinsics::sqrtf32(self.dot(self)) }
    }

    pub fn normalize(self) -> V3 {
        let len = self.len();
        self / len
    }

    pub fn dot(self, other: V3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn min(self, other: V3) -> V3 {
        V3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    pub fn max(self, other: V3) -> V3 {
        V3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }
}

impl Add for V3 {
    type Output = V3;

    fn add(self, other: V3) -> V3 {
        V3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<f32> for V3 {
    type Output = V3;

    fn add(self, other: f32) -> V3 {
        V3 {
            x: self.x + other,
            y: self.y + other,
            z: self.z + other,
        }
    }
}

impl AddAssign for V3 {
    fn add_assign(&mut self, other: V3) {
        *self = *self + other
    }
}

impl Div for V3 {
    type Output = V3;

    fn div(self, other: V3) -> V3 {
        V3 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Div<f32> for V3 {
    type Output = V3;

    fn div(self, other: f32) -> V3 {
        V3 {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl DivAssign<f32> for V3 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other
    }
}

impl Mul for V3 {
    type Output = V3;

    fn mul(self, other: V3) -> V3 {
        V3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Mul<f32> for V3 {
    type Output = V3;

    fn mul(self, other: f32) -> V3 {
        V3 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Sub for V3 {
    type Output = V3;

    fn sub(self, other: V3) -> V3 {
        V3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
