use raylib::prelude::*;

const AVAILABLE_LEVELS: [&str; 3] = ["levels/level1.txt", "levels/level2.txt", "levels/level3.txt"];
const LEVEL_TEXTURE_DIRS: [&str; 3] =
    ["assets/textures/level1", "assets/textures/level2", "assets/textures/level3"];
pub const LEVEL_COUNT: usize = AVAILABLE_LEVELS.len();

const BG_MENU: Color = Color::new(15, 15, 25, 255);
const BG_SUCCESS: Color = Color::new(10, 30, 15, 255);
const BG_GAME_OVER: Color = Color::new(30, 10, 10, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Welcome,
    LevelSelect,
    Playing,
    Success,
    Credits,
    GameOver,
}

pub fn level_path(index: usize) -> String {
    crate::paths::resolve(AVAILABLE_LEVELS[index])
}

pub fn level_exists(index: usize) -> bool {
    std::path::Path::new(&crate::paths::resolve(AVAILABLE_LEVELS[index])).exists()
}

pub fn level_texture_dir(index: usize) -> &'static str {
    LEVEL_TEXTURE_DIRS[index]
}

pub fn draw_welcome(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) {
    d.clear_background(BG_MENU);
    d.draw_text("RAY CASTER", window_width / 2 - 190, window_height / 2 - 110, 60, Color::GOLD);
    d.draw_text(
        "WASD mover  |  shift correr  |  mouse rotar  |  click disparar  |  R recargar  |  M alterna 2D/3D",
        window_width / 2 - 460,
        window_height / 2,
        20,
        Color::WHITE,
    );
    d.draw_text("ENTER para continuar", window_width / 2 - 140, window_height / 2 + 60, 26, Color::SKYBLUE);
}

pub fn draw_level_select(
    d: &mut RaylibDrawHandle,
    window_width: i32,
    window_height: i32,
    selected: usize,
    error: Option<&str>,
) {
    d.clear_background(BG_MENU);
    d.draw_text("SELECCIONA NIVEL", window_width / 2 - 230, window_height / 2 - 160, 44, Color::GOLD);

    for i in 0..LEVEL_COUNT {
        let y = window_height / 2 - 40 + i as i32 * 50;
        let available = level_exists(i);
        let color = if !available {
            Color::DARKGRAY
        } else if i == selected {
            Color::GOLD
        } else {
            Color::WHITE
        };
        let label = if available {
            format!("Nivel {}", i + 1)
        } else {
            format!("Nivel {} (no disponible)", i + 1)
        };
        d.draw_text(&label, window_width / 2 - 140, y, 30, color);
    }

    d.draw_text(
        "flechas arriba/abajo para elegir, ENTER para jugar",
        window_width / 2 - 300,
        window_height - 80,
        20,
        Color::LIGHTGRAY,
    );

    if let Some(msg) = error {
        d.draw_text(&format!("no se pudo cargar el nivel: {msg}"), window_width / 2 - 300, window_height - 50, 20, Color::RED);
    }
}

pub fn draw_success(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) {
    d.clear_background(BG_SUCCESS);
    d.draw_text("META ALCANZADA", window_width / 2 - 270, window_height / 2 - 60, 50, Color::GOLD);
    d.draw_text(
        "ENTER para volver a seleccion de nivel",
        window_width / 2 - 260,
        window_height / 2 + 20,
        22,
        Color::WHITE,
    );
}

pub fn draw_credits(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) {
    d.clear_background(BG_SUCCESS);
    d.draw_text("JUEGO TERMINADO", window_width / 2 - 300, window_height / 2 - 180, 50, Color::GOLD);
    let lines = [
        "Gracias por jugar RAY CASTER",
        "",
        "Motor de raycasting escrito en Rust con raylib",
        "3 niveles, enemigos que persiguen y disparan,",
        "armas intercambiables y HUD arcade",
        "",
        "Un proyecto de Angel Sanabria",
    ];
    for (i, line) in lines.iter().enumerate() {
        d.draw_text(line, window_width / 2 - 260, window_height / 2 - 90 + i as i32 * 30, 20, Color::WHITE);
    }
    d.draw_text(
        "ENTER para volver al inicio",
        window_width / 2 - 190,
        window_height / 2 + 160,
        22,
        Color::SKYBLUE,
    );
}

const HUD_PANEL_HEIGHT: i32 = 74;
const HUD_LIVES: i32 = 10; // segmentos de la barra de vida, estilo pips de arcade
const HUD_BORDER: Color = Color::new(255, 195, 40, 255);

