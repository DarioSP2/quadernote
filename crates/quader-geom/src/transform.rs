use crate::{PointMm, RectMm};
use serde::{Deserialize, Serialize};

pub const PIXELS_PER_MM_BASE: f32 = 3.779_527_6;

/// Viewport transform handling pan and zoom in millimeters to screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewportTransform {
    pub pan_mm: PointMm,
    pub zoom: f32,
}

impl Default for ViewportTransform {
    fn default() -> Self {
        Self {
            pan_mm: PointMm::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl ViewportTransform {
    pub const PIXELS_PER_MM_BASE: f32 = PIXELS_PER_MM_BASE;

    pub fn new(pan_mm: PointMm, zoom: f32) -> Self {
        Self {
            pan_mm,
            zoom: zoom.clamp(0.1, 10.0),
        }
    }

    pub fn mm_to_screen(&self, pt: PointMm) -> (f32, f32) {
        let scale = self.zoom * PIXELS_PER_MM_BASE;
        let sx = (pt.x - self.pan_mm.x) * scale;
        let sy = (pt.y - self.pan_mm.y) * scale;
        (sx, sy)
    }

    pub fn screen_to_mm(&self, sx: f32, sy: f32) -> PointMm {
        let scale = self.zoom * PIXELS_PER_MM_BASE;
        if scale == 0.0 {
            return self.pan_mm;
        }
        let mx = sx / scale + self.pan_mm.x;
        let my = sy / scale + self.pan_mm.y;
        PointMm::new(mx, my)
    }

    pub fn rect_mm_to_screen(&self, rect: RectMm) -> (f32, f32, f32, f32) {
        let (sx, sy) = self.mm_to_screen(rect.origin);
        let scale = self.zoom * PIXELS_PER_MM_BASE;
        let sw = rect.size.width * scale;
        let sh = rect.size.height * scale;
        (sx, sy, sw, sh)
    }

    pub fn pan_by_mm(&mut self, dx_mm: f32, dy_mm: f32) {
        self.pan_mm.x += dx_mm;
        self.pan_mm.y += dy_mm;
    }

    pub fn pan_by_screen(&mut self, dx_px: f32, dy_px: f32) {
        let scale = self.zoom * PIXELS_PER_MM_BASE;
        if scale > 0.0 {
            self.pan_mm.x -= dx_px / scale;
            self.pan_mm.y -= dy_px / scale;
        }
    }

    pub fn zoom_around_screen_point(&mut self, new_zoom: f32, screen_x: f32, screen_y: f32) {
        let clamped_zoom = new_zoom.clamp(0.1, 10.0);
        let world_before = self.screen_to_mm(screen_x, screen_y);
        self.zoom = clamped_zoom;
        let world_after = self.screen_to_mm(screen_x, screen_y);

        self.pan_mm.x += world_before.x - world_after.x;
        self.pan_mm.y += world_before.y - world_after.y;
    }
}
