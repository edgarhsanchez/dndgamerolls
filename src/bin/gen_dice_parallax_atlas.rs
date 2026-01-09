use image::{ImageBuffer, Luma, Rgba, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};

const ATLAS_COLS: u32 = 5;
const ATLAS_ROWS: u32 = 5;
const CELL_SIZE: u32 = 128;

fn main() {
    let out_dir = PathBuf::from("assets/textures/dice_numbers");
    fs::create_dir_all(&out_dir).expect("create output dir");

    let (color, depth, normal) = build_atlas();

    write_png(&out_dir.join("atlas_color.png"), &color);
    write_png(&out_dir.join("atlas_depth.png"), &depth);
    write_png(&out_dir.join("atlas_normal.png"), &normal);

    println!("Wrote atlas textures to {}", out_dir.display());
}

fn write_png(path: &Path, image: &RgbaImage) {
    image
        .save(path)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
}

fn build_atlas() -> (RgbaImage, RgbaImage, RgbaImage) {
    let width = ATLAS_COLS * CELL_SIZE;
    let height = ATLAS_ROWS * CELL_SIZE;

    let mut color = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));
    // Depth map for Bevy parallax:
    //   White (255) = surface level (no displacement)
    //   Black (0)   = maximum depth (carved into surface)
    // We want numbers to appear ENGRAVED, so:
    //   Background = white (surface)
    //   Number fill = darker gray (carved in)
    let mut depth = ImageBuffer::from_pixel(width, height, Luma([255u8]));

    for value in 0u32..=20u32 {
        let (cell_x, cell_y) = cell_origin(value);
        draw_number_cell(&mut color, &mut depth, cell_x, cell_y, value);
    }

    let normal = depth_to_normal_map(&depth);

    let depth_rgba = depth_luma_to_rgba(&depth);
    (color, depth_rgba, normal)
}

fn cell_origin(value: u32) -> (u32, u32) {
    let idx = value.min(ATLAS_COLS * ATLAS_ROWS - 1);
    let col = idx % ATLAS_COLS;
    let row = idx / ATLAS_COLS;
    (col * CELL_SIZE, row * CELL_SIZE)
}

fn draw_number_cell(color: &mut RgbaImage, depth: &mut ImageBuffer<Luma<u8>, Vec<u8>>, x0: u32, y0: u32, value: u32) {
    let text = value.to_string();

    // Layout inside the cell
    let padding = (CELL_SIZE as f32 * 0.18) as i32;
    let inner_w = CELL_SIZE as i32 - padding * 2;
    let inner_h = CELL_SIZE as i32 - padding * 2;

    // Scale digits based on count (1 or 2 chars).
    let digit_count = text.len() as i32;
    let gap = (CELL_SIZE as f32 * 0.06) as i32;
    let total_gap = gap * (digit_count - 1).max(0);
    let digit_w = ((inner_w - total_gap) / digit_count).max(1);
    let digit_h = inner_h;

    let start_x = x0 as i32 + padding;
    let start_y = y0 as i32 + padding;

    // Draw numbers with engraved depth effect:
    // - Outline (black): slightly raised edge (depth ~200)
    // - Fill (white): carved into surface (depth ~80, darker = deeper)
    for (pass, thickness, rgba, depth_value) in [
        (0, (CELL_SIZE as f32 * 0.10) as i32, Rgba([0, 0, 0, 255]), 180u8),
        (1, (CELL_SIZE as f32 * 0.07) as i32, Rgba([255, 255, 255, 255]), 100u8),
    ] {
        let _ = pass;
        for (i, ch) in text.chars().enumerate() {
            let dx = i as i32 * (digit_w + gap);
            draw_seven_segment_digit(
                color,
                depth,
                start_x + dx,
                start_y,
                digit_w,
                digit_h,
                ch,
                thickness,
                rgba,
                depth_value,
            );
        }
    }
}

