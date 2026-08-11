use raylib::prelude::*;

const TEXTURE_PATHS: [&str; 4] = [
    "assets/textures/wall1.png",
    "assets/textures/wall2.png",
    "assets/textures/wall3.png",
    "assets/textures/wall4.png",
];

pub struct TextureManager {
    width: i32,
    height: i32,
    pixels: Vec<Vec<Color>>,
}

impl TextureManager {
    pub fn new() -> Self {
        let images: Vec<Image> = TEXTURE_PATHS
            .iter()
            .map(|path| {
                Image::load_image(path)
                    .unwrap_or_else(|e| panic!("no se pudo cargar la textura {path}: {e}"))
            })
            .collect();

        let width = images[0].width();
        let height = images[0].height();
        let pixels = images.iter().map(|img| img.get_image_data().to_vec()).collect();

        Self { width, height, pixels }
    }

    pub fn sample(&self, wall_index: usize, tx: f32, ty: f32) -> Color {
        let x = (tx.clamp(0.0, 0.9999) * self.width as f32) as usize;
        let y = (ty.clamp(0.0, 0.9999) * self.height as f32) as usize;
        self.pixels[wall_index][y * self.width as usize + x]
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}
