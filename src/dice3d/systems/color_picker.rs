//! # Color Picker UI Module
//!
//! A HSV color picker with a circular hue/saturation wheel and brightness/alpha sliders.
//!
//! ## Bevy UI Coordinate System - CRITICAL for Mouse/UI Interaction
//!
//! When implementing custom mouse interactions with Bevy UI nodes, understanding the
//! coordinate system is essential. There are two coordinate spaces:
//!
//! ### Logical vs Physical Pixels
//!
//! - **Logical pixels**: Screen coordinates divided by the scale factor. Used for layout.
//! - **Physical pixels**: Raw screen coordinates (what the GPU renders). Used by transforms.
//!
//! On a 200% DPI display (scale_factor = 2.0):
//! - A 200x200 logical pixel UI node is actually 400x400 physical pixels
//! - A click at logical (100, 100) is at physical (200, 200)
//!
//! ### Key Bevy APIs and Their Coordinate Spaces
//!
//! | API                                  | Coordinate Space | Notes                          |
//! |--------------------------------------|------------------|--------------------------------|
//! | `Window::cursor_position()`          | **Logical**      | Divided by scale factor        |
//! | `Window::physical_cursor_position()` | **Physical**     | Raw pixel coordinates          |
//! | `ComputedNode::size()`               | **Physical**     | Node size in physical pixels   |
//! | `UiGlobalTransform`                  | **Physical**     | Transform matrix in physical   |
//! | `ComputedNode::normalize_point()`    | **Physical**     | Expects physical coordinates   |
//! | `Node` width/height `Val::Px()`      | **Logical**      | Layout uses logical pixels     |
//!
//! ### The Common Pitfall (What We Fixed Here)
//!
//! **WRONG**: Using `window.cursor_position()` with `normalize_point()`:
//! ```ignore
//! let cursor = window.cursor_position(); // LOGICAL pixels
//! let normalized = computed_node.normalize_point(transform, cursor); // Expects PHYSICAL!
//! // Result: On high-DPI, click position appears offset from actual cursor
//! ```
//!
//! **CORRECT**: Use `window.physical_cursor_position()`:
//! ```ignore
//! let cursor = window.physical_cursor_position(); // PHYSICAL pixels
//! let normalized = computed_node.normalize_point(transform, cursor); // Both physical ✓
//! // Result: Click position matches cursor exactly on any DPI
//! ```
//!
//! ### How Bevy's Picking Backend Does It
//!
//! Reference: `bevy_ui/src/picking_backend.rs` lines 137-146
//! ```ignore
//! // Bevy converts logical pointer position to physical:
//! let mut pointer_pos = pointer_location.position * camera_data.target_scaling_factor().unwrap_or(1.);
//! // Then uses it with node.transform and node.node.contains_point()
//! ```
//!
//! ### Quick Reference for Future UI Interactions
//!
//! 1. **Hit testing / point-in-node**: Use `physical_cursor_position()` with `contains_point()`
//! 2. **Normalizing to node-local coords**: Use `physical_cursor_position()` with `normalize_point()`
//! 3. **Manual transform math**: Multiply logical by `window.scale_factor()` first
//! 4. **Positioning UI elements**: Use logical `Val::Px()` values
//!
//! See also: `bevy_window/src/window.rs` for cursor position methods documentation.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::FocusPolicy;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;
use bevy_material_ui::prelude::*;
use bevy_material_ui::slider::{spawn_slider_control_with, MaterialSlider, SliderChangeEvent};
use std::f32::consts::PI;

use crate::dice3d::types::{ColorSetting, SettingsState};

