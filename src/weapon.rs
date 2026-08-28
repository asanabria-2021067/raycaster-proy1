use std::f32::consts::PI;

use raylib::prelude::*;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::pixelart::{beveled_rect, blank_canvas, fill_circle, fill_rect, rect_outline, rivet};
use crate::sprites::Enemy;

const HIT_RADIUS: f32 = 20.0; // radio de colision angular del enemigo, en px de mundo
const GUN_SCALE: f32 = 0.45; // porcion del ancho de ventana que ocupa el arma
const ALPHA_THRESHOLD: u8 = 20;
const PICKUP_RADIUS: f32 = 24.0; // px, distancia jugador-caja para recogerla
const PICKUP_SIZE: f32 = 26.0; // px de mundo, tamaño del brillo del pickup en pantalla
const AMMO_PICKUP_AMOUNT: i32 = 6; // municion que da cada caja recogida
const HEALTH_PICKUP_AMOUNT: i32 = 35; // vida que da cada botiquin recogido

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Pistol,
    Rifle,
    Shotgun,
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
            WeaponKind::Shotgun => WeaponStats { fire_duration: 0.5, max_ammo: 6, reload_duration: 1.8 },
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WeaponKind::Pistol => "PISTOLA",
            WeaponKind::Rifle => "RIFLE",
            WeaponKind::Shotgun => "ESCOPETA",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    Ammo,
    Health,
}

// caja de municion o botiquin tirado en el mapa; se recoge al pasar por encima
pub struct Pickup {
    pub x: f32,
    pub y: f32,
    pub kind: PickupKind,
}

pub struct GunSprite {
    width: i32,
    height: i32,
    frames: Vec<Vec<Color>>,
}

// destello de disparo: tres circulos superpuestos, simula una explosion irregular
fn muzzle_flash(buf: &mut [Color], w: i32, h: i32, tip_x: i32, tip_y: i32, size: i32) {
    let flash = Color::new(255, 225, 110, 255);
    let core = Color::new(255, 250, 220, 255);
    fill_circle(buf, w, h, tip_x, tip_y, size, flash);
    fill_circle(buf, w, h, tip_x + size / 2, tip_y - size / 2, size / 2, flash);
    fill_circle(buf, w, h, tip_x + size / 2, tip_y + size / 2, size / 2, flash);
    fill_circle(buf, w, h, tip_x, tip_y, (size / 2).max(2), core);
}

// stage: 0 = reposo, 1 = disparo (destello grande), 2 = disparo (destello chico, retrocediendo)
fn build_pistol(stage: usize) -> (i32, i32, Vec<Color>) {
    let (w, h) = (100, 80);
    let mut buf = blank_canvas(w, h);
    let metal = Color::new(72, 72, 80, 255);
    let dark = Color::new(42, 42, 48, 255);
    let highlight = Color::new(205, 205, 210, 255);
    let kick = if stage == 0 { 0 } else { 6 - stage as i32 * 2 };

    beveled_rect(&mut buf, w, h, 38, 38 - kick, 26, 40, dark); // empuñadura
    beveled_rect(&mut buf, w, h, 22, 14 - kick, 58, 20, metal); // corredera
    fill_rect(&mut buf, w, h, 70, 6 - kick, 8, 10, highlight); // mira
    rect_outline(&mut buf, w, h, 44, 50 - kick, 16, 12, 2, dark); // guardamonte
    rivet(&mut buf, w, h, 34, 24 - kick);
    rivet(&mut buf, w, h, 62, 24 - kick);

    if stage > 0 {
        muzzle_flash(&mut buf, w, h, 82, 20 - kick, if stage == 1 { 14 } else { 8 });
    }
    (w, h, buf)
}

fn build_rifle(stage: usize) -> (i32, i32, Vec<Color>) {
    let (w, h) = (190, 80);
    let mut buf = blank_canvas(w, h);
    let wood = Color::new(115, 88, 56, 255);
    let olive = Color::new(78, 88, 62, 255);
    let dark = Color::new(38, 38, 42, 255);
    let accent = Color::new(120, 172, 222, 255);
    let kick = if stage == 0 { 0 } else { 5 - stage as i32 * 2 };

    beveled_rect(&mut buf, w, h, 0, 34 - kick, 42, 24, wood); // culata
    beveled_rect(&mut buf, w, h, 36, 22 - kick, 78, 28, olive); // cuerpo/receptor
    beveled_rect(&mut buf, w, h, 60, 48 - kick, 16, 26, dark); // cargador
    rect_outline(&mut buf, w, h, 44, 48 - kick, 14, 22, 2, dark); // empuñadura
    beveled_rect(&mut buf, w, h, 108, 30 - kick, 74, 10, dark); // cañon
    fill_rect(&mut buf, w, h, 76, 10 - kick, 10, 12, accent); // mira/riel
    rivet(&mut buf, w, h, 46, 34 - kick);
    rivet(&mut buf, w, h, 100, 34 - kick);

    if stage > 0 {
        muzzle_flash(&mut buf, w, h, 178, 35 - kick, if stage == 1 { 13 } else { 7 });
    }
    (w, h, buf)
}

