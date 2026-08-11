use raylib::prelude::Color;

use crate::caster::Intersect;
use crate::framebuffer::Framebuffer;
use crate::maze::wall_type_index;
use crate::textures::TextureManager;

const CEILING_COLOR: Color = Color::new(40, 40, 55, 255);
const FLOOR_COLOR: Color = Color::new(60, 50, 40, 255);

fn fill_column(fb: &mut Framebuffer, x0: i32, x1: i32, y0: i32, y1: i32, color: Color) {
    fb.set_current_color(color);
    for y in y0..y1 {
        for x in x0..x1 {
            fb.point(x, y);
        }
    }
}

fn shade_color(base: Color, shade: f32) -> Color {
    Color::new(
        (base.r as f32 * shade) as u8,
        (base.g as f32 * shade) as u8,
        (base.b as f32 * shade) as u8,
        255,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_3d(
    fb: &mut Framebuffer,
    rays: &[Intersect],
    player_angle: f32,
    window_width: i32,
    window_height: i32,
    block_size: i32,
    fov: f32,
    textures: &TextureManager,
) {
    let num_rays = rays.len().max(1);
    let column_width = (window_width as f32 / num_rays as f32).ceil() as i32;
    let dist_to_projection_plane = (window_width as f32 / 2.0) / (fov / 2.0).tan();

    for (i, ray) in rays.iter().enumerate() {
        // correccion de fisheye: se usa la distancia perpendicular al plano de la camara,
        // no la distancia real del rayo
        let corrected_dist = (ray.distance * (ray.angle - player_angle).cos()).max(1.0);
        let stake_height = (block_size as f32 / corrected_dist) * dist_to_projection_plane;

        let center = window_height as f32 / 2.0;
        let top = (center - stake_height / 2.0).max(0.0) as i32;
        let bottom = (center + stake_height / 2.0).min(window_height as f32) as i32;

        let wall_index = wall_type_index(ray.impact).unwrap_or(0);
        let shade = if ray.is_vertical { 1.0 } else { 0.7 };

        let x0 = i as i32 * column_width;
        let x1 = x0 + column_width;

        fill_column(fb, x0, x1, 0, top, CEILING_COLOR);

        let span = (bottom - top).max(1);
        for y in top..bottom {
            let ty = (y - top) as f32 / span as f32;
            let sample = textures.sample(wall_index, ray.tx, ty);
            fb.set_current_color(shade_color(sample, shade));
            for x in x0..x1 {
                fb.point(x, y);
            }
        }

        fill_column(fb, x0, x1, bottom, window_height, FLOOR_COLOR);
    }
}
