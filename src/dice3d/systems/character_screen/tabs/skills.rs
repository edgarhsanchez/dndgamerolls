//! Skills tab content
//!
//! This module contains the UI for the Skills section of the character sheet,
//! showing all character skills with proficiency indicators and modifiers.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use super::super::*;
use crate::dice3d::types::*;

/// Spawn the Skills tab content
pub fn spawn_skills_content(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    spawn_skills_group(parent, sheet, edit_state, adding_state, icon_assets, theme);
}

/// Spawn the Skills group card.
///
/// This is intentionally reusable between the tabbed view and a future "page" view.
pub fn spawn_skills_group(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let group_type = GroupType::Skills;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    // Card container
    parent
        .spawn((
            CardBuilder::new().outlined().padding(16.0).build(theme),
            StatGroup {
                name: "Skills".to_string(),
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
            spawn_group_header(card, "Skills", group_type.clone(), edit_state, theme);

            // Sort skills alphabetically by display name
            let mut skills: Vec<_> = sheet.skills.iter().collect();
            skills.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));

            for (skill_id, skill) in skills {
                spawn_skill_row(card, skill_id, skill, is_editing, icon_assets, theme);
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(card, group_type, adding_state, icon_assets, theme);
            }
        });
}

/// Spawn a single skill row
fn spawn_skill_row(
    parent: &mut ChildSpawnerCommands,
    skill_id: &str,
    skill: &Skill,
    is_editing: bool,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();
    let skill_id_owned = skill_id.to_string();
    let has_proficiency = skill.proficient || skill.expertise.unwrap_or(false);

    // Convert name to Title Case-ish
    let display_name = camel_to_title_case(&skill.name);

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::vertical(Val::Px(3.0)),
                ..default()
            },
            SkillRow {
                skill_id: skill_id_owned.clone(),
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
                // Dice roll button
                {
                    let icon_name = "casino";
                    let icon_color = MaterialIconButton::new(icon_name)
                        .with_variant(IconButtonVariant::FilledTonal)
                        .icon_color(theme);
                    left.spawn((
                        IconButtonBuilder::new(icon_name)
                            .filled_tonal()
                            .build(theme),
                        RollSkillButton {
                            skill_id: skill_id_owned.clone(),
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
                            btn.spawn(icon.with_color(icon_color).with_size(16.0));
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
                    Button,
                    Interaction::None,
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
                        target: ProficiencyTarget::Skill(skill_id_owned.clone()),
                    },
                ));

                // Skill name
                let label_field = EditingField::SkillLabel(skill_id_owned.clone());

                if is_editing {
                    // Editable label
                    let label_text = display_name.clone();
                    left.spawn((
                        MaterialButtonBuilder::new(label_text.clone())
                            .text()
                            .build(theme),
                        EditableLabelButton {
                            field: label_field.clone(),
                            current_name: skill.name.clone(),
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
                                font_size: 13.0,
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
                            font_size: 13.0,
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

            // Right: modifier value and optional delete button
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|right| {
                let mod_str = if skill.modifier >= 0 {
                    format!("+{}", skill.modifier)
                } else {
                    skill.modifier.to_string()
                };
                let field = EditingField::Skill(skill_id_owned.clone());

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
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        min_width: Val::Px(44.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|btn| {
                        btn.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(mod_str),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(theme.on_surface),
                            StatFieldValue { field },
                        ));
                    });

                // Last roll result (filled when the dice roller completes)
                right.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                    SkillRollResultText {
                        skill: skill_id_owned.clone(),
                    },
                ));

                // Delete button
                if is_editing {
                    spawn_delete_button(
                        right,
                        GroupType::Skills,
                        &skill_id_owned,
                        icon_assets,
                        theme,
                    );
                }
            });
        });
}

/// Convert camelCase to Title Case
fn camel_to_title_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i == 0 {
            result.push(c.to_uppercase().next().unwrap());
        } else if c.is_uppercase() {
            result.push(' ');
            result.push(c);
        } else {
            result.push(c);
        }
    }
    result
}
