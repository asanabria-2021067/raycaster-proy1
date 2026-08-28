use std::f32::consts::PI;

use raylib::prelude::*;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::{is_wall, Maze};
use crate::pixelart::{beveled_rect, blank_canvas, fill_circle, fill_rect, rivet};

const SPRITE_FRAME_COUNT: usize = 4;
const ALPHA_THRESHOLD: u8 = 20;
const FOV_MARGIN: f32 = 0.3; // margen para no recortar sprites a medio entrar en pantalla
const ANIM_FRAME_SECONDS: f32 = 0.15;

const CHASE_RANGE: f32 = 500.0; // px, radio de deteccion del perseguidor
const CHASE_SPEED: f32 = 90.0; // px/seg
const STOP_DISTANCE: f32 = 24.0; // no se acerca mas al jugador que esto
// fraccion de block_size usada como hitbox; debe cubrir el ancho visual del sprite
// (aspect ~0.67 => medio ancho ~0.33*block_size) o el enemigo se ve traslapando la pared
const ENEMY_RADIUS_FACTOR: f32 = 0.35;

const SHOOTER_RANGE: f32 = 500.0; // px, radio de deteccion del tirador
const SHOOTER_PREFERRED_DIST: f32 = 260.0; // se acerca hasta esta distancia, no mas
const SHOOTER_FIRE_COOLDOWN: f32 = 1.4; // seg entre disparos
const SHOOTER_DAMAGE: f32 = 10.0;

const ALERT_DURATION: f32 = 6.0; // seg que persiguen tras oir un disparo, aunque el jugador salga de rango

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

// silueta humanoide dibujada en codigo, igual que las armas en weapon.rs; el offset de piernas
// por frame produce el ciclo de caminata sin depender de PNGs externos
fn build_humanoid(frame: usize, helmet: Color, jacket: Color, pants: Color, skin: Color, accent: Color) -> (i32, i32, Vec<Color>) {
    let (w, h) = (40, 64);
    let mut buf = blank_canvas(w, h);
    let boot = Color::new(28, 26, 24, 255);
    let stride = match frame % 4 {
        0 => 0,
        1 => 4,
        2 => 0,
        _ => -4,
    };

    // piernas: se turnan para adelante/atras segun el frame
    beveled_rect(&mut buf, w, h, 11 - stride / 2, 46, 8, 16, pants);
    beveled_rect(&mut buf, w, h, 21 + stride / 2, 46, 8, 16, pants);
    fill_rect(&mut buf, w, h, 11 - stride / 2, 58, 8, 6, boot);
    fill_rect(&mut buf, w, h, 21 + stride / 2, 58, 8, 6, boot);

    beveled_rect(&mut buf, w, h, 9, 22, 22, 26, jacket); // torso
    beveled_rect(&mut buf, w, h, 4, 24, 6, 18, jacket); // brazo izq
    beveled_rect(&mut buf, w, h, 30, 24, 6, 18, jacket); // brazo der
    fill_rect(&mut buf, w, h, 4, 40, 6, 4, skin); // mano izq
    fill_rect(&mut buf, w, h, 30, 40, 6, 4, skin); // mano der

    fill_circle(&mut buf, w, h, 20, 14, 9, skin); // cabeza
    beveled_rect(&mut buf, w, h, 10, 4, 20, 10, helmet); // casco
    fill_rect(&mut buf, w, h, 10, 12, 20, 3, accent); // visera/banda
    rivet(&mut buf, w, h, 14, 30);
    rivet(&mut buf, w, h, 26, 30);

    (w, h, buf)
}

fn build_chaser(frame: usize) -> (i32, i32, Vec<Color>) {
    build_humanoid(
        frame,
        Color::new(46, 58, 36, 255),  // casco verde oliva oscuro
        Color::new(78, 88, 62, 255),  // chaqueta verde militar
        Color::new(52, 46, 34, 255),  // pantalon marron oscuro
        Color::new(206, 172, 138, 255),
        Color::new(150, 40, 30, 255), // banda roja, distingue al perseguidor de lejos
    )
}

fn build_shooter(frame: usize) -> (i32, i32, Vec<Color>) {
    let (w, h, mut buf) = build_humanoid(
        frame,
        Color::new(35, 55, 95, 255),   // casco azul acero
        Color::new(60, 95, 150, 255),  // armadura azul
        Color::new(30, 34, 40, 255),   // pantalon gris oscuro
        Color::new(206, 172, 138, 255),
        Color::new(150, 200, 240, 255), // visera celeste
    );
    // rifle cruzado al pecho, silueta distinta a la del perseguidor a mano limpia
    fill_rect(&mut buf, w, h, 2, 30, 30, 4, Color::new(40, 40, 44, 255));
    fill_rect(&mut buf, w, h, 26, 26, 5, 12, Color::new(60, 60, 66, 255));
    (w, h, buf)
}

pub struct SpriteManager {
    width: i32,
    height: i32,
    chaser_frames: Vec<Vec<Color>>,
    shooter_frames: Vec<Vec<Color>>,
}

