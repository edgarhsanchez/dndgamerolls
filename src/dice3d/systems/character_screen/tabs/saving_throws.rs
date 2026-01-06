//! Saving Throws tab content
//!
//! This module contains the UI for the Saving Throws section of the character sheet,
//! showing proficiency and modifiers for each ability save.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use super::super::*;
use crate::dice3d::types::*;

/// Spawn the Saving Throws tab content
pub fn spawn_saving_throws_content(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    spawn_saving_throws_group(parent, sheet, edit_state, adding_state, icon_assets, theme);
}

/// Spawn the Saving Throws group card.
///
/// This is intentionally reusable between the tabbed view and a future "page" view.
pub fn spawn_saving_throws_group(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let group_type = GroupType::SavingThrows;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    // Card container
    parent
        .spawn((
            CardBuilder::new().outlined().padding(16.0).build(theme),
            StatGroup {
                name: "Saving Throws".to_string(),
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
            spawn_group_header(card, "Saving Throws", group_type.clone(), edit_state, theme);

            // Standard abilities
            let abilities = [
                "strength",
                "dexterity",
                "constitution",
                "intelligence",
                "wisdom",
                "charisma",
            ];

            for ability in abilities {
                if let Some(save) = sheet.saving_throws.get(ability) {
                    spawn_saving_throw_row(card, ability, save, is_editing, icon_assets, theme);
                }
            }

            // Custom saving throws
            for (save_name, save) in sheet.saving_throws.iter() {
                if !abilities.contains(&save_name.as_str()) {
                    spawn_saving_throw_row(card, save_name, save, is_editing, icon_assets, theme);
                }
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(card, group_type, adding_state, icon_assets, theme);
            }
        });
}

/// Spawn a single saving throw row
fn spawn_saving_throw_row(
    parent: &mut ChildSpawnerCommands,
    ability: &str,
    save: &SavingThrow,
    is_editing: bool,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let ability_owned = ability.to_string();
    let has_proficiency = save.modifier != 0;

    // Format display name (capitalize first letter, truncate to 3 chars)
    let display_name = format!(
        "{}{}",
        ability.chars().next().unwrap().to_uppercase(),
        &ability[1..3]
    );

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::vertical(Val::Px(4.0)),
                ..default()
            },
            SavingThrowRow {
                ability: ability_owned.clone(),
            },
        ))
        .with_children(|row| {
            // Left: proficiency indicator and name
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|left| {
                // Proficiency indicator
                left.spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if has_proficiency {
                        MD3_SUCCESS
                    } else {
                        Color::NONE
                    }),
                    BorderColor::from(if has_proficiency {
                        MD3_SUCCESS
                    } else {
                        MD3_OUTLINE
                    }),
                    BorderRadius::all(Val::Px(4.0)),
                    ProficiencyCheckbox {
                        target: ProficiencyTarget::SavingThrow(ability_owned.clone()),
                    },
                ));

                // Ability name
                let label_field = EditingField::SavingThrowLabel(ability_owned.clone());

                if is_editing {
                    // Editable label
                    let label_text = display_name.clone();
                    left.spawn((
                        MaterialButtonBuilder::new(label_text.clone())
                            .text()
                            .build(theme),
                        EditableLabelButton {
                            field: label_field.clone(),
                            current_name: ability_owned.clone(),
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
                            TextColor(if has_proficiency {
                                MD3_SUCCESS
                            } else {
                                theme.on_surface_variant
                            }),
                            EditableLabelText { field: label_field },
                        ));
                    });
                } else {
                    left.spawn((
                        Text::new(&display_name),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(if has_proficiency {
                            MD3_SUCCESS
                        } else {
                            MD3_ON_SURFACE_VARIANT
                        }),
                    ));
                }
            });

            // Right: modifier value
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|right| {
                let mod_str = if save.modifier >= 0 {
                    format!("+{}", save.modifier)
                } else {
                    save.modifier.to_string()
                };
                let field = EditingField::SavingThrow(ability_owned.clone());

                right
                    .spawn((
                        MaterialButtonBuilder::new(mod_str.clone())
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
                    .with_children(|btn| {
                        btn.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(mod_str),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(theme.on_surface),
                            StatFieldValue { field },
                        ));
                    });

                // Delete button
                if is_editing {
                    spawn_delete_button(
                        right,
                        GroupType::SavingThrows,
                        &ability_owned,
                        icon_assets,
                        theme,
                    );
                }
            });
        });
}
