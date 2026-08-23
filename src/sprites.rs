use std::f32::consts::PI;

use raylib::prelude::*;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::{is_wall, Maze};

const SPRITE_PATHS: [&str; 4] = [
    "assets/sprites/enemy_00.png",
    "assets/sprites/enemy_01.png",
    "assets/sprites/enemy_02.png",
    "assets/sprites/enemy_03.png",
];

const SILHOUETTE_COLOR: Color = Color::new(180, 20, 20, 255);
const ALPHA_THRESHOLD: u8 = 20;
const FOV_MARGIN: f32 = 0.3; // margen para no recortar sprites a medio entrar en pantalla
const ANIM_FRAME_SECONDS: f32 = 0.15;

const CHASE_RANGE: f32 = 400.0; // px, radio de deteccion
const CHASE_SPEED: f32 = 90.0; // px/seg
const STOP_DISTANCE: f32 = 24.0; // no se acerca mas al jugador que esto
const ENEMY_RADIUS: f32 = 10.0;

pub fn advance_frame(current: usize, timer: &mut f32, dt: f32, frame_count: usize) -> usize {
    if frame_count == 0 {
        return 0;
    }
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
        let loaded: Vec<Option<(i32, i32, Vec<Color>)>> = SPRITE_PATHS
            .iter()
            .map(|path| match Image::load_image(path) {
                Ok(image) => Some((image.width(), image.height(), image.get_image_data().to_vec())),
                Err(e) => {
                    eprintln!("advertencia: no se pudo cargar el sprite {path}: {e}, se usara un reemplazo");
                    None
                }
            })
            .collect();

        if loaded.iter().all(Option::is_none) {
            eprintln!("advertencia: ningun sprite de enemigo cargo, enemigos deshabilitados");
            return Self { width: 0, height: 0, frames: Vec::new() };
        }

        let (width, height) = loaded.iter().flatten().next().map(|(w, h, _)| (*w, *h)).unwrap();
        let frames = loaded
            .into_iter()
            .map(|data| match data {
                Some((w, h, px)) if w == width && h == height => px,
                _ => vec![SILHOUETTE_COLOR; (width * height) as usize],
            })
            .collect();

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

impl Enemy {
    // persigue al jugador si esta en rango y hay linea de vision directa (sin paredes de por medio)
    pub fn update_ai(&mut self, maze: &Maze, block_size: i32, player_x: f32, player_y: f32, dt: f32) {
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if !(STOP_DISTANCE..=CHASE_RANGE).contains(&dist) {
            return;
        }

        let angle = dy.atan2(dx);
        let wall_dist = cast_ray(maze, self.x, self.y, angle, block_size).distance;
        if wall_dist < dist {
            return;
        }

        let step = CHASE_SPEED * dt / dist;
        let new_x = self.x + dx * step;
        let new_y = self.y + dy * step;
        if !enemy_collides(maze, block_size, new_x, self.y) {
            self.x = new_x;
        }
        if !enemy_collides(maze, block_size, self.x, new_y) {
            self.y = new_y;
        }
    }
}

fn enemy_collides(maze: &Maze, block_size: i32, x: f32, y: f32) -> bool {
    let bs = block_size as f32;
    let corners = [
        (x - ENEMY_RADIUS, y - ENEMY_RADIUS),
        (x + ENEMY_RADIUS, y - ENEMY_RADIUS),
        (x - ENEMY_RADIUS, y + ENEMY_RADIUS),
        (x + ENEMY_RADIUS, y + ENEMY_RADIUS),
    ];
    corners.iter().any(|&(cx, cy)| {
        let i = cx / bs;
        let j = cy / bs;
        i < 0.0 || j < 0.0 || is_wall(maze, i as usize, j as usize)
    })
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
    if sprites.frame_count() == 0 {
        return;
    }

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
