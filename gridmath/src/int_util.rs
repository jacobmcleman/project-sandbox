

pub fn remap(val: i32, from_low: i32, from_high: i32, to_low: i32, to_high: i32) -> i32 {
    (((val - from_low) * (to_high - to_low)) / (from_high - from_low)) + to_low
}

pub fn remap_clamped(val: i32, from_low: i32, from_high: i32, to_low: i32, to_high: i32) -> i32 {
    (((val.clamp(from_low, from_high) - from_low) * (to_high - to_low)) / (from_high - from_low)) + to_low
}