use serde::{Deserialize, Serialize};

/// 2D size in millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SizeMm {
    pub width: f32,
    pub height: f32,
}

impl SizeMm {
    #[inline]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
