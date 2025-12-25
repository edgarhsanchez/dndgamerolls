use bevy::prelude::*;
use bevy_material_ui::prelude::{MaterialSelect, SelectOptionItem};
use bevy_material_ui::select::SelectOptionLabelText;

use crate::dice3d::types::ColorSetting;

fn relative_luminance_srgb(c: Color) -> f32 {
    // Convert to linear RGB for relative luminance.
    // Bevy's Color conversions are a bit version-specific; keep it simple and robust.
    let s = c.to_srgba();
    let r = srgb_to_linear(s.red);
    let g = srgb_to_linear(s.green);
    let b = srgb_to_linear(s.blue);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_to_linear(u: f32) -> f32 {
    if u <= 0.04045 {
        u / 12.92
    } else {
        ((u + 0.055) / 1.055).powf(2.4)
    }
}

fn set_option_text_color_recursive(
    root: Entity,
    desired: Color,
    children: &Query<&Children>,
    labels: &mut Query<&mut TextColor, With<SelectOptionLabelText>>,
    depth: u8,
) {
    if depth == 0 {
        return;
    }

    if let Ok(mut tc) = labels.get_mut(root) {
        *tc = TextColor(desired);
    }

    let Ok(kids) = children.get(root) else {
        return;
    };

    for child in kids.iter() {
        set_option_text_color_recursive(child, desired, children, labels, depth - 1);
    }
}

/// Tint the "Recent themes" select dropdown option rows based on their hex value.
///
/// We run this in `PostUpdate` so it can override the default MD3 menu styling.
pub fn tint_recent_theme_dropdown_items(
    parents: Query<&ChildOf>,
    selects: Query<&MaterialSelect>,
    mut option_rows: Query<(Entity, &SelectOptionItem, &mut BackgroundColor, &Children)>,
    children: Query<&Children>,
    mut label_colors: Query<&mut TextColor, With<SelectOptionLabelText>>,
) {
    for (row_entity, option_item, mut bg, row_children) in option_rows.iter_mut() {
        // Walk up to the owning MaterialSelect.
        let mut current = Some(row_entity);
        let mut owner_select: Option<&MaterialSelect> = None;
        for _ in 0..32 {
            let Some(e) = current else { break };
            if let Ok(select) = selects.get(e) {
                owner_select = Some(select);
                break;
            }
            current = parents.get(e).ok().map(|p| p.0);
        }

        let Some(select) = owner_select else {
            continue;
        };

        // Only apply to the "Recent themes" dropdown.
        if select.label.as_deref() != Some("Recent themes") {
            continue;
        }

        let Some(opt) = select.options.get(option_item.index) else {
            continue;
        };

        let raw = opt.value.as_deref().unwrap_or(opt.label.as_str());
        let Some(mut parsed) = ColorSetting::parse(raw) else {
            continue;
        };

        // Seed alpha is ignored for themes; keep the swatch opaque.
        parsed.a = 1.0;
        let swatch = parsed.to_color();
        *bg = BackgroundColor(swatch);

        // Improve readability: choose black/white based on swatch luminance.
        let lum = relative_luminance_srgb(swatch);
        let text = if lum >= 0.55 {
            Color::BLACK
        } else {
            Color::WHITE
        };

        for child in row_children.iter() {
            set_option_text_color_recursive(child, text, &children, &mut label_colors, 8);
        }
    }
}
