//! Settings UI systems
//!
//! This module contains systems for the settings button, modal, and color picker.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::{ComputedUiTargetCamera, UiGlobalTransform};

use bevy::window::PrimaryWindow;
use bevy_material_ui::prelude::*;
use std::cmp::Ordering;

use crate::dice3d::types::*;
use bevy_material_ui::prelude::SwitchChangeEvent;

use super::settings_tabs;

const SETTINGS_DIALOG_WIDTH: f32 = 780.0;
const SETTINGS_DIALOG_HEIGHT: f32 = 720.0;

/// Persist settings changes to SQLite.
///
/// Many UI interactions update settings continuously (dragging panels, curve edits).
/// Instead of writing on every change, systems should set `SettingsState.is_modified = true`.
/// This system flushes once per frame.
pub fn persist_settings_to_db(
    mut settings_state: ResMut<SettingsState>,
    db: Option<Res<CharacterDatabase>>,
) {
    if !settings_state.is_modified {
        return;
    }

    let Some(db) = db else {
        return;
    };

    match settings_state.settings.save_to_db(&db) {
        Ok(()) => settings_state.is_modified = false,
        Err(e) => warn!("Failed to persist settings to SQLite: {}", e),
    }
}

/// Spawn the settings (gear) icon button in the dice roller view.
pub fn spawn_settings_button(
    commands: &mut Commands,
    theme: &MaterialTheme,
    icon_font: Handle<Font>,
) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                // Keep below the app-level tab bar which sits at top=0.
                top: Val::Px(TAB_HEIGHT_SECONDARY + 12.0),
                width: Val::Px(ICON_BUTTON_SIZE),
                height: Val::Px(ICON_BUTTON_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(200),
            DiceRollerRoot,
        ))
        .with_children(|slot| {
            slot.spawn((
                IconButtonBuilder::new("settings").standard().build(theme),
                TooltipTrigger::new("Settings").top(),
                SettingsButton,
            ))
            .with_children(|b| {
                let icon = MaterialIcon::from_name("settings").unwrap_or_else(MaterialIcon::search);
                b.spawn((
                    Text::new(icon.as_str()),
                    TextFont {
                        font: icon_font,
                        font_size: ICON_SIZE,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });
        })
        .id()
}

fn spawn_settings_modal(
    commands: &mut Commands,
    theme: &MaterialTheme,
    settings_state: &SettingsState,
    shake_config: &ContainerShakeConfig,
) {
    let options = [
        DiceTypeSetting::D4,
        DiceTypeSetting::D6,
        DiceTypeSetting::D8,
        DiceTypeSetting::D10,
        DiceTypeSetting::D12,
        DiceTypeSetting::D20,
    ];

    let selected_index = options
        .iter()
        .position(|d| *d == settings_state.quick_roll_editing_die)
        .unwrap_or(5);

    let select_options: Vec<SelectOption> = options
        .iter()
        .map(|d| SelectOption::new(d.label()).value(d.label()))
        .collect();

    // Custom dialog bundle so we can enforce a fixed size (DialogBuilder clamps width).
    let dialog = MaterialDialog::new()
        .title("Settings")
        .open(true)
        .modal(true);

    let dialog_bg = dialog.surface_color(theme);

    let dialog_entity = commands
        .spawn((
            dialog,
            Node {
                display: Display::None, // synced by DialogPlugin
                position_type: PositionType::Absolute,
                width: Val::Px(SETTINGS_DIALOG_WIDTH),
                height: Val::Px(SETTINGS_DIALOG_HEIGHT),
                min_width: Val::Px(SETTINGS_DIALOG_WIDTH),
                max_width: Val::Px(SETTINGS_DIALOG_WIDTH),
                min_height: Val::Px(SETTINGS_DIALOG_HEIGHT),
                max_height: Val::Px(SETTINGS_DIALOG_HEIGHT),
                padding: UiRect::all(Val::Px(Spacing::EXTRA_LARGE)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(dialog_bg),
            BorderRadius::all(Val::Px(CornerRadius::EXTRA_LARGE)),
            BoxShadow::default(),
            SettingsModal,
        ))
        .id();

    let scrim_entity = commands
        .spawn((
            create_dialog_scrim_for(theme, dialog_entity, true),
            SettingsModalOverlay,
        ))
        .id();

    commands.entity(scrim_entity).add_child(dialog_entity);

    commands.entity(dialog_entity).with_children(|dialog| {
        dialog
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|content| {
                let editing_color = &settings_state.editing_color;
                let editing_highlight_color = &settings_state.editing_highlight_color;

                content.spawn((
                    Text::new("Settings"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                ));

                // Tabs: Dice / Colors / Shake Curve / Layout
                let mut tabs_cmd = content.spawn((
                    MaterialTabs::new()
                        .with_variant(TabVariant::Secondary)
                        .selected(0),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(TAB_HEIGHT_SECONDARY),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    BackgroundColor(theme.surface),
                ));
                let tabs_entity = tabs_cmd.id();
                tabs_cmd.with_children(|tabs| {
                    fn spawn_tab_label(
                        t: &mut ChildSpawnerCommands,
                        theme: &MaterialTheme,
                        label: &str,
                    ) {
                        t.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));
                    }

                    tabs.spawn((
                        MaterialTab::new(0, "Dice").selected(true),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Dice"));

                    tabs.spawn((
                        MaterialTab::new(1, "Colors"),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Colors"));

                    tabs.spawn((
                        MaterialTab::new(2, "Shake Curve"),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Shake Curve"));

                    tabs.spawn((
                        MaterialTab::new(3, "Layout"),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Layout"));
                });

                // Scrollable content area. Each tab is a scroll container.
                content
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(0.0),
                        overflow: Overflow::clip(),
                        ..default()
                    })
                    .with_children(|tab_area| {
                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            0,
                            true,
                            |tab| {
                                settings_tabs::dice::build_dice_tab(
                                    tab,
                                    theme,
                                    select_options.clone(),
                                    selected_index,
                                    settings_state.default_roll_uses_shake_editing,
                                );
                            },
                        );

                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            1,
                            false,
                            |tab| {
                                settings_tabs::colors::build_colors_tab(
                                    tab,
                                    theme,
                                    editing_color,
                                    editing_highlight_color,
                                );
                            },
                        );

                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            2,
                            false,
                            |tab| {
                                settings_tabs::shake_curve::build_shake_curve_tab(
                                    tab,
                                    theme,
                                    settings_state,
                                    shake_config,
                                );
                            },
                        );

                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            3,
                            false,
                            |tab| {
                                settings_tabs::layout::build_layout_tab(tab, theme);
                            },
                        );
                    });

                // Buttons row
                content
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(10.0),
                        width: Val::Percent(100.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        // Cancel button
                        buttons
                            .spawn(Node {
                                width: Val::Px(100.0),
                                height: Val::Px(36.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                slot.spawn((
                                    MaterialButtonBuilder::new("Cancel").outlined().build(theme),
                                    SettingsCancelButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Cancel"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(theme.primary),
                                        ButtonLabel,
                                    ));
                                });
                            });

                        // OK button
                        buttons
                            .spawn(Node {
                                width: Val::Px(100.0),
                                height: Val::Px(36.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                slot.spawn((
                                    MaterialButtonBuilder::new("OK").filled().build(theme),
                                    SettingsOkButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("OK"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(theme.on_primary),
                                        ButtonLabel,
                                    ));
                                });
                            });
                    });
            });
    });
}

/// Helper to spawn a color slider row
pub(crate) fn spawn_color_slider(
    parent: &mut ChildSpawnerCommands,
    component: ColorComponent,
    label: &str,
    value: f32,
    track_color: Color,
    theme: &MaterialTheme,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            height: Val::Px(30.0),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(track_color),
            ));

            // Slider
            row.spawn(Node {
                width: Val::Px(180.0),
                height: Val::Px(30.0),
                ..default()
            })
            .with_children(|slot| {
                let slider = MaterialSlider::new(0.0, 1.0)
                    .with_value(value)
                    .track_height(6.0)
                    .thumb_radius(8.0);
                spawn_slider_control_with(slot, theme, slider, ColorSlider { component });
            });

            // Value label
            row.spawn((
                Text::new(format!("{:.2}", value)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
                ColorValueLabel { component },
            ));
        });
}