#[derive(Resource, Default)]
pub struct ColorPickerState {
    pub active: bool,
    pub dragging_wheel: bool,
    pub drag_start_cursor: Option<Vec2>, // Cursor position when drag started
    pub drag_start_hsv: Option<Vec2>,    // (hue, saturation) when drag started
    pub target: Option<ColorPickerTarget>,
    pub original_color: Srgba,
    pub current_hsv: Vec3, // x=H(0-1), y=S(0-1), z=V(0-1)
    pub current_alpha: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorPickerTarget {
    Background,
    Theme,
    Highlight,
}

#[derive(Resource)]
pub struct ColorWheelTexture(pub Handle<Image>);

#[derive(Component)]
pub struct ColorPickerRoot;

#[derive(Component)]
pub struct ColorWheelImage;

#[derive(Component)]
pub struct ColorPickerMarker;

#[derive(Component)]
pub struct ColorPickerPreview;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColorPickerValueSlider;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColorPickerAlphaSlider;

#[derive(Component)]
pub struct ColorPickerSelectButton;

#[derive(Component)]
pub struct ColorPickerCancelButton;

pub fn setup_color_wheel_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = 256;
    let mut data = Vec::with_capacity(size * size * 4);

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - size as f32 / 2.0;
            let dy = y as f32 - size as f32 / 2.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let radius = size as f32 / 2.0;

            if dist > radius {
                // Transparent outside circle
                data.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                // Use Cartesian coordinates (Y-up) for angle to match standard Hue layout (Red=0, Yellow=60, Green=120...)
                // Image Y is down, so we invert dy
                let angle = (-dy).atan2(dx);
                // Normalize angle to 0..2PI
                let alpha = if angle < 0.0 { angle + 2.0 * PI } else { angle };
                let hue = alpha / (2.0 * PI); // 0 to 1

                let saturation = dist / radius; // 0 to 1
                let value = 1.0;

                let (r, g, b) = hsv_to_rgb(hue, saturation, value);

                data.push((r * 255.0) as u8);
                data.push((g * 255.0) as u8);
                data.push((b * 255.0) as u8);
                data.push(255);
            }
        }
    }

    let image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    commands.insert_resource(ColorWheelTexture(images.add(image)));
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let i = i as i32 % 6;
    match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

fn rgb_to_hsv(color: Srgba) -> Vec3 {
    let r = color.red;
    let g = color.green;
    let b = color.blue;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;

    let mut h = 0.0;
    let s = if max == 0.0 { 0.0 } else { d / max };
    let v = max;

    if max != min {
        if max == r {
            h = (g - b) / d + (if g < b { 6.0 } else { 0.0 });
        } else if max == g {
            h = (b - r) / d + 2.0;
        } else {
            h = (r - g) / d + 4.0;
        }
        h /= 6.0;
    }

    Vec3::new(h, s, v)
}

pub fn open_color_picker(
    target: ColorPickerTarget,
    current_color: Srgba,
    state: &mut ColorPickerState,
) {
    state.active = true;
    state.dragging_wheel = false;
    state.drag_start_cursor = None;
    state.drag_start_hsv = None;
    state.target = Some(target);
    state.original_color = current_color;
    state.current_hsv = rgb_to_hsv(current_color);
    state.current_alpha = current_color.alpha;
}

