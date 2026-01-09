//! Character list panel (left side of character screen)
//!
//! This module contains the character list UI showing all available characters,
//! with a modified indicator (*) for unsaved changes.

use bevy::prelude::*;
use bevy_material_ui::icons::MaterialIcon;
use bevy_material_ui::list::{
    ListBuilder, ListItemBody, ListItemBuilder, ListItemClickEvent, ListItemHeadline,
    ListItemSupportingText, ListSelectionMode,
};
use bevy_material_ui::prelude::{
    ButtonClickEvent, IconButtonBuilder, IconButtonClickEvent, IconButtonVariant,
    MaterialButtonBuilder, MaterialIconButton, MaterialTheme,
};
use bevy_material_ui::tokens::Spacing;

use crate::dice3d::types::*;

// ============================================================================
// Character List Panel
// ============================================================================

/// Spawn the character list panel on the left side of the screen
pub fn spawn_character_list_panel(
    parent: &mut ChildSpawnerCommands,
    character_manager: &CharacterManager,
    character_data: &CharacterData,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();

    parent
        .spawn((
            Node {
                width: Val::Px(260.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::right(Val::Px(1.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(theme.surface_container),
            BorderColor::from(theme.outline_variant),
            CharacterListPanel,
        ))
        .with_children(|panel| {
            // Header row with "Characters" title and Roll All button
            spawn_list_header(panel, dice_icon, theme);

            // New Character button
            spawn_new_character_button(panel, theme);

            // Divider
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(theme.outline_variant),
            ));

            // Scrollable Material list (like the showcase)
            panel
                .spawn((
                    // Use a scrollable list with single selection so the selected character
                    // is styled consistently and long lists scroll.
                    ListBuilder::new()
                        .selection_mode(ListSelectionMode::Single)
                        .build_scrollable(),
                    BackgroundColor(theme.surface_container_low),
                    BorderRadius::all(Val::Px(12.0)),
                    Interaction::None,
                ))
                // Replace the Node from the bundle to make it fill remaining space.
                .insert(Node {
                    flex_grow: 1.0,
                    width: Val::Percent(100.0),
                    // In a flex column, scrollable children must be allowed to shrink
                    // so overflow can occur and the scrollbar can appear.
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::vertical(Val::Px(Spacing::SMALL)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                })
                .with_children(|list| {
                    spawn_character_list_items(list, character_manager, character_data, theme);
                });
        });
}

fn spawn_list_header(
    panel: &mut ChildSpawnerCommands,
    dice_icon: Option<Handle<Image>>,
    theme: &MaterialTheme,
) {
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|header| {
            // Title
            header.spawn((
                Text::new("Characters"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(theme.on_surface),
            ));

            // Roll All Stats button (dice icon)
            // Prefer Material icon button.
            // If the Material icon name isn't available at runtime, fall back to the existing dice image.
            {
                let icon_name = "casino";
                let icon_color = MaterialIconButton::new(icon_name)
                    .with_variant(IconButtonVariant::FilledTonal)
                    .icon_color(theme);
                let mut icon_button = header.spawn((
                    IconButtonBuilder::new(icon_name)
                        .filled_tonal()
                        .build(theme),
                    RollAllStatsButton,
                ));

                icon_button.with_children(|btn| {
                    if let Some(icon) = MaterialIcon::from_name(icon_name) {
                        btn.spawn(icon.with_color(icon_color).with_size(24.0));
                    } else if let Some(handle) = dice_icon {
                        btn.spawn((
                            ImageNode::new(handle),
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                ..default()
                            },
                        ));
                    } else {
                        btn.spawn((
                            Text::new("🎲"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(theme.on_surface),
                        ));
                    }
                });
            }
        });
}

fn spawn_new_character_button(panel: &mut ChildSpawnerCommands, theme: &MaterialTheme) {
    // Wrapper provides spacing without replacing the MD3 button's Node.
    panel
        .spawn(Node {
            width: Val::Percent(100.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        })
        .with_children(|wrapper| {
            let label = "+ New Character";

            wrapper
                .spawn((
                    MaterialButtonBuilder::new(label).filled().build(theme),
                    NewCharacterButton,
                ))
                // Replace Node to make it full-width while preserving MD3 padding.
                .insert(Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(Spacing::EXTRA_LARGE), Val::Px(Spacing::MEDIUM)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|btn| {
                    btn.spawn((
                        bevy_material_ui::button::ButtonLabel,
                        Text::new(label),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_primary),
                    ));
                });
        });
}

fn spawn_character_list_items(
    panel: &mut ChildSpawnerCommands,
    character_manager: &CharacterManager,
    character_data: &CharacterData,
    theme: &MaterialTheme,
) {
    for (i, char_entry) in character_manager.characters.iter().enumerate() {
        let is_current = character_manager
            .current_character_id
            .map(|id| id == char_entry.id)
            .unwrap_or(false);

        // Show asterisk for modified current character
        let display_name = if is_current && character_data.is_modified {
            format!("{}*", char_entry.name)
        } else {
            char_entry.name.clone()
        };
        let base_name = char_entry.name.clone();

        let supporting = if char_entry.class.trim().is_empty() {
            format!("Level {}", char_entry.level.max(1))
        } else {
            format!("Level {} • {}", char_entry.level.max(1), char_entry.class)
        };

        panel
            .spawn((
                ListItemBuilder::new(&display_name)
                    .two_line()
                    .supporting_text(&supporting)
                    .selected(is_current)
                    .build(theme),
                CharacterListItem { index: i },
            ))
            .with_children(|item| {
                item.spawn((
                    ListItemBody,
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        ..default()
                    },
                ))
                .with_children(|body| {
                    body.spawn((
                        ListItemHeadline,
                        Text::new(&display_name),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(theme.on_surface),
                        CharacterListItemText {
                            index: i,
                            base_name,
                        },
                    ));

                    body.spawn((
                        ListItemSupportingText,
                        Text::new(&supporting),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));
                });
            });
    }
}

// ============================================================================
// Character List Event Handlers
// ============================================================================

/// Handle clicks on character list items
pub fn handle_character_list_clicks(
    mut click_events: MessageReader<ListItemClickEvent>,
    clicked_items: Query<&CharacterListItem>,
    mut character_manager: ResMut<CharacterManager>,
    mut character_data: ResMut<CharacterData>,
    db: Res<CharacterDatabase>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(list_item) = clicked_items.get(event.entity) else {
            continue;
        };

        if let Some(entry) = character_manager.characters.get(list_item.index) {
            // Load the selected character
            let char_id = entry.id;
            if let Ok(mut sheet) = db.load_character(char_id) {
                // Ensure older data (pre stable-uuid/id fields) is upgraded in-memory.
                // This keeps UI components (skill/save/custom rows) stable across sessions.
                sheet.migrate_to_ids();

                // Persist the upgraded sheet so we don't regenerate ids every load.
                if let Err(err) = db.save_character(Some(char_id), &sheet) {
                    bevy::log::warn!("Failed to persist upgraded character {char_id}: {err}");
                }

                character_manager.current_character_id = Some(char_id);
                character_data.character_id = Some(char_id);
                character_data.sheet = Some(sheet);
                character_data.is_modified = false;
                character_data.needs_refresh = true;
            }
        }
    }
}

/// Handle click on new character button
pub fn handle_new_character_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    buttons: Query<(), With<NewCharacterButton>>,
    mut character_manager: ResMut<CharacterManager>,
    mut character_data: ResMut<CharacterData>,
    _db: Res<CharacterDatabase>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        if buttons.get(event.entity).is_err() {
            continue;
        }

        // Create a new character in-memory.
        // Saving immediately can fail (missing DB/table, locked file, etc.) and would look
        // like the button "does nothing".
        let new_sheet = CharacterSheet::default();

        character_manager.current_character_id = None;
        character_data.sheet = Some(new_sheet);
        character_data.is_modified = true;
    }
}

