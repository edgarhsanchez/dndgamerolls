use bevy::prelude::*;

use crate::dice3d::systems::color_picker::{
    open_color_picker, ColorPickerState, ColorPickerTarget,
};
use crate::dice3d::types::{ColorPreview, HighlightColorPreview, SettingsState, ThemeColorPreview};

pub fn handle_color_preview_clicks(
    mut color_picker_state: ResMut<ColorPickerState>,
    _settings: Res<SettingsState>,

    // Queries for the preview boxes
    bg_query: Query<(&Interaction, &BackgroundColor), (Changed<Interaction>, With<ColorPreview>)>,
    highlight_query: Query<
        (&Interaction, &BackgroundColor),
        (Changed<Interaction>, With<HighlightColorPreview>),
    >,
    theme_query: Query<
        (&Interaction, &BackgroundColor),
        (Changed<Interaction>, With<ThemeColorPreview>),
    >,
) {
    if color_picker_state.active {
        return;
    }

    // Background Color
    if let Some((interaction, bg_color)) = bg_query.iter().next() {
        if *interaction == Interaction::Pressed {
            open_color_picker(
                ColorPickerTarget::Background,
                bg_color.0.to_srgba(),
                &mut color_picker_state,
            );
        }
    }

    // Highlight Color
    if let Some((interaction, bg_color)) = highlight_query.iter().next() {
        if *interaction == Interaction::Pressed {
            open_color_picker(
                ColorPickerTarget::Highlight,
                bg_color.0.to_srgba(),
                &mut color_picker_state,
            );
        }
    }

    // Theme Color
    if let Some((interaction, bg_color)) = theme_query.iter().next() {
        if *interaction == Interaction::Pressed {
            open_color_picker(
                ColorPickerTarget::Theme,
                bg_color.0.to_srgba(),
                &mut color_picker_state,
            );
        }
    }
}
