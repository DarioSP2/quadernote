//! Spatial geometry for QuaderNote in millimeters (mm).

pub mod hit_test;
pub mod point;
pub mod rect;
pub mod size;
pub mod spatial_index;
pub mod transform;

#[cfg(test)]
mod tests;

pub use hit_test::{point_to_segment_distance_mm, polyline_hit_test_mm};
pub use point::PointMm;
pub use rect::RectMm;
pub use size::SizeMm;
pub use spatial_index::SpatialIndex;
pub use transform::ViewportTransform;
