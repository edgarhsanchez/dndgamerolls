//! Settings UI systems
//!
//! This module contains systems for the settings button, modal, and color picker.

use bevy::prelude::*;

use crate::dice3d::types::{
    ColorComponent, ColorPreview, ColorSetting, ColorSlider, ColorSliderHandle, ColorSliderTrack,
    ColorTextInput, ColorValueLabel, DiceRollerRoot, IconAssets, IconType, SettingsButton,
    SettingsCancelButton, SettingsModal, SettingsModalOverlay, SettingsOkButton, SettingsState,
};

// ============================================================================
// Constants
// ============================================================================

const MODAL_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
const PANEL_BG: Color = Color::srgb(0.15, 0.15, 0.2);
const BUTTON_BG: Color = Color::srgb(0.25, 0.25, 0.3);
const BUTTON_HOVER: Color = Color::srgb(0.35, 0.35, 0.4);
const BUTTON_OK: Color = Color::srgb(0.2, 0.5, 0.3);
const BUTTON_CANCEL: Color = Color::srgb(0.5, 0.2, 0.2);
const TEXT_PRIMARY: Color = Color::srgb(0.9, 0.9, 0.9);
const TEXT_SECONDARY: Color = Color::srgb(0.7, 0.7, 0.7);
const SLIDER_TRACK_BG: Color = Color::srgb(0.1, 0.1, 0.1);
const INPUT_BG: Color = Color::srgb(0.1, 0.1, 0.12);

// ============================================================================
// Setup Systems
// ============================================================================

/// Spawn the settings button (gear icon) in the dice view
pub fn spawn_settings_button(commands: &mut Commands, icon_assets: Res<IconAssets>) {
    let icon_handle = icon_assets.icons.get(&IconType::Settings).cloned();

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    right: Val::Px(10.0), // Rightmost side
                    width: Val::Px(36.0),
                    height: Val::Px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: BackgroundColor(BUTTON_BG),
                border_color: BorderColor(Color::srgb(0.4, 0.4, 0.45)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            SettingsButton,
            DiceRollerRoot,
        ))
        .with_children(|parent| {
            if let Some(handle) = icon_handle {
                // Use icon image
                parent.spawn(ImageBundle {
                    image: UiImage::new(handle),
                    style: Style {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                    ..default()
                });
            } else {
                // Fallback to unicode
                parent.spawn(TextBundle::from_section(
                    "⚙",
                    TextStyle {
                        font_size: 22.0,
                        color: TEXT_PRIMARY,
                        ..default()
                    },
                ));
            }
        });
}