impl SpriteManager {
    pub fn new() -> Self {
        let (width, height, _) = build_chaser(0);
        let chaser_frames = (0..SPRITE_FRAME_COUNT).map(|f| build_chaser(f).2).collect();
        let shooter_frames = (0..SPRITE_FRAME_COUNT).map(|f| build_shooter(f).2).collect();
        Self { width, height, chaser_frames, shooter_frames }
    }

    pub fn frame_count(&self) -> usize {
        SPRITE_FRAME_COUNT
    }

    // ancho/alto del lienzo; usarlo evita estirar sprites que no son cuadrados
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    fn frames_for(&self, kind: EnemyKind) -> &[Vec<Color>] {
        match kind {
            EnemyKind::Chaser => &self.chaser_frames,
            EnemyKind::Shooter => &self.shooter_frames,
        }
    }

    fn sample(&self, kind: EnemyKind, frame: usize, tx: f32, ty: f32) -> Color {
        let x = (tx.clamp(0.0, 0.9999) * self.width as f32) as usize;
        let y = (ty.clamp(0.0, 0.9999) * self.height as f32) as usize;
        self.frames_for(kind)[frame][y * self.width as usize + x]
    }
}

impl Default for SpriteManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Chaser,
    Shooter,
}

pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub kind: EnemyKind,
    fire_cooldown: f32,
    alert_timer: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32, kind: EnemyKind) -> Self {
        Self { x, y, kind, fire_cooldown: 0.0, alert_timer: 0.0 }
    }

    // el enemigo oyo un disparo: persigue al jugador aunque este fuera de su rango normal
    pub fn alert(&mut self) {
        self.alert_timer = ALERT_DURATION;
    }

    // avanza siempre hacia el jugador si esta en rango, deslizandose contra las paredes
    // que encuentre en el camino (no requiere linea de vision directa)
    fn approach(&mut self, maze: &Maze, block_size: i32, dx: f32, dy: f32, dist: f32, dt: f32) {
        let step = CHASE_SPEED * dt / dist.max(1.0);
        let new_x = self.x + dx * step;
        let new_y = self.y + dy * step;
        if !enemy_collides(maze, block_size, new_x, self.y) {
            self.x = new_x;
        }
        if !enemy_collides(maze, block_size, self.x, new_y) {
            self.y = new_y;
        }
    }

    // actualiza movimiento/disparo; devuelve Some(daño) si le acierta un tiro al jugador este frame
    pub fn update_ai(&mut self, maze: &Maze, block_size: i32, player_x: f32, player_y: f32, dt: f32) -> Option<f32> {
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        self.alert_timer = (self.alert_timer - dt).max(0.0);
        let alerted = self.alert_timer > 0.0;

        match self.kind {
            EnemyKind::Chaser => {
                if dist > STOP_DISTANCE && (dist <= CHASE_RANGE || alerted) {
                    self.approach(maze, block_size, dx, dy, dist, dt);
                }
                None
            }
            EnemyKind::Shooter => {
                if dist > SHOOTER_RANGE && !alerted {
                    return None;
                }
                if dist > SHOOTER_PREFERRED_DIST {
                    self.approach(maze, block_size, dx, dy, dist, dt);
                }
                let angle = dy.atan2(dx);
                let has_los = cast_ray(maze, self.x, self.y, angle, block_size).distance >= dist;
                if has_los && dist > STOP_DISTANCE && self.fire_cooldown <= 0.0 {
                    self.fire_cooldown = SHOOTER_FIRE_COOLDOWN;
                    return Some(SHOOTER_DAMAGE);
                }
                None
            }
        }
    }
}

fn enemy_collides(maze: &Maze, block_size: i32, x: f32, y: f32) -> bool {
    let bs = block_size as f32;
    let r = bs * ENEMY_RADIUS_FACTOR;
    let corners = [(x - r, y - r), (x + r, y - r), (x - r, y + r), (x + r, y + r)];
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
        let sprite_height = (block_size as f32 / corrected_dist) * dist_to_projection_plane;
        let sprite_width = sprite_height * sprites.aspect();
        let screen_x = window_width as f32 / 2.0 + rel_angle.tan() * dist_to_projection_plane;

        let x0 = (screen_x - sprite_width / 2.0).max(0.0) as i32;
        let x1 = (screen_x + sprite_width / 2.0).min(window_width as f32) as i32;
        let center_y = window_height as f32 / 2.0;
        let y0 = (center_y - sprite_height / 2.0).max(0.0) as i32;
        let y1 = (center_y + sprite_height / 2.0).min(window_height as f32) as i32;

        for x in x0..x1 {
            if x < 0 || x >= window_width || zbuffer.is_empty() {
                continue;
            }
            // zbuffer trae un valor por columna de rayo, no por pixel; hay que mapear
            let ray_index = (x as usize * zbuffer.len()) / window_width as usize;
            if corrected_dist >= zbuffer[ray_index] {
                continue;
            }
            let tx = (x - x0) as f32 / sprite_width.max(1.0);
            for y in y0..y1 {
                let ty = (y as f32 - (center_y - sprite_height / 2.0)) / sprite_height.max(1.0);
                let color = sprites.sample(enemy.kind, frame, tx, ty);
                if color.a < ALPHA_THRESHOLD {
                    continue;
                }
                fb.set_current_color(color);
                fb.point(x, y);
            }
        }
    }
}
