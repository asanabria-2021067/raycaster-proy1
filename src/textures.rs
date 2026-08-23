use raylib::prelude::*;

const TEXTURE_PATHS: [&str; 4] = [
    "assets/textures/wall1.png",
    "assets/textures/wall2.png",
    "assets/textures/wall3.png",
    "assets/textures/wall4.png",
];

const FLOOR_PATH: &str = "assets/textures/floor.png";
const CEILING_PATH: &str = "assets/textures/ceiling.png";

const FALLBACK_SIZE: i32 = 64;
const CHECKER_TILE: i32 = 8;

const WALL_FALLBACK_COLORS: [(Color, Color); 4] = [
    (Color::new(200, 50, 50, 255), Color::new(120, 20, 20, 255)),
    (Color::new(50, 200, 50, 255), Color::new(20, 120, 20, 255)),
    (Color::new(50, 50, 200, 255), Color::new(20, 20, 120, 255)),
    (Color::new(200, 200, 50, 255), Color::new(120, 120, 20, 255)),
];
const FLOOR_FALLBACK: (Color, Color) = (Color::new(90, 90, 90, 255), Color::new(60, 60, 60, 255));
const CEILING_FALLBACK: (Color, Color) = (Color::new(40, 60, 90, 255), Color::new(20, 30, 60, 255));

fn checkerboard(width: i32, height: i32, color_a: Color, color_b: Color) -> Vec<Color> {
    (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| if ((x / CHECKER_TILE) + (y / CHECKER_TILE)) % 2 == 0 { color_a } else { color_b })
        .collect()
}

fn try_load(path: &str) -> Option<(i32, i32, Vec<Color>)> {
    match Image::load_image(path) {
        Ok(image) => Some((image.width(), image.height(), image.get_image_data().to_vec())),
        Err(e) => {
            eprintln!("advertencia: no se pudo cargar la textura {path}: {e}, se usara un reemplazo");
            None
        }
    }
}

fn load_pixels_or_fallback(path: &str, fallback: (Color, Color)) -> (i32, i32, Vec<Color>) {
    match try_load(path) {
        Some(data) => data,
        None => (FALLBACK_SIZE, FALLBACK_SIZE, checkerboard(FALLBACK_SIZE, FALLBACK_SIZE, fallback.0, fallback.1)),
    }
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
        let loaded: Vec<Option<(i32, i32, Vec<Color>)>> = TEXTURE_PATHS.iter().map(|path| try_load(path)).collect();
        let (width, height) =
            loaded.iter().flatten().next().map(|(w, h, _)| (*w, *h)).unwrap_or((FALLBACK_SIZE, FALLBACK_SIZE));

        let pixels = loaded
            .into_iter()
            .enumerate()
            .map(|(i, data)| match data {
                Some((w, h, px)) if w == width && h == height => px,
                _ => {
                    let (a, b) = WALL_FALLBACK_COLORS[i % WALL_FALLBACK_COLORS.len()];
                    checkerboard(width, height, a, b)
                }
            })
            .collect();

        let (floor_width, floor_height, floor_pixels) = load_pixels_or_fallback(FLOOR_PATH, FLOOR_FALLBACK);
        let (ceiling_width, ceiling_height, ceiling_pixels) = load_pixels_or_fallback(CEILING_PATH, CEILING_FALLBACK);

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
