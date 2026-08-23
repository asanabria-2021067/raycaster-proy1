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

const HIT_RADIUS: f32 = 20.0; // radio de colision angular del enemigo, en px de mundo
const GUN_SCALE: f32 = 0.45; // porcion del ancho de ventana que ocupa el arma
const ALPHA_THRESHOLD: u8 = 20;
const SILHOUETTE_COLOR: Color = Color::new(90, 90, 90, 255);
const PICKUP_RADIUS: f32 = 24.0; // px, distancia jugador-arma para recogerla
const PICKUP_SIZE: f32 = 26.0; // px de mundo, tamaño del brillo del pickup en pantalla

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Pistol,
    Rifle,
}

struct WeaponStats {
    fire_duration: f32,
    max_ammo: i32,
    reload_duration: f32,
}

impl WeaponKind {
    fn stats(self) -> WeaponStats {
        match self {
            WeaponKind::Pistol => WeaponStats { fire_duration: 0.2, max_ammo: 12, reload_duration: 1.2 },
            WeaponKind::Rifle => WeaponStats { fire_duration: 0.1, max_ammo: 24, reload_duration: 1.6 },
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WeaponKind::Pistol => "PISTOLA",
            WeaponKind::Rifle => "RIFLE",
        }
    }

    // tinte multiplicativo para distinguir armas sin arte nuevo (blanco = sin cambio)
    fn tint(self) -> Color {
        match self {
            WeaponKind::Pistol => Color::WHITE,
            WeaponKind::Rifle => Color::new(170, 210, 255, 255),
        }
    }
}

pub struct Pickup {
    pub x: f32,
    pub y: f32,
    pub kind: WeaponKind,
}

pub struct GunSprite {
    width: i32,
    height: i32,
    frames: Vec<Vec<Color>>,
}

impl GunSprite {
    pub fn new() -> Self {
        let loaded: Vec<Option<(i32, i32, Vec<Color>)>> = GUN_PATHS
            .iter()
            .map(|path| match Image::load_image(&crate::paths::resolve(path)) {
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

    // ancho/alto del PNG fuente; usarlo evita estirar el arma si no es cuadrada
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl Default for GunSprite {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Weapon {
    kind: WeaponKind,
    fire_timer: f32,
    reload_timer: f32,
    ammo: i32,
}

impl Weapon {
    pub fn new(kind: WeaponKind) -> Self {
        Self { kind, fire_timer: 0.0, reload_timer: 0.0, ammo: kind.stats().max_ammo }
    }

    pub fn kind(&self) -> WeaponKind {
        self.kind
    }

    pub fn ammo(&self) -> i32 {
        self.ammo
    }

    pub fn max_ammo(&self) -> i32 {
        self.kind.stats().max_ammo
    }

    pub fn is_reloading(&self) -> bool {
        self.reload_timer > 0.0
    }

    pub fn update(&mut self, dt: f32) {
        self.fire_timer = (self.fire_timer - dt).max(0.0);
        if self.reload_timer > 0.0 {
            self.reload_timer = (self.reload_timer - dt).max(0.0);
            if self.reload_timer == 0.0 {
                self.ammo = self.kind.stats().max_ammo;
            }
        }
    }

    // dispara si hay municion, no esta recargando ni en cooldown; retorna true si el disparo se realizo
    pub fn try_fire(&mut self) -> bool {
        if self.fire_timer > 0.0 || self.is_reloading() || self.ammo <= 0 {
            return false;
        }
        self.fire_timer = self.kind.stats().fire_duration;
        self.ammo -= 1;
        true
    }

    // inicia recarga si hay espacio y no se esta recargando ya; retorna true si arranco
    pub fn try_reload(&mut self) -> bool {
        let max_ammo = self.kind.stats().max_ammo;
        if self.is_reloading() || self.ammo >= max_ammo {
            return false;
        }
        self.reload_timer = self.kind.stats().reload_duration;
        true
    }

    fn frame_index(&self) -> usize {
        let fire_duration = self.kind.stats().fire_duration;
        if self.fire_timer <= 0.0 {
            0
        } else if self.fire_timer > fire_duration * 0.5 {
            1
        } else {
            2
        }
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
    let tint = weapon.kind().tint();
    let sprite_w = window_width as f32 * GUN_SCALE;
    let sprite_h = sprite_w / gun.aspect();
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
            let color = Color::new(
                ((color.r as u32 * tint.r as u32) / 255) as u8,
                ((color.g as u32 * tint.g as u32) / 255) as u8,
                ((color.b as u32 * tint.b as u32) / 255) as u8,
                color.a,
            );
            fb.set_current_color(color);
            fb.point(x0 + x, y0 + y);
        }
    }
}

// recoge el pickup mas cercano si el jugador esta encima; devuelve el arma recogida
pub fn try_collect_pickup(pickups: &mut Vec<Pickup>, player_x: f32, player_y: f32) -> Option<WeaponKind> {
    let idx = pickups.iter().position(|p| {
        let dx = p.x - player_x;
        let dy = p.y - player_y;
        (dx * dx + dy * dy).sqrt() < PICKUP_RADIUS
    })?;
    Some(pickups.remove(idx).kind)
}

// brillo tipo diamante que marca las armas tiradas en el mapa; reusa la proyeccion de billboard
#[allow(clippy::too_many_arguments)]
pub fn render_pickups(
    fb: &mut Framebuffer,
    pickups: &[Pickup],
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    fov: f32,
    window_width: i32,
    window_height: i32,
    zbuffer: &[f32],
) {
    if zbuffer.is_empty() {
        return;
    }
    let dist_to_projection_plane = (window_width as f32 / 2.0) / (fov / 2.0).tan();
    let glow = Color::new(255, 215, 90, 255);

    for pickup in pickups {
        let dx = pickup.x - player_x;
        let dy = pickup.y - player_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 1.0 {
            continue;
        }

        let mut rel_angle = dy.atan2(dx) - player_angle;
        while rel_angle > PI {
            rel_angle -= 2.0 * PI;
        }
        while rel_angle < -PI {
            rel_angle += 2.0 * PI;
        }
        if rel_angle.abs() > fov / 2.0 + 0.3 {
            continue;
        }

        let corrected_dist = (dist * rel_angle.cos()).max(1.0);
        let size = (PICKUP_SIZE / corrected_dist) * dist_to_projection_plane;
        let screen_x = window_width as f32 / 2.0 + rel_angle.tan() * dist_to_projection_plane;
        let center_y = window_height as f32 * 0.62; // apoyado sobre el piso, no al centro de la vista

        let x0 = (screen_x - size / 2.0).max(0.0) as i32;
        let x1 = (screen_x + size / 2.0).min(window_width as f32) as i32;
        let y0 = (center_y - size / 2.0).max(0.0) as i32;
        let y1 = (center_y + size / 2.0).min(window_height as f32) as i32;

        for x in x0..x1 {
            if x < 0 || x >= window_width {
                continue;
            }
            let ray_index = (x as usize * zbuffer.len()) / window_width as usize;
            if corrected_dist >= zbuffer[ray_index] {
                continue;
            }
            let tx = (x - x0) as f32 / size.max(1.0);
            for y in y0..y1 {
                let ty = (y - y0) as f32 / size.max(1.0);
                // silueta de diamante: solo pinta cerca del centro del cuadro
                if (tx - 0.5).abs() + (ty - 0.5).abs() > 0.5 {
                    continue;
                }
                fb.set_current_color(glow);
                fb.point(x, y);
            }
        }
    }
}