/// Handle click on roll all stats button
pub fn handle_roll_all_stats_click(
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<(), With<RollAllStatsButton>>,
    mut character_data: ResMut<CharacterData>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        if buttons.get(event.entity).is_err() {
            continue;
        }

        let Some(sheet) = &mut character_data.sheet else {
            continue;
        };

        // Roll 4d6 drop lowest for each attribute
        use rand::Rng;
        let mut rng = rand::rng();

        let roll_4d6_drop_lowest = |rng: &mut rand::rngs::ThreadRng| {
            let mut rolls: Vec<i32> = (0..4).map(|_| rng.random_range(1..=6)).collect();
            rolls.sort();
            rolls.iter().skip(1).sum::<i32>()
        };

        sheet.attributes.strength = roll_4d6_drop_lowest(&mut rng);
        sheet.attributes.dexterity = roll_4d6_drop_lowest(&mut rng);
        sheet.attributes.constitution = roll_4d6_drop_lowest(&mut rng);
        sheet.attributes.intelligence = roll_4d6_drop_lowest(&mut rng);
        sheet.attributes.wisdom = roll_4d6_drop_lowest(&mut rng);
        sheet.attributes.charisma = roll_4d6_drop_lowest(&mut rng);

        // Update modifiers based on new attribute values
        sheet.modifiers.strength = Attributes::calculate_modifier(sheet.attributes.strength);
        sheet.modifiers.dexterity = Attributes::calculate_modifier(sheet.attributes.dexterity);
        sheet.modifiers.constitution =
            Attributes::calculate_modifier(sheet.attributes.constitution);
        sheet.modifiers.intelligence =
            Attributes::calculate_modifier(sheet.attributes.intelligence);
        sheet.modifiers.wisdom = Attributes::calculate_modifier(sheet.attributes.wisdom);
        sheet.modifiers.charisma = Attributes::calculate_modifier(sheet.attributes.charisma);

        character_data.is_modified = true;
    }
}

/// Update the modified indicator in character list
pub fn update_character_list_modified_indicator(
    character_manager: Res<CharacterManager>,
    character_data: Res<CharacterData>,
    mut text_query: Query<(&CharacterListItemText, &mut Text)>,
) {
    if !character_data.is_changed() && !character_manager.is_changed() {
        return;
    }

    for (item_text, mut text) in text_query.iter_mut() {
        if let Some(entry) = character_manager.characters.get(item_text.index) {
            let is_current = character_manager
                .current_character_id
                .map(|id| id == entry.id)
                .unwrap_or(false);

            let display_name = if is_current && character_data.is_modified {
                format!("{}*", item_text.base_name)
            } else {
                item_text.base_name.clone()
            };

            *text = Text::new(display_name);
        }
    }
}
