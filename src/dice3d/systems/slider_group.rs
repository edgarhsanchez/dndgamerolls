//! Draggable slider group UI (dice view)
//!
//! Wraps the zoom + strength sliders into a single draggable panel and persists
//! its location via `SettingsState.settings`.

use bevy::prelude::*;

use crate::dice3d::types::{
    AppTab, AppTabBar, CommandHistoryPanelDragState, CommandHistoryPanelHandle,
    CommandHistoryPanelRoot, QuickRollPanel, QuickRollPanelDragState, QuickRollPanelHandle,
    ResultsPanelDragState, ResultsPanelHandle, ResultsPanelRoot, SettingsState, SliderGroupDragState,
    SliderGroupHandle, SliderGroupRoot, UiState,
    DiceBoxControlsPanelDragState, DiceBoxControlsPanelHandle, DiceBoxControlsPanelRoot,
};

#[derive(Clone, Copy, Debug, Default)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn left(&self) -> f32 {
        self.x
    }

    fn right(&self) -> f32 {
        self.x + self.w
    }

    fn top(&self) -> f32 {
        self.y
    }

    fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

fn save_slider_group_position(settings_state: &mut SettingsState, x: f32, y: f32) {
    let slot = &mut settings_state.settings.slider_group_position;
    if (slot.x - x).abs() < 0.5 && (slot.y - y).abs() < 0.5 {
        return;
    }

    slot.x = x;
    slot.y = y;

    if let Err(e) = settings_state.settings.save() {
        eprintln!("Failed to save settings: {e}");
    }
}

fn save_quick_roll_position(settings_state: &mut SettingsState, x: f32, y: f32) {
    let slot = &mut settings_state.settings.quick_roll_panel_position;
    if (slot.x - x).abs() < 0.5 && (slot.y - y).abs() < 0.5 {
        return;
    }

    slot.x = x;
    slot.y = y;

    if let Err(e) = settings_state.settings.save() {
        eprintln!("Failed to save settings: {e}");
    }
}

fn save_command_history_position(settings_state: &mut SettingsState, x: f32, y: f32) {
    let slot = &mut settings_state.settings.command_history_panel_position;
    if (slot.x - x).abs() < 0.5 && (slot.y - y).abs() < 0.5 {
        return;
    }

    slot.x = x;
    slot.y = y;

    if let Err(e) = settings_state.settings.save() {
        eprintln!("Failed to save settings: {e}");
    }
}

fn save_results_panel_position(settings_state: &mut SettingsState, x: f32, y: f32) {
    let slot = &mut settings_state.settings.results_panel_position;
    if (slot.x - x).abs() < 0.5 && (slot.y - y).abs() < 0.5 {
        return;
    }

    slot.x = x;
    slot.y = y;

    if let Err(e) = settings_state.settings.save() {
        eprintln!("Failed to save settings: {e}");
    }
}

fn save_dice_box_controls_panel_position(settings_state: &mut SettingsState, x: f32, y: f32) {
    let slot = &mut settings_state.settings.dice_box_controls_panel_position;
    if (slot.x - x).abs() < 0.5 && (slot.y - y).abs() < 0.5 {
        return;
    }

    slot.x = x;
    slot.y = y;

    if let Err(e) = settings_state.settings.save() {
        eprintln!("Failed to save settings: {e}");
    }
}

