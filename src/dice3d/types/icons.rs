//! Icon resources and loading
//!
//! This module provides embedded icons for the application UI.
//! Icons are stored as base64-encoded PNG data and loaded at runtime.

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

/// Resource containing all loaded icon handles
#[derive(Resource, Default)]
pub struct IconAssets {
    pub icons: HashMap<IconType, Handle<Image>>,
}

/// Available icon types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconType {
    Dice,
    Character,
    Info,
    Settings,
    Add,
    Delete,
    Edit,
    Save,
    Cancel,
    Check,
    Roll,
    Expand,
    Collapse,
}

impl IconType {
    /// Get all icon types for loading
    pub fn all() -> &'static [IconType] {
        &[
            IconType::Dice,
            IconType::Character,
            IconType::Info,
            IconType::Settings,
            IconType::Add,
            IconType::Delete,
            IconType::Edit,
            IconType::Save,
            IconType::Cancel,
            IconType::Check,
            IconType::Roll,
            IconType::Expand,
            IconType::Collapse,
        ]
    }
}

/// Generate a simple icon as RGBA pixel data
/// Icons are 32x32 pixels
fn generate_icon_pixels(icon_type: IconType) -> Vec<u8> {
    let size = 32;
    let mut pixels = vec![0u8; size * size * 4]; // RGBA

    match icon_type {
        IconType::Dice => draw_dice_icon(&mut pixels, size),
        IconType::Character => draw_character_icon(&mut pixels, size),
        IconType::Info => draw_info_icon(&mut pixels, size),
        IconType::Settings => draw_settings_icon(&mut pixels, size),
        IconType::Add => draw_add_icon(&mut pixels, size),
        IconType::Delete => draw_delete_icon(&mut pixels, size),
        IconType::Edit => draw_edit_icon(&mut pixels, size),
        IconType::Save => draw_save_icon(&mut pixels, size),
        IconType::Cancel => draw_cancel_icon(&mut pixels, size),
        IconType::Check => draw_check_icon(&mut pixels, size),
        IconType::Roll => draw_roll_icon(&mut pixels, size),
        IconType::Expand => draw_expand_icon(&mut pixels, size),
        IconType::Collapse => draw_collapse_icon(&mut pixels, size),
    }

    pixels
}

// Helper to set a pixel (with bounds checking)
#[allow(clippy::too_many_arguments)]
fn set_pixel(pixels: &mut [u8], size: usize, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if x >= 0 && x < size as i32 && y >= 0 && y < size as i32 {
        let idx = ((y as usize) * size + (x as usize)) * 4;
        pixels[idx] = r;
        pixels[idx + 1] = g;
        pixels[idx + 2] = b;
        pixels[idx + 3] = a;
    }
}

// Draw a filled circle
#[allow(clippy::too_many_arguments)]
fn draw_circle(
    pixels: &mut [u8],
    size: usize,
    cx: i32,
    cy: i32,
    radius: i32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                set_pixel(pixels, size, cx + dx, cy + dy, r, g, b, a);
            }
        }
    }
}

// Draw a circle outline
#[allow(clippy::too_many_arguments)]
fn draw_circle_outline(
    pixels: &mut [u8],
    size: usize,
    cx: i32,
    cy: i32,
    radius: i32,
    thickness: i32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    let inner_r2 = (radius - thickness) * (radius - thickness);
    let outer_r2 = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let d2 = dx * dx + dy * dy;
            if d2 <= outer_r2 && d2 >= inner_r2 {
                set_pixel(pixels, size, cx + dx, cy + dy, r, g, b, a);
            }
        }
    }
}

// Draw a filled rectangle
#[allow(clippy::too_many_arguments)]
fn draw_rect(
    pixels: &mut [u8],
    size: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    for dy in 0..h {
        for dx in 0..w {
            set_pixel(pixels, size, x + dx, y + dy, r, g, b, a);
        }
    }
}