// ============================================================================
// Interaction Systems
// ============================================================================

/// Handle settings button click
pub fn handle_settings_button_click(
    mut click_events: MessageReader<IconButtonClickEvent>,
    button_query: Query<(), With<SettingsButton>>,
    mut settings_state: ResMut<SettingsState>,
    shake_config: Res<ContainerShakeConfig>,
) {
    for event in click_events.read() {
        if button_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.show_modal = true;
        settings_state.modal_kind = crate::dice3d::types::ActiveModalKind::DiceRollerSettings;
        settings_state.editing_color = settings_state.settings.background_color.clone();
        settings_state.color_input_text = settings_state.editing_color.to_hex();

        settings_state.editing_highlight_color =
            settings_state.settings.dice_box_highlight_color.clone();
        settings_state.highlight_input_text = settings_state.editing_highlight_color.to_hex();

        settings_state.quick_roll_editing_die = settings_state.settings.quick_roll_default_die;
        settings_state.default_roll_uses_shake_editing =
            settings_state.settings.default_roll_uses_shake;

        // Copy current shake settings into an editable staging area.
        settings_state.editing_shake_config = shake_config.clone();

        // Keep autosave snapshot aligned so opening the modal doesn't immediately rewrite.
        settings_state.last_saved_shake_config_json =
            serde_json::to_string(&settings_state.settings.shake_config).unwrap_or_default();

        settings_state.shake_duration_input_text = format!(
            "{:.3}",
            settings_state
                .editing_shake_config
                .duration_seconds
                .max(0.0)
        );
    }
}

/// Persist shake curve changes immediately (every edit) and apply them to runtime.
///
/// The curve editor stores values in normalized percentages (value in [-1..1]).
/// At shake time, the runtime converts to actual movement using:
///   offset = (curve_value * shake_config.distance) * strength
pub fn autosave_and_apply_shake_config(
    mut settings_state: ResMut<SettingsState>,
    mut shake_config: ResMut<ContainerShakeConfig>,
) {
    if !settings_state.show_modal
        || settings_state.modal_kind != crate::dice3d::types::ActiveModalKind::DiceRollerSettings
    {
        return;
    }

    // Convert the editing config into the persisted representation.
    let persisted = ShakeConfigSetting::from_runtime(&settings_state.editing_shake_config);
    let Ok(json) = serde_json::to_string(&persisted) else {
        return;
    };

    if json == settings_state.last_saved_shake_config_json {
        return;
    }

    settings_state.settings.shake_config = persisted;
    settings_state.last_saved_shake_config_json = json;

    // Apply to runtime immediately so the shake feature uses the latest curve without
    // requiring an explicit OK click.
    *shake_config = settings_state.editing_shake_config.clone();

    settings_state.is_modified = true;
}

/// Apply persisted shake config on startup.
pub fn apply_initial_shake_config(
    settings_state: Res<SettingsState>,
    mut shake_config: ResMut<ContainerShakeConfig>,
) {
    *shake_config = settings_state.settings.shake_config.to_runtime();
}