/// Spawn the settings modal
pub fn spawn_settings_modal(commands: &mut Commands, settings_state: &SettingsState) {
    let editing_color = &settings_state.editing_color;

    // Modal overlay (darkens background)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(MODAL_BG),
                z_index: ZIndex::Global(100),
                ..default()
            },
            SettingsModalOverlay,
        ))
        .with_children(|overlay| {
            // Modal window
            overlay
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(450.0),
                            min_height: Val::Px(400.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(20.0)),
                            row_gap: Val::Px(15.0),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(PANEL_BG),
                        border_color: BorderColor(Color::srgb(0.4, 0.4, 0.5)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    SettingsModal,
                ))
                .with_children(|modal| {
                    // Title
                    modal.spawn(TextBundle::from_section(
                        "Settings",
                        TextStyle {
                            font_size: 24.0,
                            color: TEXT_PRIMARY,
                            ..default()
                        },
                    ));

                    // Background Color section
                    modal.spawn(TextBundle::from_section(
                        "Background Color",
                        TextStyle {
                            font_size: 18.0,
                            color: TEXT_SECONDARY,
                            ..default()
                        },
                    ));

                    // Color preview and sliders container
                    modal
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(20.0),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            // Color preview box
                            row.spawn((
                                NodeBundle {
                                    style: Style {
                                        width: Val::Px(80.0),
                                        height: Val::Px(120.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    background_color: BackgroundColor(editing_color.to_color()),
                                    border_color: BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                ColorPreview,
                            ));

                            // Sliders column
                            row.spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    flex_grow: 1.0,
                                    row_gap: Val::Px(10.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|sliders| {
                                // Alpha slider
                                spawn_color_slider(
                                    sliders,
                                    ColorComponent::Alpha,
                                    "A",
                                    editing_color.a,
                                    Color::WHITE,
                                );
                                // Red slider
                                spawn_color_slider(
                                    sliders,
                                    ColorComponent::Red,
                                    "R",
                                    editing_color.r,
                                    Color::srgb(1.0, 0.3, 0.3),
                                );
                                // Green slider
                                spawn_color_slider(
                                    sliders,
                                    ColorComponent::Green,
                                    "G",
                                    editing_color.g,
                                    Color::srgb(0.3, 1.0, 0.3),
                                );
                                // Blue slider
                                spawn_color_slider(
                                    sliders,
                                    ColorComponent::Blue,
                                    "B",
                                    editing_color.b,
                                    Color::srgb(0.3, 0.3, 1.0),
                                );
                            });
                        });

                    // Text input section
                    modal.spawn(TextBundle::from_section(
                        "Enter color (hex, ARGB, or labeled):",
                        TextStyle {
                            font_size: 14.0,
                            color: TEXT_SECONDARY,
                            ..default()
                        },
                    ));

                    // Text input field
                    modal
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(32.0),
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                background_color: BackgroundColor(INPUT_BG),
                                border_color: BorderColor(Color::srgb(0.3, 0.3, 0.35)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            ColorTextInput,
                        ))
                        .with_children(|input| {
                            input.spawn(TextBundle::from_section(
                                editing_color.to_hex(),
                                TextStyle {
                                    font_size: 16.0,
                                    color: TEXT_PRIMARY,
                                    ..default()
                                },
                            ));
                        });

                    // Format hints
                    modal.spawn(TextBundle::from_section(
                        "Formats: #AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, or 1,0.5,0.3,0.2",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::srgb(0.5, 0.5, 0.5),
                            ..default()
                        },
                    ));

                    // Spacer
                    modal.spawn(NodeBundle {
                        style: Style {
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });

                    // Buttons row
                    modal
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::FlexEnd,
                                column_gap: Val::Px(10.0),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|buttons| {
                            // Cancel button
                            buttons
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(100.0),
                                            height: Val::Px(36.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        background_color: BackgroundColor(BUTTON_CANCEL),
                                        border_color: BorderColor(Color::srgb(0.6, 0.3, 0.3)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SettingsCancelButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        "Cancel",
                                        TextStyle {
                                            font_size: 16.0,
                                            color: TEXT_PRIMARY,
                                            ..default()
                                        },
                                    ));
                                });

                            // OK button
                            buttons
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(100.0),
                                            height: Val::Px(36.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        background_color: BackgroundColor(BUTTON_OK),
                                        border_color: BorderColor(Color::srgb(0.3, 0.6, 0.4)),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    SettingsOkButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        "OK",
                                        TextStyle {
                                            font_size: 16.0,
                                            color: TEXT_PRIMARY,
                                            ..default()
                                        },
                                    ));
                                });
                        });
                });
        });
}

/// Helper to spawn a color slider row
fn spawn_color_slider(
    parent: &mut ChildBuilder,
    component: ColorComponent,
    label: &str,
    value: f32,
    track_color: Color,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                height: Val::Px(24.0),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 16.0,
                    color: track_color,
                    ..default()
                },
            ));

            // Slider track
            row.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(180.0),
                        height: Val::Px(16.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(SLIDER_TRACK_BG),
                    border_color: BorderColor(Color::srgb(0.3, 0.3, 0.35)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                ColorSliderTrack { component },
                ColorSlider { component },
                Interaction::None,
            ))
            .with_children(|track| {
                // Slider handle
                track.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(value * 100.0 - 4.0), // Center the 14px handle
                            top: Val::Px(1.0),
                            width: Val::Px(14.0),
                            height: Val::Px(14.0),
                            ..default()
                        },
                        background_color: BackgroundColor(track_color),
                        border_radius: BorderRadius::all(Val::Px(7.0)),
                        ..default()
                    },
                    ColorSliderHandle { component },
                ));
            });

            // Value label
            row.spawn((
                TextBundle::from_section(
                    format!("{:.2}", value),
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_SECONDARY,
                        ..default()
                    },
                ),
                ColorValueLabel { component },
            ));
        });
}

