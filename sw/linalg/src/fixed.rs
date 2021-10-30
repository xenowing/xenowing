// TODO: Consider fixed point type with shift as const parameter

pub fn mul(x: i32, y: i32, shift: u32) -> i32 {
    ((x as i64 * y as i64) >> shift) as _
}
