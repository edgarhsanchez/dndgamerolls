use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::prelude::*;
use bevy_material_ui::tokens::CornerRadius;

use crate::dice3d::systems::settings::spawn_dice_scale_slider;
use crate::dice3d::types::{
    DefaultRollUsesShakeSwitch, DiceFxParamKind, DiceFxParamSlider, DiceFxParamValueLabel,
    DiceRollFxKind, DiceRollFxMappingSelect, DiceScaleSettings, DiceType, SettingsState,
};

pub fn build_dice_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    select_options: Vec<SelectOption>,
    selected_index: usize,
    default_roll_uses_shake: bool,
    dice_scales: &DiceScaleSettings,
    preview_image: Option<Handle<Image>>,
    settings_state: &SettingsState,
) {
    parent.spawn((
        Text::new("Quick Rolls"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    // Global Dice FX visuals
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|col| {
            fn spawn_param_slider(
                col: &mut ChildSpawnerCommands,
                theme: &MaterialTheme,
                label: &str,
                kind: DiceFxParamKind,
                min: f32,
                max: f32,
                value: f32,
                width_px: f32,
            ) {
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    height: Val::Px(30.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));

                    row.spawn(Node {
                        width: Val::Px(width_px),
                        height: Val::Px(30.0),
                        ..default()
                    })
                    .with_children(|slot| {
                        let slider = MaterialSlider::new(min, max)
                            .with_value(value.clamp(min, max))
                            .track_height(6.0)
                            .thumb_radius(8.0);
                        spawn_slider_control_with(slot, theme, slider, DiceFxParamSlider { kind });
                    });

                    row.spawn((
                        Text::new(format!("{:.2}", value)),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                        DiceFxParamValueLabel { kind },
                    ));
                });
            }

            spawn_param_slider(
                col,
                theme,
                "Surface opacity",
                DiceFxParamKind::SurfaceOpacity,
                0.0,
                1.0,
                settings_state.editing_dice_fx_surface_opacity,
                260.0,
            );

            spawn_param_slider(
                col,
                theme,
                "Plume height",
                DiceFxParamKind::PlumeHeight,
                0.25,
                3.0,
                settings_state.editing_dice_fx_plume_height_multiplier,
                260.0,
            );

            spawn_param_slider(
                col,
                theme,
                "Plume radius",
                DiceFxParamKind::PlumeRadius,
                0.25,
                3.0,
                settings_state.editing_dice_fx_plume_radius_multiplier,
                260.0,
            );
        });

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

    parent.spawn(Node {
        height: Val::Px(16.0),
        ..default()
    });

    parent.spawn((
        Text::new("Dice Sizes"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Adjust the 3D size of each die type"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    // Sliders + preview side-by-side.
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(16.0),
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|row| {
            // Left: sliders
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|sliders| {
                // Keep ordering invariant by providing an ordered set of sliders.
                spawn_dice_scale_slider(
                    sliders,
                    DiceType::D4,
                    "d4",
                    dice_scales.scale_for(DiceType::D4),
                    theme,
                );
                spawn_dice_scale_slider(
                    sliders,
                    DiceType::D6,
                    "d6",
                    dice_scales.scale_for(DiceType::D6),
                    theme,
                );
                spawn_dice_scale_slider(
                    sliders,
                    DiceType::D8,
                    "d8",
                    dice_scales.scale_for(DiceType::D8),
                    theme,
                );
                spawn_dice_scale_slider(
                    sliders,
                    DiceType::D10,
                    "d10",
                    dice_scales.scale_for(DiceType::D10),
                    theme,
                );
                spawn_dice_scale_slider(
                    sliders,
                    DiceType::D12,
                    "d12",
                    dice_scales.scale_for(DiceType::D12),
                    theme,
                );
                spawn_dice_scale_slider(
                    sliders,
                    DiceType::D20,
                    "d20",
                    dice_scales.scale_for(DiceType::D20),
                    theme,
                );
            });

            // Right: live 3D preview
            row.spawn((
                Node {
                    width: Val::Px(360.0),
                    height: Val::Px(220.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(theme.surface_container),
                BorderColor::all(theme.outline_variant),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_children(|preview| {
                if let Some(handle) = preview_image {
                    preview.spawn((
                        bevy::ui::widget::ImageNode::new(handle),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                } else {
                    preview.spawn((
                        Text::new("Preview unavailable"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));
                }
            });
        });

    // ---------------------------------------------------------------------
    // Dice Roll Effects (hardcoded FX, mapped per die face value)
    // ---------------------------------------------------------------------

    parent.spawn(Node {
        height: Val::Px(18.0),
        ..default()
    });

    parent.spawn((
        Text::new("Dice Roll Effects"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Choose which effect to play for each die face value."),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    fn fx_kind_options() -> Vec<SelectOption> {
        vec![
            SelectOption::new(DiceRollFxKind::None.label()).value("none"),
            SelectOption::new(DiceRollFxKind::Fire.label()).value("fire"),
            SelectOption::new(DiceRollFxKind::Electricity.label()).value("electricity"),
            SelectOption::new(DiceRollFxKind::Fireworks.label()).value("fireworks"),
            SelectOption::new(DiceRollFxKind::Explosion.label()).value("explosion"),
            SelectOption::new(DiceRollFxKind::Plasma.label()).value("plasma"),
        ]
    }

    fn kind_to_index(kind: DiceRollFxKind) -> usize {
        match kind {
            DiceRollFxKind::None => 0,
            DiceRollFxKind::Fire => 1,
            DiceRollFxKind::Electricity => 2,
            DiceRollFxKind::Fireworks => 3,
            DiceRollFxKind::Explosion => 4,
            DiceRollFxKind::Plasma => 5,
        }
    }

    fn editing_roll_fx_for(
        settings_state: &SettingsState,
        die_type: DiceType,
        value: u32,
    ) -> DiceRollFxKind {
        if value == 0 {
            return DiceRollFxKind::None;
        }

        settings_state
            .editing_dice_roll_fx_mappings
            .iter()
            .find(|m| m.die_type == die_type)
            .map(|m| m.get(value))
            .unwrap_or(DiceRollFxKind::None)
    }

    let dice_types = [
        DiceType::D4,
        DiceType::D6,
        DiceType::D8,
        DiceType::D10,
        DiceType::D12,
        DiceType::D20,
    ];
    let options = fx_kind_options();

    for die_type in dice_types {
        parent.spawn(Node {
            height: Val::Px(10.0),
            ..default()
        });

        parent.spawn((
            Text::new(die_type.name()),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(theme.on_surface_variant),
        ));

        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(22.0),
                row_gap: Val::Px(18.0),
                width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|wrap| {
                for value in 1..=die_type.max_value() {
                    let selected =
                        kind_to_index(editing_roll_fx_for(settings_state, die_type, value));

                    wrap.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!("{}:", value)),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));

                        row.spawn((
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(32.0),
                                ..default()
                            },
                            DiceRollFxMappingSelect { die_type, value },
                        ))
                        .with_children(|slot| {
                            let builder = SelectBuilder::new(options.clone())
                                .outlined()
                                .label("")
                                .selected(selected)
                                .width(Val::Px(150.0));
                            slot.spawn_select_with(theme, builder);
                        });
                    });
                }
            });
    }
}