// Draw a line (Bresenham's algorithm)
#[allow(clippy::too_many_arguments)]
fn draw_line(
    pixels: &mut [u8],
    size: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    thickness: i32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        // Draw thick line
        for t in -(thickness / 2)..=(thickness / 2) {
            set_pixel(pixels, size, x + t, y, r, g, b, a);
            set_pixel(pixels, size, x, y + t, r, g, b, a);
        }

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

// 🎲 Dice icon - D20 shape
fn draw_dice_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (220, 180, 100, 255); // Gold

    // Draw a hexagon-ish D20 shape
    let points = [
        (c, 4),       // top
        (c + 10, 10), // top-right
        (c + 10, 22), // bottom-right
        (c, 28),      // bottom
        (c - 10, 22), // bottom-left
        (c - 10, 10), // top-left
    ];

    // Draw edges
    for i in 0..points.len() {
        let (x0, y0) = points[i];
        let (x1, y1) = points[(i + 1) % points.len()];
        draw_line(
            pixels, size, x0, y0, x1, y1, 2, color.0, color.1, color.2, color.3,
        );
    }

    // Draw internal lines
    draw_line(
        pixels, size, c, 4, c, 28, 2, color.0, color.1, color.2, color.3,
    );
    draw_line(
        pixels,
        size,
        c - 10,
        10,
        c + 10,
        22,
        2,
        color.0,
        color.1,
        color.2,
        color.3,
    );
    draw_line(
        pixels,
        size,
        c + 10,
        10,
        c - 10,
        22,
        2,
        color.0,
        color.1,
        color.2,
        color.3,
    );

    // Draw "20" text (simplified)
    draw_circle(
        pixels,
        size,
        c - 2,
        c,
        3,
        color.0,
        color.1,
        color.2,
        color.3,
    );
    draw_circle(
        pixels,
        size,
        c + 4,
        c,
        3,
        color.0,
        color.1,
        color.2,
        color.3,
    );
}

// 📋 Character icon - person silhouette
fn draw_character_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (150, 200, 255, 255); // Light blue

    // Head
    draw_circle(pixels, size, c, 9, 5, color.0, color.1, color.2, color.3);

    // Body (torso)
    draw_rect(
        pixels,
        size,
        c - 6,
        15,
        12,
        10,
        color.0,
        color.1,
        color.2,
        color.3,
    );

    // Shoulders curve
    draw_circle(
        pixels,
        size,
        c - 6,
        17,
        3,
        color.0,
        color.1,
        color.2,
        color.3,
    );
    draw_circle(
        pixels,
        size,
        c + 6,
        17,
        3,
        color.0,
        color.1,
        color.2,
        color.3,
    );
}

// ℹ️ Info icon - "i" in circle
fn draw_info_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (100, 180, 255, 255); // Blue

    // Circle outline
    draw_circle_outline(
        pixels, size, c, c, 12, 2, color.0, color.1, color.2, color.3,
    );

    // Dot of "i"
    draw_circle(pixels, size, c, 10, 2, color.0, color.1, color.2, color.3);

    // Stem of "i"
    draw_rect(
        pixels,
        size,
        c - 2,
        14,
        4,
        10,
        color.0,
        color.1,
        color.2,
        color.3,
    );
}

// ⚙️ Settings icon - gear
fn draw_settings_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (180, 180, 190, 255); // Silver

    // Outer gear shape (circle with notches)
    draw_circle(pixels, size, c, c, 10, color.0, color.1, color.2, color.3);

    // Inner hole
    draw_circle(pixels, size, c, c, 4, 0, 0, 0, 0);

    // Gear teeth (8 teeth around the circle)
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI / 4.0;
        let tx = c + (12.0 * angle.cos()) as i32;
        let ty = c + (12.0 * angle.sin()) as i32;
        draw_rect(
            pixels,
            size,
            tx - 2,
            ty - 2,
            5,
            5,
            color.0,
            color.1,
            color.2,
            color.3,
        );
    }
}

// ➕ Add icon - plus sign
fn draw_add_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (100, 220, 100, 255); // Green

    // Horizontal bar
    draw_rect(
        pixels,
        size,
        c - 10,
        c - 2,
        20,
        5,
        color.0,
        color.1,
        color.2,
        color.3,
    );

    // Vertical bar
    draw_rect(
        pixels,
        size,
        c - 2,
        c - 10,
        5,
        20,
        color.0,
        color.1,
        color.2,
        color.3,
    );
}

// 🗑️ Delete icon - trash can
fn draw_delete_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (220, 80, 80, 255); // Red

    // Lid
    draw_rect(
        pixels,
        size,
        c - 10,
        6,
        20,
        3,
        color.0,
        color.1,
        color.2,
        color.3,
    );
    draw_rect(
        pixels,
        size,
        c - 4,
        4,
        8,
        2,
        color.0,
        color.1,
        color.2,
        color.3,
    );

    // Body
    draw_rect(
        pixels,
        size,
        c - 8,
        10,
        16,
        16,
        color.0,
        color.1,
        color.2,
        color.3,
    );

    // Lines inside
    draw_rect(pixels, size, c - 5, 13, 2, 10, 80, 30, 30, 255);
    draw_rect(pixels, size, c - 1, 13, 2, 10, 80, 30, 30, 255);
    draw_rect(pixels, size, c + 3, 13, 2, 10, 80, 30, 30, 255);
}

// ✏️ Edit icon - pencil
fn draw_edit_icon(pixels: &mut [u8], size: usize) {
    let color = (255, 200, 100, 255); // Yellow/orange

    // Pencil body (diagonal)
    draw_line(
        pixels, size, 6, 26, 22, 10, 4, color.0, color.1, color.2, color.3,
    );

    // Pencil tip
    draw_line(pixels, size, 4, 28, 8, 24, 2, 200, 150, 80, 255);

    // Eraser end
    draw_line(pixels, size, 22, 10, 26, 6, 3, 220, 100, 100, 255);
}

