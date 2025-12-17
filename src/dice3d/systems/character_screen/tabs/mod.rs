//! Character sheet layout module
//!
//! This module renders the character sheet as a wrapping grid of group cards.

use bevy::prelude::*;
use bevy_material_ui::icons::MaterialIconFont;
use bevy_material_ui::prelude::*;

use super::*;
use crate::dice3d::types::*;

// Submodules for each tab
mod attributes;
mod basic_info;
mod combat;
mod saving_throws;
mod skills;

// Re-export tab content builders
pub use attributes::spawn_attributes_content;
pub use basic_info::spawn_basic_info_content;
pub use combat::spawn_combat_content;
pub use saving_throws::spawn_saving_throws_content;
pub use skills::spawn_skills_content;

// ============================================================================
// Character Sheet Tab Container
// ============================================================================

/// Setup the character screen with Material Design tabs
pub fn setup_character_screen(
    mut commands: Commands,
    character_data: Res<CharacterData>,
    character_manager: Res<CharacterManager>,
    edit_state: Res<GroupEditState>,
    adding_state: Res<AddingEntryState>,
    icon_assets: Res<IconAssets>,
    icon_font: Res<MaterialIconFont>,
    theme: Option<Res<MaterialTheme>>,
) {
    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let icon_font = icon_font.0.clone();

    // Root container (hidden by default, shown when tab is active)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(48.0), // Below app tab bar
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(MD3_SURFACE),
            Visibility::Hidden,
            CharacterScreenRoot,
        ))
        .with_children(|parent| {
            // Left panel - Character list
            spawn_character_list_panel(
                parent,
                &character_manager,
                &character_data,
                &icon_assets,
                icon_font.clone(),
                &theme,
            );

            // Right panel - Tabbed character sheet content
            spawn_tabbed_content_panel(
                parent,
                &character_data,
                &character_manager,
                &edit_state,
                &adding_state,
                &icon_assets,
                icon_font.clone(),
                &theme,
            );
        });
}

/// Spawn the right panel with the character sheet content.
///
/// This is public so the UI can be rebuilt when `CharacterData` changes
/// (e.g., after clicking Create New Character).
pub fn spawn_tabbed_content_panel(
    parent: &mut ChildSpawnerCommands,
    character_data: &CharacterData,
    character_manager: &CharacterManager,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            CharacterStatsPanel,
        ))
        .with_children(|container| {
            // Single scrollable area that shows all group cards at once (wrapping grid)
            spawn_all_groups_area(
                container,
                character_data,
                character_manager,
                edit_state,
                adding_state,
                icon_assets,
                icon_font,
                theme,
            );
        });
}

/// Spawn the scrollable content area showing all group cards at once (wrapping grid)
fn spawn_all_groups_area(
    parent: &mut ChildSpawnerCommands,
    character_data: &CharacterData,
    character_manager: &CharacterManager,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    parent
        .spawn((
            ScrollContainer::both(),
            ScrollPosition::default(),
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                // Allow both vertical and horizontal scrolling when space is constrained.
                overflow: Overflow {
                    x: OverflowAxis::Scroll,
                    y: OverflowAxis::Scroll,
                },
                // Important for scroll containers inside flex columns.
                min_height: Val::Px(0.0),
                ..default()
            },
        ))
        .with_children(|container| {
            container
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },))
                .with_children(|content| {
                    if let Some(sheet) = &character_data.sheet {
                        spawn_header_row(
                            content,
                            sheet,
                            character_data.is_modified,
                            icon_assets,
                            icon_font.clone(),
                            theme,
                        );

                        content
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(16.0),
                                row_gap: Val::Px(16.0),
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            })
                            .with_children(|grid| {
                                spawn_basic_info_content(
                                    grid,
                                    sheet,
                                    edit_state,
                                    adding_state,
                                    icon_assets,
                                    icon_font.clone(),
                                    theme,
                                );
                                spawn_attributes_content(
                                    grid,
                                    sheet,
                                    edit_state,
                                    adding_state,
                                    icon_assets,
                                    icon_font.clone(),
                                    theme,
                                );
                                spawn_combat_content(
                                    grid,
                                    sheet,
                                    edit_state,
                                    adding_state,
                                    icon_assets,
                                    icon_font.clone(),
                                    theme,
                                );
                                spawn_saving_throws_content(
                                    grid,
                                    sheet,
                                    edit_state,
                                    adding_state,
                                    icon_assets,
                                    icon_font.clone(),
                                    theme,
                                );
                                spawn_skills_content(
                                    grid,
                                    sheet,
                                    edit_state,
                                    adding_state,
                                    icon_assets,
                                    icon_font.clone(),
                                    theme,
                                );
                            });
                    } else {
                        let has_any_characters = !character_manager.characters.is_empty();
                        spawn_no_character_message(content, theme, has_any_characters);
                    }
                });
        });
}

/// Spawn header row with character name and save button
fn spawn_header_row(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    is_modified: bool,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    let _ = icon_assets;

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::bottom(Val::Px(8.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            BorderColor::from(MD3_OUTLINE_VARIANT),
        ))
        .with_children(|header| {
            // Character name
            header.spawn((
                Text::new(&sheet.character.name),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE),
            ));

            // Header actions: settings + save
            header
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|actions| {
                    // Character sheet dice settings (gear icon)
                    actions
                        .spawn((
                            IconButtonBuilder::new("settings").standard().build(theme),
                            CharacterSheetSettingsButton,
                        ))
                        .insert(Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|button| {
                            button.spawn((
                                Text::new(MaterialIcon::settings().as_str()),
                                TextFont {
                                    font: icon_font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(theme.on_surface_variant),
                            ));
                        });

                    // Save button (MD3 Material button)
                    actions
                        .spawn((
                            MaterialButtonBuilder::new("Save")
                                .filled()
                                .disabled(!is_modified)
                                .build(theme),
                            SaveButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                ButtonLabel,
                                Text::new("Save"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                // Will be kept in sync by bevy_material_ui style systems.
                                TextColor(theme.on_primary),
                            ));
                        });
                });
        });
}

/// Spawn the "no character" message with create button
fn spawn_no_character_message(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    has_any_characters: bool,
) {
    parent
        .spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            min_height: Val::Px(300.0),
            ..default()
        })
        .with_children(|center| {
            // Message
            center.spawn((
                Text::new(if has_any_characters {
                    "Select a character from the list"
                } else {
                    "No character loaded"
                }),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE_VARIANT),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            if !has_any_characters {
                // Create button
                center.spawn(Node::default()).with_children(|wrapper| {
                    wrapper
                        .spawn((
                            MaterialButtonBuilder::new("+ Create First Character")
                                .filled()
                                .build(theme),
                            NewCharacterButton,
                        ))
                        .insert(Node {
                            padding: UiRect::axes(Val::Px(24.0), Val::Px(16.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|btn| {
                            // Ensure label size matches the previous design.
                            btn.spawn((
                                ButtonLabel,
                                Text::new("+ Create First Character"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(theme.on_primary),
                            ));
                        });
                });
            }
        });
}
