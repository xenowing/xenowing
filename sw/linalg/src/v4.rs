use core::intrinsics;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub};

#[derive(Clone, Copy)]
pub struct V4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl V4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> V4 {
        V4 {
            x,
            y,
            z,
            w,
        }
    }

    pub fn splat(value: f32) -> V4 {
        V4 {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    pub fn zero() -> V4 {
        V4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn z(&self) -> f32 {
        self.z
    }

    pub fn w(&self) -> f32 {
        self.w
    }

    pub fn len(self) -> f32 {
        unsafe { intrinsics::sqrtf32(self.dot(self)) }
    }

    pub fn normalize(self) -> V4 {
        let len = self.len();
        self / len
    }

    pub fn dot(self, other: V4) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn min(self, other: V4) -> V4 {
        V4 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
            w: self.w.min(other.w),
        }
    }

    pub fn max(self, other: V4) -> V4 {
        V4 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
            w: self.w.max(other.w),
        }
    }
}

impl Add for V4 {
    type Output = V4;

    fn add(self, other: V4) -> V4 {
        V4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl AddAssign for V4 {
    fn add_assign(&mut self, other: V4) {
        *self = *self + other
    }
}

impl Div for V4 {
    type Output = V4;

    fn div(self, other: V4) -> V4 {
        V4 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
            w: self.w / other.w,
        }
    }
}

impl Div<f32> for V4 {
    type Output = V4;

    fn div(self, other: f32) -> V4 {
        V4 {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
            w: self.w / other,
        }
    }
}

impl DivAssign<f32> for V4 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other
    }
}

impl Mul for V4 {
    type Output = V4;

    fn mul(self, other: V4) -> V4 {
        V4 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

impl Mul<f32> for V4 {
    type Output = V4;

    fn mul(self, other: f32) -> V4 {
        V4 {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
            w: self.w * other,
        }
    }
}

impl Sub for V4 {
    type Output = V4;

    fn sub(self, other: V4) -> V4 {
        V4 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}
