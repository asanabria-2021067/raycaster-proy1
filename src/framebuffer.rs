use raylib::prelude::*;

pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    color_buffer: Vec<u8>,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, background_color: Color) -> Self {
        let mut fb = Self {
            width,
            height,
            color_buffer: vec![0; (width * height * 4) as usize],
            background_color,
            current_color: Color::WHITE,
        };
        fb.clear();
        fb
    }

    pub fn clear(&mut self) {
        for px in self.color_buffer.chunks_exact_mut(4) {
            px[0] = self.background_color.r;
            px[1] = self.background_color.g;
            px[2] = self.background_color.b;
            px[3] = self.background_color.a;
        }
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn point(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.color_buffer[idx] = self.current_color.r;
        self.color_buffer[idx + 1] = self.current_color.g;
        self.color_buffer[idx + 2] = self.current_color.b;
        self.color_buffer[idx + 3] = self.current_color.a;
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.color_buffer
    }

    pub fn swap(&self, texture: &mut Texture2D) {
        texture
            .update_texture(self.as_bytes())
            .expect("el buffer no coincide con el tamano de la textura");
    }
}
