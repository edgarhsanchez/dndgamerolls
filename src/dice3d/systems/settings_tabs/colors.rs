use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy_material_ui::prelude::*;

use crate::dice3d::systems::settings::spawn_color_slider;
use crate::dice3d::types::{
    ColorComponent, ColorPreview, ColorSetting, ColorTextInput, HighlightColorPreview,
    HighlightColorTextInput,
};

pub fn build_colors_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    editing_color: &ColorSetting,
    editing_highlight_color: &ColorSetting,
) {
    parent.spawn((
        Text::new("Background Color"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(120.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(editing_color.to_color()),
                BorderColor::from(theme.outline_variant),
                BorderRadius::all(Val::Px(6.0)),
                ColorPreview,
            ));

            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|sliders| {
                spawn_color_slider(
                    sliders,
                    ColorComponent::Alpha,
                    "A",
                    editing_color.a,
                    theme.on_surface,
                    theme,
                );
                spawn_color_slider(
                    sliders,
                    ColorComponent::Red,
                    "R",
                    editing_color.r,
                    theme.error,
                    theme,
                );
                spawn_color_slider(
                    sliders,
                    ColorComponent::Green,
                    "G",
                    editing_color.g,
                    theme.tertiary,
                    theme,
                );
                spawn_color_slider(
                    sliders,
                    ColorComponent::Blue,
                    "B",
                    editing_color.b,
                    theme.primary,
                    theme,
                );
            });
        });

    parent.spawn((
        Text::new("Enter color (hex, ARGB, or labeled):"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|slot| {
            let builder = TextFieldBuilder::new()
                .outlined()
                .label("Color")
                .value(editing_color.to_hex())
                .supporting_text("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2")
                .width(Val::Percent(100.0));
            spawn_text_field_control_with(slot, theme, builder, ColorTextInput);
        });

    parent.spawn((
        Text::new("Dice Box Highlight Color"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(editing_highlight_color.to_color()),
                BorderColor::from(theme.outline_variant),
                BorderRadius::all(Val::Px(6.0)),
                HighlightColorPreview,
            ));

            row.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|slot| {
                let builder = TextFieldBuilder::new()
                    .outlined()
                    .label("Highlight")
                    .value(editing_highlight_color.to_hex())
                    .supporting_text("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2")
                    .width(Val::Percent(100.0));
                spawn_text_field_control_with(slot, theme, builder, HighlightColorTextInput);
            });
        });
}
