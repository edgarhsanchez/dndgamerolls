use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use crate::dice3d::systems::settings::spawn_dice_scale_slider;
use crate::dice3d::types::{
    DefaultRollUsesShakeSwitch, DiceFxEffectKind, DiceFxParamKind, DiceFxParamSlider,
    DiceFxParamValueLabel, DiceScaleSettings, DiceType, SettingsState,
};

pub fn build_dice_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    select_options: Vec<SelectOption>,
    selected_index: usize,
    default_roll_uses_shake: bool,
    dice_scales: &DiceScaleSettings,
    _preview_image: Option<Handle<Image>>,
    _blank_image: Option<Handle<Image>>,
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

    // ---------------------------------------------------------------------
    // Global Dice FX visuals
    // ---------------------------------------------------------------------
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
                    width: Val::Percent(100.0),
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
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|slot| {
                        let slider = MaterialSlider::new(min, max)
                            .with_value(value.clamp(min, max))
                            .track_height(6.0)
                            .thumb_radius(8.0);
                        slot.spawn(Node {
                            width: Val::Px(width_px),
                            height: Val::Px(30.0),
                            ..default()
                        })
                        .with_children(|slider_slot| {
                            spawn_slider_control_with(slider_slot, theme, slider, DiceFxParamSlider { kind });
                        });
                    });

                    row.spawn((
                        Text::new(format!("{:.2}", value)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ))
                    .insert(DiceFxParamValueLabel { kind });
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
                240.0,
            );
            spawn_param_slider(
                col,
                theme,
                "Plume height",
                DiceFxParamKind::PlumeHeight,
                0.25,
                3.0,
                settings_state.editing_dice_fx_plume_height_multiplier,
                240.0,
            );
            spawn_param_slider(
                col,
                theme,
                "Plume radius",
                DiceFxParamKind::PlumeRadius,
                0.25,
                3.0,
                settings_state.editing_dice_fx_plume_radius_multiplier,
                240.0,
            );
        });

    // ---------------------------------------------------------------------
    // Quick roll die select + shake switch
    // ---------------------------------------------------------------------
    parent.spawn(Node {
        height: Val::Px(10.0),
        ..default()
    });

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|slot| {
                let builder = SelectBuilder::new(select_options)
                    .outlined()
                    .label("Quick roll die")
                    .selected(selected_index)
                    .width(Val::Percent(100.0));
                slot.spawn_select_with(theme, builder);
            });

            // Manual switch spawn so the SwitchChangeEvent entity can be tagged.
            let mut sw = MaterialSwitch::new();
            sw.selected = default_roll_uses_shake;
            sw.animation_progress = if default_roll_uses_shake { 1.0 } else { 0.0 };
            let bg_color = sw.track_color(theme);
            let border_color = sw.track_outline_color(theme);
            let handle_color = sw.handle_color(theme);
            let handle_size = sw.handle_size();
            let has_border = !sw.selected;
            let justify = if sw.selected {
                JustifyContent::FlexEnd
            } else {
                JustifyContent::FlexStart
            };

            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|switch_row| {
                switch_row
                    .spawn((
                        sw,
                        Button,
                        Interaction::None,
                        RippleHost::new(),
                        Node {
                            width: Val::Px(52.0),
                            height: Val::Px(32.0),
                            justify_content: justify,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(2.0)),
                            border: UiRect::all(Val::Px(if has_border { 2.0 } else { 0.0 })),
                            ..default()
                        },
                        BackgroundColor(bg_color),
                        BorderColor::all(border_color),
                        BorderRadius::all(Val::Px(16.0)),
                        DefaultRollUsesShakeSwitch,
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

                switch_row.spawn((
                    Text::new("Use container shake"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                ));
            });
        });

    // Dice scale sliders
    parent.spawn(Node {
        height: Val::Px(10.0),
        ..default()
    });
    for die_type in [
        DiceType::D4,
        DiceType::D6,
        DiceType::D8,
        DiceType::D10,
        DiceType::D12,
        DiceType::D20,
    ] {
        let value = dice_scales.scale_for(die_type);
        spawn_dice_scale_slider(
            parent,
            die_type,
            &format!("{} scale", die_type.name()),
            value,
            theme,
        );
    }

    // ---------------------------------------------------------------------
    // Dice Roll Effects (built-in mapping)
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

    let die_type = settings_state.quick_roll_editing_die.to_dice_type();
    parent.spawn((
        Text::new(format!(
            "Choose which effect to play when a {} settles on each face value.",
            die_type.name()
        )),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    let max = die_type.max_value();
    let options = vec![
        SelectOption::new("None").value("none"),
        SelectOption::new("Fire").value("fire"),
        SelectOption::new("Lightning").value("lightning"),
        SelectOption::new("Firework").value("firework"),
        SelectOption::new("Explosion").value("explosion"),
    ];

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|col| {
            for value in 1..=max {
                let current = settings_state
                    .editing_dice_fx_roll_effects
                    .effect_for(die_type, value);

                let selected = match current {
                    DiceFxEffectKind::None => 0,
                    DiceFxEffectKind::Fire => 1,
                    DiceFxEffectKind::Lightning => 2,
                    DiceFxEffectKind::Firework => 3,
                    DiceFxEffectKind::Explosion => 4,
                };

                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!("{value}")),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));

                    row.spawn(Node {
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|slot| {
                        let builder = SelectBuilder::new(options.clone())
                            .outlined()
                            .label(format!("Effect for {value}"))
                            .selected(selected)
                            .width(Val::Percent(100.0));
                        slot.spawn_select_with(theme, builder);
                    });
                });
            }
        });
    }

    /* Legacy Dice FX curve/custom UI removed
                        crate::dice3d::types::DiceFxEffectKind::Lightning => 2,
                                    crate::dice3d::types::DiceFxEffectKind::Firework => 3,
                                    crate::dice3d::types::DiceFxEffectKind::Explosion => 4,
                                };

                                col.spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        Text::new(format!("{}", value)),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(theme.on_surface_variant),
                                    ));

                                    row.spawn(Node {
                                        flex_grow: 1.0,
                                        min_width: Val::Px(0.0),
                                        ..default()
                                    })
                                    .with_children(|slot| {
                                        let builder = SelectBuilder::new(options.clone())
                                            .outlined()
                                            .label(format!("Effect for {}", value))
                                            .selected(selected)
                                            .width(Val::Percent(100.0));
                                        slot.spawn_select_with(theme, builder)
                                            .insert(DiceFxRollEffectSelect { die_type, value });
                                    });
                                });
                            }
                        });
                ..default()
            })
            .with_children(|slot| {
                let builder = TextFieldBuilder::new()
                    .outlined()
                    .label("Trigger value")
                    .value(settings_state.dice_fx_trigger_value_input_text.clone())
                    .supporting_text("Used by Total ≥ value and Any die equals value")
                    .width(Val::Percent(100.0));
                spawn_text_field_control_with(slot, theme, builder, DiceFxTriggerValueTextInput);
            });

            row.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|slot| {
                let builder = TextFieldBuilder::new()
                    .outlined()
                    .label("Effect duration (seconds)")
                    .value(settings_state.dice_fx_duration_input_text.clone())
                    .supporting_text("How long the effect stays active")
                    .width(Val::Percent(100.0));
                spawn_text_field_control_with(slot, theme, builder, DiceFxDurationTextInput);
            });
        });

    parent.spawn((
        Text::new(
            "Curve graph area (click to add; drag points/Bezier handles). Choose Add/Delete, then click.",
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    // Curve chips (Add/Delete + Mask/Noise/Ramp)
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(8.0),
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            // Add/Delete
            row.spawn(
                ChipBuilder::filter("Add")
                    .selected(settings_state.dice_fx_curve_edit_mode == ShakeCurveEditMode::Add)
                    .value("dice_fx_curve_mode_add")
                    .build(theme),
            )
            .insert(DiceFxCurveEditModeChip {
                mode: ShakeCurveEditMode::Add,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Add"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });

            row.spawn(
                ChipBuilder::filter("Delete")
                    .selected(settings_state.dice_fx_curve_edit_mode == ShakeCurveEditMode::Delete)
                    .value("dice_fx_curve_mode_delete")
                    .build(theme),
            )
            .insert(DiceFxCurveEditModeChip {
                mode: ShakeCurveEditMode::Delete,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Delete"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });

            // Channels
            row.spawn(
                ChipBuilder::filter("Mask")
                    .selected(settings_state.dice_fx_curve_add_mask)
                    .value("dice_fx_curve_mask")
                    .build(theme),
            )
            .insert(DiceFxCurveChannelChip {
                channel: DiceFxCurveChannel::Mask,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Mask"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });

            row.spawn(
                ChipBuilder::filter("Noise")
                    .selected(settings_state.dice_fx_curve_add_noise)
                    .value("dice_fx_curve_noise")
                    .build(theme),
            )
            .insert(DiceFxCurveChannelChip {
                channel: DiceFxCurveChannel::Noise,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Noise"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });

            row.spawn(
                ChipBuilder::filter("Ramp")
                    .selected(settings_state.dice_fx_curve_add_ramp)
                    .value("dice_fx_curve_ramp")
                    .build(theme),
            )
            .insert(DiceFxCurveChannelChip {
                channel: DiceFxCurveChannel::Ramp,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Ramp"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });

            row.spawn(
                ChipBuilder::filter("Opacity")
                    .selected(settings_state.dice_fx_curve_add_opacity)
                    .value("dice_fx_curve_opacity")
                    .build(theme),
            )
            .insert(DiceFxCurveChannelChip {
                channel: DiceFxCurveChannel::Opacity,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Opacity"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });

            row.spawn(
                ChipBuilder::filter("Plume H")
                    .selected(settings_state.dice_fx_curve_add_plume_height)
                    .value("dice_fx_curve_plume_height")
                    .build(theme),
            )
            .insert(DiceFxCurveChannelChip {
                channel: DiceFxCurveChannel::PlumeHeight,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Plume H"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });

            row.spawn(
                ChipBuilder::filter("Plume R")
                    .selected(settings_state.dice_fx_curve_add_plume_radius)
                    .value("dice_fx_curve_plume_radius")
                    .build(theme),
            )
            .insert(DiceFxCurveChannelChip {
                channel: DiceFxCurveChannel::PlumeRadius,
            })
            .with_children(|chip| {
                chip.spawn((
                    ChipLabel,
                    Text::new("Plume R"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });
        });

    // Graph
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                height: Val::Px(220.0),
                min_height: Val::Px(220.0),
                flex_shrink: 0.0,
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme.surface_container_highest),
            BorderColor::from(theme.outline_variant),
            BorderRadius::all(Val::Px(8.0)),
            DiceFxCurveGraphRoot,
        ))
        .with_children(|graph| {
            graph
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(8.0),
                        right: Val::Px(8.0),
                        top: Val::Px(8.0),
                        bottom: Val::Px(8.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    DiceFxCurveGraphPlotRoot,
                ))
                .with_children(|plot| {
                    // Grid lines (subtle), matching the Shake Curve editor.
                    let grid_color = theme.outline_variant.with_alpha(0.35);
                    let half_grid_color = theme.outline_variant.with_alpha(0.22);

                    // 100% line (slightly inset so max values are fully visible)
                    plot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Px(7.0),
                            height: Val::Px(1.0),
                            ..default()
                        },
                        FocusPolicy::Pass,
                        BackgroundColor(grid_color),
                    ));

                    // 75% line
                    plot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Percent(25.0),
                            height: Val::Px(1.0),
                            ..default()
                        },
                        FocusPolicy::Pass,
                        BackgroundColor(half_grid_color),
                    ));

                    // 50% midline
                    plot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Percent(50.0),
                            height: Val::Px(2.0),
                            ..default()
                        },
                        FocusPolicy::Pass,
                        BackgroundColor(theme.outline_variant.with_alpha(0.45)),
                    ));

                    // 25% line
                    plot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Percent(75.0),
                            height: Val::Px(1.0),
                            ..default()
                        },
                        FocusPolicy::Pass,
                        BackgroundColor(half_grid_color),
                    ));

                    // 0% line (slightly inset so min values are fully visible)
                    plot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            bottom: Val::Px(7.0),
                            height: Val::Px(1.0),
                            ..default()
                        },
                        FocusPolicy::Pass,
                        BackgroundColor(grid_color),
                    ));

                    // Sample dots
                    const DOTS: usize = 80;
                    for channel in [
                        DiceFxCurveChannel::Mask,
                        DiceFxCurveChannel::Noise,
                        DiceFxCurveChannel::Ramp,
                        DiceFxCurveChannel::Opacity,
                        DiceFxCurveChannel::PlumeHeight,
                        DiceFxCurveChannel::PlumeRadius,
                    ] {
                        let color = match channel {
                            DiceFxCurveChannel::Mask => theme.primary,
                            DiceFxCurveChannel::Noise => theme.secondary,
                            DiceFxCurveChannel::Ramp => theme.tertiary,
                            DiceFxCurveChannel::Opacity => theme.error,
                            DiceFxCurveChannel::PlumeHeight => theme.on_surface,
                            DiceFxCurveChannel::PlumeRadius => theme.on_surface_variant,
                        };

                        for i in 0..DOTS {
                            plot.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    top: Val::Px(0.0),
                                    width: Val::Px(3.0),
                                    height: Val::Px(3.0),
                                    ..default()
                                },
                                BackgroundColor(color),
                                BorderRadius::all(Val::Px(2.0)),
                                DiceFxCurveGraphDot { channel, index: i },
                            ));
                        }
                    }

                    // Handles for existing points across all curves
                    for p in settings_state
                        .editing_custom_dice_fx
                        .curve_points_mask
                        .iter()
                        .chain(settings_state.editing_custom_dice_fx.curve_points_noise.iter())
                        .chain(settings_state.editing_custom_dice_fx.curve_points_ramp.iter())
                        .chain(settings_state.editing_custom_dice_fx.curve_points_opacity.iter())
                        .chain(
                            settings_state
                                .editing_custom_dice_fx
                                .curve_points_plume_height
                                .iter(),
                        )
                        .chain(
                            settings_state
                                .editing_custom_dice_fx
                                .curve_points_plume_radius
                                .iter(),
                        )
                    {
                        plot.spawn((
                            Button,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Px(14.0),
                                height: Val::Px(14.0),
                                ..default()
                            },
                            BackgroundColor(theme.surface_container_high),
                            BorderRadius::all(Val::Px(7.0)),
                            BorderColor::from(theme.outline_variant),
                            Interaction::None,
                            DiceFxCurvePointHandle { id: p.id },
                        ));
                    }
                });
        });

    // Extra breathing room so the graph can be scrolled fully into view.
    parent.spawn(Node {
        height: Val::Px(24.0),
        ..default()
    });
}
*/
