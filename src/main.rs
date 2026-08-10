mod framebuffer;
mod maze;

use framebuffer::Framebuffer;
use maze::Maze;
use raylib::prelude::*;

const WINDOW_WIDTH: i32 = 1300;
const WINDOW_HEIGHT: i32 = 900;
const MARKER_SIZE: i32 = 6;

fn draw_marker(fb: &mut Framebuffer, cell: (usize, usize), cell_w: i32, cell_h: i32) {
    let cx = cell.0 as i32 * cell_w + cell_w / 2;
    let cy = cell.1 as i32 * cell_h + cell_h / 2;
    for dy in -MARKER_SIZE / 2..MARKER_SIZE / 2 {
        for dx in -MARKER_SIZE / 2..MARKER_SIZE / 2 {
            fb.point(cx + dx, cy + dy);
        }
    }
}

fn main() {
    let level: Maze = maze::load_maze("levels/level1.txt");
    let spawn = maze::find_char(&level, 'p').expect("el mapa no tiene spawn 'p'");
    let goal = maze::find_char(&level, 'g').expect("el mapa no tiene meta 'g'");
    assert!(!maze::is_wall(&level, spawn.0, spawn.1), "el spawn cae sobre una pared");
    assert!(!maze::is_wall(&level, goal.0, goal.1), "la meta cae sobre una pared");

    let cols = level.first().map_or(0, |row| row.len()) as i32;
    let rows = level.len() as i32;
    println!("mapa cargado: {rows} filas x {cols} columnas, spawn {spawn:?}, meta {goal:?}");

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Ray Caster")
        .build();

    let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT, Color::BLACK);
    let blank_image = Image::gen_image_color(WINDOW_WIDTH, WINDOW_HEIGHT, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &blank_image)
        .expect("no se pudo crear la textura del framebuffer");

    let cell_w = WINDOW_WIDTH / cols;
    let cell_h = WINDOW_HEIGHT / rows;

    while !rl.window_should_close() {
        framebuffer.clear();

        framebuffer.set_current_color(Color::RED);
        draw_marker(&mut framebuffer, spawn, cell_w, cell_h);

        framebuffer.set_current_color(Color::LIME);
        draw_marker(&mut framebuffer, goal, cell_w, cell_h);

        framebuffer.swap(&mut texture);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
