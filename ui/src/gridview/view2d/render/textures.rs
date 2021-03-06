use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::srgb_texture2d::SrgbTexture2d;
use send_wrapper::SendWrapper;
use std::cell::RefCell;

use crate::DISPLAY;

#[derive(Default)]
pub struct CachedSrgbTexture2d {
    cached: Option<SrgbTexture2d>,
    current_size: Option<(u32, u32)>,
}
impl CachedSrgbTexture2d {
    pub fn set_min_size(&mut self, w: u32, h: u32) -> (f64, f64) {
        if let Some((current_w, current_h)) = self.current_size {
            if current_w >= w && current_h >= h {
                return (w as f64 / current_w as f64, h as f64 / current_h as f64);
            }
        }
        self.set_size(w, h);
        (1.0, 1.0)
    }
    pub fn set_size(&mut self, w: u32, h: u32) {
        if self.current_size != Some((w, h)) {
            self.cached =
                Some(SrgbTexture2d::empty(&**DISPLAY, w, h).expect("Failed to create texture"));
            self.current_size = Some((w, h));
        }
    }
    pub fn at_size(&mut self, w: u32, h: u32) -> (&SrgbTexture2d, SimpleFrameBuffer) {
        self.set_size(w, h);
        (self.unwrap(), self.make_fbo())
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn unwrap(&self) -> &SrgbTexture2d {
        self.cached.as_ref().unwrap()
    }
    pub fn make_fbo(&self) -> SimpleFrameBuffer {
        SimpleFrameBuffer::new(&**DISPLAY, self.unwrap()).expect("Failed to create frame buffer")
    }
}

#[derive(Default)]
pub struct TextureCache {
    pub unscaled_cells: CachedSrgbTexture2d,
    pub scaled_cells: CachedSrgbTexture2d,
    pub gridlines: CachedSrgbTexture2d,
}

lazy_static! {
    pub static ref CACHE: SendWrapper<RefCell<TextureCache>> =
        SendWrapper::new(RefCell::new(TextureCache::default()));
}
