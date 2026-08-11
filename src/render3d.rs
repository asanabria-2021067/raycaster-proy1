use raylib::prelude::Color;

use crate::caster::Intersect;
use crate::framebuffer::Framebuffer;
use crate::maze::wall_type_index;
use crate::textures::TextureManager;

const CEILING_SHADE: f32 = 0.6;
const FLOOR_SHADE: f32 = 0.75;

// coordenadas de mundo -> UV de textura tileable (0..1, se repite cada block_size)
fn floor_uv(world_x: f32, world_y: f32, block_size: i32) -> (f32, f32) {
    let bs = block_size as f32;
    ((world_x / bs).rem_euclid(1.0), (world_y / bs).rem_euclid(1.0))
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
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    window_width: i32,
    window_height: i32,
    block_size: i32,
    fov: f32,
    textures: &TextureManager,
) -> Vec<f32> {
    let num_rays = rays.len().max(1);
    let column_width = (window_width as f32 / num_rays as f32).ceil() as i32;
    let dist_to_projection_plane = (window_width as f32 / 2.0) / (fov / 2.0).tan();
    let mut zbuffer = Vec::with_capacity(num_rays);

    let center = window_height as f32 / 2.0;
    let half_block = block_size as f32 / 2.0;

    for (i, ray) in rays.iter().enumerate() {
        // correccion de fisheye: se usa la distancia perpendicular al plano de la camara,
        // no la distancia real del rayo
        let rel_cos = (ray.angle - player_angle).cos();
        let corrected_dist = (ray.distance * rel_cos).max(1.0);
        zbuffer.push(corrected_dist);
        let stake_height = (block_size as f32 / corrected_dist) * dist_to_projection_plane;

        let top = (center - stake_height / 2.0).max(0.0) as i32;
        let bottom = (center + stake_height / 2.0).min(window_height as f32) as i32;

        let wall_index = wall_type_index(ray.impact).unwrap_or(0);
        let shade = if ray.is_vertical { 1.0 } else { 0.7 };

        let x0 = i as i32 * column_width;
        let x1 = x0 + column_width;

        // casteo de piso/techo por fila: la distancia real a lo largo del rayo se
        // recupera dividiendo por el mismo coseno usado para corregir el fisheye
        let inv_cos_rel = 1.0 / rel_cos;
        let cos_a = ray.angle.cos();
        let sin_a = ray.angle.sin();

        for y in 0..top {
            let p = (center - y as f32).max(1.0);
            let row_dist = half_block * dist_to_projection_plane / p;
            let true_dist = row_dist * inv_cos_rel;
            let world_x = player_x + true_dist * cos_a;
            let world_y = player_y + true_dist * sin_a;
            let (tx, ty) = floor_uv(world_x, world_y, block_size);
            let sample = textures.sample_ceiling(tx, ty);
            fb.set_current_color(shade_color(sample, CEILING_SHADE));
            for x in x0..x1 {
                fb.point(x, y);
            }
        }

        let span = (bottom - top).max(1);
        for y in top..bottom {
            let ty = (y - top) as f32 / span as f32;
            let sample = textures.sample_wall(wall_index, ray.tx, ty);
            fb.set_current_color(shade_color(sample, shade));
            for x in x0..x1 {
                fb.point(x, y);
            }
        }

        for y in bottom..window_height {
            let p = (y as f32 - center).max(1.0);
            let row_dist = half_block * dist_to_projection_plane / p;
            let true_dist = row_dist * inv_cos_rel;
            let world_x = player_x + true_dist * cos_a;
            let world_y = player_y + true_dist * sin_a;
            let (tx, ty) = floor_uv(world_x, world_y, block_size);
            let sample = textures.sample_floor(tx, ty);
            fb.set_current_color(shade_color(sample, FLOOR_SHADE));
            for x in x0..x1 {
                fb.point(x, y);
            }
        }
    }

    zbuffer
}
