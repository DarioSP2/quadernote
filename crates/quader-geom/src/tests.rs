use crate::{
    hit_test::{point_to_segment_distance_mm, polyline_hit_test_mm},
    point::PointMm,
    rect::RectMm,
    spatial_index::SpatialIndex,
    transform::{ViewportTransform, PIXELS_PER_MM_BASE},
};

#[test]
fn test_point_distance() {
    let p1 = PointMm::new(0.0, 0.0);
    let p2 = PointMm::new(3.0, 4.0);
    assert_eq!(p1.distance(p2), 5.0);
}

#[test]
fn test_rect_constructors_and_properties() {
    let r1 = RectMm::new(10.0, 20.0, 30.0, 40.0);
    assert_eq!(r1.min(), PointMm::new(10.0, 20.0));
    assert_eq!(r1.max(), PointMm::new(40.0, 60.0));
    assert_eq!(r1.right(), 40.0);
    assert_eq!(r1.bottom(), 60.0);
    assert_eq!(r1.center(), PointMm::new(25.0, 40.0));

    let r2 = RectMm::from_min_max(PointMm::new(10.0, 20.0), PointMm::new(40.0, 60.0));
    assert_eq!(r1, r2);

    let points = vec![
        PointMm::new(10.0, 60.0),
        PointMm::new(40.0, 20.0),
        PointMm::new(25.0, 30.0),
    ];
    let r3 = RectMm::from_points(&points).expect("Should construct rect from points");
    assert_eq!(r3, r1);

    assert!(RectMm::from_points(&[]).is_none());
}

#[test]
fn test_rect_inflate() {
    let r = RectMm::new(10.0, 10.0, 20.0, 20.0);
    let inflated = r.inflate(5.0);
    assert_eq!(inflated, RectMm::new(5.0, 5.0, 30.0, 30.0));
}

#[test]
fn test_rect_contains_and_intersects() {
    let r1 = RectMm::new(0.0, 0.0, 10.0, 10.0);
    assert!(r1.contains(PointMm::new(5.0, 5.0)));
    assert!(r1.contains(PointMm::new(0.0, 0.0)));
    assert!(r1.contains(PointMm::new(10.0, 10.0)));
    assert!(!r1.contains(PointMm::new(11.0, 5.0)));

    let r2 = RectMm::new(5.0, 5.0, 10.0, 10.0);
    assert!(r1.intersects(&r2));

    let r3 = RectMm::new(20.0, 20.0, 10.0, 10.0);
    assert!(!r1.intersects(&r3));
}

#[test]
fn test_rect_union_and_intersection() {
    let r1 = RectMm::new(0.0, 0.0, 10.0, 10.0);
    let r2 = RectMm::new(5.0, 5.0, 10.0, 10.0);

    let union_r = r1.union(&r2);
    assert_eq!(union_r, RectMm::new(0.0, 0.0, 15.0, 15.0));

    let inter_r = r1.intersection(&r2).expect("Should intersect");
    assert_eq!(inter_r, RectMm::new(5.0, 5.0, 5.0, 5.0));

    let r3 = RectMm::new(20.0, 20.0, 5.0, 5.0);
    assert!(r1.intersection(&r3).is_none());
}

#[test]
fn test_viewport_transform() {
    let vp = ViewportTransform::new(PointMm::new(10.0, 10.0), 2.0);
    let world_pt = PointMm::new(20.0, 30.0);

    let (sx, sy) = vp.mm_to_screen(world_pt);
    let scale = 2.0 * PIXELS_PER_MM_BASE;
    assert_eq!(sx, (20.0 - 10.0) * scale);
    assert_eq!(sy, (30.0 - 10.0) * scale);

    let back_pt = vp.screen_to_mm(sx, sy);
    assert!((back_pt.x - world_pt.x).abs() < 1e-5);
    assert!((back_pt.y - world_pt.y).abs() < 1e-5);

    let rect = RectMm::new(10.0, 10.0, 50.0, 50.0);
    let (rx, ry, rw, rh) = vp.rect_mm_to_screen(rect);
    assert_eq!(rx, 0.0);
    assert_eq!(ry, 0.0);
    assert_eq!(rw, 50.0 * scale);
    assert_eq!(rh, 50.0 * scale);
}

#[test]
fn test_viewport_panning_and_zooming() {
    let mut vp = ViewportTransform::default();
    vp.pan_by_mm(5.0, -5.0);
    assert_eq!(vp.pan_mm, PointMm::new(5.0, -5.0));

    vp.pan_by_screen(PIXELS_PER_MM_BASE, PIXELS_PER_MM_BASE);
    assert_eq!(vp.pan_mm, PointMm::new(4.0, -6.0));

    let screen_x = 100.0;
    let screen_y = 100.0;
    let pt_before = vp.screen_to_mm(screen_x, screen_y);

    vp.zoom_around_screen_point(2.0, screen_x, screen_y);
    let pt_after = vp.screen_to_mm(screen_x, screen_y);

    assert!((pt_before.x - pt_after.x).abs() < 1e-4);
    assert!((pt_before.y - pt_after.y).abs() < 1e-4);
    assert_eq!(vp.zoom, 2.0);
}

#[test]
fn test_hit_testing() {
    let p1 = PointMm::new(0.0, 0.0);
    let p2 = PointMm::new(10.0, 0.0);

    assert_eq!(
        point_to_segment_distance_mm(PointMm::new(5.0, 3.0), p1, p2),
        3.0
    );
    assert_eq!(
        point_to_segment_distance_mm(PointMm::new(-4.0, 0.0), p1, p2),
        4.0
    );

    let polyline = vec![
        PointMm::new(0.0, 0.0),
        PointMm::new(10.0, 0.0),
        PointMm::new(10.0, 10.0),
    ];

    assert!(polyline_hit_test_mm(&polyline, PointMm::new(5.0, 1.0), 1.5));
    assert!(!polyline_hit_test_mm(
        &polyline,
        PointMm::new(5.0, 5.0),
        1.5
    ));
    assert!(polyline_hit_test_mm(
        &polyline,
        PointMm::new(10.0, 5.0),
        1.5
    ));
}

#[test]
fn test_spatial_index() {
    let mut index: SpatialIndex<i32> = SpatialIndex::new();
    assert!(index.is_empty());

    let id1 = 1;
    let rect1 = RectMm::new(0.0, 0.0, 10.0, 10.0);
    index.insert(id1, rect1);

    let id2 = 2;
    let rect2 = RectMm::new(20.0, 20.0, 10.0, 10.0);
    index.insert(id2, rect2);

    assert_eq!(index.len(), 2);

    let pts_query = index.query_point(PointMm::new(5.0, 5.0));
    assert_eq!(pts_query, vec![1]);

    let rect_query = index.query_rect(&RectMm::new(5.0, 5.0, 20.0, 20.0));
    assert_eq!(rect_query, vec![1, 2]);

    assert!(index.remove(&1));
    assert_eq!(index.len(), 1);
    let empty_vec: Vec<i32> = Vec::new();
    assert_eq!(index.query_point(PointMm::new(5.0, 5.0)), empty_vec);

    index.clear();
    assert!(index.is_empty());
}

#[test]
fn test_serde_roundtrip() {
    let rect = RectMm::new(1.0, 2.0, 3.0, 4.0);
    let json = serde_json::to_string(&rect).expect("Serialization failed");
    let deserialized: RectMm = serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(rect, deserialized);
}