/// Spawn/despawn settings modal based on state
pub fn manage_settings_modal(
    mut commands: Commands,
    settings_state: Res<SettingsState>,
    theme: Res<MaterialTheme>,
    modal_query: Query<Entity, With<SettingsModalOverlay>>,
    children_query: Query<&Children>,
) {
    if !settings_state.is_changed() {
        return;
    }

    if settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings
    {
        // Spawn modal if not exists
        if modal_query.is_empty() {
            spawn_settings_modal(
                &mut commands,
                &theme,
                &settings_state,
                &settings_state.editing_shake_config,
            );
        }
    } else {
        // Despawn modal
        for entity in modal_query.iter() {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Handle OK button click
pub fn handle_settings_ok_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    ok_query: Query<(), With<SettingsOkButton>>,
    mut settings_state: ResMut<SettingsState>,
    mut clear_color: ResMut<ClearColor>,
    mut shake_config: ResMut<ContainerShakeConfig>,
) {
    for event in click_events.read() {
        if ok_query.get(event.entity).is_err() {
            continue;
        }
        // Apply the editing colors
        settings_state.settings.background_color = settings_state.editing_color.clone();
        settings_state.settings.dice_box_highlight_color =
            settings_state.editing_highlight_color.clone();

        settings_state.settings.quick_roll_default_die = settings_state.quick_roll_editing_die;

        settings_state.settings.default_roll_uses_shake =
            settings_state.default_roll_uses_shake_editing;

        // Update the clear color
        clear_color.0 = settings_state.settings.background_color.to_color();

        // Apply shake settings from the editor
        *shake_config = settings_state.editing_shake_config.clone();

        settings_state.is_modified = true;

        // Close modal
        settings_state.show_modal = false;
        settings_state.modal_kind = crate::dice3d::types::ActiveModalKind::None;
    }
}

/// Handle switch changes in the dice roller settings modal.
pub fn handle_default_roll_uses_shake_switch_change(
    mut events: MessageReader<SwitchChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
    switch_query: Query<(), With<DefaultRollUsesShakeSwitch>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in events.read() {
        if switch_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.default_roll_uses_shake_editing = event.selected;
    }
}

/// Handle selection changes in the dice roller settings modal (Quick Rolls die).
pub fn handle_quick_roll_die_type_select_change(
    mut events: MessageReader<SelectChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    let options = [
        DiceTypeSetting::D4,
        DiceTypeSetting::D6,
        DiceTypeSetting::D8,
        DiceTypeSetting::D10,
        DiceTypeSetting::D12,
        DiceTypeSetting::D20,
    ];

    for event in events.read() {
        if let Some(setting) = options.get(event.index).copied() {
            settings_state.quick_roll_editing_die = setting;
        }
    }
}

fn sort_curve_points(points: &mut [ShakeCurvePoint]) {
    points.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(Ordering::Equal));
}

fn find_curve_point_index(points: &[ShakeCurvePoint], id: u64) -> Option<usize> {
    points.iter().position(|p| p.id == id)
}

fn curve_points(cfg: &ContainerShakeConfig, axis: ShakeAxis) -> &[ShakeCurvePoint] {
    match axis {
        ShakeAxis::X => &cfg.curve_points_x,
        ShakeAxis::Y => &cfg.curve_points_y,
        ShakeAxis::Z => &cfg.curve_points_z,
    }
}

fn curve_points_mut(cfg: &mut ContainerShakeConfig, axis: ShakeAxis) -> &mut Vec<ShakeCurvePoint> {
    match axis {
        ShakeAxis::X => &mut cfg.curve_points_x,
        ShakeAxis::Y => &mut cfg.curve_points_y,
        ShakeAxis::Z => &mut cfg.curve_points_z,
    }
}

fn find_curve_point_axis(cfg: &ContainerShakeConfig, id: u64) -> Option<ShakeAxis> {
    if cfg.curve_points_x.iter().any(|p| p.id == id) {
        return Some(ShakeAxis::X);
    }
    if cfg.curve_points_y.iter().any(|p| p.id == id) {
        return Some(ShakeAxis::Y);
    }
    if cfg.curve_points_z.iter().any(|p| p.id == id) {
        return Some(ShakeAxis::Z);
    }
    None
}

fn remove_curve_point_by_id(cfg: &mut ContainerShakeConfig, id: u64) -> bool {
    let Some(axis) = find_curve_point_axis(cfg, id) else {
        return false;
    };
    let points = curve_points_mut(cfg, axis);
    if points.len() <= 2 {
        return false;
    }
    if let Some(i) = find_curve_point_index(points, id) {
        points.remove(i);
        return true;
    }
    false
}

fn window_physical_cursor_to_ui_node_local_logical_px(
    cursor_in_ui_target_physical: Vec2,
    transform: &UiGlobalTransform,
    computed: &ComputedNode,
) -> Option<Vec2> {
    // Hit-testing uses physical pixels, but UI layout values (Val::Px) are in logical UI pixels.
    // Convert the physical-local point into logical-local using the node's inverse scale factor.
    let inv = transform.try_inverse()?;
    let size_physical = computed.size();
    let local_physical = inv.transform_point2(cursor_in_ui_target_physical) + size_physical * 0.5;
    Some(local_physical * computed.inverse_scale_factor())
}

fn ui_target_cursor_physical_px(window: &Window, ui_camera: Option<&Camera>) -> Option<Vec2> {
    let cursor_window = window.physical_cursor_position()?;
    let viewport_min = ui_camera
        .and_then(|c| c.physical_viewport_rect().map(|r| r.min.as_vec2()))
        .unwrap_or(Vec2::ZERO);
    Some(cursor_window - viewport_min)
}

fn shake_curve_t_v_to_local_px(graph_size: Vec2, t: f32, v: f32) -> Option<Vec2> {
    if graph_size.x <= 1.0 || graph_size.y <= 1.0 {
        return None;
    }

    // Keep a small inset so +/-100% is fully visible with point/handle sizes.
    const EDGE_PAD_PX: f32 = 7.0;
    let w = (graph_size.x - 2.0 * EDGE_PAD_PX).max(1.0);
    let h = (graph_size.y - 2.0 * EDGE_PAD_PX).max(1.0);

    let t = t.clamp(0.0, 1.0);
    let v = v.clamp(-1.0, 1.0);
    let x = EDGE_PAD_PX + t * w;
    let y = EDGE_PAD_PX + ((1.0 - v) * 0.5) * h;
    Some(Vec2::new(x, y))
}

fn graph_local_px_to_t_v(graph_size: Vec2, local: Vec2) -> Option<(f32, f32)> {
    if graph_size.x <= 1.0 || graph_size.y <= 1.0 {
        return None;
    }

    const EDGE_PAD_PX: f32 = 7.0;
    let w = (graph_size.x - 2.0 * EDGE_PAD_PX).max(1.0);
    let h = (graph_size.y - 2.0 * EDGE_PAD_PX).max(1.0);

    let lx = (local.x - EDGE_PAD_PX).clamp(0.0, w);
    let ly = (local.y - EDGE_PAD_PX).clamp(0.0, h);

    let t: f32 = (lx / w).clamp(0.0, 1.0);
    let y01: f32 = (ly / h).clamp(0.0, 1.0);
    let v: f32 = (1.0 - 2.0 * y01).clamp(-1.0, 1.0);
    Some((t, v))
}

fn add_curve_point(cfg: &mut ContainerShakeConfig, axis: ShakeAxis, t: f32, value: f32) -> u64 {
    let new_id = cfg.next_curve_point_id;
    cfg.next_curve_point_id += 1;
    curve_points_mut(cfg, axis).push(ShakeCurvePoint {
        id: new_id,
        t: t.clamp(0.0, 1.0),
        value: value.clamp(-1.0, 1.0),
        in_handle: None,
        out_handle: None,
    });
    sort_curve_points(curve_points_mut(cfg, axis));
    new_id
}

fn axis_enabled(settings_state: &SettingsState, axis: ShakeAxis) -> bool {
    match axis {
        ShakeAxis::X => settings_state.shake_curve_add_x,
        ShakeAxis::Y => settings_state.shake_curve_add_y,
        ShakeAxis::Z => settings_state.shake_curve_add_z,
    }
}

fn find_nearest_curve_point_id(
    cfg: &ContainerShakeConfig,
    graph_size: Vec2,
    cursor_local: Vec2,
    settings_state: &SettingsState,
    threshold_px: f32,
) -> Option<u64> {
    let mut best: Option<(u64, f32, u8)> = None;

    // Prefer deterministic axis ordering when points overlap.
    let axis_rank = |axis: ShakeAxis| match axis {
        ShakeAxis::X => 0u8,
        ShakeAxis::Y => 1u8,
        ShakeAxis::Z => 2u8,
    };

    for axis in [ShakeAxis::X, ShakeAxis::Y, ShakeAxis::Z] {
        if !axis_enabled(settings_state, axis) {
            continue;
        }

        for p in curve_points(cfg, axis) {
            let Some(pos) = shake_curve_t_v_to_local_px(graph_size, p.t, p.value) else {
                continue;
            };
            let d = cursor_local.distance(pos);
            if d > threshold_px {
                continue;
            }

            let rank = axis_rank(axis);
            match best {
                None => best = Some((p.id, d, rank)),
                Some((_, best_d, best_rank)) => {
                    if d < best_d || (d == best_d && rank < best_rank) {
                        best = Some((p.id, d, rank));
                    }
                }
            }
        }
    }

    best.map(|(id, _, _)| id)
}

/// Handle chip clicks for shake curve edit mode and axis selection.
pub fn handle_shake_curve_chip_clicks(
    mut click_events: MessageReader<ChipClickEvent>,
    edit_mode_chips: Query<&ShakeCurveEditModeChip>,
    axis_chips: Query<&ShakeCurveAxisChip>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !settings_state.show_modal {
        return;
    }

    for ev in click_events.read() {
        if let Ok(chip) = edit_mode_chips.get(ev.entity) {
            settings_state.shake_curve_edit_mode =
                if settings_state.shake_curve_edit_mode == chip.mode {
                    ShakeCurveEditMode::None
                } else {
                    chip.mode
                };

            // Delete mode cancels any in-progress drag.
            if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
                settings_state.dragging_shake_curve_point_id = None;
                settings_state.dragging_shake_curve_bezier = None;
            }

            continue;
        }

        if let Ok(chip) = axis_chips.get(ev.entity) {
            let axis = chip.axis;
            match chip.axis {
                ShakeAxis::X => {
                    settings_state.shake_curve_add_x = !settings_state.shake_curve_add_x
                }
                ShakeAxis::Y => {
                    settings_state.shake_curve_add_y = !settings_state.shake_curve_add_y
                }
                ShakeAxis::Z => {
                    settings_state.shake_curve_add_z = !settings_state.shake_curve_add_z
                }
            }

            // Never allow the user to disable all axes; that makes the editor feel broken
            // (no points are selectable and Add/Delete may appear to do nothing).
            if !settings_state.shake_curve_add_x
                && !settings_state.shake_curve_add_y
                && !settings_state.shake_curve_add_z
            {
                match axis {
                    ShakeAxis::X => settings_state.shake_curve_add_x = true,
                    ShakeAxis::Y => settings_state.shake_curve_add_y = true,
                    ShakeAxis::Z => settings_state.shake_curve_add_z = true,
                }
            }

            // If the currently-selected point is on a now-disabled axis, deselect it.
            if let Some(selected_id) = settings_state.selected_shake_curve_point_id {
                if let Some(axis) =
                    find_curve_point_axis(&settings_state.editing_shake_config, selected_id)
                {
                    if !axis_enabled(&settings_state, axis) {
                        settings_state.selected_shake_curve_point_id = None;
                        if settings_state.dragging_shake_curve_point_id == Some(selected_id) {
                            settings_state.dragging_shake_curve_point_id = None;
                        }
                        settings_state.dragging_shake_curve_bezier = None;
                    }
                }
            }
        }
    }
}

