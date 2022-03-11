pub fn lerp(from: f32, to: f32, by: f32) -> f32 {
    from + (to - from) * by.clamp(0.0, 1.0)
}

pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if a != b {
        ((value - a) / (b - a)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta  
    }
}
