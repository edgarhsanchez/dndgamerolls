use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::prelude::*;
use bevy_material_ui::text_field::spawn_text_field_control_with;

use crate::dice3d::types::{
    ColorPreview, ColorSetting, ColorTextInput, HighlightColorPreview, HighlightColorTextInput,
    ThemeColorPreview, ThemeSeedTextInput,
};

pub fn build_colors_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    editing_color: &ColorSetting,
    editing_highlight_color: &ColorSetting,
    theme_seed_input_text: &str,
    recent_theme_seeds: &[String],
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
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(editing_color.to_color()),
                BorderColor::from(theme.outline_variant),
                BorderRadius::all(Val::Px(6.0)),
                ColorPreview,
                Interaction::default(),
            ));

            row.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|slot| {
                let builder = TextFieldBuilder::new()
                    .outlined()
                    .label("Color")
                    .value(editing_color.to_hex())
                    .supporting_text(
                        "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray",
                    )
                    .width(Val::Percent(100.0));
                spawn_text_field_control_with(slot, theme, builder, ColorTextInput);
            });
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
                Interaction::default(),
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
                    .supporting_text(
                        "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray",
                    )
                    .width(Val::Percent(100.0));
                spawn_text_field_control_with(slot, theme, builder, HighlightColorTextInput);
            });
        });

    parent.spawn(Node {
        height: Val::Px(16.0),
        ..default()
    });

    parent.spawn((
        Text::new("Theme"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Theme seed (hex or name)"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|row| {
            // Recent theme seeds dropdown
            if !recent_theme_seeds.is_empty() {
                let options: Vec<SelectOption> = recent_theme_seeds
                    .iter()
                    .map(|hex| SelectOption::new(hex).value(hex))
                    .collect();

                let mut builder = SelectBuilder::new(options)
                    .outlined()
                    .label("Recent themes");
                if let Some(idx) = recent_theme_seeds
                    .iter()
                    .position(|s| s.eq_ignore_ascii_case(theme_seed_input_text))
                {
                    builder = builder.selected(idx);
                }

                row.spawn(Node {
                    width: Val::Px(210.0),
                    ..default()
                })
                .with_children(|slot| {
                    slot.spawn_select_with(theme, builder.width(Val::Px(210.0)));
                });
            }

            // Theme seed hex input
            row.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|slot| {
                // Preview box for theme seed
                let seed_color = ColorSetting::parse(theme_seed_input_text).unwrap_or_default();
                slot.spawn((
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(seed_color.to_color()),
                    BorderColor::from(theme.outline_variant),
                    BorderRadius::all(Val::Px(4.0)),
                    ThemeColorPreview,
                    Interaction::default(),
                ));

                let builder = TextFieldBuilder::new()
                    .outlined()
                    .label("Seed")
                    .value(theme_seed_input_text)
                    .supporting_text("#RRGGBB or #AARRGGBB (alpha ignored)")
                    .width(Val::Percent(100.0));
                spawn_text_field_control_with(slot, theme, builder, ThemeSeedTextInput);
            });
        });
}