/// Keep chip selected state in sync with `SettingsState`.
pub fn sync_shake_curve_chip_ui(
    settings_state: Res<SettingsState>,
    mut edit_mode_chips: Query<
        (&ShakeCurveEditModeChip, &mut MaterialChip),
        Without<ShakeCurveAxisChip>,
    >,
    mut axis_chips: Query<
        (&ShakeCurveAxisChip, &mut MaterialChip),
        Without<ShakeCurveEditModeChip>,
    >,
) {
    if !settings_state.show_modal {
        return;
    }

    if !settings_state.is_changed() {
        return;
    }

    for (chip, mut material) in edit_mode_chips.iter_mut() {
        material.selected = settings_state.shake_curve_edit_mode == chip.mode;
    }

    for (chip, mut material) in axis_chips.iter_mut() {
        material.selected = match chip.axis {
            ShakeAxis::X => settings_state.shake_curve_add_x,
            ShakeAxis::Y => settings_state.shake_curve_add_y,
            ShakeAxis::Z => settings_state.shake_curve_add_z,
        };
    }
}

/// Click the graph background to add a point anywhere.
pub fn handle_shake_curve_graph_click_to_add_point(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<ShakeCurveGraphPlotRoot>,
    >,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let mode = settings_state.shake_curve_edit_mode;

    // If we're already dragging something, don't also process a background click.
    if settings_state.dragging_shake_curve_point_id.is_some()
        || settings_state.dragging_shake_curve_bezier.is_some()
    {
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let window = windows.iter().next();

    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    // Use window physical cursor mapping (robust under DPI scaling).
    let cursor_local = if let Some(window) = window {
        let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
        cursor_in_ui_target
            .and_then(|c| window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed))
    } else {
        None
    };

    let Some(cursor_local) = cursor_local else {
        return;
    };

    // Require the click be inside the plot rect.
    if cursor_local.x < 0.0
        || cursor_local.y < 0.0
        || cursor_local.x > size.x
        || cursor_local.y > size.y
    {
        return;
    }

    let Some((t, v)) = graph_local_px_to_t_v(size, cursor_local) else {
        return;
    };

    // Read chip toggles before mut-borrowing the config.
    let add_x = settings_state.shake_curve_add_x;
    let add_y = settings_state.shake_curve_add_y;
    let add_z = settings_state.shake_curve_add_z;

    match mode {
        ShakeCurveEditMode::Add => {
            // In Add mode, clicking near an existing point selects/drags it instead of creating a new one.
            if let Some(id) = find_nearest_curve_point_id(
                &settings_state.editing_shake_config,
                size,
                cursor_local,
                &settings_state,
                16.0,
            ) {
                settings_state.selected_shake_curve_point_id = Some(id);
                settings_state.dragging_shake_curve_point_id = Some(id);
                return;
            }

            let new_selected: Option<u64> = {
                let cfg = &mut settings_state.editing_shake_config;
                let mut new_selected: Option<u64> = None;
                if add_x {
                    new_selected = Some(add_curve_point(cfg, ShakeAxis::X, t, v));
                }
                if add_y {
                    new_selected = Some(add_curve_point(cfg, ShakeAxis::Y, t, v));
                }
                if add_z {
                    new_selected = Some(add_curve_point(cfg, ShakeAxis::Z, t, v));
                }
                new_selected
            };

            if let Some(id) = new_selected {
                settings_state.selected_shake_curve_point_id = Some(id);
                settings_state.dragging_shake_curve_point_id = Some(id);
            }
        }
        ShakeCurveEditMode::Delete => {
            // Background click deletes nearest point handle.
            let removed_id: Option<u64> = {
                let cfg = &mut settings_state.editing_shake_config;
                let mut best: Option<(u64, f32)> = None;
                let consider = |best: &mut Option<(u64, f32)>, id: u64, dist: f32| match best {
                    None => *best = Some((id, dist)),
                    Some((_, best_dist)) if dist < *best_dist => *best = Some((id, dist)),
                    _ => {}
                };

                // Threshold in px from point center.
                let threshold = 22.0_f32;
                for axis in [ShakeAxis::X, ShakeAxis::Y, ShakeAxis::Z] {
                    let axis_on = match axis {
                        ShakeAxis::X => add_x,
                        ShakeAxis::Y => add_y,
                        ShakeAxis::Z => add_z,
                    };
                    if !axis_on {
                        continue;
                    }
                    for p in curve_points(cfg, axis) {
                        let Some(pos) = shake_curve_t_v_to_local_px(size, p.t, p.value) else {
                            continue;
                        };
                        let d = cursor_local.distance(pos);
                        if d <= threshold {
                            consider(&mut best, p.id, d);
                        }
                    }
                }

                if let Some((id, _)) = best {
                    if remove_curve_point_by_id(cfg, id) {
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(id) = removed_id {
                if settings_state.selected_shake_curve_point_id == Some(id) {
                    settings_state.selected_shake_curve_point_id = None;
                }
                if settings_state.dragging_shake_curve_point_id == Some(id) {
                    settings_state.dragging_shake_curve_point_id = None;
                }
            }
        }
        ShakeCurveEditMode::None => {
            // Select/drag if clicking near a point; otherwise deselect.
            if let Some(id) = find_nearest_curve_point_id(
                &settings_state.editing_shake_config,
                size,
                cursor_local,
                &settings_state,
                16.0,
            ) {
                settings_state.selected_shake_curve_point_id = Some(id);
                settings_state.dragging_shake_curve_point_id = Some(id);
            } else {
                settings_state.selected_shake_curve_point_id = None;
                settings_state.dragging_shake_curve_point_id = None;
                settings_state.dragging_shake_curve_bezier = None;
            }
        }
    }
}

/// Start dragging a Bezier handle when it is pressed.
pub fn handle_shake_curve_bezier_handle_press(
    mut settings_state: ResMut<SettingsState>,
    mut interactions: Query<(&Interaction, &ShakeCurveBezierHandle), Changed<Interaction>>,
) {
    if !settings_state.show_modal {
        return;
    }

    if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    for (interaction, handle) in interactions.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        settings_state.selected_shake_curve_point_id = Some(handle.point_id);
        settings_state.dragging_shake_curve_point_id = None;
        settings_state.dragging_shake_curve_bezier = Some((handle.point_id, handle.kind));
    }
}

/// Drag a Bezier handle for the currently-selected point.
pub fn drag_shake_curve_bezier_handle(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<ShakeCurveGraphPlotRoot>,
    >,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }
    if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        settings_state.dragging_shake_curve_bezier = None;
        return;
    }

    let Some((point_id, kind)) = settings_state.dragging_shake_curve_bezier else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        settings_state.dragging_shake_curve_bezier = None;
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };

    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
    let local = cursor_in_ui_target
        .and_then(|c| window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed));
    let Some(local) = local else {
        return;
    };
    let Some((t, v)) = graph_local_px_to_t_v(size, local) else {
        return;
    };

    let cfg = &mut settings_state.editing_shake_config;
    let Some(axis) = find_curve_point_axis(cfg, point_id) else {
        return;
    };
    let points = curve_points_mut(cfg, axis);
    sort_curve_points(points);
    let Some(i) = find_curve_point_index(points, point_id) else {
        return;
    };

    let pt_t = points[i].t;
    let prev_t = if i > 0 { points[i - 1].t } else { pt_t };
    let next_t = if i + 1 < points.len() {
        points[i + 1].t
    } else {
        pt_t
    };

    let v = v.clamp(-1.0, 1.0);
    let handle_t = match kind {
        ShakeCurveBezierHandleKind::In => t.clamp(prev_t.min(pt_t), pt_t.max(prev_t)),
        ShakeCurveBezierHandleKind::Out => t.clamp(pt_t.min(next_t), next_t.max(pt_t)),
    };

    let handle_pos = Vec2::new(handle_t, v);
    match kind {
        ShakeCurveBezierHandleKind::In => points[i].in_handle = Some(handle_pos),
        ShakeCurveBezierHandleKind::Out => points[i].out_handle = Some(handle_pos),
    }
}