#[allow(clippy::too_many_arguments)]
pub fn draw_hud(
    d: &mut RaylibDrawHandle,
    window_width: i32,
    window_height: i32,
    health: f32,
    ammo: i32,
    max_ammo: i32,
    enemies_left: usize,
    reloading: bool,
    weapon_label: &str,
    multiple_weapons: bool,
) {
    let panel_y = window_height - HUD_PANEL_HEIGHT;
    d.draw_rectangle(0, panel_y, window_width, HUD_PANEL_HEIGHT, Color::new(8, 8, 12, 215));
    d.draw_line(0, panel_y, window_width, panel_y, HUD_BORDER);

    let health_color =
        if health > 50.0 { Color::LIME } else if health > 20.0 { Color::GOLD } else { Color::RED };

    // panel de vida: pips estilo arcade, uno se apaga cada 100/HUD_LIVES % de vida perdida
    let label_x = 18;
    let label_y = panel_y + 10;
    d.draw_text("VIDA", label_x, label_y, 20, HUD_BORDER);
    let bar_x = label_x + 62;
    let bar_y = label_y + 1;
    let seg_w = 16;
    let seg_h = 20;
    let seg_gap = 3;
    let filled = ((health.clamp(0.0, 100.0) / 100.0) * HUD_LIVES as f32).ceil() as i32;
    for i in 0..HUD_LIVES {
        let sx = bar_x + i * (seg_w + seg_gap);
        let fill_color = if i < filled { health_color } else { Color::new(35, 35, 35, 255) };
        d.draw_rectangle(sx, bar_y, seg_w, seg_h, fill_color);
        d.draw_rectangle_lines(sx, bar_y, seg_w, seg_h, Color::BLACK);
    }
    d.draw_text(&format!("{}", health.max(0.0) as i32), bar_x + HUD_LIVES * (seg_w + seg_gap) + 8, label_y, 20, Color::WHITE);

    // panel de municion: una bala por tiro disponible, estilo cargador visible
    let ammo_x = window_width / 2 - 110;
    d.draw_text("MUN", ammo_x, label_y, 20, Color::SKYBLUE);
    let bullets_x = ammo_x + 55;
    for i in 0..max_ammo {
        let bx = bullets_x + i * 11;
        let lit = !reloading && i < ammo;
        let color = if lit { Color::YELLOW } else { Color::new(45, 45, 45, 255) };
        d.draw_rectangle(bx, bar_y + 2, 7, seg_h - 4, color);
    }
    if reloading {
        d.draw_text("RECARGANDO", ammo_x, bar_y + seg_h + 4, 16, Color::GOLD);
    } else if ammo == 0 {
        d.draw_text("[R] RECARGAR", ammo_x, bar_y + seg_h + 4, 16, Color::RED);
    }

    let weapon_display =
        if multiple_weapons { format!("{weapon_label} [1][2]") } else { weapon_label.to_string() };
    d.draw_text(&weapon_display, bullets_x + max_ammo * 11 + 14, label_y, 16, Color::GOLD);

    // contador de enemigos, alineado a la derecha
    let enemies_text = format!("ENEMIGOS {enemies_left}");
    let text_w = d.measure_text(&enemies_text, 22);
    let enemies_x = window_width - text_w - 34;
    d.draw_circle(enemies_x - 14, label_y + 12, 7.0, Color::RED);
    d.draw_text(&enemies_text, enemies_x, label_y, 22, Color::LIGHTGRAY);
}

pub fn draw_fps(d: &mut RaylibDrawHandle, window_width: i32, fps: f32) {
    d.draw_text(&format!("FPS {fps:.0}"), window_width - 130, 10, 22, Color::LIME);
}

pub fn draw_toast(d: &mut RaylibDrawHandle, window_width: i32, message: &str) {
    let text_w = d.measure_text(message, 24);
    let box_w = text_w + 40;
    let x = window_width / 2 - box_w / 2;
    d.draw_rectangle(x, 60, box_w, 44, Color::new(20, 20, 15, 220));
    d.draw_rectangle_lines(x, 60, box_w, 44, Color::new(255, 215, 90, 255));
    d.draw_text(message, x + 20, 72, 24, Color::new(255, 215, 90, 255));
}

pub fn draw_game_over(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) {
    d.clear_background(BG_GAME_OVER);
    d.draw_text("HAS MUERTO", window_width / 2 - 210, window_height / 2 - 60, 50, Color::RED);
    d.draw_text(
        "ENTER para volver a seleccion de nivel",
        window_width / 2 - 260,
        window_height / 2 + 20,
        22,
        Color::WHITE,
    );
}
