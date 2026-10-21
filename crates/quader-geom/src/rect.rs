use crate::{PointMm, SizeMm};
use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box in millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct RectMm {
    pub origin: PointMm,
    pub size: SizeMm,
}

impl RectMm {
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: PointMm::new(x, y),
            size: SizeMm::new(width, height),
        }
    }

    #[inline]
    pub fn from_min_max(min: PointMm, max: PointMm) -> Self {
        let width = (max.x - min.x).max(0.0);
        let height = (max.y - min.y).max(0.0);
        Self {
            origin: min,
            size: SizeMm::new(width, height),
        }
    }

    pub fn from_points(points: &[PointMm]) -> Option<Self> {
        let first = points.first()?;
        let mut min_x = first.x;
        let mut max_x = first.x;
        let mut min_y = first.y;
        let mut max_y = first.y;

        for pt in points.iter().skip(1) {
            if pt.x < min_x {
                min_x = pt.x;
            }
            if pt.x > max_x {
                max_x = pt.x;
            }
            if pt.y < min_y {
                min_y = pt.y;
            }
            if pt.y > max_y {
                max_y = pt.y;
            }
        }

        Some(Self::from_min_max(
            PointMm::new(min_x, min_y),
            PointMm::new(max_x, max_y),
        ))
    }

    #[inline]
    pub fn min(&self) -> PointMm {
        self.origin
    }

    #[inline]
    pub fn max(&self) -> PointMm {
        PointMm::new(self.right(), self.bottom())
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.origin.x + self.size.width
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.origin.y + self.size.height
    }

    #[inline]
    pub fn center(&self) -> PointMm {
        PointMm::new(
            self.origin.x + self.size.width * 0.5,
            self.origin.y + self.size.height * 0.5,
        )
    }

    pub fn inflate(&self, margin_mm: f32) -> Self {
        Self::new(
            self.origin.x - margin_mm,
            self.origin.y - margin_mm,
            (self.size.width + margin_mm * 2.0).max(0.0),
            (self.size.height + margin_mm * 2.0).max(0.0),
        )
    }

    #[inline]
    pub fn contains(&self, pt: PointMm) -> bool {
        pt.x >= self.origin.x
            && pt.x <= self.right()
            && pt.y >= self.origin.y
            && pt.y <= self.bottom()
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.origin.x < other.right()
            && self.right() > other.origin.x
            && self.origin.y < other.bottom()
            && self.bottom() > other.origin.y
    }

    pub fn union(&self, other: &Self) -> Self {
        let min_x = self.origin.x.min(other.origin.x);
        let min_y = self.origin.y.min(other.origin.y);
        let max_x = self.right().max(other.right());
        let max_y = self.bottom().max(other.bottom());
        Self::from_min_max(PointMm::new(min_x, min_y), PointMm::new(max_x, max_y))
    }

    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }
        let min_x = self.origin.x.max(other.origin.x);
        let min_y = self.origin.y.max(other.origin.y);
        let max_x = self.right().min(other.right());
        let max_y = self.bottom().min(other.bottom());
        if min_x <= max_x && min_y <= max_y {
            Some(Self::from_min_max(
                PointMm::new(min_x, min_y),
                PointMm::new(max_x, max_y),
            ))
        } else {
            None
        }
    }
}