/// Start selecting/dragging curve points when their handle is pressed.
pub fn handle_shake_curve_point_press(
    mut settings_state: ResMut<SettingsState>,
    mut interactions: Query<(&Interaction, &ShakeCurvePointHandle), Changed<Interaction>>,
    graph: Query<&Node, With<ShakeCurveGraphPlotRoot>>,
) {
    if !settings_state.show_modal {
        return;
    }

    // Only active when the graph is visible.
    let Some(node) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    for (interaction, handle) in interactions.iter_mut() {
        if *interaction == Interaction::Pressed {
            if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
                // Deletion is handled on press in delete mode.
                let id = handle.id;
                let removed = {
                    let cfg = &mut settings_state.editing_shake_config;
                    remove_curve_point_by_id(cfg, id)
                };
                if removed {
                    if settings_state.selected_shake_curve_point_id == Some(id) {
                        settings_state.selected_shake_curve_point_id = None;
                    }
                    if settings_state.dragging_shake_curve_point_id == Some(id) {
                        settings_state.dragging_shake_curve_point_id = None;
                    }
                }
            } else {
                settings_state.selected_shake_curve_point_id = Some(handle.id);
                settings_state.dragging_shake_curve_point_id = Some(handle.id);
                settings_state.dragging_shake_curve_bezier = None;
            }
        }
    }
}

/// Drag the selected curve point within the graph bounds.
pub fn drag_shake_curve_point(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<ShakeCurveGraphPlotRoot>,
    >,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }

    if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };

    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    if mouse.just_released(MouseButton::Left) {
        settings_state.dragging_shake_curve_point_id = None;
        return;
    }

    let Some(drag_id) = settings_state.dragging_shake_curve_point_id else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        settings_state.dragging_shake_curve_point_id = None;
        return;
    }

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
    let local = cursor_in_ui_target
        .and_then(|c| window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed));
    let Some(local) = local else {
        return;
    };
    let Some((t, v)) = graph_local_px_to_t_v(size, local) else {
        return;
    };

    let cfg = &mut settings_state.editing_shake_config;
    if let Some(axis) = find_curve_point_axis(cfg, drag_id) {
        let points = curve_points_mut(cfg, axis);
        if let Some(i) = find_curve_point_index(points, drag_id) {
            let old_t = points[i].t;
            let old_v = points[i].value;
            points[i].t = t;
            points[i].value = v;

            // Move existing handles along with the point so they stay attached.
            let dt = points[i].t - old_t;
            let dv = points[i].value - old_v;
            if let Some(h) = points[i].in_handle {
                points[i].in_handle = Some(Vec2::new(
                    (h.x + dt).clamp(0.0, 1.0),
                    (h.y + dv).clamp(-1.0, 1.0),
                ));
            }
            if let Some(h) = points[i].out_handle {
                points[i].out_handle = Some(Vec2::new(
                    (h.x + dt).clamp(0.0, 1.0),
                    (h.y + dv).clamp(-1.0, 1.0),
                ));
            }
            sort_curve_points(points);
        }
    }
}

fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let omt = 1.0 - u;
    (omt * omt * omt) * p0
        + (3.0 * omt * omt * u) * p1
        + (3.0 * omt * u * u) * p2
        + (u * u * u) * p3
}