fn draw_seven_segment_digit(
    color: &mut RgbaImage,
    depth: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    ch: char,
    thickness: i32,
    rgba: Rgba<u8>,
    depth_value: u8,
) {
    let segments = segments_for(ch);
    if segments.is_none() {
        return;
    }
    let segments = segments.unwrap();

    let t = thickness.max(1);
    let inset = (t as f32 * 0.6) as i32;

    // Horizontal segments: A (top), G (middle), D (bottom)
    // Vertical segments: F (top-left), B (top-right), E (bottom-left), C (bottom-right)

    // A
    if segments[0] {
        fill_rect(color, depth, x + inset, y + inset, x + w - inset, y + inset + t, rgba, depth_value);
    }
    // B
    if segments[1] {
        fill_rect(
            color,
            depth,
            x + w - inset - t,
            y + inset,
            x + w - inset,
            y + h / 2 - inset,
            rgba,
            depth_value,
        );
    }
    // C
    if segments[2] {
        fill_rect(
            color,
            depth,
            x + w - inset - t,
            y + h / 2 + inset,
            x + w - inset,
            y + h - inset,
            rgba,
            depth_value,
        );
    }
    // D
    if segments[3] {
        fill_rect(
            color,
            depth,
            x + inset,
            y + h - inset - t,
            x + w - inset,
            y + h - inset,
            rgba,
            depth_value,
        );
    }
    // E
    if segments[4] {
        fill_rect(
            color,
            depth,
            x + inset,
            y + h / 2 + inset,
            x + inset + t,
            y + h - inset,
            rgba,
            depth_value,
        );
    }
    // F
    if segments[5] {
        fill_rect(
            color,
            depth,
            x + inset,
            y + inset,
            x + inset + t,
            y + h / 2 - inset,
            rgba,
            depth_value,
        );
    }
    // G
    if segments[6] {
        fill_rect(
            color,
            depth,
            x + inset,
            y + h / 2 - t / 2,
            x + w - inset,
            y + h / 2 + t / 2,
            rgba,
            depth_value,
        );
    }
}

fn segments_for(ch: char) -> Option<[bool; 7]> {
    // [A, B, C, D, E, F, G]
    Some(match ch {
        '0' => [true, true, true, true, true, true, false],
        '1' => [false, true, true, false, false, false, false],
        '2' => [true, true, false, true, true, false, true],
        '3' => [true, true, true, true, false, false, true],
        '4' => [false, true, true, false, false, true, true],
        '5' => [true, false, true, true, false, true, true],
        '6' => [true, false, true, true, true, true, true],
        '7' => [true, true, true, false, false, false, false],
        '8' => [true, true, true, true, true, true, true],
        '9' => [true, true, true, true, false, true, true],
        _ => return None,
    })
}

fn fill_rect(
    color: &mut RgbaImage,
    depth: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    rgba: Rgba<u8>,
    depth_value: u8,
) {
    let x0 = x0.max(0) as u32;
    let y0 = y0.max(0) as u32;
    let x1 = x1.max(0) as u32;
    let y1 = y1.max(0) as u32;

    let w = color.width();
    let h = color.height();

    let x1 = x1.min(w);
    let y1 = y1.min(h);

    for yy in y0..y1 {
        for xx in x0..x1 {
            // Alpha overwrite is fine: later passes are inside previous pass.
            color.put_pixel(xx, yy, rgba);
            // Only write depth where the digit is opaque.
            if rgba[3] > 0 {
                depth.put_pixel(xx, yy, Luma([depth_value]));
            }
        }
    }
}

fn depth_luma_to_rgba(depth: &ImageBuffer<Luma<u8>, Vec<u8>>) -> RgbaImage {
    let mut out = RgbaImage::new(depth.width(), depth.height());
    for (x, y, p) in depth.enumerate_pixels() {
        out.put_pixel(x, y, Rgba([p[0], p[0], p[0], 255]));
    }
    out
}

fn depth_to_normal_map(depth: &ImageBuffer<Luma<u8>, Vec<u8>>) -> RgbaImage {
    let w = depth.width() as i32;
    let h = depth.height() as i32;
    let mut out = RgbaImage::new(depth.width(), depth.height());

    let strength = 4.0; // tweakable

    for y in 0..h {
        for x in 0..w {
            let l = sample_depth(depth, x - 1, y) as f32;
            let r = sample_depth(depth, x + 1, y) as f32;
            let u = sample_depth(depth, x, y - 1) as f32;
            let d = sample_depth(depth, x, y + 1) as f32;

            // Depth is 0..255 where 0 is "high" and 255 is "low".
            // Convert to height where higher = larger.
            let hl = 1.0 - (l / 255.0);
            let hr = 1.0 - (r / 255.0);
            let hu = 1.0 - (u / 255.0);
            let hd = 1.0 - (d / 255.0);

            let dx = (hr - hl) * strength;
            let dy = (hd - hu) * strength;

            let nx = -dx;
            let ny = -dy;
            let nz = 1.0;
            let inv_len = 1.0 / (nx * nx + ny * ny + nz * nz).sqrt();

            let nx = nx * inv_len;
            let ny = ny * inv_len;
            let nz = nz * inv_len;

            let to_u8 = |v: f32| ((v * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;

            out.put_pixel(x as u32, y as u32, Rgba([to_u8(nx), to_u8(ny), to_u8(nz), 255]));
        }
    }

    out
}

fn sample_depth(depth: &ImageBuffer<Luma<u8>, Vec<u8>>, x: i32, y: i32) -> u8 {
    let x = x.clamp(0, depth.width() as i32 - 1) as u32;
    let y = y.clamp(0, depth.height() as i32 - 1) as u32;
    depth.get_pixel(x, y)[0]
}
