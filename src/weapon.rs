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
const SILHOUETTE_COLOR: Color = Color::new(90, 90, 90, 255);
pub const MAX_AMMO: i32 = 12;
const RELOAD_DURATION: f32 = 1.2;

pub struct GunSprite {
    width: i32,
    height: i32,
    frames: Vec<Vec<Color>>,
}

impl GunSprite {
    pub fn new() -> Self {
        let loaded: Vec<Option<(i32, i32, Vec<Color>)>> = GUN_PATHS
            .iter()
            .map(|path| match Image::load_image(path) {
                Ok(image) => Some((image.width(), image.height(), image.get_image_data().to_vec())),
                Err(e) => {
                    eprintln!("advertencia: no se pudo cargar el arma {path}: {e}, se usara un reemplazo");
                    None
                }
            })
            .collect();

        if loaded.iter().all(Option::is_none) {
            eprintln!("advertencia: ningun sprite de arma cargo, arma deshabilitada");
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
    reload_timer: f32,
    ammo: i32,
}

impl Weapon {
    pub fn new() -> Self {
        Self { fire_timer: 0.0, reload_timer: 0.0, ammo: MAX_AMMO }
    }

    pub fn ammo(&self) -> i32 {
        self.ammo
    }

    pub fn is_reloading(&self) -> bool {
        self.reload_timer > 0.0
    }

    pub fn update(&mut self, dt: f32) {
        self.fire_timer = (self.fire_timer - dt).max(0.0);
        if self.reload_timer > 0.0 {
            self.reload_timer = (self.reload_timer - dt).max(0.0);
            if self.reload_timer == 0.0 {
                self.ammo = MAX_AMMO;
            }
        }
    }

    // dispara si hay municion, no esta recargando ni en cooldown; retorna true si el disparo se realizo
    pub fn try_fire(&mut self) -> bool {
        if self.fire_timer > 0.0 || self.is_reloading() || self.ammo <= 0 {
            return false;
        }
        self.fire_timer = FIRE_DURATION;
        self.ammo -= 1;
        true
    }

    // inicia recarga si hay espacio y no se esta recargando ya; retorna true si arranco
    pub fn try_reload(&mut self) -> bool {
        if self.is_reloading() || self.ammo >= MAX_AMMO {
            return false;
        }
        self.reload_timer = RELOAD_DURATION;
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
    if gun.frames.is_empty() {
        return;
    }
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