fn cubic_bezier_derivative(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    // d/du of cubic bezier
    let omt = 1.0 - u;
    3.0 * omt * omt * (p1 - p0) + 6.0 * omt * u * (p2 - p1) + 3.0 * u * u * (p3 - p2)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn sample_curve(points: &[ShakeCurvePoint], t: f32) -> f32 {
    if points.is_empty() {
        return 0.0;
    }
    if points.len() == 1 {
        return points[0].value;
    }

    // Non-looping start->finish curve.
    let t = t.clamp(0.0, 1.0);
    let mut points_sorted: std::borrow::Cow<'_, [ShakeCurvePoint]> =
        std::borrow::Cow::Borrowed(points);
    // If points are not sorted (should be), sort a copy.
    if !points.windows(2).all(|w| w[0].t <= w[1].t) {
        let mut tmp = points.to_vec();
        tmp.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(Ordering::Equal));
        points_sorted = std::borrow::Cow::Owned(tmp);
    }
    let points = points_sorted.as_ref();

    if t <= points[0].t {
        return points[0].value;
    }
    if t >= points[points.len() - 1].t {
        return points[points.len() - 1].value;
    }

    for w in points.windows(2) {
        let a = w[0];
        let b = w[1];
        if t >= a.t && t <= b.t {
            let dt = (b.t - a.t).max(0.0001);
            let initial_u = ((t - a.t) / dt).clamp(0.0, 1.0);

            // Resolve control points.
            let mut p1 = a
                .out_handle
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 1.0 / 3.0), a.value));
            let mut p2 = b
                .in_handle
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 2.0 / 3.0), b.value));

            // Clamp handle x within the segment to keep x(u) monotonic-ish.
            p1.x = p1.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p2.x = p2.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p1.y = p1.y.clamp(-1.0, 1.0);
            p2.y = p2.y.clamp(-1.0, 1.0);

            // Invert x(u) to find u such that x(u) == t.
            let mut u = initial_u;
            for _ in 0..8 {
                let x = cubic_bezier(a.t, p1.x, p2.x, b.t, u);
                let dx = cubic_bezier_derivative(a.t, p1.x, p2.x, b.t, u);
                if dx.abs() < 1e-5 {
                    break;
                }
                u = (u - (x - t) / dx).clamp(0.0, 1.0);
            }

            return cubic_bezier(a.value, p1.y, p2.y, b.value, u);
        }
    }

    points[0].value
}

/// Keep graph dots and point handles positioned to match the current curve.
pub fn sync_shake_curve_graph_ui(
    mut commands: Commands,
    theme: Res<MaterialTheme>,
    settings_state: Res<SettingsState>,
    graph: Query<(Entity, &ComputedNode), With<ShakeCurveGraphPlotRoot>>,
    mut dots: Query<
        (&ShakeCurveGraphDot, &mut Node),
        (
            Without<ShakeCurvePointHandle>,
            Without<ShakeCurveBezierHandle>,
        ),
    >,
    mut handles: Query<
        (
            Entity,
            &ShakeCurvePointHandle,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Without<ShakeCurveGraphDot>, Without<ShakeCurveBezierHandle>),
    >,
    mut bezier_handles: Query<
        (
            Entity,
            &ShakeCurveBezierHandle,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Without<ShakeCurvePointHandle>, Without<ShakeCurveGraphDot>),
    >,
    graph_children: Query<&Children>,
) {
    if !settings_state.show_modal {
        return;
    }

    let Some((graph_entity, computed)) = graph.iter().next() else {
        return;
    };

    // UI layout uses logical pixels (Val::Px). `ComputedNode::size()` is in physical pixels.
    // Convert once so point placement math matches cursor mapping under DPI scaling.
    let size_physical = computed.size();
    let size = size_physical * computed.inverse_scale_factor();
    if size.x <= 1.0 || size.y <= 1.0 {
        return;
    }
    let cfg = &settings_state.editing_shake_config;

    // Ensure handle entities exist for each curve point id; remove extras.
    let mut existing_ids: std::collections::HashMap<u64, Entity> = std::collections::HashMap::new();
    for (e, h, _node, _bg, _border) in handles.iter_mut() {
        existing_ids.insert(h.id, e);
    }

    let mut desired_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for axis in [ShakeAxis::X, ShakeAxis::Y, ShakeAxis::Z] {
        for p in curve_points(cfg, axis) {
            desired_ids.insert(p.id);
            if !existing_ids.contains_key(&p.id) {
                // spawn missing
                commands.entity(graph_entity).with_children(|graph| {
                    graph.spawn((
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
                });
            }
        }
    }

    for (e, h, _node, _bg, _border) in handles.iter_mut() {
        if !desired_ids.contains(&h.id) {
            commands.entity(e).despawn();
        }
    }

    // Update handle positions/colors.
    let selected = settings_state.selected_shake_curve_point_id;
    for (_e, h, mut node, mut bg, mut border) in handles.iter_mut() {
        let Some(axis) = find_curve_point_axis(cfg, h.id) else {
            continue;
        };

        // Axis chips control which points are selectable; when disabled, hide the handle hitbox.
        if !axis_enabled(&settings_state, axis) {
            node.display = Display::None;
            continue;
        }
        node.display = Display::Flex;

        let axis_color = match axis {
            ShakeAxis::X => theme.primary,
            ShakeAxis::Y => theme.secondary,
            ShakeAxis::Z => theme.tertiary,
        };
        *border = BorderColor::all(axis_color);

        let Some(p) = curve_points(cfg, axis).iter().find(|p| p.id == h.id) else {
            continue;
        };
        let Some(pos) = shake_curve_t_v_to_local_px(size, p.t, p.value) else {
            continue;
        };
        node.left = Val::Px((pos.x - 7.0).clamp(0.0, (size.x - 14.0).max(0.0)));
        node.top = Val::Px((pos.y - 7.0).clamp(0.0, (size.y - 14.0).max(0.0)));
        *bg = if selected == Some(h.id) {
            BackgroundColor(axis_color)
        } else {
            BackgroundColor(theme.surface_container_high)
        };
    }

    // Spawn/despawn and position Bezier handles for the selected point.
    {
        use std::collections::{HashMap, HashSet};

        // Index existing Bezier handles.
        let mut existing: HashMap<(u64, ShakeCurveBezierHandleKind), Entity> = HashMap::new();
        for (e, h, _node, _bg, _border) in bezier_handles.iter_mut() {
            existing.insert((h.point_id, h.kind), e);
        }

        let mut desired: HashSet<(u64, ShakeCurveBezierHandleKind)> = HashSet::new();
        if let Some(sel_id) = selected {
            if let Some(axis) = find_curve_point_axis(cfg, sel_id) {
                let pts = curve_points(cfg, axis);
                if let Some(i) = pts.iter().position(|p| p.id == sel_id) {
                    if i > 0 {
                        desired.insert((sel_id, ShakeCurveBezierHandleKind::In));
                    }
                    if i + 1 < pts.len() {
                        desired.insert((sel_id, ShakeCurveBezierHandleKind::Out));
                    }

                    // Spawn any missing handle entities.
                    let axis_color = match axis {
                        ShakeAxis::X => theme.primary,
                        ShakeAxis::Y => theme.secondary,
                        ShakeAxis::Z => theme.tertiary,
                    };

                    for (pid, kind) in desired.iter().copied() {
                        if existing.contains_key(&(pid, kind)) {
                            continue;
                        }

                        commands.entity(graph_entity).with_children(|graph| {
                            graph.spawn((
                                Button,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    top: Val::Px(0.0),
                                    width: Val::Px(10.0),
                                    height: Val::Px(10.0),
                                    ..default()
                                },
                                BackgroundColor(theme.surface_container_high),
                                BorderRadius::all(Val::Px(5.0)),
                                BorderColor::all(axis_color),
                                Interaction::None,
                                ShakeCurveBezierHandle {
                                    point_id: pid,
                                    kind,
                                },
                            ));
                        });
                    }
                }
            }
        }

        // Despawn any stale Bezier handles.
        for (e, h, _node, _bg, _border) in bezier_handles.iter_mut() {
            if !desired.contains(&(h.point_id, h.kind)) {
                commands.entity(e).despawn();
            }
        }

        // Position Bezier handles that remain.
        if let Some(sel_id) = selected {
            if let Some(axis) = find_curve_point_axis(cfg, sel_id) {
                let pts = curve_points(cfg, axis);
                if let Some(i) = pts.iter().position(|p| p.id == sel_id) {
                    let p = pts[i];
                    let prev = if i > 0 { Some(pts[i - 1]) } else { None };
                    let next = if i + 1 < pts.len() {
                        Some(pts[i + 1])
                    } else {
                        None
                    };

                    let axis_color = match axis {
                        ShakeAxis::X => theme.primary,
                        ShakeAxis::Y => theme.secondary,
                        ShakeAxis::Z => theme.tertiary,
                    };

                    let default_in = prev.map(|a| {
                        let dt = (p.t - a.t).abs() * 0.25;
                        Vec2::new((p.t - dt).clamp(a.t.min(p.t), a.t.max(p.t)), p.value)
                    });
                    let default_out = next.map(|b| {
                        let dt = (b.t - p.t).abs() * 0.25;
                        Vec2::new((p.t + dt).clamp(p.t.min(b.t), p.t.max(b.t)), p.value)
                    });

                    for (_e, h, mut node, mut bg, mut border) in bezier_handles.iter_mut() {
                        if h.point_id != sel_id {
                            continue;
                        }

                        let handle_pos = match h.kind {
                            ShakeCurveBezierHandleKind::In => p
                                .in_handle
                                .or(default_in)
                                .unwrap_or(Vec2::new(p.t, p.value)),
                            ShakeCurveBezierHandleKind::Out => p
                                .out_handle
                                .or(default_out)
                                .unwrap_or(Vec2::new(p.t, p.value)),
                        };

                        let Some(pos) =
                            shake_curve_t_v_to_local_px(size, handle_pos.x, handle_pos.y)
                        else {
                            continue;
                        };
                        node.left = Val::Px((pos.x - 5.0).clamp(0.0, (size.x - 10.0).max(0.0)));
                        node.top = Val::Px((pos.y - 5.0).clamp(0.0, (size.y - 10.0).max(0.0)));

                        *border = BorderColor::all(axis_color);
                        *bg = BackgroundColor(theme.surface_container_high);
                    }
                }
            }
        }
    }

    for (dot, mut node) in dots.iter_mut() {
        // DOTS count is fixed at spawn time.
        let n = 80usize;
        let t = (dot.index as f32) / (n.saturating_sub(1) as f32).max(1.0);
        let v = sample_curve(curve_points(cfg, dot.axis), t).clamp(-1.0, 1.0);
        let Some(pos) = shake_curve_t_v_to_local_px(size, t, v) else {
            continue;
        };
        node.left = Val::Px((pos.x - 1.5).clamp(0.0, (size.x - 3.0).max(0.0)));
        node.top = Val::Px((pos.y - 1.5).clamp(0.0, (size.y - 3.0).max(0.0)));
    }

    // Ensure dots/handles are still children (in case of reparent issues).
    let _ = graph_children.get(graph_entity);
}

/// Handle Cancel button click
pub fn handle_settings_cancel_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    cancel_query: Query<(), With<SettingsCancelButton>>,
    mut settings_state: ResMut<SettingsState>,
) {
    for event in click_events.read() {
        if cancel_query.get(event.entity).is_err() {
            continue;
        }

        // Discard changes and close modal
        settings_state.show_modal = false;
        settings_state.modal_kind = crate::dice3d::types::ActiveModalKind::None;
    }
}

