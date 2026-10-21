use crate::PointMm;

/// Calculate shortest distance from a point to a line segment in millimeters.
pub fn point_to_segment_distance_mm(pt: PointMm, seg_start: PointMm, seg_end: PointMm) -> f32 {
    let dx = seg_end.x - seg_start.x;
    let dy = seg_end.y - seg_start.y;

    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        return pt.distance(seg_start);
    }

    let t = (((pt.x - seg_start.x) * dx + (pt.y - seg_start.y) * dy) / l2).clamp(0.0, 1.0);
    let projection = PointMm::new(seg_start.x + t * dx, seg_start.y + t * dy);
    pt.distance(projection)
}

/// Hit-test if target point is within tolerance_mm of any segment in a polyline.
pub fn polyline_hit_test_mm(points: &[PointMm], target: PointMm, tolerance_mm: f32) -> bool {
    if points.is_empty() {
        return false;
    }
    if points.len() == 1 {
        if let Some(first) = points.first() {
            return target.distance(*first) <= tolerance_mm;
        }
        return false;
    }

    for window in points.windows(2) {
        if let (Some(p1), Some(p2)) = (window.first(), window.get(1)) {
            if point_to_segment_distance_mm(target, *p1, *p2) <= tolerance_mm {
                return true;
            }
        }
    }

    false
}
