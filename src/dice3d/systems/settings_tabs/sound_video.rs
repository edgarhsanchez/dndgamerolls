use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::prelude::*;
use bevy_material_ui::slider::spawn_slider_control_with;
use bevy_material_ui::switch::{SWITCH_TRACK_HEIGHT, SWITCH_TRACK_WIDTH};
use bevy_material_ui::tokens::CornerRadius;

use crate::dice3d::types::{MasterVolumeSlider, MasterVolumeValueLabel, VsyncSwitch};

pub fn build_sound_video_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    master_volume: f32,
    vsync_enabled: bool,
) {
    // Sound
    parent.spawn((
        Text::new("Sound"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Adjust in-game volume (does not change system volume)."),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            height: Val::Px(36.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new("Game volume"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));

            row.spawn(Node {
                width: Val::Px(260.0),
                height: Val::Px(32.0),
                ..default()
            })
            .with_children(|slot| {
                let slider = MaterialSlider::new(0.0, 1.0)
                    .with_value(master_volume.clamp(0.0, 1.0))
                    .track_height(6.0)
                    .thumb_radius(8.0);
                spawn_slider_control_with(slot, theme, slider, MasterVolumeSlider);
            });

            row.spawn((
                Text::new(format!("{:.0}%", master_volume * 100.0)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
                MasterVolumeValueLabel,
            ));
        });

    parent.spawn(Node {
        height: Val::Px(16.0),
        ..default()
    });

    // Video
    parent.spawn((
        Text::new("Video"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Disable VSync to allow higher frame rates (may increase GPU load)."),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    let switch = MaterialSwitch::new().selected(vsync_enabled);
    let bg_color = switch.track_color(theme);
    let border_color = switch.track_outline_color(theme);
    let handle_color = switch.handle_color(theme);
    let handle_size = switch.handle_size();
    let has_border = !switch.selected;
    let justify = if switch.selected {
        JustifyContent::FlexEnd
    } else {
        JustifyContent::FlexStart
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                VsyncSwitch,
                switch,
                Button,
                Interaction::None,
                RippleHost::new(),
                Node {
                    width: Val::Px(SWITCH_TRACK_WIDTH),
                    height: Val::Px(SWITCH_TRACK_HEIGHT),
                    justify_content: justify,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(2.0)),
                    border: UiRect::all(Val::Px(if has_border { 2.0 } else { 0.0 })),
                    ..default()
                },
                BackgroundColor(bg_color),
                BorderColor::all(border_color),
                BorderRadius::all(Val::Px(CornerRadius::FULL)),
            ))
            .with_children(|track| {
                track.spawn((
                    SwitchHandle,
                    Node {
                        width: Val::Px(handle_size),
                        height: Val::Px(handle_size),
                        ..default()
                    },
                    BackgroundColor(handle_color),
                    BorderRadius::all(Val::Px(handle_size / 2.0)),
                ));
            });

            row.spawn((
                Text::new("Enable VSync"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));
        });
}
