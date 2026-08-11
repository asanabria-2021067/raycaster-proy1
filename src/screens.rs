use raylib::prelude::*;

const AVAILABLE_LEVELS: [&str; 3] = ["levels/level1.txt", "levels/level2.txt", "levels/level3.txt"];
pub const LEVEL_COUNT: usize = AVAILABLE_LEVELS.len();

const BG_MENU: Color = Color::new(15, 15, 25, 255);
const BG_SUCCESS: Color = Color::new(10, 30, 15, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Welcome,
    LevelSelect,
    Playing,
    Success,
}

pub fn level_path(index: usize) -> &'static str {
    AVAILABLE_LEVELS[index]
}

pub fn level_exists(index: usize) -> bool {
    std::path::Path::new(AVAILABLE_LEVELS[index]).exists()
}

pub fn draw_welcome(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32) {
    d.clear_background(BG_MENU);
    d.draw_text("RAY CASTER", window_width / 2 - 190, window_height / 2 - 110, 60, Color::GOLD);
    d.draw_text(
        "WASD mover  |  mouse rotar  |  click disparar  |  M alterna 2D/3D",
        window_width / 2 - 330,
        window_height / 2,
        20,
        Color::WHITE,
    );
    d.draw_text("ENTER para continuar", window_width / 2 - 140, window_height / 2 + 60, 26, Color::SKYBLUE);
}

pub fn draw_level_select(d: &mut RaylibDrawHandle, window_width: i32, window_height: i32, selected: usize) {
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
