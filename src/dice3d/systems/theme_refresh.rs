use bevy::prelude::*;
use bevy_material_ui::prelude::{
    MaterialTheme, ScrollbarThumbHorizontal, ScrollbarThumbVertical, ScrollbarTrackHorizontal,
    ScrollbarTrackVertical,
};

pub fn refresh_scrollbar_colors_on_theme_change(
    theme: Res<MaterialTheme>,
    mut parts: Query<
        (
            &mut BackgroundColor,
            Option<&ScrollbarTrackVertical>,
            Option<&ScrollbarThumbVertical>,
            Option<&ScrollbarTrackHorizontal>,
            Option<&ScrollbarThumbHorizontal>,
        ),
        Or<(
            With<ScrollbarTrackVertical>,
            With<ScrollbarThumbVertical>,
            With<ScrollbarTrackHorizontal>,
            With<ScrollbarThumbHorizontal>,
        )>,
    >,
) {
    if !theme.is_changed() {
        return;
    }

    let track_color = theme.surface_container_highest.with_alpha(0.5);
    let thumb_color = theme.primary.with_alpha(0.7);

    for (mut bg, track_v, thumb_v, track_h, thumb_h) in parts.iter_mut() {
        let is_track = track_v.is_some() || track_h.is_some();
        let is_thumb = thumb_v.is_some() || thumb_h.is_some();

        if is_track {
            *bg = BackgroundColor(track_color);
        } else if is_thumb {
            *bg = BackgroundColor(thumb_color);
        }
    }
}
