use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use crate::dice3d::types::SettingsResetLayoutButton;

pub fn build_layout_tab(parent: &mut ChildSpawnerCommands, theme: &MaterialTheme) {
    parent.spawn((
        Text::new("Layout"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Reset draggable panel positions to defaults."),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent
        .spawn(Node {
            width: Val::Px(200.0),
            height: Val::Px(36.0),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn((
                MaterialButtonBuilder::new("Reset layout")
                    .outlined()
                    .build(theme),
                SettingsResetLayoutButton,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Reset layout"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(theme.primary),
                    ButtonLabel,
                ));
            });
        });
}