/// Reset panel layout to a predictable side-by-side arrangement under the results panel.
pub fn handle_settings_reset_layout_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    reset_query: Query<(), With<SettingsResetLayoutButton>>,
    mut settings_state: ResMut<SettingsState>,
    mut panel_nodes: ParamSet<(
        Query<&mut Node, With<crate::dice3d::types::SliderGroupRoot>>,
        Query<&mut Node, With<crate::dice3d::types::QuickRollPanel>>,
        Query<&mut Node, With<crate::dice3d::types::CommandHistoryPanelRoot>>,
        Query<&mut Node, With<crate::dice3d::types::ResultsPanelRoot>>,
    )>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in click_events.read() {
        if reset_query.get(event.entity).is_err() {
            continue;
        }

        // Layout: results stays top-left; other panels go below it side-by-side.
        // Note: y=230 is chosen to be "after results" with typical font sizes.
        settings_state.settings.results_panel_position.x = 10.0;
        settings_state.settings.results_panel_position.y = 50.0;

        let y = 230.0;
        settings_state.settings.slider_group_position.x = 10.0;
        settings_state.settings.slider_group_position.y = y;

        settings_state.settings.command_history_panel_position.x = 132.0;
        settings_state.settings.command_history_panel_position.y = y;

        settings_state.settings.quick_roll_panel_position.x = 342.0;
        settings_state.settings.quick_roll_panel_position.y = y;

        // Immediately move any spawned panels so the user sees it.
        if let Some(mut node) = panel_nodes.p3().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.results_panel_position.x);
            node.top = Val::Px(settings_state.settings.results_panel_position.y);
        }
        if let Some(mut node) = panel_nodes.p0().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.slider_group_position.x);
            node.top = Val::Px(settings_state.settings.slider_group_position.y);
        }
        if let Some(mut node) = panel_nodes.p2().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.command_history_panel_position.x);
            node.top = Val::Px(settings_state.settings.command_history_panel_position.y);
        }
        if let Some(mut node) = panel_nodes.p1().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.quick_roll_panel_position.x);
            node.top = Val::Px(settings_state.settings.quick_roll_panel_position.y);
        }

        settings_state.is_modified = true;
    }
}

/// Handle RGBA slider changes
pub fn handle_color_slider_changes(
    mut events: MessageReader<SliderChangeEvent>,
    slider_query: Query<&ColorSlider>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !settings_state.show_modal {
        return;
    }

    for event in events.read() {
        let Ok(slider) = slider_query.get(event.entity) else {
            continue;
        };

        let value = event.value.clamp(0.0, 1.0);
        match slider.component {
            ColorComponent::Alpha => settings_state.editing_color.a = value,
            ColorComponent::Red => settings_state.editing_color.r = value,
            ColorComponent::Green => settings_state.editing_color.g = value,
            ColorComponent::Blue => settings_state.editing_color.b = value,
        }

        settings_state.color_input_text = settings_state.editing_color.to_hex();
    }
}