// 💾 Save icon - floppy disk
fn draw_save_icon(pixels: &mut [u8], size: usize) {
    let color = (100, 150, 220, 255); // Blue

    // Main body
    draw_rect(
        pixels, size, 6, 6, 20, 20, color.0, color.1, color.2, color.3,
    );

    // Label area (white)
    draw_rect(pixels, size, 10, 6, 12, 8, 230, 230, 230, 255);

    // Metal slider
    draw_rect(pixels, size, 10, 16, 12, 10, 60, 60, 80, 255);
    draw_rect(pixels, size, 12, 18, 8, 6, 40, 40, 50, 255);
}

// ❌ Cancel icon - X
fn draw_cancel_icon(pixels: &mut [u8], size: usize) {
    let color = (220, 100, 100, 255); // Red

    // Diagonal lines forming X
    draw_line(
        pixels, size, 8, 8, 24, 24, 3, color.0, color.1, color.2, color.3,
    );
    draw_line(
        pixels, size, 24, 8, 8, 24, 3, color.0, color.1, color.2, color.3,
    );
}

// ✓ Check icon - checkmark
fn draw_check_icon(pixels: &mut [u8], size: usize) {
    let color = (100, 220, 100, 255); // Green

    // Checkmark
    draw_line(
        pixels, size, 6, 16, 12, 24, 3, color.0, color.1, color.2, color.3,
    );
    draw_line(
        pixels, size, 12, 24, 26, 8, 3, color.0, color.1, color.2, color.3,
    );
}

// 🎲 Roll icon - spinning dice
fn draw_roll_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (220, 220, 100, 255); // Yellow

    // Curved arrow (simplified)
    draw_circle_outline(
        pixels, size, c, c, 10, 2, color.0, color.1, color.2, color.3,
    );

    // Arrow head
    draw_line(
        pixels,
        size,
        c + 8,
        c - 6,
        c + 10,
        c - 2,
        2,
        color.0,
        color.1,
        color.2,
        color.3,
    );
    draw_line(
        pixels,
        size,
        c + 4,
        c - 4,
        c + 10,
        c - 2,
        2,
        color.0,
        color.1,
        color.2,
        color.3,
    );

    // Clear part of circle to make arrow
    for dy in -4..4 {
        for dx in 0..6 {
            set_pixel(pixels, size, c + 8 + dx, c + dy, 0, 0, 0, 0);
        }
    }
}

// ▼ Expand icon - down arrow
fn draw_expand_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (180, 180, 200, 255); // Light gray

    // Triangle pointing down
    for row in 0..10 {
        let half_width = row;
        for dx in -half_width..=half_width {
            set_pixel(
                pixels,
                size,
                c + dx,
                10 + row,
                color.0,
                color.1,
                color.2,
                color.3,
            );
        }
    }
}

// ▲ Collapse icon - up arrow
fn draw_collapse_icon(pixels: &mut [u8], size: usize) {
    let c = (size / 2) as i32;
    let color = (180, 180, 200, 255); // Light gray

    // Triangle pointing up
    for row in 0..10 {
        let half_width = 9 - row;
        for dx in -half_width..=half_width {
            set_pixel(
                pixels,
                size,
                c + dx,
                10 + row,
                color.0,
                color.1,
                color.2,
                color.3,
            );
        }
    }
}

/// System to load/generate icons at startup
pub fn load_icons(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut icon_assets = IconAssets::default();
    let size = 32u32;

    for icon_type in IconType::all() {
        let pixels = generate_icon_pixels(*icon_type);

        let image = Image::new(
            Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        );

        let handle = images.add(image);
        icon_assets.icons.insert(*icon_type, handle);
    }

    commands.insert_resource(icon_assets);
    println!("Loaded {} icons", IconType::all().len());
}

/// Helper component to create an icon button
#[derive(Component)]
pub struct IconButton {
    pub icon_type: IconType,
}

/// Spawn an icon button with the given icon
pub fn spawn_icon_button(
    parent: &mut ChildBuilder,
    icon_assets: &IconAssets,
    icon_type: IconType,
    size: f32,
    tooltip: &str,
) -> Entity {
    let icon_handle = icon_assets.icons.get(&icon_type).cloned();

    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(size),
                    height: Val::Px(size),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.8)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            IconButton { icon_type },
            Name::new(tooltip.to_string()),
        ))
        .with_children(|btn| {
            if let Some(handle) = icon_handle {
                btn.spawn(ImageBundle {
                    image: UiImage::new(handle),
                    style: Style {
                        width: Val::Px(size - 8.0),
                        height: Val::Px(size - 8.0),
                        ..default()
                    },
                    ..default()
                });
            }
        })
        .id()
}
