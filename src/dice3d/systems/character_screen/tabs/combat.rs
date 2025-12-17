//! Combat tab content
//!
//! This module contains the UI for the Combat section of the character sheet,
//! including AC, initiative, speed, HP, and proficiency bonus.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use crate::dice3d::types::*;
use super::super::*;

/// Spawn the Combat tab content
pub fn spawn_combat_content(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    let group_type = GroupType::Combat;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    // Card container
    parent
        .spawn((
            CardBuilder::new()
                .outlined()
                .padding(16.0)
                .build(theme),
            StatGroup {
                name: "Combat".to_string(),
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
                "Combat",
                group_type.clone(),
                edit_state,
                icon_font.clone(),
                theme,
            );

            // Armor Class
            spawn_stat_field(
                card,
                "Armor Class",
                &sheet.combat.armor_class.to_string(),
                EditingField::ArmorClass,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("armor_class"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            // Initiative
            spawn_stat_field(
                card,
                "Initiative",
                &format!(
                    "{}{}",
                    if sheet.combat.initiative >= 0 { "+" } else { "" },
                    sheet.combat.initiative
                ),
                EditingField::Initiative,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("initiative"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            // Speed
            spawn_stat_field(
                card,
                "Speed",
                &format!("{} ft", sheet.combat.speed),
                EditingField::Speed,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("speed"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            // Proficiency Bonus
            spawn_stat_field(
                card,
                "Proficiency",
                &format!("+{}", sheet.proficiency_bonus),
                EditingField::ProficiencyBonus,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("proficiency_bonus"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            // Hit Points
            if let Some(hp) = &sheet.combat.hit_points {
                spawn_hp_field(card, hp, is_editing, theme);
            }

            // Custom combat stats
            for (stat_name, stat_value) in sheet.custom_combat.iter() {
                spawn_custom_field_row(
                    card,
                    stat_name,
                    stat_value,
                    GroupType::Combat,
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

/// Spawn the HP field with current/maximum display
fn spawn_hp_field(parent: &mut ChildSpawnerCommands, hp: &HitPoints, is_editing: bool, theme: &MaterialTheme) {
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
                Text::new("Hit Points"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE_VARIANT),
            ));

            // Value container
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|values| {
                // Current HP
                let hp_current_field = EditingField::HitPointsCurrent;
                let current_color = if hp.current < hp.maximum / 2 {
                    MD3_ERROR
                } else {
                    MD3_ON_SURFACE
                };

                let current_text = hp.current.to_string();
                values
                    .spawn((
                        MaterialButtonBuilder::new(current_text.clone())
                            .outlined()
                            .disabled(is_editing)
                            .build(theme),
                        StatField {
                            field: hp_current_field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .insert(Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        min_width: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|field| {
                        field.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(current_text),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(if is_editing { theme.on_surface_variant } else { current_color }),
                            StatFieldValue { field: hp_current_field },
                        ));
                    });

                // Separator
                values.spawn((
                    Text::new("/"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(MD3_ON_SURFACE_VARIANT),
                ));

                // Max HP
                let hp_max_field = EditingField::HitPointsMaximum;
                let max_text = hp.maximum.to_string();
                values
                    .spawn((
                        MaterialButtonBuilder::new(max_text.clone())
                            .outlined()
                            .disabled(is_editing)
                            .build(theme),
                        StatField {
                            field: hp_max_field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .insert(Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        min_width: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|field| {
                        field.spawn((
                            bevy_material_ui::button::ButtonLabel,
                            Text::new(max_text),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(theme.on_surface),
                            StatFieldValue { field: hp_max_field },
                        ));
                    });

                // Temporary HP
                if hp.temporary > 0 {
                    values.spawn((
                        Text::new(format!("(+{} temp)", hp.temporary)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(MD3_SECONDARY),
                    ));
                }
            });
        });
}
