//! Basic Info tab content
//!
//! This module contains the UI for the Basic Info section of the character sheet,
//! including name, class, race, level, and custom fields.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use super::super::*;
use crate::dice3d::types::*;

/// Spawn the Basic Info tab content
pub fn spawn_basic_info_content(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    let group_type = GroupType::BasicInfo;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    // Card container
    parent
        .spawn((
            CardBuilder::new().outlined().padding(16.0).build(theme),
            StatGroup {
                name: "Basic Info".to_string(),
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
                "Basic Info",
                group_type.clone(),
                edit_state,
                icon_font.clone(),
                theme,
            );

            // Core fields
            spawn_stat_field(
                card,
                "Name",
                &sheet.character.name,
                EditingField::CharacterName,
                false,
                is_editing,
                Some(group_type.clone()),
                Some("name"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            spawn_stat_field(
                card,
                "Class",
                &sheet.character.class,
                EditingField::CharacterClass,
                false,
                is_editing,
                Some(group_type.clone()),
                Some("class"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            spawn_stat_field(
                card,
                "Race",
                &sheet.character.race,
                EditingField::CharacterRace,
                false,
                is_editing,
                Some(group_type.clone()),
                Some("race"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            spawn_stat_field(
                card,
                "Level",
                &sheet.character.level.to_string(),
                EditingField::CharacterLevel,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("level"),
                icon_assets,
                icon_font.clone(),
                theme,
            );

            // Optional fields (read-only)
            if let Some(subclass) = &sheet.character.subclass {
                spawn_readonly_field(card, "Subclass", subclass);
            }
            if let Some(background) = &sheet.character.background {
                spawn_readonly_field(card, "Background", background);
            }
            if let Some(alignment) = &sheet.character.alignment {
                spawn_readonly_field(card, "Alignment", alignment);
            }

            // Custom fields
            for (field_name, field_value) in sheet.custom_basic_info.iter() {
                spawn_custom_field_row(
                    card,
                    field_name,
                    field_value,
                    GroupType::BasicInfo,
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