pub fn spawn_color_picker_ui(
    mut commands: Commands,
    state: Res<ColorPickerState>,
    wheel_texture: Res<ColorWheelTexture>,
    theme: Res<MaterialTheme>,
    root_query: Query<Entity, With<ColorPickerRoot>>,
) {
    // If active and not spawned, spawn it
    if state.active && root_query.is_empty() {
        let dialog = MaterialDialog::new()
            .title("Select Color")
            .open(true)
            .modal(true);

        let dialog_bg = dialog.surface_color(&theme);

        let dialog_entity = commands
            .spawn((
                dialog,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(340.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                GlobalTransform::default(),
                Transform::default(),
                Visibility::default(),
                BackgroundColor(dialog_bg),
                BorderRadius::all(Val::Px(16.0)),
                BoxShadow::default(),
            ))
            .id();

        let scrim_entity = commands
            .spawn((
                create_dialog_scrim_for(&theme, dialog_entity, false), // Handle close manually if needed
                ColorPickerRoot,
            ))
            .insert(GlobalZIndex(2000))
            .id();

        commands.entity(scrim_entity).add_child(dialog_entity);

        commands.entity(dialog_entity).with_children(|container| {
            // Color Wheel
            let visual_brightness = state.current_hsv.z.max(0.1);
            let visual_alpha = state.current_alpha.max(0.2);

            let wheel_tint = Color::Srgba(Srgba::new(
                visual_brightness,
                visual_brightness,
                visual_brightness,
                visual_alpha,
            ));

            container
                .spawn((
                    ImageNode {
                        image: wheel_texture.0.clone(),
                        color: wheel_tint,
                        ..default()
                    },
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(200.0),
                        ..default()
                    },
                    GlobalTransform::default(),
                    Transform::default(),
                    Visibility::default(),
                    BackgroundColor(Color::NONE), // Ensure hit test works
                    FocusPolicy::Block,           // Ensure it receives interaction
                    ColorWheelImage,
                    Interaction::default(),
                ))
                .with_children(|wheel| {
                    // Marker
                    wheel.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(12.0),
                            height: Val::Px(12.0),
                            // Initial position will be updated by system
                            left: Val::Px(94.0),
                            top: Val::Px(94.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::from(Color::WHITE),
                        BorderRadius::all(Val::Percent(50.0)),
                        FocusPolicy::Pass,
                        ColorPickerMarker,
                    ));
                });

            // Preview & Sliders
            container
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Preview Box
                    row.spawn((
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::Srgba(
                            Srgba::from_vec3(
                                hsv_to_rgb(
                                    state.current_hsv.x,
                                    state.current_hsv.y,
                                    state.current_hsv.z,
                                )
                                .into(),
                            )
                            .with_alpha(state.current_alpha),
                        )),
                        BorderColor::from(theme.outline),
                        BorderRadius::all(Val::Px(4.0)),
                        ColorPickerPreview,
                    ));

                    // Sliders Column
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|sliders| {
                        // Value Slider
                        sliders.spawn((
                            Text::new("Brightness"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));

                        sliders
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(20.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                let slider = MaterialSlider::new(0.0, 1.0)
                                    .with_value(state.current_hsv.z)
                                    .track_height(6.0)
                                    .thumb_radius(8.0);
                                spawn_slider_control_with(
                                    slot,
                                    &theme,
                                    slider,
                                    ColorPickerValueSlider,
                                );
                            });

                        // Alpha Slider
                        sliders.spawn((
                            Text::new("Alpha"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));

                        sliders
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(20.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                let slider = MaterialSlider::new(0.0, 1.0)
                                    .with_value(state.current_alpha)
                                    .track_height(6.0)
                                    .thumb_radius(8.0);
                                spawn_slider_control_with(
                                    slot,
                                    &theme,
                                    slider,
                                    ColorPickerAlphaSlider,
                                );
                            });
                    });
                });

            // Buttons
            container
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::FlexEnd,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|btns| {
                    // Cancel
                    btns.spawn((
                        MaterialButtonBuilder::new("Cancel").text().build(&theme),
                        ColorPickerCancelButton,
                    ));

                    // Select
                    btns.spawn((
                        MaterialButtonBuilder::new("Select").filled().build(&theme),
                        ColorPickerSelectButton,
                    ));
                });
        });
    } else if !state.active && !root_query.is_empty() {
        // Despawn if not active
        if let Some(entity) = root_query.iter().next() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_color_picker_interactions(
    mut state: ResMut<ColorPickerState>,
    mut settings: ResMut<SettingsState>,
    _theme_res: ResMut<MaterialTheme>,

    // Wheel interaction - use ComputedNode and UiGlobalTransform for position calculation
    wheel_query: Query<(&Interaction, &ComputedNode, &UiGlobalTransform), With<ColorWheelImage>>,
    // Slider interactions
    mut slider_events: MessageReader<SliderChangeEvent>,
    value_slider_query: Query<&ColorPickerValueSlider>,
    alpha_slider_query: Query<&ColorPickerAlphaSlider>,

    // Button interactions
    _button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    select_query: Query<&Interaction, (With<ColorPickerSelectButton>, Changed<Interaction>)>,
    cancel_query: Query<&Interaction, (With<ColorPickerCancelButton>, Changed<Interaction>)>,

    // Input for global drag tracking
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !state.active {
        return;
    }

    // Wheel Logic (Click/Drag) - use normalize_point for accurate cursor mapping
    // IMPORTANT: UiGlobalTransform and ComputedNode.size() work in PHYSICAL pixels.
    // Bevy's picking backend converts pointer_location.position (logical) to physical by
    // multiplying by target_scaling_factor. We must do the same when using normalize_point.
    if let Some((interaction, computed_node, ui_transform)) = wheel_query.iter().next() {
        if let Some(window) = windows.iter().next() {
            // Helper to calculate HSV from cursor position using normalize_point
            // CRITICAL: Use physical_cursor_position() because UiGlobalTransform operates
            // in physical pixel space. See bevy_ui/src/picking_backend.rs lines 137-146.
            let calc_hsv_from_cursor = |cursor_pos_physical: Vec2| -> Option<(f32, f32)> {
                // normalize_point returns centered coords: center=(0,0), corners at +/-0.5
                if let Some(normalized) =
                    computed_node.normalize_point(*ui_transform, cursor_pos_physical)
                {
                    // normalized: (-0.5 to 0.5, -0.5 to 0.5) with center at (0,0)
                    // Distance from center (0 to ~0.707 at corners, but UI is square so 0.5 at edges)
                    let dist = (normalized.x * normalized.x + normalized.y * normalized.y).sqrt();
                    let saturation = (dist * 2.0).clamp(0.0, 1.0); // 0.5 = sat 1.0

                    // Angle: -normalized.y because normalize_point has Y-down
                    let angle = (-normalized.y).atan2(normalized.x);
                    let alpha = if angle < 0.0 { angle + 2.0 * PI } else { angle };
                    let hue = alpha / (2.0 * PI);

                    Some((hue, saturation))
                } else {
                    None
                }
            };

            // Start dragging if pressed on the wheel
            if *interaction == Interaction::Pressed && !state.dragging_wheel {
                state.dragging_wheel = true;

                // Use physical_cursor_position() - this matches how Bevy's picking works
                if let Some(cursor_pos_physical) = window.physical_cursor_position() {
                    if let Some((hue, saturation)) = calc_hsv_from_cursor(cursor_pos_physical) {
                        state.current_hsv.x = hue;
                        state.current_hsv.y = saturation;

                        info!(
                            "Click: cursor_physical={:?} hue={:.3} sat={:.3}",
                            cursor_pos_physical, hue, saturation
                        );
                    }
                }
            }

            // Process drag - continuously update from cursor position
            if state.dragging_wheel {
                // Use physical_cursor_position() for consistency with UiGlobalTransform
                if let Some(cursor_pos_physical) = window.physical_cursor_position() {
                    if let Some((hue, saturation)) = calc_hsv_from_cursor(cursor_pos_physical) {
                        state.current_hsv.x = hue;
                        state.current_hsv.y = saturation;
                    }
                }
            }
        }
    }

    // Stop dragging if mouse released anywhere
    if mouse_button_input.just_released(MouseButton::Left) {
        state.dragging_wheel = false;
    }

    // Slider Logic
    for event in slider_events.read() {
        if value_slider_query.contains(event.entity) {
            state.current_hsv.z = event.value.clamp(0.0, 1.0);
        } else if alpha_slider_query.contains(event.entity) {
            state.current_alpha = event.value.clamp(0.0, 1.0);
        }
    }

    // Buttons
    if let Some(interaction) = cancel_query.iter().next() {
        if *interaction == Interaction::Pressed {
            state.active = false;
        }
    }

    if let Some(interaction) = select_query.iter().next() {
        if *interaction == Interaction::Pressed {
            // Apply changes
            let (r, g, b) = hsv_to_rgb(
                state.current_hsv.x,
                state.current_hsv.y,
                state.current_hsv.z,
            );
            let new_color = ColorSetting {
                r,
                g,
                b,
                a: state.current_alpha,
            };

            if let Some(target) = state.target {
                match target {
                    ColorPickerTarget::Background => {
                        settings.settings.background_color = new_color.clone();
                        settings.editing_color = new_color;
                    }
                    ColorPickerTarget::Theme => {
                        // Update theme seed
                        let hex = new_color.to_hex();
                        settings.theme_seed_input_text = hex.clone();
                        // Note: Theme update logic is handled in colors.rs via text input change
                    }
                    ColorPickerTarget::Highlight => {
                        settings.settings.dice_box_highlight_color = new_color.clone();
                        settings.editing_highlight_color = new_color;
                    }
                }
            }
            state.active = false;
        }
    }
}

// System to continuously update the preview color while picker is open
pub fn update_color_picker_preview(
    state: Res<ColorPickerState>,
    mut preview_query: Query<&mut BackgroundColor, With<ColorPickerPreview>>,
    mut wheel_query: Query<(&mut ImageNode, &Node), With<ColorWheelImage>>,
    mut marker_query: Query<&mut Node, (With<ColorPickerMarker>, Without<ColorWheelImage>)>,
    mut value_slider_query: Query<
        &mut MaterialSlider,
        (
            With<ColorPickerValueSlider>,
            Without<ColorPickerAlphaSlider>,
        ),
    >,
    mut alpha_slider_query: Query<
        &mut MaterialSlider,
        (
            With<ColorPickerAlphaSlider>,
            Without<ColorPickerValueSlider>,
        ),
    >,
) {
    if !state.active {
        return;
    }

    let (r, g, b) = hsv_to_rgb(
        state.current_hsv.x,
        state.current_hsv.y,
        state.current_hsv.z,
    );
    let color = Color::Srgba(Srgba::new(r, g, b, state.current_alpha));

    for mut bg in preview_query.iter_mut() {
        *bg = BackgroundColor(color);
    }

    // Update wheel tint based on brightness and alpha
    // Ensure it remains visible even at low brightness/alpha so user can still interact
    let visual_brightness = state.current_hsv.z.max(0.1);
    let visual_alpha = state.current_alpha.max(0.2);

    let wheel_tint = Color::Srgba(Srgba::new(
        visual_brightness,
        visual_brightness,
        visual_brightness,
        visual_alpha,
    ));

    // Get the logical size from Node (Val::Px values) for marker positioning
    let mut logical_radius = 100.0; // Default: 200px / 2

    for (mut image_node, node) in wheel_query.iter_mut() {
        image_node.color = wheel_tint;
        // Use logical size from Node
        if let Val::Px(w) = node.width {
            logical_radius = w / 2.0;
        }
    }

    // Update marker position using logical coordinates
    let angle = state.current_hsv.x * 2.0 * PI;
    let radius = state.current_hsv.y * logical_radius;

    // Cartesian to Screen Space (Y-up to Y-down)
    let x = angle.cos() * radius;
    let y = -angle.sin() * radius;

    let marker_left = logical_radius + x - 6.0;
    let marker_top = logical_radius + y - 6.0;

    for mut node in marker_query.iter_mut() {
        node.left = Val::Px(marker_left);
        node.top = Val::Px(marker_top);
    }

    // Update slider values if they changed externally (e.g. by wheel)
    // Note: MaterialSlider handles its own internal state, but we should sync it if state changes otherwise
    for mut slider in value_slider_query.iter_mut() {
        if (slider.value - state.current_hsv.z).abs() > 0.001 {
            slider.value = state.current_hsv.z;
        }
    }
    for mut slider in alpha_slider_query.iter_mut() {
        if (slider.value - state.current_alpha).abs() > 0.001 {
            slider.value = state.current_alpha;
        }
    }
}
