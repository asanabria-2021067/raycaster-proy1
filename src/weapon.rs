use std::f32::consts::PI;

use raylib::prelude::*;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::sprites::Enemy;

const GUN_PATHS: [&str; 3] = [
    "assets/sprites/gun_00.png",
    "assets/sprites/gun_01.png",
    "assets/sprites/gun_02.png",
];

const FIRE_DURATION: f32 = 0.2; // duracion total de la animacion de disparo/retroceso
const HIT_RADIUS: f32 = 20.0; // radio de colision angular del enemigo, en px de mundo
const GUN_SCALE: f32 = 0.6; // porcion del ancho de ventana que ocupa el arma
const ALPHA_THRESHOLD: u8 = 20;

pub struct GunSprite {
    width: i32,
    height: i32,
    frames: Vec<Vec<Color>>,
}

impl GunSprite {
    pub fn new() -> Self {
        let images: Vec<Image> = GUN_PATHS
            .iter()
            .map(|path| {
                Image::load_image(path).unwrap_or_else(|e| panic!("no se pudo cargar el arma {path}: {e}"))
            })
            .collect();

        let width = images[0].width();
        let height = images[0].height();
        let frames = images.iter().map(|img| img.get_image_data().to_vec()).collect();

        Self { width, height, frames }
    }

    fn sample(&self, frame: usize, tx: f32, ty: f32) -> Color {
        let x = (tx.clamp(0.0, 0.9999) * self.width as f32) as usize;
        let y = (ty.clamp(0.0, 0.9999) * self.height as f32) as usize;
        self.frames[frame][y * self.width as usize + x]
    }
}

impl Default for GunSprite {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Weapon {
    fire_timer: f32,
}

impl Weapon {
    pub fn new() -> Self {
        Self { fire_timer: 0.0 }
    }

    pub fn update(&mut self, dt: f32) {
        self.fire_timer = (self.fire_timer - dt).max(0.0);
    }

    // dispara si no esta en cooldown; retorna true si el disparo se realizo
    pub fn try_fire(&mut self) -> bool {
        if self.fire_timer > 0.0 {
            return false;
        }
        self.fire_timer = FIRE_DURATION;
        true
    }

    fn frame_index(&self) -> usize {
        if self.fire_timer <= 0.0 {
            0
        } else if self.fire_timer > FIRE_DURATION * 0.5 {
            1
        } else {
            2
        }
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Self::new()
    }
}

// hitbox angular: elimina al enemigo mas cercano dentro del cono central si no hay pared antes
pub fn hitscan(
    enemies: &mut Vec<Enemy>,
    maze: &Maze,
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    block_size: i32,
) -> bool {
    let wall_dist = cast_ray(maze, player_x, player_y, player_angle, block_size).distance;

    let mut best: Option<(usize, f32)> = None;
    for (i, enemy) in enemies.iter().enumerate() {
        let dx = enemy.x - player_x;
        let dy = enemy.y - player_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1.0 || dist >= wall_dist {
            continue;
        }

        let mut rel_angle = dy.atan2(dx) - player_angle;
        while rel_angle > PI {
            rel_angle -= 2.0 * PI;
        }
        while rel_angle < -PI {
            rel_angle += 2.0 * PI;
        }

        let angular_radius = (HIT_RADIUS / dist).atan();
        if rel_angle.abs() <= angular_radius && best.is_none_or(|(_, d)| dist < d) {
            best = Some((i, dist));
        }
    }

    match best {
        Some((i, _)) => {
            enemies.remove(i);
            true
        }
        None => false,
    }
}

pub fn draw_gun(fb: &mut Framebuffer, gun: &GunSprite, weapon: &Weapon, window_width: i32, window_height: i32) {
    let frame = weapon.frame_index();
    let sprite_w = window_width as f32 * GUN_SCALE;
    let sprite_h = sprite_w;
    let x0 = ((window_width as f32 - sprite_w) / 2.0) as i32;
    let y0 = (window_height as f32 - sprite_h) as i32;

    for y in 0..sprite_h as i32 {
        let ty = y as f32 / sprite_h;
        for x in 0..sprite_w as i32 {
            let tx = x as f32 / sprite_w;
            let color = gun.sample(frame, tx, ty);
            if color.a < ALPHA_THRESHOLD {
                continue;
            }
            fb.set_current_color(color);
            fb.point(x0 + x, y0 + y);
        }
    }
}
