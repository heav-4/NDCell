use super::*;

// TODO rewrite zoom

#[derive(Debug, Default)]
pub struct Viewport2D {
    /// Cell position that is at the center of the viewport.
    pub pos: Vec2D,
    /// Offset along the X axis (0..1).
    pub x_offset: f32,
    /// Offset along the Y axis (0..1).
    pub y_offset: f32,
    /// The zoom level.
    pub zoom: Zoom2D,
}

impl Viewport2D {
    /// Scroll the viewport by the given number of pixels along each axis.
    pub fn scroll_pixels(&mut self, mut dx: f32, mut dy: f32) {
        self.scroll_cells(
            dx * self.zoom.cells_per_pixel(),
            dy * self.zoom.cells_per_pixel(),
        );
    }
    /// Scroll the viewport by the given number of cells along each axis.
    pub fn scroll_cells(&mut self, mut dx: f32, mut dy: f32) {
        // Add dx and dy.
        self.x_offset += dx;
        self.y_offset += dy;
        // Remove the integral part from self.x_offset and self.y_offset,
        // leaving only the fraction part between 0 and 1.
        let int_dx = self.x_offset.floor();
        let int_dy = self.y_offset.floor();
        self.x_offset -= int_dx;
        self.y_offset -= int_dy;
        // Add the integral part that we removed onto self.pos.
        let int_dx = int_dx as isize;
        let int_dy = int_dy as isize;
        self.pos += Vec2D::from([int_dx, int_dy]);
    }
    /// Snap to the nearest integer cell position.
    pub fn snap_pos(&mut self) {
        if self.x_offset >= 0.5 {
            self.pos += Vec2D::from([1, 0]);
        }
        if self.y_offset >= 0.5 {
            self.pos += Vec2D::from([0, 1]);
        }
        self.x_offset = 0.0;
        self.y_offset = 0.0;
    }
    /// Set the zoom level.
    pub fn set_zoom(&mut self, new_zoom: Zoom2D) {
        self.zoom = new_zoom.clamp();
    }
    /// Zoom in or out by the given factor.
    pub fn zoom_by(&mut self, factor: f32) {
        assert!(
            factor > 0.0,
            "Zoom factor must be a positive number, not {}",
            factor
        );
        self.set_zoom(self.zoom * factor);
    }
}