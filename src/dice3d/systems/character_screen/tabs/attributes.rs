//! Attributes tab content
//!
//! This module contains the UI for the Attributes section of the character sheet,
//! including the six core ability scores and their modifiers.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use super::super::*;
use crate::dice3d::types::*;

/// Spawn the Attributes tab content
pub fn spawn_attributes_content(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    let group_type = GroupType::Attributes;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    // Card container
    parent
        .spawn((
            CardBuilder::new().outlined().padding(16.0).build(theme),
            StatGroup {
                name: "Attributes".to_string(),
                group_type: group_type.clone(),
            },
        ))
        .insert(Node {
            width: Val::Px(360.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        })
        .with_children(|card| {
            // Group header
            spawn_group_header(
                card,
                "Attributes",
                group_type.clone(),
                edit_state,
                icon_font.clone(),
                theme,
            );

            // Core attributes
            let attrs = [
                (
                    "Strength",
                    sheet.attributes.strength,
                    sheet.modifiers.strength,
                    EditingField::AttributeStrength,
                ),
                (
                    "Dexterity",
                    sheet.attributes.dexterity,
                    sheet.modifiers.dexterity,
                    EditingField::AttributeDexterity,
                ),
                (
                    "Constitution",
                    sheet.attributes.constitution,
                    sheet.modifiers.constitution,
                    EditingField::AttributeConstitution,
                ),
                (
                    "Intelligence",
                    sheet.attributes.intelligence,
                    sheet.modifiers.intelligence,
                    EditingField::AttributeIntelligence,
                ),
                (
                    "Wisdom",
                    sheet.attributes.wisdom,
                    sheet.modifiers.wisdom,
                    EditingField::AttributeWisdom,
                ),
                (
                    "Charisma",
                    sheet.attributes.charisma,
                    sheet.modifiers.charisma,
                    EditingField::AttributeCharisma,
                ),
            ];

            for (name, score, modifier, field) in attrs {
                spawn_attribute_row(
                    card,
                    name,
                    score,
                    modifier,
                    field,
                    is_editing,
                    icon_assets,
                    icon_font.clone(),
                    theme,
                );
            }

            // Custom attributes
            for (attr_name, attr_score) in sheet.custom_attributes.iter() {
                let modifier = Attributes::calculate_modifier(*attr_score);
                spawn_custom_attribute_row(
                    card,
                    attr_name,
                    *attr_score,
                    modifier,
                    is_editing,
                    icon_assets,
                    icon_font.clone(),
                    theme,
                );
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(
                    card,
                    group_type,
                    adding_state,
                    icon_assets,
                    icon_font,
                    theme,
                );
            }
        });
}

/// Spawn a row for a standard attribute
fn spawn_attribute_row(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    score: i32,
    modifier: i32,
    field: EditingField,
    is_editing: bool,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();
    let attr_name = name.to_string();

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Left side: name with dice button
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|name_row| {
                // Dice roll button
                {
                    let icon_name = "casino";
                    let icon_color = MaterialIconButton::new(icon_name)
                        .with_variant(IconButtonVariant::FilledTonal)
                        .icon_color(theme);
                    name_row
                        .spawn((
                            IconButtonBuilder::new(icon_name)
                                .filled_tonal()
                                .build(theme),
                            RollAttributeButton {
                                attribute: attr_name.clone(),
                            },
                        ))
                        .insert(Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|btn| {
                            if let Some(icon) = MaterialIcon::from_name(icon_name) {
                                btn.spawn((
                                    Text::new(icon.as_str()),
                                    TextFont {
                                        font: icon_font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(icon_color),
                                ));
                            } else if let Some(handle) = dice_icon {
                                btn.spawn((
                                    ImageNode::new(handle),
                                    Node {
                                        width: Val::Px(16.0),
                                        height: Val::Px(16.0),
                                        ..default()
                                    },
                                ));
                            } else {
                                btn.spawn((
                                    Text::new("🎲"),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface),
                                ));
                            }
                        });
                }

                // Attribute name
                name_row.spawn((
                    Text::new(name),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(MD3_ON_SURFACE_VARIANT),
                ));
            });

            // Right side: score and modifier
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|values| {
                // Score (editable)
                let score_text = score.to_string();
                values
                    .spawn((
                        MaterialButtonBuilder::new(score_text.clone())
                            .outlined()
                            .disabled(is_editing)
                            .build(theme),
                        StatField {
                            field: field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .insert(Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        min_width: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|field_node| {
                        field_node.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(score_text),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(theme.on_surface),
                            StatFieldValue { field },
                        ));
                    });

                // Modifier (readonly)
                let mod_str = if modifier >= 0 {
                    format!("+{}", modifier)
                } else {
                    modifier.to_string()
                };
                values.spawn((
                    Text::new(format!("({})", mod_str)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(MD3_ON_SURFACE_VARIANT),
                ));

                // Last roll result (filled when the dice roller completes)
                values.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                    AttributeRollResultText {
                        attribute: attr_name.clone(),
                    },
                ));
            });

            // Delete button (only for custom attributes, not shown for standard ones)
        });
}

