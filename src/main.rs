mod framebuffer;
mod maze;
mod render2d;

use framebuffer::Framebuffer;
use maze::Maze;
use raylib::prelude::*;

const WINDOW_WIDTH: i32 = 1300;
const WINDOW_HEIGHT: i32 = 900;
const MARKER_SIZE: i32 = 6;

fn draw_marker(fb: &mut Framebuffer, cell: (usize, usize), origin_x: i32, origin_y: i32, block_size: i32) {
    let cx = origin_x + cell.0 as i32 * block_size + block_size / 2;
    let cy = origin_y + cell.1 as i32 * block_size + block_size / 2;
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

    let block_size = (WINDOW_WIDTH / cols).min(WINDOW_HEIGHT / rows);
    let origin_x = (WINDOW_WIDTH - block_size * cols) / 2;
    let origin_y = (WINDOW_HEIGHT - block_size * rows) / 2;

    while !rl.window_should_close() {
        framebuffer.clear();

        render2d::render_2d(&mut framebuffer, &level, origin_x, origin_y, block_size);

        framebuffer.set_current_color(Color::WHITE);
        draw_marker(&mut framebuffer, spawn, origin_x, origin_y, block_size);

        framebuffer.set_current_color(Color::GOLD);
        draw_marker(&mut framebuffer, goal, origin_x, origin_y, block_size);

        framebuffer.swap(&mut texture);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
