use core::intrinsics;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub};

#[derive(Clone, Copy)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub fn new(x: f32, y: f32) -> V2 {
        V2 {
            x,
            y,
        }
    }

    pub fn splat(value: f32) -> V2 {
        V2 {
            x: value,
            y: value,
        }
    }

    pub fn zero() -> V2 {
        V2 {
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn len(self) -> f32 {
        unsafe { intrinsics::sqrtf32(self.dot(self)) }
    }

    pub fn normalize(self) -> V2 {
        let len = self.len();
        self / len
    }

    pub fn dot(self, other: V2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn min(self, other: V2) -> V2 {
        V2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(self, other: V2) -> V2 {
        V2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl Add for V2 {
    type Output = V2;

    fn add(self, other: V2) -> V2 {
        V2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<f32> for V2 {
    type Output = V2;

    fn add(self, other: f32) -> V2 {
        V2 {
            x: self.x + other,
            y: self.y + other,
        }
    }
}

impl AddAssign for V2 {
    fn add_assign(&mut self, other: V2) {
        *self = *self + other
    }
}

impl Div for V2 {
    type Output = V2;

    fn div(self, other: V2) -> V2 {
        V2 {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}

impl Div<f32> for V2 {
    type Output = V2;

    fn div(self, other: f32) -> V2 {
        V2 {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl DivAssign<f32> for V2 {
    fn div_assign(&mut self, other: f32) {
        *self = *self / other
    }
}

impl Mul for V2 {
    type Output = V2;

    fn mul(self, other: V2) -> V2 {
        V2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl Mul<f32> for V2 {
    type Output = V2;

    fn mul(self, other: f32) -> V2 {
        V2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Sub for V2 {
    type Output = V2;

    fn sub(self, other: V2) -> V2 {
        V2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Sub<f32> for V2 {
    type Output = V2;

    fn sub(self, other: f32) -> V2 {
        V2 {
            x: self.x - other,
            y: self.y - other,
        }
    }
}
