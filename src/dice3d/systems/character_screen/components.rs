//! Shared UI components for the character screen
//!
//! This module contains reusable UI components used across multiple tabs,
//! such as stat fields, group headers, delete buttons, etc.

use bevy::prelude::*;
use bevy_material_ui::icons::MaterialIcon;
use bevy_material_ui::prelude::*;

use super::*;
use crate::dice3d::types::*;

// ============================================================================
// Group Header Component
// ============================================================================

/// Spawn a group header with title and edit toggle button
pub fn spawn_group_header(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    group_type: GroupType,
    edit_state: &GroupEditState,
    theme: &MaterialTheme,
) {
    let is_editing = edit_state.editing_groups.contains(&group_type);
    let edit_icon_name = if is_editing { "check" } else { "edit" };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            padding: UiRect::bottom(Val::Px(8.0)),
            border: UiRect::bottom(Val::Px(1.0)),
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .insert(BorderColor::from(MD3_OUTLINE_VARIANT))
        .with_children(|header| {
            // Title
            header.spawn((
                Text::new(title),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE),
            ));

            // Edit toggle button
            {
                let icon_color = MaterialIconButton::new(edit_icon_name)
                    .with_variant(IconButtonVariant::Standard)
                    .icon_color(theme);
                let builder = IconButtonBuilder::new(edit_icon_name).standard();

                header
                    .spawn((
                        builder.build(theme),
                        GroupEditButton {
                            group_type: group_type.clone(),
                        },
                    ))
                    .with_children(|btn| {
                        if let Some(icon) = MaterialIcon::from_name(edit_icon_name) {
                            btn.spawn(icon.with_color(icon_color).with_size(ICON_SIZE));
                        }
                    });
            }
        });
}

// ============================================================================
// Stat Field Component
// ============================================================================

/// Spawn an editable stat field with label and value
#[allow(clippy::too_many_arguments)]
pub fn spawn_stat_field(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    field: EditingField,
    is_numeric: bool,
    is_editing: bool,
    group_type: Option<GroupType>,
    entry_id: Option<&str>,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE_VARIANT),
            ));

            // Value container with optional delete button
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|right| {
                // Clickable value field (MD3 Material button)
                let value_text = value.to_string();
                right
                    .spawn((
                        MaterialButtonBuilder::new(value_text.clone())
                            .outlined()
                            .disabled(is_editing)
                            .build(theme),
                        StatField {
                            field: field.clone(),
                            is_numeric,
                        },
                    ))
                    // Override sizing/layout to match prior compact pill.
                    .insert(Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        min_width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|field_node| {
                        field_node.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(value_text),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            // Button label color is driven by MD3 systems via `ButtonLabel`.
                            TextColor(theme.on_surface),
                            StatFieldValue { field },
                        ));
                    });

                // Delete button (shown in edit mode)
                if is_editing {
                    if let (Some(gt), Some(eid)) = (group_type, entry_id) {
                        spawn_delete_button(right, gt, eid, icon_assets, theme);
                    }
                }
            });
        });
}

/// Spawn a read-only field (no button, just text)
pub fn spawn_readonly_field(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE_VARIANT),
            ));

            row.spawn((
                Text::new(value),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE_VARIANT),
            ));
        });
}

// ============================================================================
// Custom Field Row Component
// ============================================================================

/// Spawn a row for a custom field (editable name and value with delete button)
pub fn spawn_custom_field_row(
    parent: &mut ChildSpawnerCommands,
    field_name: &str,
    field_value: &str,
    group_type: GroupType,
    is_editing: bool,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    // Determine editing fields based on group type
    let (label_field, value_field) = match &group_type {
        GroupType::BasicInfo => (
            EditingField::CustomBasicInfoLabel(field_name.to_string()),
            EditingField::CustomBasicInfo(field_name.to_string()),
        ),
        GroupType::Combat => (
            EditingField::CustomCombatLabel(field_name.to_string()),
            EditingField::CustomCombat(field_name.to_string()),
        ),
        _ => return, // Other groups don't use custom fields this way
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Editable label (when in edit mode)
            if is_editing {
                let label_text = field_name.to_string();
                row.spawn((
                    MaterialButtonBuilder::new(label_text.clone())
                        .text()
                        .build(theme),
                    EditableLabelButton {
                        field: label_field.clone(),
                        current_name: field_name.to_string(),
                    },
                ))
                .insert(Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    ..default()
                })
                .with_children(|btn| {
                    btn.spawn((
                        bevy_material_ui::button::ButtonLabel,
                        Text::new(label_text),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                        EditableLabelText { field: label_field },
                    ));
                });
            } else {
                row.spawn((
                    Text::new(field_name),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(MD3_ON_SURFACE_VARIANT),
                ));
            }

            // Value container with optional delete button
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|right| {
                // Editable value
                let value_text = field_value.to_string();
                right
                    .spawn((
                        MaterialButtonBuilder::new(value_text.clone())
                            .outlined()
                            .disabled(is_editing)
                            .build(theme),
                        StatField {
                            field: value_field.clone(),
                            is_numeric: false,
                        },
                    ))
                    .insert(Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        min_width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|field_node| {
                        field_node.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(value_text),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(theme.on_surface),
                            StatFieldValue { field: value_field },
                        ));
                    });

                // Delete button
                if is_editing {
                    spawn_delete_button(right, group_type, field_name, icon_assets, theme);
                }
            });
        });
}