/// Drag the slider group around the screen by grabbing the handle.
///
/// - Only active on the Dice Roller tab
/// - Disabled while the settings modal is open
/// - Auto-saves panel positions to `settings.json` while dragging
pub fn handle_slider_group_drag(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    ui_state: Res<UiState>,
    mut settings_state: ResMut<SettingsState>,
    app_tab_bar: Query<&ComputedNode, With<AppTabBar>>,
    slider_handle_interaction: Query<(&Interaction, &ChildOf), (With<SliderGroupHandle>, Changed<Interaction>)>,
    quick_handle_interaction: Query<(&Interaction, &ChildOf), (With<QuickRollPanelHandle>, Changed<Interaction>)>,
    history_handle_interaction: Query<(&Interaction, &ChildOf), (With<CommandHistoryPanelHandle>, Changed<Interaction>)>,
    results_handle_interaction: Query<(&Interaction, &ChildOf), (With<ResultsPanelHandle>, Changed<Interaction>)>,
    dice_box_controls_handle_interaction: Query<(&Interaction, &ChildOf), (With<DiceBoxControlsPanelHandle>, Changed<Interaction>)>,
    slider_group_size: Query<&ComputedNode, With<SliderGroupRoot>>,
    quick_roll_size: Query<&ComputedNode, With<QuickRollPanel>>,
    command_history_size: Query<&ComputedNode, With<CommandHistoryPanelRoot>>,
    results_panel_size: Query<&ComputedNode, With<ResultsPanelRoot>>,
    dice_box_controls_size: Query<&ComputedNode, With<DiceBoxControlsPanelRoot>>,
    mut panel_queries: ParamSet<(
        Query<(&mut Node, &mut SliderGroupDragState, &ComputedNode), With<SliderGroupRoot>>,
        Query<(&mut Node, &mut QuickRollPanelDragState, &ComputedNode), With<QuickRollPanel>>,
        Query<(&mut Node, &mut CommandHistoryPanelDragState, &ComputedNode), With<CommandHistoryPanelRoot>>,
        Query<(&mut Node, &mut ResultsPanelDragState, &ComputedNode), With<ResultsPanelRoot>>,
        Query<(&mut Node, &mut DiceBoxControlsPanelDragState, &ComputedNode), With<DiceBoxControlsPanelRoot>>,
    )>,
) {
    if ui_state.active_tab != AppTab::DiceRoller {
        return;
    }

    // Modal dialog open: don't allow moving the panel.
    if settings_state.show_modal {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let tab_bar_height = app_tab_bar
        .iter()
        .next()
        .map(|c| c.size().y.max(0.0))
        .unwrap_or(48.0);

    let snap_px: f32 = 10.0;

    let rects_overlap = |a0: f32, a1: f32, b0: f32, b1: f32| {
        let left = a0.max(b0);
        let right = a1.min(b1);
        (right - left) > 1.0
    };

    let apply_snapping = |mut x: f32,
                          mut y: f32,
                          w: f32,
                          h: f32,
                          others: &[Rect],
                          win_w: f32,
                          win_h: f32,
                          min_y: f32| {
        // Window edge snapping ------------------------------------------------
        let left_edge = 0.0;
        let right_edge = (win_w - w).max(0.0);
        let top_edge = min_y;
        let bottom_edge = (win_h - h).max(min_y);

        if (x - left_edge).abs() <= snap_px {
            x = left_edge;
        }
        if (x - right_edge).abs() <= snap_px {
            x = right_edge;
        }
        if (y - top_edge).abs() <= snap_px {
            y = top_edge;
        }
        if (y - bottom_edge).abs() <= snap_px {
            y = bottom_edge;
        }

        // Snap next to / align with other panels -----------------------------
        for other in others {
            let moving_left = x;
            let moving_top = y;
            let moving_bottom = y + h;

            // Horizontal snapping: require some vertical overlap
            if rects_overlap(moving_top, moving_bottom, other.top(), other.bottom()) {
                // Align left edges
                if (moving_left - other.left()).abs() <= snap_px {
                    x = other.left();
                }
                // Align right edges
                if ((x + w) - other.right()).abs() <= snap_px {
                    x = other.right() - w;
                }
                // Place to the right of other
                if (x - other.right()).abs() <= snap_px {
                    x = other.right();
                }
                // Place to the left of other
                if ((x + w) - other.left()).abs() <= snap_px {
                    x = other.left() - w;
                }
            }

            // Vertical snapping: require some horizontal overlap
            if rects_overlap(x, x + w, other.left(), other.right()) {
                // Align top edges
                if (moving_top - other.top()).abs() <= snap_px {
                    y = other.top();
                }
                // Align bottom edges
                if ((y + h) - other.bottom()).abs() <= snap_px {
                    y = other.bottom() - h;
                }
                // Place below other
                if (y - other.bottom()).abs() <= snap_px {
                    y = other.bottom();
                }
                // Place above other
                if ((y + h) - other.top()).abs() <= snap_px {
                    y = other.top() - h;
                }
            }
        }

        (x, y)
    };

    let clamp_to_window = |x: f32, y: f32, computed: &ComputedNode| {
        let win_w = window.resolution.width();
        let win_h = window.resolution.height();
        let panel_w = computed.size().x.max(1.0);
        let panel_h = computed.size().y.max(1.0);

        let clamped_x = x.clamp(0.0, (win_w - panel_w).max(0.0));
        let clamped_y = y.clamp(tab_bar_height, (win_h - panel_h).max(tab_bar_height));

        (clamped_x, clamped_y)
    };

    // Results panel --------------------------------------------------------
    {
        let mut results_panel_query = panel_queries.p3();

        // Start dragging when the handle is pressed.
        for (interaction, child_of) in results_handle_interaction.iter() {
            if *interaction != Interaction::Pressed {
                continue;
            }

            if !mouse.just_pressed(MouseButton::Left) {
                continue;
            }

            let parent_entity = child_of.parent();
            let Ok((node, mut drag_state, _computed)) =
                results_panel_query.get_mut(parent_entity)
            else {
                continue;
            };

            let current_x = match node.left {
                Val::Px(px) => px,
                _ => 0.0,
            };
            let current_y = match node.top {
                Val::Px(px) => px,
                _ => 0.0,
            };

            drag_state.dragging = true;
            drag_state.grab_offset = Vec2::new(
                cursor_position.x - current_x,
                cursor_position.y - current_y,
            );
        }

        // Update position while dragging.
        for (mut node, mut drag_state, computed) in results_panel_query.iter_mut() {
            if !drag_state.dragging {
                continue;
            }

            if mouse.just_released(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            if !mouse.pressed(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            let mut new_x = cursor_position.x - drag_state.grab_offset.x;
            let mut new_y = cursor_position.y - drag_state.grab_offset.y;

            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            let mut other_rects = Vec::with_capacity(3);
            if let Some(c) = slider_group_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.slider_group_position.x,
                    y: settings_state.settings.slider_group_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = quick_roll_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.quick_roll_panel_position.x,
                    y: settings_state.settings.quick_roll_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = command_history_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.command_history_panel_position.x,
                    y: settings_state.settings.command_history_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = dice_box_controls_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.dice_box_controls_panel_position.x,
                    y: settings_state.settings.dice_box_controls_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }

            (new_x, new_y) = apply_snapping(
                new_x,
                new_y,
                computed.size().x.max(1.0),
                computed.size().y.max(1.0),
                &other_rects,
                window.resolution.width(),
                window.resolution.height(),
                tab_bar_height,
            );
            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            node.left = Val::Px(new_x);
            node.top = Val::Px(new_y);

            save_results_panel_position(&mut settings_state, new_x, new_y);
        }
    }

    // Dice box controls panel ---------------------------------------------
    {
        let mut controls_panel_query = panel_queries.p4();

        // Start dragging when the handle is pressed.
        for (interaction, child_of) in dice_box_controls_handle_interaction.iter() {
            if *interaction != Interaction::Pressed {
                continue;
            }

            if !mouse.just_pressed(MouseButton::Left) {
                continue;
            }

            let parent_entity = child_of.parent();
            let Ok((node, mut drag_state, _computed)) =
                controls_panel_query.get_mut(parent_entity)
            else {
                continue;
            };

            let current_x = match node.left {
                Val::Px(px) => px,
                _ => 0.0,
            };
            let current_y = match node.top {
                Val::Px(px) => px,
                _ => 0.0,
            };

            drag_state.dragging = true;
            drag_state.grab_offset = Vec2::new(
                cursor_position.x - current_x,
                cursor_position.y - current_y,
            );
        }

        // Update position while dragging.
        for (mut node, mut drag_state, computed) in controls_panel_query.iter_mut() {
            if !drag_state.dragging {
                continue;
            }

            if mouse.just_released(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            if !mouse.pressed(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            let mut new_x = cursor_position.x - drag_state.grab_offset.x;
            let mut new_y = cursor_position.y - drag_state.grab_offset.y;

            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            let mut other_rects = Vec::with_capacity(4);
            if let Some(c) = slider_group_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.slider_group_position.x,
                    y: settings_state.settings.slider_group_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = quick_roll_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.quick_roll_panel_position.x,
                    y: settings_state.settings.quick_roll_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = command_history_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.command_history_panel_position.x,
                    y: settings_state.settings.command_history_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = results_panel_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.results_panel_position.x,
                    y: settings_state.settings.results_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }

            (new_x, new_y) = apply_snapping(
                new_x,
                new_y,
                computed.size().x.max(1.0),
                computed.size().y.max(1.0),
                &other_rects,
                window.resolution.width(),
                window.resolution.height(),
                tab_bar_height,
            );
            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            node.left = Val::Px(new_x);
            node.top = Val::Px(new_y);

            save_dice_box_controls_panel_position(&mut settings_state, new_x, new_y);
        }
    }

    // Slider group panel ----------------------------------------------------
    {
        let mut slider_group_query = panel_queries.p0();

        // Start dragging when the handle is pressed.
        for (interaction, child_of) in slider_handle_interaction.iter() {
            if *interaction != Interaction::Pressed {
                continue;
            }

            if !mouse.just_pressed(MouseButton::Left) {
                continue;
            }

            let parent_entity = child_of.parent();
            let Ok((node, mut drag_state, _computed)) =
                slider_group_query.get_mut(parent_entity)
            else {
                continue;
            };

            let current_x = match node.left {
                Val::Px(px) => px,
                _ => 0.0,
            };
            let current_y = match node.top {
                Val::Px(px) => px,
                _ => 0.0,
            };

            drag_state.dragging = true;
            drag_state.grab_offset = Vec2::new(
                cursor_position.x - current_x,
                cursor_position.y - current_y,
            );
        }

        // Update position while dragging.
        for (mut node, mut drag_state, computed) in slider_group_query.iter_mut() {
            if !drag_state.dragging {
                continue;
            }

            if mouse.just_released(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            if !mouse.pressed(MouseButton::Left) {
                // Safety: if focus is lost, stop dragging.
                drag_state.dragging = false;
                continue;
            }

            let mut new_x = cursor_position.x - drag_state.grab_offset.x;
            let mut new_y = cursor_position.y - drag_state.grab_offset.y;

            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            let mut other_rects = Vec::with_capacity(4);
            if let Some(c) = quick_roll_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.quick_roll_panel_position.x,
                    y: settings_state.settings.quick_roll_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = command_history_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.command_history_panel_position.x,
                    y: settings_state.settings.command_history_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = results_panel_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.results_panel_position.x,
                    y: settings_state.settings.results_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = dice_box_controls_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.dice_box_controls_panel_position.x,
                    y: settings_state.settings.dice_box_controls_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }

            (new_x, new_y) = apply_snapping(
                new_x,
                new_y,
                computed.size().x.max(1.0),
                computed.size().y.max(1.0),
                &other_rects,
                window.resolution.width(),
                window.resolution.height(),
                tab_bar_height,
            );
            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            node.left = Val::Px(new_x);
            node.top = Val::Px(new_y);

            save_slider_group_position(&mut settings_state, new_x, new_y);
        }
    }

    // Quick Rolls panel -----------------------------------------------------
    {
        let mut quick_panel_query = panel_queries.p1();

        // Start dragging when the handle is pressed.
        for (interaction, child_of) in quick_handle_interaction.iter() {
            if *interaction != Interaction::Pressed {
                continue;
            }

            if !mouse.just_pressed(MouseButton::Left) {
                continue;
            }

            let parent_entity = child_of.parent();
            let Ok((node, mut drag_state, _computed)) =
                quick_panel_query.get_mut(parent_entity)
            else {
                continue;
            };

            let current_x = match node.left {
                Val::Px(px) => px,
                _ => 0.0,
            };
            let current_y = match node.top {
                Val::Px(px) => px,
                _ => 0.0,
            };

            drag_state.dragging = true;
            drag_state.grab_offset = Vec2::new(
                cursor_position.x - current_x,
                cursor_position.y - current_y,
            );
        }

        // Update position while dragging.
        for (mut node, mut drag_state, computed) in quick_panel_query.iter_mut() {
            if !drag_state.dragging {
                continue;
            }

            if mouse.just_released(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            if !mouse.pressed(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            let mut new_x = cursor_position.x - drag_state.grab_offset.x;
            let mut new_y = cursor_position.y - drag_state.grab_offset.y;

            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            let mut other_rects = Vec::with_capacity(4);
            if let Some(c) = slider_group_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.slider_group_position.x,
                    y: settings_state.settings.slider_group_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = command_history_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.command_history_panel_position.x,
                    y: settings_state.settings.command_history_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = results_panel_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.results_panel_position.x,
                    y: settings_state.settings.results_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = dice_box_controls_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.dice_box_controls_panel_position.x,
                    y: settings_state.settings.dice_box_controls_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }

            (new_x, new_y) = apply_snapping(
                new_x,
                new_y,
                computed.size().x.max(1.0),
                computed.size().y.max(1.0),
                &other_rects,
                window.resolution.width(),
                window.resolution.height(),
                tab_bar_height,
            );
            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            node.left = Val::Px(new_x);
            node.top = Val::Px(new_y);

            save_quick_roll_position(&mut settings_state, new_x, new_y);
        }
    }

    // Command History panel -------------------------------------------------
    {
        let mut history_panel_query = panel_queries.p2();

        // Start dragging when the handle is pressed.
        for (interaction, child_of) in history_handle_interaction.iter() {
            if *interaction != Interaction::Pressed {
                continue;
            }

            if !mouse.just_pressed(MouseButton::Left) {
                continue;
            }

            let parent_entity = child_of.parent();
            let Ok((node, mut drag_state, _computed)) =
                history_panel_query.get_mut(parent_entity)
            else {
                continue;
            };

            let current_x = match node.left {
                Val::Px(px) => px,
                _ => 0.0,
            };
            let current_y = match node.top {
                Val::Px(px) => px,
                _ => 0.0,
            };

            drag_state.dragging = true;
            drag_state.grab_offset = Vec2::new(
                cursor_position.x - current_x,
                cursor_position.y - current_y,
            );
        }

        // Update position while dragging.
        for (mut node, mut drag_state, computed) in history_panel_query.iter_mut() {
            if !drag_state.dragging {
                continue;
            }

            if mouse.just_released(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            if !mouse.pressed(MouseButton::Left) {
                drag_state.dragging = false;
                continue;
            }

            let mut new_x = cursor_position.x - drag_state.grab_offset.x;
            let mut new_y = cursor_position.y - drag_state.grab_offset.y;

            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            let mut other_rects = Vec::with_capacity(4);
            if let Some(c) = slider_group_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.slider_group_position.x,
                    y: settings_state.settings.slider_group_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = quick_roll_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.quick_roll_panel_position.x,
                    y: settings_state.settings.quick_roll_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = results_panel_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.results_panel_position.x,
                    y: settings_state.settings.results_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }
            if let Some(c) = dice_box_controls_size.iter().next() {
                other_rects.push(Rect {
                    x: settings_state.settings.dice_box_controls_panel_position.x,
                    y: settings_state.settings.dice_box_controls_panel_position.y,
                    w: c.size().x.max(1.0),
                    h: c.size().y.max(1.0),
                });
            }

            (new_x, new_y) = apply_snapping(
                new_x,
                new_y,
                computed.size().x.max(1.0),
                computed.size().y.max(1.0),
                &other_rects,
                window.resolution.width(),
                window.resolution.height(),
                tab_bar_height,
            );
            (new_x, new_y) = clamp_to_window(new_x, new_y, &computed);

            node.left = Val::Px(new_x);
            node.top = Val::Px(new_y);

            save_command_history_position(&mut settings_state, new_x, new_y);
        }
    }
}
