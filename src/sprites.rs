use std::f32::consts::PI;

use raylib::prelude::*;

use crate::framebuffer::Framebuffer;

const SPRITE_PATHS: [&str; 4] = [
    "assets/sprites/enemy_00.png",
    "assets/sprites/enemy_01.png",
    "assets/sprites/enemy_02.png",
    "assets/sprites/enemy_03.png",
];

const ALPHA_THRESHOLD: u8 = 20;
const FOV_MARGIN: f32 = 0.3; // margen para no recortar sprites a medio entrar en pantalla
const ANIM_FRAME_SECONDS: f32 = 0.15;

pub fn advance_frame(current: usize, timer: &mut f32, dt: f32, frame_count: usize) -> usize {
    *timer += dt;
    if *timer >= ANIM_FRAME_SECONDS {
        *timer -= ANIM_FRAME_SECONDS;
        (current + 1) % frame_count
    } else {
        current
    }
}

pub struct SpriteManager {
    width: i32,
    height: i32,
    frames: Vec<Vec<Color>>,
}

impl SpriteManager {
    pub fn new() -> Self {
        let images: Vec<Image> = SPRITE_PATHS
            .iter()
            .map(|path| {
                Image::load_image(path)
                    .unwrap_or_else(|e| panic!("no se pudo cargar el sprite {path}: {e}"))
            })
            .collect();

        let width = images[0].width();
        let height = images[0].height();
        let frames = images.iter().map(|img| img.get_image_data().to_vec()).collect();

        Self { width, height, frames }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    fn sample(&self, frame: usize, tx: f32, ty: f32) -> Color {
        let x = (tx.clamp(0.0, 0.9999) * self.width as f32) as usize;
        let y = (ty.clamp(0.0, 0.9999) * self.height as f32) as usize;
        self.frames[frame][y * self.width as usize + x]
    }
}

impl Default for SpriteManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Enemy {
    pub x: f32,
    pub y: f32,
}

fn normalize_angle(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}

#[allow(clippy::too_many_arguments)]
pub fn render_sprites(
    fb: &mut Framebuffer,
    enemies: &[Enemy],
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    fov: f32,
    window_width: i32,
    window_height: i32,
    block_size: i32,
    zbuffer: &[f32],
    sprites: &SpriteManager,
    frame: usize,
) {
    let dist_to_projection_plane = (window_width as f32 / 2.0) / (fov / 2.0).tan();

    // pintar del mas lejano al mas cercano para que los sprites cercanos tapen a los lejanos
    let mut order: Vec<usize> = (0..enemies.len()).collect();
    order.sort_by(|&a, &b| {
        let da = (enemies[a].x - player_x).powi(2) + (enemies[a].y - player_y).powi(2);
        let db = (enemies[b].x - player_x).powi(2) + (enemies[b].y - player_y).powi(2);
        db.partial_cmp(&da).unwrap()
    });

    for &idx in &order {
        let enemy = &enemies[idx];
        let dx = enemy.x - player_x;
        let dy = enemy.y - player_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1.0 {
            continue;
        }

        let rel_angle = normalize_angle(dy.atan2(dx) - player_angle);
        if rel_angle.abs() > fov / 2.0 + FOV_MARGIN {
            continue;
        }

        let corrected_dist = (dist * rel_angle.cos()).max(1.0);
        let sprite_size = (block_size as f32 / corrected_dist) * dist_to_projection_plane;
        let screen_x = window_width as f32 / 2.0 + rel_angle.tan() * dist_to_projection_plane;

        let x0 = (screen_x - sprite_size / 2.0).max(0.0) as i32;
        let x1 = (screen_x + sprite_size / 2.0).min(window_width as f32) as i32;
        let center_y = window_height as f32 / 2.0;
        let y0 = (center_y - sprite_size / 2.0).max(0.0) as i32;
        let y1 = (center_y + sprite_size / 2.0).min(window_height as f32) as i32;

        for x in x0..x1 {
            if x < 0 || x as usize >= zbuffer.len() || corrected_dist >= zbuffer[x as usize] {
                continue;
            }
            let tx = (x - x0) as f32 / sprite_size.max(1.0);
            for y in y0..y1 {
                let ty = (y as f32 - (center_y - sprite_size / 2.0)) / sprite_size.max(1.0);
                let color = sprites.sample(frame, tx, ty);
                if color.a < ALPHA_THRESHOLD {
                    continue;
                }
                fb.set_current_color(color);
                fb.point(x, y);
            }
        }
    }
}
