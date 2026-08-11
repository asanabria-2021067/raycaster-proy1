use raylib::prelude::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::render2d::{draw_line, render_2d};

const MINIMAP_BLOCK: i32 = 8;
const MINIMAP_MARGIN: i32 = 16;
const MARKER_RADIUS: i32 = 3;
const DIR_LEN: f32 = 12.0;

#[allow(clippy::too_many_arguments)]
pub fn draw_minimap(
    fb: &mut Framebuffer,
    maze: &Maze,
    player_world: (f32, f32),
    player_angle: f32,
    goal_world: (f32, f32),
    block_size: i32,
    window_width: i32,
) {
    let cols = maze.first().map_or(0, |row| row.len()) as i32;
    let map_width = cols * MINIMAP_BLOCK;
    let origin_x = window_width - map_width - MINIMAP_MARGIN;
    let origin_y = MINIMAP_MARGIN;

    render_2d(fb, maze, origin_x, origin_y, MINIMAP_BLOCK);

    let scale = MINIMAP_BLOCK as f32 / block_size as f32;
    let (px, py) = (origin_x as f32 + player_world.0 * scale, origin_y as f32 + player_world.1 * scale);
    let (gx, gy) = (origin_x as f32 + goal_world.0 * scale, origin_y as f32 + goal_world.1 * scale);

    fb.set_current_color(Color::GOLD);
    fill_marker(fb, gx as i32, gy as i32);

    draw_line(
        fb,
        px as i32,
        py as i32,
        (px + player_angle.cos() * DIR_LEN) as i32,
        (py + player_angle.sin() * DIR_LEN) as i32,
        Color::SKYBLUE,
    );

    fb.set_current_color(Color::SKYBLUE);
    fill_marker(fb, px as i32, py as i32);
}

fn fill_marker(fb: &mut Framebuffer, cx: i32, cy: i32) {
    for dy in -MARKER_RADIUS..=MARKER_RADIUS {
        for dx in -MARKER_RADIUS..=MARKER_RADIUS {
            fb.point(cx + dx, cy + dy);
        }
    }
}