// ============================================================================
// Interaction Systems
// ============================================================================

/// Handle settings button click
pub fn handle_settings_button_click(
    interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsButton>),
    >,
    mut settings_state: ResMut<SettingsState>,
) {
    for (interaction, _bg) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            settings_state.show_modal = true;
            settings_state.editing_color = settings_state.settings.background_color.clone();
            settings_state.color_input_text = settings_state.editing_color.to_hex();
        }
    }
}

/// Handle settings button hover
pub fn handle_settings_button_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsButton>),
    >,
) {
    for (interaction, mut bg) in interaction_query.iter_mut() {
        *bg = BackgroundColor(match interaction {
            Interaction::Hovered => BUTTON_HOVER,
            _ => BUTTON_BG,
        });
    }
}

/// Spawn/despawn settings modal based on state
pub fn manage_settings_modal(
    mut commands: Commands,
    settings_state: Res<SettingsState>,
    modal_query: Query<Entity, With<SettingsModalOverlay>>,
) {
    if !settings_state.is_changed() {
        return;
    }

    if settings_state.show_modal {
        // Spawn modal if not exists
        if modal_query.is_empty() {
            spawn_settings_modal(&mut commands, &settings_state);
        }
    } else {
        // Despawn modal
        for entity in modal_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Handle OK button click
pub fn handle_settings_ok_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SettingsOkButton>)>,
    mut settings_state: ResMut<SettingsState>,
    mut clear_color: ResMut<ClearColor>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Apply the editing color
            settings_state.settings.background_color = settings_state.editing_color.clone();

            // Update the clear color
            clear_color.0 = settings_state.settings.background_color.to_color();

            // Save to file
            if let Err(e) = settings_state.settings.save() {
                eprintln!("Failed to save settings: {}", e);
            }

            // Close modal
            settings_state.show_modal = false;
        }
    }
}

/// Handle Cancel button click
pub fn handle_settings_cancel_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SettingsCancelButton>)>,
    mut settings_state: ResMut<SettingsState>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Discard changes and close modal
            settings_state.show_modal = false;
        }
    }
}

/// Handle color slider interaction
pub fn handle_color_slider_drag(
    mut interaction_query: Query<
        (&Interaction, &ColorSlider, &Node, &GlobalTransform),
        Changed<Interaction>,
    >,
    mut settings_state: ResMut<SettingsState>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    for (interaction, slider, _node, _transform) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings_state.dragging_slider = Some(slider.component);
        }
    }

    // Handle ongoing drag
    if let Some(component) = settings_state.dragging_slider {
        // Get track bounds from any slider with matching component
        for (_interaction, slider, node, transform) in interaction_query.iter() {
            if slider.component == component {
                let rect = node.logical_rect(transform);
                let relative_x = (cursor_pos.x - rect.min.x) / rect.width();
                let value = relative_x.clamp(0.0, 1.0);

                match component {
                    ColorComponent::Alpha => settings_state.editing_color.a = value,
                    ColorComponent::Red => settings_state.editing_color.r = value,
                    ColorComponent::Green => settings_state.editing_color.g = value,
                    ColorComponent::Blue => settings_state.editing_color.b = value,
                }
            }
        }
    }
}

/// Release slider drag on mouse release
pub fn handle_slider_release(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
) {
    if mouse_button.just_released(MouseButton::Left) {
        settings_state.dragging_slider = None;
    }
}