fn build_shotgun(stage: usize) -> (i32, i32, Vec<Color>) {
    let (w, h) = (170, 90);
    let mut buf = blank_canvas(w, h);
    let wood = Color::new(122, 88, 52, 255);
    let dark_wood = Color::new(96, 68, 40, 255);
    let metal = Color::new(78, 78, 86, 255);
    let accent = Color::new(228, 122, 42, 255);
    let kick = if stage == 0 { 0 } else { 7 - stage as i32 * 3 };

    beveled_rect(&mut buf, w, h, 0, 42 - kick, 48, 28, wood); // culata
    beveled_rect(&mut buf, w, h, 42, 30 - kick, 42, 36, metal); // receptor
    beveled_rect(&mut buf, w, h, 78, 46 - kick, 38, 18, dark_wood); // guardamano
    beveled_rect(&mut buf, w, h, 80, 26 - kick, 82, 9, metal); // cañon superior
    beveled_rect(&mut buf, w, h, 80, 38 - kick, 82, 9, metal); // cañon inferior
    fill_rect(&mut buf, w, h, 44, 30 - kick, 34, 6, accent); // banda de color
    rivet(&mut buf, w, h, 52, 40 - kick);
    rivet(&mut buf, w, h, 68, 40 - kick);

    if stage > 0 {
        muzzle_flash(&mut buf, w, h, 158, 34 - kick, if stage == 1 { 18 } else { 10 });
    }
    (w, h, buf)
}

impl GunSprite {
    pub fn procedural(kind: WeaponKind) -> Self {
        let build: fn(usize) -> (i32, i32, Vec<Color>) = match kind {
            WeaponKind::Pistol => build_pistol,
            WeaponKind::Rifle => build_rifle,
            WeaponKind::Shotgun => build_shotgun,
        };
        let (width, height, _) = build(0);
        let frames = (0..3).map(|stage| build(stage).2).collect();
        Self { width, height, frames }
    }

    fn sample(&self, frame: usize, tx: f32, ty: f32) -> Color {
        let x = (tx.clamp(0.0, 0.9999) * self.width as f32) as usize;
        let y = (ty.clamp(0.0, 0.9999) * self.height as f32) as usize;
        self.frames[frame][y * self.width as usize + x]
    }

    // ancho/alto del lienzo; usarlo evita estirar el arma si no es cuadrada
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
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

    pub fn add_ammo(&mut self, amount: i32) {
        self.ammo = (self.ammo + amount).min(self.kind.stats().max_ammo);
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
            fb.set_current_color(color);
            fb.point(x0 + x, y0 + y);
        }
    }
}

// recoge la caja mas cercana si el jugador esta encima; devuelve su tipo y la cantidad otorgada
pub fn try_collect_pickup(pickups: &mut Vec<Pickup>, player_x: f32, player_y: f32) -> Option<(PickupKind, i32)> {
    let idx = pickups.iter().position(|p| {
        let dx = p.x - player_x;
        let dy = p.y - player_y;
        (dx * dx + dy * dy).sqrt() < PICKUP_RADIUS
    })?;
    let kind = pickups[idx].kind;
    pickups.remove(idx);
    let amount = match kind {
        PickupKind::Ammo => AMMO_PICKUP_AMOUNT,
        PickupKind::Health => HEALTH_PICKUP_AMOUNT,
    };
    Some((kind, amount))
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
    let ammo_glow = Color::new(255, 215, 90, 255);
    let health_glow = Color::new(90, 235, 120, 255);

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
                let (visible, glow) = match pickup.kind {
                    // silueta de diamante: solo pinta cerca del centro del cuadro
                    PickupKind::Ammo => ((tx - 0.5).abs() + (ty - 0.5).abs() <= 0.5, ammo_glow),
                    // cruz de botiquin: banda horizontal o vertical centrada
                    PickupKind::Health => {
                        ((tx - 0.5).abs() < 0.16 || (ty - 0.5).abs() < 0.16, health_glow)
                    }
                };
                if !visible {
                    continue;
                }
                fb.set_current_color(glow);
                fb.point(x, y);
            }
        }
    }
}