/// Spawn a row for a custom attribute (with delete button in edit mode)
fn spawn_custom_attribute_row(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    score: i32,
    modifier: i32,
    is_editing: bool,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();
    let attr_name = name.to_string();
    let label_field = EditingField::CustomAttributeLabel(name.to_string());
    let value_field = EditingField::CustomAttribute(name.to_string());

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Left side: name with dice button
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|name_row| {
                // Dice roll button
                {
                    let icon_name = "casino";
                    let icon_color = MaterialIconButton::new(icon_name)
                        .with_variant(IconButtonVariant::FilledTonal)
                        .icon_color(theme);
                    name_row
                        .spawn((
                            IconButtonBuilder::new(icon_name)
                                .filled_tonal()
                                .build(theme),
                            RollAttributeButton {
                                attribute: attr_name.clone(),
                            },
                        ))
                        .insert(Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|btn| {
                            if let Some(icon) = MaterialIcon::from_name(icon_name) {
                                btn.spawn((
                                    Text::new(icon.as_str()),
                                    TextFont {
                                        font: icon_font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(icon_color),
                                ));
                            } else if let Some(handle) = dice_icon {
                                btn.spawn((
                                    ImageNode::new(handle),
                                    Node {
                                        width: Val::Px(16.0),
                                        height: Val::Px(16.0),
                                        ..default()
                                    },
                                ));
                            } else {
                                btn.spawn((
                                    Text::new("🎲"),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(theme.on_surface),
                                ));
                            }
                        });
                }

                // Editable name (when in edit mode)
                if is_editing {
                    name_row
                        .spawn((
                            MaterialButtonBuilder::new(name).text().build(theme),
                            EditableLabelButton {
                                field: label_field.clone(),
                                current_name: name.to_string(),
                            },
                        ))
                        .insert(Node {
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                            ..default()
                        })
                        .with_children(|btn| {
                            btn.spawn((
                                bevy_material_ui::button::ButtonLabel,
                                Text::new(name),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(theme.on_surface_variant),
                                EditableLabelText { field: label_field },
                            ));
                        });
                } else {
                    name_row.spawn((
                        Text::new(name),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));
                }
            });

            // Right side: score, modifier, and delete button
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|values| {
                // Score (editable)
                let score_text = score.to_string();
                values
                    .spawn((
                        MaterialButtonBuilder::new(score_text.clone())
                            .outlined()
                            .disabled(is_editing)
                            .build(theme),
                        StatField {
                            field: value_field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .insert(Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        min_width: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|field_node| {
                        field_node.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(score_text),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(theme.on_surface),
                            StatFieldValue { field: value_field },
                        ));
                    });

                // Modifier
                let mod_str = if modifier >= 0 {
                    format!("+{}", modifier)
                } else {
                    modifier.to_string()
                };
                values.spawn((
                    Text::new(format!("({})", mod_str)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(MD3_ON_SURFACE_VARIANT),
                ));

                // Last roll result (filled when the dice roller completes)
                values.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                    AttributeRollResultText {
                        attribute: attr_name.clone(),
                    },
                ));

                // Delete button (shown in edit mode)
                if is_editing {
                    spawn_delete_button(
                        values,
                        GroupType::Attributes,
                        name,
                        icon_assets,
                        icon_font,
                        theme,
                    );
                }
            });
        });
}
