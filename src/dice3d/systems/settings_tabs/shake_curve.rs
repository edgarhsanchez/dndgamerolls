use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_material_ui::prelude::*;
use bevy_material_ui::text_field::spawn_text_field_control_with;

use crate::dice3d::types::{
    ContainerShakeConfig, SettingsState, ShakeAxis, ShakeCurveAxisChip, ShakeCurveEditMode,
    ShakeCurveEditModeChip, ShakeCurveGraphDot, ShakeCurveGraphPlotRoot, ShakeCurveGraphRoot,
    ShakeCurvePointHandle, ShakeDurationTextInput,
};

fn spawn_filter_chip_in<M: Component>(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    label: &str,
    selected: bool,
    value: &str,
    marker: M,
) {
    parent
        .spawn(
            ChipBuilder::filter(label)
                .selected(selected)
                .value(value)
                .build(theme),
        )
        .insert(marker)
        .with_children(|chip| {
            // Minimal content: label only. Chip style systems will recolor it.
            chip.spawn((
                ChipLabel,
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));
        });
}

pub fn build_shake_curve_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    settings_state: &SettingsState,
    shake_config: &ContainerShakeConfig,
) {
    parent.spawn((
        Text::new("Curve (full shake: start → finish). Choose Add/Delete, then click."),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    // Duration input
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|slot| {
            let builder = TextFieldBuilder::new()
                .outlined()
                .label("Shake duration (seconds)")
                .value(format!("{:.3}", shake_config.duration_seconds.max(0.0)))
                .supporting_text("Total time from t=0 to t=1")
                .width(Val::Percent(100.0));
            spawn_text_field_control_with(slot, theme, builder, ShakeDurationTextInput);
        });

    // Mode + axis chips
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(8.0),
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            // Add/Delete mode (mutually exclusive via systems)
            spawn_filter_chip_in(
                row,
                theme,
                "Add",
                settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Add,
                "shake_curve_mode_add",
                ShakeCurveEditModeChip {
                    mode: ShakeCurveEditMode::Add,
                },
            );

            spawn_filter_chip_in(
                row,
                theme,
                "Delete",
                settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete,
                "shake_curve_mode_delete",
                ShakeCurveEditModeChip {
                    mode: ShakeCurveEditMode::Delete,
                },
            );

            // Axis selection for Add/select (also helps disambiguate overlapping points)
            spawn_filter_chip_in(
                row,
                theme,
                "X",
                settings_state.shake_curve_add_x,
                "shake_axis_x",
                ShakeCurveAxisChip { axis: ShakeAxis::X },
            );

            spawn_filter_chip_in(
                row,
                theme,
                "Y",
                settings_state.shake_curve_add_y,
                "shake_axis_y",
                ShakeCurveAxisChip { axis: ShakeAxis::Y },
            );

            spawn_filter_chip_in(
                row,
                theme,
                "Z",
                settings_state.shake_curve_add_z,
                "shake_axis_z",
                ShakeCurveAxisChip { axis: ShakeAxis::Z },
            );
        });

    // Graph area
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                height: Val::Px(240.0),
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme.surface_container_highest),
            BorderColor::from(theme.outline_variant),
            BorderRadius::all(Val::Px(8.0)),
            ShakeCurveGraphRoot,
        ))
        .with_children(|graph| {
            // Inner plot area (inset from the border) so the curve/handles never touch edges.
            // All dots/handles are parented under this node.
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
                    ShakeCurveGraphPlotRoot,
                ))
                .with_children(|plot| {
                    // Grid lines to make the +/- range clearer (values are normalized -1..+1).
                    // These are intentionally subtle and sit behind the curve dots/handles.
                    let grid_color = theme.outline_variant.with_alpha(0.35);
                    let half_grid_color = theme.outline_variant.with_alpha(0.22);

                    // +100% line (slightly inset so max values are fully visible)
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

                    // +50% line
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

                    // -50% line
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

                    // -100% line (slightly inset so max values are fully visible)
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

                    // Sample dots for the curve line
                    const DOTS: usize = 80;
                    for axis in [ShakeAxis::X, ShakeAxis::Y, ShakeAxis::Z] {
                        let color = match axis {
                            ShakeAxis::X => theme.primary,
                            ShakeAxis::Y => theme.secondary,
                            ShakeAxis::Z => theme.tertiary,
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
                                ShakeCurveGraphDot { axis, index: i },
                            ));
                        }
                    }

                    // Draggable handles for existing points
                    for p in shake_config
                        .curve_points_x
                        .iter()
                        .chain(shake_config.curve_points_y.iter())
                        .chain(shake_config.curve_points_z.iter())
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
                            ShakeCurvePointHandle { id: p.id },
                        ));
                    }
                });
        });
}
