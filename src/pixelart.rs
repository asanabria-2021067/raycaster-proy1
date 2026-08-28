use raylib::prelude::*;

pub fn blank_canvas(w: i32, h: i32) -> Vec<Color> {
    vec![Color::new(0, 0, 0, 0); (w * h) as usize]
}

pub fn fill_rect(buf: &mut [Color], w: i32, h: i32, x: i32, y: i32, rw: i32, rh: i32, color: Color) {
    for yy in y..(y + rh) {
        if yy < 0 || yy >= h {
            continue;
        }
        for xx in x..(x + rw) {
            if xx < 0 || xx >= w {
                continue;
            }
            buf[(yy * w + xx) as usize] = color;
        }
    }
}

pub fn fill_circle(buf: &mut [Color], w: i32, h: i32, cx: i32, cy: i32, r: i32, color: Color) {
    for yy in (cy - r)..(cy + r) {
        if yy < 0 || yy >= h {
            continue;
        }
        for xx in (cx - r)..(cx + r) {
            if xx < 0 || xx >= w {
                continue;
            }
            let (dx, dy) = (xx - cx, yy - cy);
            if dx * dx + dy * dy <= r * r {
                buf[(yy * w + xx) as usize] = color;
            }
        }
    }
}

pub fn shift(c: Color, amount: i32) -> Color {
    let clamp = |v: u8| (v as i32 + amount).clamp(0, 255) as u8;
    Color::new(clamp(c.r), clamp(c.g), clamp(c.b), c.a)
}

// rectangulo con borde superior claro e inferior oscuro, simula volumen sin arte real
pub fn beveled_rect(buf: &mut [Color], w: i32, h: i32, x: i32, y: i32, rw: i32, rh: i32, color: Color) {
    fill_rect(buf, w, h, x, y, rw, rh, color);
    fill_rect(buf, w, h, x, y, rw, 2.min(rh), shift(color, 35));
    if rh > 2 {
        fill_rect(buf, w, h, x, y + rh - 2, rw, 2, shift(color, -35));
    }
}

// contorno rectangular hueco
#[allow(clippy::too_many_arguments)]
pub fn rect_outline(buf: &mut [Color], w: i32, h: i32, x: i32, y: i32, rw: i32, rh: i32, thick: i32, color: Color) {
    fill_rect(buf, w, h, x, y, rw, thick, color);
    fill_rect(buf, w, h, x, y + rh - thick, rw, thick, color);
    fill_rect(buf, w, h, x, y, thick, rh, color);
    fill_rect(buf, w, h, x + rw - thick, y, thick, rh, color);
}

pub fn rivet(buf: &mut [Color], w: i32, h: i32, x: i32, y: i32) {
    fill_circle(buf, w, h, x, y, 2, Color::new(25, 25, 28, 255));
}