/// Update color preview and slider handles when editing color changes
pub fn update_color_ui(
    settings_state: Res<SettingsState>,
    mut preview_queries: ParamSet<(
        Query<&mut BackgroundColor, With<ColorPreview>>,
        Query<&mut BackgroundColor, With<HighlightColorPreview>>,
    )>,
    mut slider_query: Query<(&ColorSlider, &mut MaterialSlider)>,
    mut label_query: Query<(&ColorValueLabel, &mut Text)>,
    mut input_queries: ParamSet<(
        Query<&mut MaterialTextField, With<ColorTextInput>>,
        Query<&mut MaterialTextField, With<HighlightColorTextInput>>,
    )>,
) {
    if !settings_state.is_changed() {
        return;
    }

    if !settings_state.show_modal {
        return;
    }

    let color = &settings_state.editing_color;
    let highlight_color = &settings_state.editing_highlight_color;

    // Update preview
    for mut bg in preview_queries.p0().iter_mut() {
        bg.0 = color.to_color();
    }

    for mut bg in preview_queries.p1().iter_mut() {
        bg.0 = highlight_color.to_color();
    }

    // Sync slider values
    for (slider, mut material_slider) in slider_query.iter_mut() {
        material_slider.value = match slider.component {
            ColorComponent::Alpha => color.a,
            ColorComponent::Red => color.r,
            ColorComponent::Green => color.g,
            ColorComponent::Blue => color.b,
        };
    }

    // Update value labels
    for (label, mut text) in label_query.iter_mut() {
        let value = match label.component {
            ColorComponent::Alpha => color.a,
            ColorComponent::Red => color.r,
            ColorComponent::Green => color.g,
            ColorComponent::Blue => color.b,
        };
        **text = format!("{:.2}", value);
    }

    // Sync text field value (avoid stomping while the user is typing)
    for mut field in input_queries.p0().iter_mut() {
        if field.focused {
            continue;
        }

        field.value = settings_state.color_input_text.clone();
        field.has_content = !field.value.is_empty();
        if ColorSetting::parse(&field.value).is_some() {
            field.error = false;
            field.error_text = None;
        } else {
            field.error = true;
            field.error_text = Some("Invalid color format".to_string());
        }
        field.supporting_text =
            Some("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2".to_string());
    }

    // Sync highlight text field value (avoid stomping while the user is typing)
    for mut field in input_queries.p1().iter_mut() {
        if field.focused {
            continue;
        }

        field.value = settings_state.highlight_input_text.clone();
        field.has_content = !field.value.is_empty();
        if ColorSetting::parse(&field.value).is_some() {
            field.error = false;
            field.error_text = None;
        } else {
            field.error = true;
            field.error_text = Some("Invalid color format".to_string());
        }
        field.supporting_text =
            Some("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2".to_string());
    }
}

/// Handle keyboard input for color text field
pub fn handle_color_text_input(
    mut settings_state: ResMut<SettingsState>,
    mut change_events: MessageReader<TextFieldChangeEvent>,
    mut submit_events: MessageReader<TextFieldSubmitEvent>,
    mut field_queries: ParamSet<(
        Query<&mut MaterialTextField, With<ColorTextInput>>,
        Query<&mut MaterialTextField, With<HighlightColorTextInput>>,
    )>,
) {
    if !settings_state.show_modal {
        return;
    }

    // Live update from the text field
    for ev in change_events.read() {
        if let Ok(mut field) = field_queries.p0().get_mut(ev.entity) {
            settings_state.color_input_text = ev.value.clone();

            if let Some(parsed) = ColorSetting::parse(&settings_state.color_input_text) {
                settings_state.editing_color = parsed;
                field.error = false;
                field.error_text = None;
                field.supporting_text =
                    Some("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2".to_string());
            } else {
                field.error = true;
                field.error_text = Some("Invalid color format".to_string());
            }

            continue;
        }

        if let Ok(mut field) = field_queries.p1().get_mut(ev.entity) {
            settings_state.highlight_input_text = ev.value.clone();

            if let Some(parsed) = ColorSetting::parse(&settings_state.highlight_input_text) {
                settings_state.editing_highlight_color = parsed;
                field.error = false;
                field.error_text = None;
                field.supporting_text =
                    Some("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2".to_string());
            } else {
                field.error = true;
                field.error_text = Some("Invalid color format".to_string());
            }
        }
    }

    // On submit (Enter), normalize to canonical hex if valid
    for ev in submit_events.read() {
        if let Ok(mut field) = field_queries.p0().get_mut(ev.entity) {
            if let Some(parsed) = ColorSetting::parse(&ev.value) {
                let canonical_hex = parsed.to_hex();
                settings_state.editing_color = parsed;
                settings_state.color_input_text = canonical_hex;

                field.value = settings_state.color_input_text.clone();
                field.has_content = !field.value.is_empty();
                field.error = false;
                field.error_text = None;
                field.supporting_text =
                    Some("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2".to_string());
            } else {
                field.error = true;
                field.error_text = Some("Invalid color format".to_string());
            }

            continue;
        }

        if let Ok(mut field) = field_queries.p1().get_mut(ev.entity) {
            if let Some(parsed) = ColorSetting::parse(&ev.value) {
                let canonical_hex = parsed.to_hex();
                settings_state.editing_highlight_color = parsed;
                settings_state.highlight_input_text = canonical_hex;

                field.value = settings_state.highlight_input_text.clone();
                field.has_content = !field.value.is_empty();
                field.error = false;
                field.error_text = None;
                field.supporting_text =
                    Some("#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2".to_string());
            } else {
                field.error = true;
                field.error_text = Some("Invalid color format".to_string());
            }
        }
    }
}

/// Handle keyboard input for the shake duration text field.
pub fn handle_shake_duration_text_input(
    mut settings_state: ResMut<SettingsState>,
    mut change_events: MessageReader<TextFieldChangeEvent>,
    mut submit_events: MessageReader<TextFieldSubmitEvent>,
    mut field_query: Query<&mut MaterialTextField, With<ShakeDurationTextInput>>,
) {
    if !settings_state.show_modal {
        return;
    }

    for ev in change_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };

        settings_state.shake_duration_input_text = ev.value.clone();

        let parsed = ev.value.trim().parse::<f32>();
        if let Ok(mut seconds) = parsed {
            if seconds.is_finite() && seconds > 0.0 {
                seconds = seconds.max(0.01);
                settings_state.editing_shake_config.duration_seconds = seconds;
                field.error = false;
                field.error_text = None;
            } else {
                field.error = true;
                field.error_text = Some("Enter a positive number".to_string());
            }
        } else {
            field.error = true;
            field.error_text = Some("Enter a number".to_string());
        }
    }

    for ev in submit_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };

        let parsed = ev.value.trim().parse::<f32>();
        if let Ok(mut seconds) = parsed {
            if seconds.is_finite() && seconds > 0.0 {
                seconds = seconds.max(0.01);
                settings_state.editing_shake_config.duration_seconds = seconds;
                settings_state.shake_duration_input_text = format!("{:.3}", seconds);

                field.value = settings_state.shake_duration_input_text.clone();
                field.has_content = !field.value.is_empty();
                field.error = false;
                field.error_text = None;
            } else {
                field.error = true;
                field.error_text = Some("Enter a positive number".to_string());
            }
        } else {
            field.error = true;
            field.error_text = Some("Enter a number".to_string());
        }
    }
}

/// Apply settings on startup
pub fn apply_initial_settings(
    settings_state: Res<SettingsState>,
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = settings_state.settings.background_color.to_color();
}
