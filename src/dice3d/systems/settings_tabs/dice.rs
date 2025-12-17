use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::prelude::*;
use bevy_material_ui::tokens::CornerRadius;

use crate::dice3d::types::DefaultRollUsesShakeSwitch;

pub fn build_dice_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    select_options: Vec<SelectOption>,
    selected_index: usize,
    default_roll_uses_shake: bool,
) {
    parent.spawn((
        Text::new("Quick Rolls"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Default die for Quick Rolls"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn(Node::default()).with_children(|slot| {
        let builder = SelectBuilder::new(select_options)
            .outlined()
            .label("Quick roll die")
            .selected(selected_index)
            .width(Val::Px(210.0));
        slot.spawn_select_with(theme, builder);
    });

    parent.spawn(Node {
        height: Val::Px(16.0),
        ..default()
    });

    parent.spawn((
        Text::new("Roll Behavior"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Default roll action"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    // Custom spawn so we can tag the actual switch (track) entity.
    let switch = MaterialSwitch::new().selected(default_roll_uses_shake);
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
            // Switch track (touch target)
            row.spawn((
                DefaultRollUsesShakeSwitch,
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
                Text::new("Use shake for all rolls"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface),
            ));
        });
}
