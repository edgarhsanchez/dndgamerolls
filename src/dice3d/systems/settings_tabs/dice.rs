use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_material_ui::prelude::*;
use bevy_material_ui::tokens::CornerRadius;

use crate::dice3d::systems::settings::spawn_dice_scale_slider;
use crate::dice3d::types::{
    DefaultRollUsesShakeSwitch, DiceScaleSettings, DiceType, DiceFxCurveChannel,
    DiceFxCurveChannelChip, DiceFxCurveEditModeChip, DiceFxCurveGraphDot,
    DiceFxCurveGraphPlotRoot, DiceFxCurveGraphRoot, DiceFxCurvePointHandle,
    DiceFxDurationTextInput, DiceFxTriggerValueTextInput,
    DiceFxPreviewImageKind, DiceFxPreviewImageNode, DiceFxPreviewTimeLabel,
    DiceFxPreviewTimeSlider, DiceFxUploadImageButton, SettingsState,
    ShakeCurveEditMode,
};

pub fn build_dice_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    select_options: Vec<SelectOption>,
    selected_index: usize,
    default_roll_uses_shake: bool,
    dice_scales: &DiceScaleSettings,
    preview_image: Option<Handle<Image>>,
    blank_image: Option<Handle<Image>>,
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
    // Dice Effects (custom, runtime-loadable)
    // ---------------------------------------------------------------------

    parent.spawn(Node {
        height: Val::Px(18.0),
        ..default()
    });

    parent.spawn((
        Text::new("Dice Effects"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new(
            "Upload an image to generate mask/noise/ramp, then configure when the effect triggers.",
        ),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    // Output directory
    if !settings_state.dice_fx_saved_dir_display_text.is_empty() {
        parent.spawn((
            Text::new(format!(
                "Saved to: {}",
                settings_state.dice_fx_saved_dir_display_text
            )),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(theme.on_surface_variant),
        ));
    }

    // Upload button
    parent
        .spawn(Node {
            width: Val::Px(200.0),
            height: Val::Px(36.0),
            ..default()
        })
        .with_children(|slot| {
            slot.spawn((
                MaterialButtonBuilder::new("Upload image")
                    .outlined()
                    .build(theme),
                DiceFxUploadImageButton,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Upload image"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(theme.primary),
                    ButtonLabel,
                ));
            });
        });

    // Image previews (Source/Noise/Mask/Ramp)
    let blank = blank_image.clone();
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(12.0),
            row_gap: Val::Px(12.0),
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|row| {
            fn spawn_preview(
                row: &mut ChildSpawnerCommands,
                theme: &MaterialTheme,
                label: &str,
                kind: DiceFxPreviewImageKind,
                blank: Option<Handle<Image>>,
                width: f32,
                height: f32,
            ) {
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));

                    col.spawn((
                        Node {
                            width: Val::Px(width),
                            height: Val::Px(height),
                            border: UiRect::all(Val::Px(1.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(theme.surface_container),
                        BorderColor::all(theme.outline_variant),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|box_| {
                        if let Some(blank) = blank {
                            box_.spawn((
                                bevy::ui::widget::ImageNode::new(blank),
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                DiceFxPreviewImageNode { kind },
                            ));
                        }
                    });
                });
            }

            spawn_preview(
                row,
                theme,
                "Source",
                DiceFxPreviewImageKind::Source,
                blank.clone(),
                140.0,
                90.0,
            );
            spawn_preview(
                row,
                theme,
                "Mask",
                DiceFxPreviewImageKind::Mask,
                blank.clone(),
                140.0,
                90.0,
            );
            spawn_preview(
                row,
                theme,
                "Noise",
                DiceFxPreviewImageKind::Noise,
                blank.clone(),
                140.0,
                90.0,
            );
            spawn_preview(
                row,
                theme,
                "Ramp",
                DiceFxPreviewImageKind::Ramp,
                blank.clone(),
                140.0,
                42.0,
            );
        });

    // Time scrubber for previewing curve impact over duration.
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Text::new("Preview time (0..duration)"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));

            col.spawn((
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
                DiceFxPreviewTimeLabel,
            ));

            col.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                ..default()
            })
            .with_children(|slot| {
                let slider = MaterialSlider::new(0.0, 1.0)
                    .with_value(settings_state.dice_fx_preview_time_t.clamp(0.0, 1.0))
                    .track_height(6.0)
                    .thumb_radius(8.0);
                spawn_slider_control_with(slot, theme, slider, DiceFxPreviewTimeSlider);
            });
        });

    // Trigger select
    parent.spawn(Node::default()).with_children(|slot| {
        let trigger_options = vec![
            SelectOption::new("Total ≥ value").value("total_at_least"),
            SelectOption::new("All dice are max").value("all_max"),
            SelectOption::new("Any die equals value").value("any_die_equals"),
        ];

        let selected = match settings_state.editing_custom_dice_fx.trigger_kind {
            crate::dice3d::types::CustomDiceFxTriggerKind::TotalAtLeast => 0,
            crate::dice3d::types::CustomDiceFxTriggerKind::AllMax => 1,
            crate::dice3d::types::CustomDiceFxTriggerKind::AnyDieEquals => 2,
        };

        let builder = SelectBuilder::new(trigger_options)
            .outlined()
            .label("Custom dice effect trigger")
            .selected(selected)
            .width(Val::Percent(100.0));
        slot.spawn_select_with(theme, builder);
    });

    // Trigger value + duration
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
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
        });

    // Graph
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                height: Val::Px(220.0),
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

                    // 50% line
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

                    // 0% midline
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

                    // -50% line (maps to 0.25 in 0..1 curve space)
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
                    ] {
                        let color = match channel {
                            DiceFxCurveChannel::Mask => theme.primary,
                            DiceFxCurveChannel::Noise => theme.secondary,
                            DiceFxCurveChannel::Ramp => theme.tertiary,
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
}
