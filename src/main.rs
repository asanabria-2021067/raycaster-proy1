mod caster;
mod framebuffer;
mod maze;
mod player;
mod render2d;

use framebuffer::Framebuffer;
use maze::Maze;
use player::Player;
use raylib::prelude::*;

const WINDOW_WIDTH: i32 = 1300;
const WINDOW_HEIGHT: i32 = 900;
const MARKER_SIZE: i32 = 6;

fn draw_marker(fb: &mut Framebuffer, world_x: f32, world_y: f32, origin_x: i32, origin_y: i32) {
    let cx = origin_x + world_x as i32;
    let cy = origin_y + world_y as i32;
    for dy in -MARKER_SIZE / 2..MARKER_SIZE / 2 {
        for dx in -MARKER_SIZE / 2..MARKER_SIZE / 2 {
            fb.point(cx + dx, cy + dy);
        }
    }
}

fn cell_center(cell: (usize, usize), block_size: i32) -> (f32, f32) {
    let bs = block_size as f32;
    (cell.0 as f32 * bs + bs / 2.0, cell.1 as f32 * bs + bs / 2.0)
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

    let mut player = Player::new(spawn, block_size);
    let goal_center = cell_center(goal, block_size);

    while !rl.window_should_close() {
        player.a += rl.get_frame_time();

        let intersect = caster::cast_ray(&level, player.pos_x, player.pos_y, player.a, block_size);
        let hit_x = player.pos_x + intersect.distance * player.a.cos();
        let hit_y = player.pos_y + intersect.distance * player.a.sin();

        framebuffer.clear();

        render2d::render_2d(&mut framebuffer, &level, origin_x, origin_y, block_size);

        render2d::draw_line(
            &mut framebuffer,
            origin_x + player.pos_x as i32,
            origin_y + player.pos_y as i32,
            origin_x + hit_x as i32,
            origin_y + hit_y as i32,
            Color::YELLOW,
        );

        framebuffer.set_current_color(Color::GOLD);
        draw_marker(&mut framebuffer, goal_center.0, goal_center.1, origin_x, origin_y);

        framebuffer.set_current_color(Color::SKYBLUE);
        draw_marker(&mut framebuffer, player.pos_x, player.pos_y, origin_x, origin_y);

        // color del impacto = color del tipo de pared golpeado, oscurecido en cara horizontal
        // (previsualiza el sombreado por cara vertical/horizontal de la fase 7/8)
        let wall_index = maze::wall_type_index(intersect.impact).unwrap_or(0);
        let base = render2d::WALL_COLORS[wall_index];
        let shade = if intersect.is_vertical { 1.0 } else { 0.7 };
        framebuffer.set_current_color(Color::new(
            (base.r as f32 * shade) as u8,
            (base.g as f32 * shade) as u8,
            (base.b as f32 * shade) as u8,
            255,
        ));
        draw_marker(&mut framebuffer, hit_x, hit_y, origin_x, origin_y);

        framebuffer.swap(&mut texture);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
