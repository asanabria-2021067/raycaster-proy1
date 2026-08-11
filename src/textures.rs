use raylib::prelude::*;

const TEXTURE_PATHS: [&str; 4] = [
    "assets/textures/wall1.png",
    "assets/textures/wall2.png",
    "assets/textures/wall3.png",
    "assets/textures/wall4.png",
];

const FLOOR_PATH: &str = "assets/textures/floor.png";
const CEILING_PATH: &str = "assets/textures/ceiling.png";

fn load_pixels(path: &str) -> (i32, i32, Vec<Color>) {
    let image = Image::load_image(path).unwrap_or_else(|e| panic!("no se pudo cargar la textura {path}: {e}"));
    (image.width(), image.height(), image.get_image_data().to_vec())
}

pub struct TextureManager {
    width: i32,
    height: i32,
    pixels: Vec<Vec<Color>>,
    floor_width: i32,
    floor_height: i32,
    floor_pixels: Vec<Color>,
    ceiling_width: i32,
    ceiling_height: i32,
    ceiling_pixels: Vec<Color>,
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

        let (floor_width, floor_height, floor_pixels) = load_pixels(FLOOR_PATH);
        let (ceiling_width, ceiling_height, ceiling_pixels) = load_pixels(CEILING_PATH);

        Self {
            width,
            height,
            pixels,
            floor_width,
            floor_height,
            floor_pixels,
            ceiling_width,
            ceiling_height,
            ceiling_pixels,
        }
    }

    pub fn sample_wall(&self, wall_index: usize, tx: f32, ty: f32) -> Color {
        let x = (tx.clamp(0.0, 0.9999) * self.width as f32) as usize;
        let y = (ty.clamp(0.0, 0.9999) * self.height as f32) as usize;
        self.pixels[wall_index][y * self.width as usize + x]
    }

    pub fn sample_floor(&self, tx: f32, ty: f32) -> Color {
        let x = (tx * self.floor_width as f32) as usize % self.floor_width as usize;
        let y = (ty * self.floor_height as f32) as usize % self.floor_height as usize;
        self.floor_pixels[y * self.floor_width as usize + x]
    }

    pub fn sample_ceiling(&self, tx: f32, ty: f32) -> Color {
        let x = (tx * self.ceiling_width as f32) as usize % self.ceiling_width as usize;
        let y = (ty * self.ceiling_height as f32) as usize % self.ceiling_height as usize;
        self.ceiling_pixels[y * self.ceiling_width as usize + x]
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}