// ============================================================================
// Delete Button Component
// ============================================================================

/// Spawn a delete button for an entry
pub fn spawn_delete_button(
    parent: &mut ChildSpawnerCommands,
    group_type: GroupType,
    entry_id: &str,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let delete_icon = icon_assets.icons.get(&IconType::Delete).cloned();
    let icon_name = "delete";
    let icon_color = MaterialIconButton::new(icon_name)
        .with_variant(IconButtonVariant::Standard)
        .icon_color(theme);

    parent
        .spawn((
            IconButtonBuilder::new(icon_name).standard().build(theme),
            DeleteEntryButton {
                group_type,
                entry_id: entry_id.to_string(),
            },
        ))
        // Match prior compact sizing.
        .insert(Node {
            width: Val::Px(24.0),
            height: Val::Px(24.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::left(Val::Px(4.0)),
            ..default()
        })
        .with_children(|btn| {
            if let Some(icon) = MaterialIcon::from_name(icon_name) {
                btn.spawn(icon.with_color(icon_color).with_size(16.0));
            } else if let Some(handle) = delete_icon {
                btn.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        ..default()
                    },
                ));
            }
        });
}

// ============================================================================
// Add Button Component
// ============================================================================

/// Spawn an add button at the bottom of a group (or input field if adding)
pub fn spawn_group_add_button(
    parent: &mut ChildSpawnerCommands,
    group_type: GroupType,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let is_adding = adding_state.adding_to.as_ref() == Some(&group_type);
    let check_icon = icon_assets.icons.get(&IconType::Check).cloned();
    let cancel_icon = icon_assets.icons.get(&IconType::Cancel).cloned();
    let add_icon = icon_assets.icons.get(&IconType::Add).cloned();

    if is_adding {
        // Show input field for new entry
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    margin: UiRect::top(Val::Px(12.0)),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(MD3_PRIMARY.with_alpha(0.1)),
                BorderColor::from(MD3_PRIMARY),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_children(|row| {
                // Name input display
                row.spawn((
                    Node {
                        flex_grow: 1.0,
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(MD3_SURFACE_CONTAINER_HIGH),
                    BorderColor::from(MD3_OUTLINE),
                    BorderRadius::all(Val::Px(6.0)),
                    NewEntryInput {
                        group_type: group_type.clone(),
                    },
                ))
                .with_children(|input| {
                    input.spawn((
                        Text::new(if adding_state.new_entry_name.is_empty() {
                            "Type name..."
                        } else {
                            &adding_state.new_entry_name
                        }),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(if adding_state.new_entry_name.is_empty() {
                            MD3_ON_SURFACE_VARIANT
                        } else {
                            MD3_ON_SURFACE
                        }),
                    ));
                });

                // Confirm button
                {
                    let icon_name = "check";
                    let icon_color = MaterialIconButton::new(icon_name)
                        .with_variant(IconButtonVariant::Filled)
                        .icon_color(theme);
                    row.spawn((
                        IconButtonBuilder::new(icon_name).filled().build(theme),
                        NewEntryConfirmButton {
                            group_type: group_type.clone(),
                        },
                    ))
                    .insert(Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|btn| {
                        if let Some(icon) = MaterialIcon::from_name(icon_name) {
                            btn.spawn(icon.with_color(icon_color).with_size(18.0));
                        } else if let Some(handle) = check_icon {
                            btn.spawn((
                                ImageNode::new(handle),
                                Node {
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    ..default()
                                },
                            ));
                        }
                    });
                }

                // Cancel button
                {
                    let icon_name = "close";
                    let icon_color = MaterialIconButton::new(icon_name)
                        .with_variant(IconButtonVariant::FilledTonal)
                        .icon_color(theme);
                    row.spawn((
                        IconButtonBuilder::new(icon_name)
                            .filled_tonal()
                            .build(theme),
                        NewEntryCancelButton { group_type },
                    ))
                    .insert(Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|btn| {
                        if let Some(icon) = MaterialIcon::from_name(icon_name) {
                            btn.spawn(icon.with_color(icon_color).with_size(18.0));
                        } else if let Some(handle) = cancel_icon {
                            btn.spawn((
                                ImageNode::new(handle),
                                Node {
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    ..default()
                                },
                            ));
                        }
                    });
                }
            });
    } else {
        // Show add button
        let label = "Add Field";
        parent
            .spawn((
                MaterialButtonBuilder::new(label).outlined().build(theme),
                GroupAddButton { group_type },
            ))
            .insert(Node {
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(8.0)),
                margin: UiRect::top(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|btn| {
                if let Some(icon) = MaterialIcon::from_name("add") {
                    btn.spawn(icon.with_color(theme.primary).with_size(18.0));
                } else if let Some(handle) = add_icon {
                    btn.spawn((
                        ImageNode::new(handle),
                        Node {
                            width: Val::Px(18.0),
                            height: Val::Px(18.0),
                            ..default()
                        },
                    ));
                }

                btn.spawn((
                    bevy_material_ui::button::ButtonLabel,
                    Text::new(label),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.primary),
                ));
            });
    }
}