/// Continue slider drag while mouse is held
pub fn handle_slider_drag_continuous(
    mut settings_state: ResMut<SettingsState>,
    slider_query: Query<(&ColorSlider, &Node, &GlobalTransform)>,
    windows: Query<&Window>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Some(component) = settings_state.dragging_slider else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    for (slider, node, transform) in slider_query.iter() {
        if slider.component == component {
            let rect = node.logical_rect(transform);
            let relative_x = (cursor_pos.x - rect.min.x) / rect.width();
            let value = relative_x.clamp(0.0, 1.0);

            match component {
                ColorComponent::Alpha => settings_state.editing_color.a = value,
                ColorComponent::Red => settings_state.editing_color.r = value,
                ColorComponent::Green => settings_state.editing_color.g = value,
                ColorComponent::Blue => settings_state.editing_color.b = value,
            }
            break;
        }
    }
}

/// Update color preview and slider handles when editing color changes
pub fn update_color_ui(
    settings_state: Res<SettingsState>,
    mut preview_query: Query<&mut BackgroundColor, With<ColorPreview>>,
    mut handle_query: Query<(&ColorSliderHandle, &mut Style)>,
    mut label_query: Query<(&ColorValueLabel, &mut Text)>,
    input_query: Query<&Children, With<ColorTextInput>>,
    mut text_query: Query<&mut Text, Without<ColorValueLabel>>,
) {
    if !settings_state.is_changed() {
        return;
    }

    let color = &settings_state.editing_color;

    // Update preview
    for mut bg in preview_query.iter_mut() {
        bg.0 = color.to_color();
    }

    // Update slider handles
    for (handle, mut style) in handle_query.iter_mut() {
        let value = match handle.component {
            ColorComponent::Alpha => color.a,
            ColorComponent::Red => color.r,
            ColorComponent::Green => color.g,
            ColorComponent::Blue => color.b,
        };
        // Position handle (track is 180px, handle is 14px)
        style.left = Val::Px(value * 166.0);
    }

    // Update value labels
    for (label, mut text) in label_query.iter_mut() {
        let value = match label.component {
            ColorComponent::Alpha => color.a,
            ColorComponent::Red => color.r,
            ColorComponent::Green => color.g,
            ColorComponent::Blue => color.b,
        };
        if let Some(section) = text.sections.first_mut() {
            section.value = format!("{:.2}", value);
        }
    }

    // Update text input if not currently being edited
    if settings_state.dragging_slider.is_some() {
        for children in input_query.iter() {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    if let Some(section) = text.sections.first_mut() {
                        section.value = color.to_hex();
                    }
                }
            }
        }
    }
}

/// Handle keyboard input for color text field
pub fn handle_color_text_input(
    mut settings_state: ResMut<SettingsState>,
    mut keyboard_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    input_query: Query<&Children, With<ColorTextInput>>,
    mut text_query: Query<&mut Text>,
) {
    if !settings_state.show_modal {
        return;
    }

    let mut text_changed = false;

    // Handle keyboard input
    for event in keyboard_events.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            KeyCode::Backspace => {
                settings_state.color_input_text.pop();
                text_changed = true;
            }
            KeyCode::Enter => {
                if let Some(parsed) = ColorSetting::parse(&settings_state.color_input_text) {
                    settings_state.editing_color = parsed;
                }
            }
            _ => {
                // Try to get the character from logical_key
                if let bevy::input::keyboard::Key::Character(ref s) = event.logical_key {
                    for c in s.chars() {
                        if c.is_ascii() && !c.is_control() {
                            settings_state.color_input_text.push(c);
                            text_changed = true;
                        }
                    }
                }
            }
        }
    }

    // Update display
    if text_changed {
        for children in input_query.iter() {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    if let Some(section) = text.sections.first_mut() {
                        section.value = settings_state.color_input_text.clone();
                    }
                }
            }
        }

        // Try to parse and update preview (but don't commit yet)
        if let Some(parsed) = ColorSetting::parse(&settings_state.color_input_text) {
            settings_state.editing_color = parsed;
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
