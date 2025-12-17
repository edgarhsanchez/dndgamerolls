//! Event handlers for the character screen
//!
//! This module contains all the input handlers, scroll handlers,
//! text editing handlers, and other event systems.

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_material_ui::icons::MaterialIconFont;
use bevy_material_ui::prelude::*;

use crate::dice3d::types::*;
use super::*;

// ============================================================================
// UI Fixups
// ============================================================================

/// Ensure any entity tagged with Bevy's `Button` has the required interaction components.
///
/// Many parts of the character UI spawn `Button` directly (instead of `ButtonBundle`).
/// Without `Interaction` / `FocusPolicy`, clicks won't register and handlers will appear
/// to "do nothing".
pub fn ensure_buttons_have_interaction(
    mut commands: Commands,
    missing_interaction: Query<Entity, (With<Button>, Without<Interaction>)>,
    missing_focus_policy: Query<Entity, (With<Button>, Without<FocusPolicy>)>,
) {
    for entity in missing_interaction.iter() {
        commands.entity(entity).insert(Interaction::None);
    }

    for entity in missing_focus_policy.iter() {
        commands.entity(entity).insert(FocusPolicy::Block);
    }
}

// ============================================================================
// Scroll Handling
// ============================================================================

/// Handle mouse wheel scrolling for the character stats panel
pub fn handle_scroll_input(
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut scrollable_query: Query<(&mut Node, &ComputedNode, &ChildOf), With<ScrollableContent>>,
    parent_query: Query<&ComputedNode>,
    ui_state: Res<UiState>,
    mut info_scroll_query: Query<
        (&mut Node, &ComputedNode, &ChildOf),
        (With<InfoScrollContent>, Without<ScrollableContent>),
    >,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    // Only scroll when on character sheet or info tab
    if ui_state.active_tab != AppTab::CharacterSheet && ui_state.active_tab != AppTab::DndInfo {
        return;
    }

    let scroll_speed = 30.0;
    let mut scroll_delta = 0.0;

    for event in mouse_wheel.read() {
        scroll_delta += event.y * scroll_speed;
    }

    if scroll_delta == 0.0 {
        return;
    }

    // Handle character sheet scrolling
    if ui_state.active_tab == AppTab::CharacterSheet {
        for (mut node, computed, child_of) in scrollable_query.iter_mut() {
            if let Ok(parent_computed) = parent_query.get(child_of.parent()) {
                let content_height = computed.size().y;
                let container_height = parent_computed.size().y;
                let max_scroll = (content_height - container_height).max(0.0);

                let current_top = match node.top {
                    Val::Px(px) => px,
                    _ => 0.0,
                };

                let new_top = (current_top + scroll_delta).clamp(-max_scroll, 0.0);
                node.top = Val::Px(new_top);
            }
        }
    }

    // Handle info screen scrolling
    if ui_state.active_tab == AppTab::DndInfo {
        for (mut node, computed, child_of) in info_scroll_query.iter_mut() {
            if let Ok(parent_computed) = parent_query.get(child_of.parent()) {
                let content_height = computed.size().y;
                let container_height = parent_computed.size().y;
                let max_scroll = (content_height - container_height).max(0.0);

                let current_top = match node.top {
                    Val::Px(px) => px,
                    _ => 0.0,
                };

                let new_top = (current_top + scroll_delta).clamp(-max_scroll, 0.0);
                node.top = Val::Px(new_top);
            }
        }
    }
}

// ============================================================================
// Field Click Handlers
// ============================================================================

/// Handle clicking on stat fields to start editing
pub fn handle_stat_field_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    stat_fields: Query<&StatField>,
    mut text_input: ResMut<TextInputState>,
    character_data: Res<CharacterData>,
    edit_state: Res<GroupEditState>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(stat_field) = stat_fields.get(event.entity) else {
            continue;
        };
            // Don't allow editing values while the group is in edit mode
            let is_group_editing = match &stat_field.field {
                EditingField::CharacterName
                | EditingField::CharacterClass
                | EditingField::CharacterRace
                | EditingField::CharacterLevel
                | EditingField::CustomBasicInfo(_)
                | EditingField::CustomBasicInfoLabel(_) => {
                    edit_state.editing_groups.contains(&GroupType::BasicInfo)
                }
                EditingField::AttributeStrength
                | EditingField::AttributeDexterity
                | EditingField::AttributeConstitution
                | EditingField::AttributeIntelligence
                | EditingField::AttributeWisdom
                | EditingField::AttributeCharisma
                | EditingField::CustomAttribute(_)
                | EditingField::CustomAttributeLabel(_) => {
                    edit_state.editing_groups.contains(&GroupType::Attributes)
                }
                EditingField::ArmorClass
                | EditingField::Initiative
                | EditingField::Speed
                | EditingField::HitPointsCurrent
                | EditingField::HitPointsMaximum
                | EditingField::ProficiencyBonus
                | EditingField::CustomCombat(_)
                | EditingField::CustomCombatLabel(_) => {
                    edit_state.editing_groups.contains(&GroupType::Combat)
                }
                EditingField::SavingThrow(_) | EditingField::SavingThrowLabel(_) => {
                    edit_state.editing_groups.contains(&GroupType::SavingThrows)
                }
                EditingField::Skill(_) | EditingField::SkillLabel(_) => {
                    edit_state.editing_groups.contains(&GroupType::Skills)
                }
            };

            if is_group_editing {
                continue; // Skip value editing when in group edit mode
            }

            // Get current value
            let current_value = get_field_value(&character_data, &stat_field.field);
            
            text_input.active_field = Some(stat_field.field.clone());
            text_input.current_text = current_value.clone();
            text_input.cursor_position = current_value.len();
    }
}

/// Handle clicking on editable labels
pub fn handle_label_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    buttons: Query<&EditableLabelButton>,
    mut text_input: ResMut<TextInputState>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(label_button) = buttons.get(event.entity) else { continue };
        text_input.active_field = Some(label_button.field.clone());
        text_input.current_text = label_button.current_name.clone();
        text_input.cursor_position = label_button.current_name.len();
    }
}

// ============================================================================
// Text Input Handling
// ============================================================================

/// Handle keyboard input for text editing
pub fn handle_text_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut char_events: MessageReader<KeyboardInput>,
    mut text_input: ResMut<TextInputState>,
    mut character_data: ResMut<CharacterData>,
    ui_state: Res<UiState>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    // Only process when on character sheet
    if ui_state.active_tab != AppTab::CharacterSheet {
        return;
    }

    let Some(active_field) = text_input.active_field.clone() else {
        return;
    };

    // Handle escape to cancel editing
    if keyboard.just_pressed(KeyCode::Escape) {
        text_input.active_field = None;
        text_input.current_text.clear();
        return;
    }

    // Handle enter to confirm editing
    if keyboard.just_pressed(KeyCode::Enter) {
        let current_text = text_input.current_text.clone();
        let _ = apply_field_value(&mut character_data, &mut text_input, &active_field, &current_text);
        text_input.active_field = None;
        text_input.current_text.clear();
        return;
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) && !text_input.current_text.is_empty() {
        text_input.current_text.pop();
        return;
    }

    // Check if shift is pressed for uppercase letters
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Handle character input
    for event in char_events.read() {
        if event.state.is_pressed() {
            if let Some(key_code) = event.key_code.to_char(shift_pressed) {
                // Validate character based on field type
                let valid = match &active_field {
                    EditingField::CharacterName => key_code.is_alphanumeric() || key_code == ' ',
                    EditingField::CharacterLevel
                    | EditingField::AttributeStrength
                    | EditingField::AttributeDexterity
                    | EditingField::AttributeConstitution
                    | EditingField::AttributeIntelligence
                    | EditingField::AttributeWisdom
                    | EditingField::AttributeCharisma
                    | EditingField::ArmorClass
                    | EditingField::Speed
                    | EditingField::HitPointsCurrent
                    | EditingField::HitPointsMaximum => key_code.is_ascii_digit() || key_code == '-',
                    EditingField::Initiative
                    | EditingField::ProficiencyBonus
                    | EditingField::Skill(_)
                    | EditingField::SavingThrow(_) => {
                        key_code.is_ascii_digit() || key_code == '-' || key_code == '+'
                    }
                    _ => true,
                };

                if valid {
                    text_input.current_text.push(key_code);
                }
            }
        }
    }
}

/// Helper trait to convert KeyCode to char
trait KeyCodeToChar {
    fn to_char(&self, shift_pressed: bool) -> Option<char>;
}

impl KeyCodeToChar for KeyCode {
    fn to_char(&self, shift_pressed: bool) -> Option<char> {
        let base_char = match self {
            KeyCode::KeyA => Some('a'),
            KeyCode::KeyB => Some('b'),
            KeyCode::KeyC => Some('c'),
            KeyCode::KeyD => Some('d'),
            KeyCode::KeyE => Some('e'),
            KeyCode::KeyF => Some('f'),
            KeyCode::KeyG => Some('g'),
            KeyCode::KeyH => Some('h'),
            KeyCode::KeyI => Some('i'),
            KeyCode::KeyJ => Some('j'),
            KeyCode::KeyK => Some('k'),
            KeyCode::KeyL => Some('l'),
            KeyCode::KeyM => Some('m'),
            KeyCode::KeyN => Some('n'),
            KeyCode::KeyO => Some('o'),
            KeyCode::KeyP => Some('p'),
            KeyCode::KeyQ => Some('q'),
            KeyCode::KeyR => Some('r'),
            KeyCode::KeyS => Some('s'),
            KeyCode::KeyT => Some('t'),
            KeyCode::KeyU => Some('u'),
            KeyCode::KeyV => Some('v'),
            KeyCode::KeyW => Some('w'),
            KeyCode::KeyX => Some('x'),
            KeyCode::KeyY => Some('y'),
            KeyCode::KeyZ => Some('z'),
            KeyCode::Digit0 | KeyCode::Numpad0 => Some('0'),
            KeyCode::Digit1 | KeyCode::Numpad1 => Some('1'),
            KeyCode::Digit2 | KeyCode::Numpad2 => Some('2'),
            KeyCode::Digit3 | KeyCode::Numpad3 => Some('3'),
            KeyCode::Digit4 | KeyCode::Numpad4 => Some('4'),
            KeyCode::Digit5 | KeyCode::Numpad5 => Some('5'),
            KeyCode::Digit6 | KeyCode::Numpad6 => Some('6'),
            KeyCode::Digit7 | KeyCode::Numpad7 => Some('7'),
            KeyCode::Digit8 | KeyCode::Numpad8 => Some('8'),
            KeyCode::Digit9 | KeyCode::Numpad9 => Some('9'),
            KeyCode::Space => Some(' '),
            KeyCode::Minus | KeyCode::NumpadSubtract => Some('-'),
            KeyCode::Equal | KeyCode::NumpadAdd => {
                if shift_pressed { Some('+') } else { Some('=') }
            }
            _ => None,
        };

        base_char.map(|c| if shift_pressed && c.is_ascii_lowercase() { c.to_ascii_uppercase() } else { c })
    }
}

// ============================================================================
// Display Update Systems
// ============================================================================

/// Update the display of currently editing fields
pub fn update_editing_display(
    text_input: Res<TextInputState>,
    mut field_values: Query<(&StatFieldValue, &mut Text, &mut TextColor)>,
    mut label_texts: Query<(&EditableLabelText, &mut Text, &mut TextColor), Without<StatFieldValue>>,
) {
    if !text_input.is_changed() {
        return;
    }

    // Update stat field values
    for (field_value, mut text, mut color) in field_values.iter_mut() {
        if text_input.active_field.as_ref() == Some(&field_value.field) {
            // Show current input with cursor
            let display = if text_input.current_text.is_empty() {
                "_".to_string()
            } else {
                format!("{}|", text_input.current_text)
            };
            *text = Text::new(display);
            color.0 = MD3_PRIMARY;
        }
    }

    // Update label texts
    for (label_text, mut text, mut color) in label_texts.iter_mut() {
        if text_input.active_field.as_ref() == Some(&label_text.field) {
            let display = if text_input.current_text.is_empty() {
                "_".to_string()
            } else {
                format!("{}|", text_input.current_text)
            };
            *text = Text::new(display);
            color.0 = MD3_PRIMARY;
        }
    }
}

// ============================================================================
// Group Editing Handlers
// ============================================================================

/// Handle clicking on group edit buttons
pub fn handle_group_edit_toggle(
    mut click_events: MessageReader<IconButtonClickEvent>,
    button_query: Query<&GroupEditButton>,
    mut edit_state: ResMut<GroupEditState>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(button) = button_query.get(event.entity) else { continue };
        if edit_state.editing_groups.contains(&button.group_type) {
            edit_state.editing_groups.remove(&button.group_type);
        } else {
            edit_state.editing_groups.insert(button.group_type.clone());
        }
    }
}

/// Handle clicking on group add buttons
pub fn handle_group_add_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    buttons: Query<&GroupAddButton>,
    mut adding_state: ResMut<AddingEntryState>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(button) = buttons.get(event.entity) else { continue };
        adding_state.adding_to = Some(button.group_type.clone());
        adding_state.new_entry_name.clear();
        adding_state.new_entry_value.clear();
    }
}

/// Handle confirming a new entry
pub fn handle_new_entry_confirm(
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<&NewEntryConfirmButton>,
    mut adding_state: ResMut<AddingEntryState>,
    mut character_data: ResMut<CharacterData>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(button) = buttons.get(event.entity) else { continue };

        if !adding_state.new_entry_name.is_empty() {
            // Add the new entry to the character sheet
            if let Some(sheet) = &mut character_data.sheet {
                match &button.group_type {
                    GroupType::BasicInfo => {
                        sheet.custom_basic_info.insert(
                            adding_state.new_entry_name.clone(),
                            adding_state.new_entry_value.clone(),
                        );
                    }
                    GroupType::Attributes => {
                        sheet.custom_attributes.insert(
                            adding_state.new_entry_name.clone(),
                            10, // Default attribute value
                        );
                    }
                    GroupType::Combat => {
                        sheet.custom_combat.insert(
                            adding_state.new_entry_name.clone(),
                            adding_state.new_entry_value.clone(),
                        );
                    }
                    GroupType::SavingThrows => {
                        sheet.saving_throws.insert(
                            adding_state.new_entry_name.clone(),
                            SavingThrow { modifier: 0, proficient: false },
                        );
                    }
                    GroupType::Skills => {
                        sheet.skills.insert(
                            adding_state.new_entry_name.clone(),
                            Skill {
                                modifier: 0,
                                proficient: false,
                                expertise: None,
                                proficiency_type: None,
                            },
                        );
                    }
                }
                character_data.is_modified = true;
            }
        }

        adding_state.adding_to = None;
        adding_state.new_entry_name.clear();
        adding_state.new_entry_value.clear();
    }
}

/// Handle canceling a new entry
pub fn handle_new_entry_cancel(
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<(), With<NewEntryCancelButton>>,
    mut adding_state: ResMut<AddingEntryState>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        if buttons.get(event.entity).is_err() {
            continue;
        }
        adding_state.adding_to = None;
        adding_state.new_entry_name.clear();
        adding_state.new_entry_value.clear();
    }
}

/// Handle text input for new entry name
pub fn handle_new_entry_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut char_events: MessageReader<KeyboardInput>,
    mut adding_state: ResMut<AddingEntryState>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    if adding_state.adding_to.is_none() {
        return;
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        adding_state.new_entry_name.pop();
        return;
    }

    // Handle character input
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    for event in char_events.read() {
        if event.state.is_pressed() {
            if let Some(c) = event.key_code.to_char(shift_pressed) {
                if c.is_alphanumeric() || c == ' ' {
                    adding_state.new_entry_name.push(c);
                }
            }
        }
    }
}

/// Update the display of new entry input
pub fn update_new_entry_input_display(
    adding_state: Res<AddingEntryState>,
    input_query: Query<(&NewEntryInput, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    if !adding_state.is_changed() {
        return;
    }

    for (input, children) in input_query.iter() {
        if adding_state.adding_to.as_ref() == Some(&input.group_type) {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    let display = if adding_state.new_entry_name.is_empty() {
                        "Type name...".to_string()
                    } else {
                        format!("{}|", adding_state.new_entry_name)
                    };
                    *text = Text::new(display);
                }
            }
        }
    }
}

// ============================================================================
// Delete Handler
// ============================================================================

/// Handle clicking on delete buttons
pub fn handle_delete_click(
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<&DeleteEntryButton>,
    mut character_data: ResMut<CharacterData>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(button) = buttons.get(event.entity) else { continue };
        if let Some(sheet) = &mut character_data.sheet {
            match &button.group_type {
                GroupType::BasicInfo => {
                    sheet.custom_basic_info.remove(&button.entry_id);
                }
                GroupType::Attributes => {
                    sheet.custom_attributes.remove(&button.entry_id);
                }
                GroupType::Combat => {
                    sheet.custom_combat.remove(&button.entry_id);
                }
                GroupType::SavingThrows => {
                    sheet.saving_throws.remove(&button.entry_id);
                }
                GroupType::Skills => {
                    sheet.skills.remove(&button.entry_id);
                }
            }
            character_data.is_modified = true;
        }
    }
}

// ============================================================================
// Save Handler
// ============================================================================

/// Handle save button clicks
pub fn handle_save_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    buttons: Query<(), With<SaveButton>>,
    mut text_input: ResMut<TextInputState>,
    mut character_data: ResMut<CharacterData>,
    mut character_manager: ResMut<CharacterManager>,
    db: Res<CharacterDatabase>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        if buttons.get(event.entity).is_err() {
            continue;
        }

        if character_data.sheet.is_some() {
            // If the user is currently typing (hasn't pressed Enter), commit the buffer
            // so Save persists what they see on screen.
            if let Some(active_field) = text_input.active_field.clone() {
                let current_text = text_input.current_text.clone();
                let _ = apply_field_value(
                    &mut character_data,
                    &mut text_input,
                    &active_field,
                    &current_text,
                );
                text_input.active_field = None;
                text_input.current_text.clear();
            }
        }

        if let Some(sheet) = &character_data.sheet {
            match character_manager.current_character_id {
                Some(char_id) => {
                    if let Err(err) = db.save_character(Some(char_id), sheet) {
                        bevy::log::warn!("Failed to save character {char_id}: {err}");
                    } else {
                        character_data.is_modified = false;
                    }
                }
                None => match db.save_character(None, sheet) {
                    Ok(new_id) => {
                        // Refresh list and select the newly-created character
                        if let Ok(chars) = db.list_characters() {
                            character_manager.characters = chars;
                        }
                        character_manager.current_character_id = Some(new_id);
                        character_manager.list_version += 1;
                        character_data.is_modified = false;
                    }
                    Err(err) => {
                        bevy::log::warn!("Failed to create character: {err}");
                    }
                },
            }
        }
    }
}

/// Update save button appearance based on modified state
pub fn update_save_button_appearance(
    character_data: Res<CharacterData>,
    text_input: Res<TextInputState>,
    mut query: Query<&mut bevy_material_ui::prelude::MaterialButton, With<SaveButton>>,
) {
    if !character_data.is_changed() && !text_input.is_changed() {
        return;
    }

    let has_pending_text_edits = text_input
        .active_field
        .as_ref()
        .is_some_and(|field| get_field_value(&character_data, field) != text_input.current_text);

    for mut button in query.iter_mut() {
        // When not modified, disable the button so MD3 styling renders it as inactive.
        button.disabled = !(character_data.is_modified || has_pending_text_edits);
    }
}

// ============================================================================
// Roll Attribute Handler
// ============================================================================

/// Handle clicking on attribute roll buttons
pub fn handle_roll_attribute_click(
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<&RollAttributeButton>,
    character_data: Res<CharacterData>,
    character_manager: Res<CharacterManager>,
    mut bridge: ResMut<CharacterScreenRollBridge>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dice_config: ResMut<DiceConfig>,
    mut dice_results: ResMut<DiceResults>,
    mut roll_state: ResMut<RollState>,
    mut ui_state: ResMut<UiState>,
    mut snackbar: MessageWriter<ShowSnackbar>,
    dice_query: Query<Entity, With<Die>>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    // Only allow these dice buttons on the character sheet screen.
    if ui_state.active_tab != AppTab::CharacterSheet {
        return;
    }

    for event in click_events.read() {
        let Ok(button) = buttons.get(event.entity) else { continue };
        if let Some(sheet) = &character_data.sheet {
            // Get the modifier for this attribute
            let modifier = match button.attribute.to_lowercase().as_str() {
                "strength" => sheet.modifiers.strength,
                "dexterity" => sheet.modifiers.dexterity,
                "constitution" => sheet.modifiers.constitution,
                "intelligence" => sheet.modifiers.intelligence,
                "wisdom" => sheet.modifiers.wisdom,
                "charisma" => sheet.modifiers.charisma,
                _ => {
                    // Check custom attributes
                    sheet.custom_attributes
                        .get(&button.attribute)
                        .map(|&score| Attributes::calculate_modifier(score))
                        .unwrap_or(0)
                }
            };

            let die_type = settings_state
                .settings
                .character_sheet_default_die
                .to_dice_type();

            start_character_sheet_roll(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut dice_config,
                &mut dice_results,
                &mut roll_state,
                &mut ui_state,
                &mut bridge,
                &character_manager,
                &dice_query,
                die_type,
                modifier,
                format!("{} Check", button.attribute),
                CharacterScreenRollTarget::Attribute(button.attribute.clone()),
            );

            snackbar.write(ShowSnackbar::message("Dice roll started").duration(2.0));
        }
    }
}

/// Handle clicking on skill roll buttons
pub fn handle_roll_skill_click(
    mut click_events: MessageReader<IconButtonClickEvent>,
    buttons: Query<&RollSkillButton>,
    character_data: Res<CharacterData>,
    character_manager: Res<CharacterManager>,
    mut bridge: ResMut<CharacterScreenRollBridge>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dice_config: ResMut<DiceConfig>,
    mut dice_results: ResMut<DiceResults>,
    mut roll_state: ResMut<RollState>,
    mut ui_state: ResMut<UiState>,
    mut snackbar: MessageWriter<ShowSnackbar>,
    dice_query: Query<Entity, With<Die>>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    if ui_state.active_tab != AppTab::CharacterSheet {
        return;
    }

    for event in click_events.read() {
        let Ok(button) = buttons.get(event.entity) else { continue };
        let Some(sheet) = &character_data.sheet else { continue };

        let modifier = sheet
            .skills
            .get(&button.skill)
            .map(|s| s.modifier)
            .unwrap_or(0);

        let die_type = settings_state
            .settings
            .character_sheet_default_die
            .to_dice_type();

        start_character_sheet_roll(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut dice_config,
            &mut dice_results,
            &mut roll_state,
            &mut ui_state,
            &mut bridge,
            &character_manager,
            &dice_query,
            die_type,
            modifier,
            format!("{} Skill", button.skill),
            CharacterScreenRollTarget::Skill(button.skill.clone()),
        );

        snackbar.write(ShowSnackbar::message("Dice roll started").duration(2.0));
    }
}

fn start_character_sheet_roll(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    dice_config: &mut ResMut<DiceConfig>,
    dice_results: &mut ResMut<DiceResults>,
    roll_state: &mut ResMut<RollState>,
    ui_state: &mut ResMut<UiState>,
    bridge: &mut ResMut<CharacterScreenRollBridge>,
    character_manager: &CharacterManager,
    dice_query: &Query<Entity, With<Die>>,
    die_type: DiceType,
    modifier: i32,
    modifier_name: String,
    target: CharacterScreenRollTarget,
) {
    // Remove old dice
    for entity in dice_query.iter() {
        commands.entity(entity).despawn();
    }

    // Update config
    dice_config.dice_to_roll.clear();
    dice_config.dice_to_roll.push(die_type);
    dice_config.modifier = modifier;
    dice_config.modifier_name = modifier_name;
    dice_results.results.clear();

    // Spawn new dice
    let position = super::super::calculate_dice_position(0, 1);
    let _die_entity = super::super::spawn_die(commands, meshes, materials, die_type, position);

    // Mark as rolling
    roll_state.rolling = true;

    // Bridge: remember what to write back
    bridge.pending = Some(target);
    bridge.last_character_id = character_manager.current_character_id;

    // Switch to dice roller so the user can see the roll.
    ui_state.active_tab = AppTab::DiceRoller;
}

/// When a dice roll finishes, record the final total for any pending character-screen roll.
pub fn record_character_screen_roll_on_settle(
    roll_state: Res<RollState>,
    dice_results: Res<DiceResults>,
    mut character_data: ResMut<CharacterData>,
    character_manager: Res<CharacterManager>,
    mut bridge: ResMut<CharacterScreenRollBridge>,
    mut was_rolling: Local<bool>,
) {
    let finished_this_frame = *was_rolling && !roll_state.rolling;
    *was_rolling = roll_state.rolling;

    if !finished_this_frame {
        return;
    }

    if dice_results.results.is_empty() {
        return;
    }

    let Some(target) = bridge.pending.clone() else {
        return;
    };

    // Only apply if we're still on the same character.
    if bridge.last_character_id != character_manager.current_character_id {
        bridge.pending = None;
        return;
    }

    let dice_total: i32 = dice_results
        .results
        .iter()
        .map(|(_, v)| *v as i32)
        .sum();

    match target {
        CharacterScreenRollTarget::Attribute(attr) => {
            // Apply dice-only total to the base attribute score.
            if let Some(sheet) = character_data.sheet.as_mut() {
                match attr.to_lowercase().as_str() {
                    "strength" => {
                        sheet.attributes.strength = dice_total;
                        sheet.modifiers.strength = Attributes::calculate_modifier(dice_total);
                    }
                    "dexterity" => {
                        sheet.attributes.dexterity = dice_total;
                        sheet.modifiers.dexterity = Attributes::calculate_modifier(dice_total);
                    }
                    "constitution" => {
                        sheet.attributes.constitution = dice_total;
                        sheet.modifiers.constitution = Attributes::calculate_modifier(dice_total);
                    }
                    "intelligence" => {
                        sheet.attributes.intelligence = dice_total;
                        sheet.modifiers.intelligence =
                            Attributes::calculate_modifier(dice_total);
                    }
                    "wisdom" => {
                        sheet.attributes.wisdom = dice_total;
                        sheet.modifiers.wisdom = Attributes::calculate_modifier(dice_total);
                    }
                    "charisma" => {
                        sheet.attributes.charisma = dice_total;
                        sheet.modifiers.charisma = Attributes::calculate_modifier(dice_total);
                    }
                    _ => {
                        // Custom attribute: store as a score.
                        sheet.custom_attributes.insert(attr.clone(), dice_total);
                    }
                }

                character_data.is_modified = true;
                character_data.needs_refresh = true;
            }

            // Keep the roll-result text in sync (store dice-only total).
            bridge.last_attribute_totals.insert(attr, dice_total);
        }
        CharacterScreenRollTarget::Skill(skill) => {
            // Apply dice-only total to the base skill modifier.
            if let Some(sheet) = character_data.sheet.as_mut() {
                if let Some(sk) = sheet.skills.get_mut(&skill) {
                    sk.modifier = dice_total;
                } else {
                    sheet.skills.insert(
                        skill.clone(),
                        Skill {
                            modifier: dice_total,
                            ..Default::default()
                        },
                    );
                }

                character_data.is_modified = true;
                character_data.needs_refresh = true;
            }

            // Keep the roll-result text in sync (store dice-only total).
            bridge.last_skill_totals.insert(skill, dice_total);
        }
    }

    bridge.pending = None;
}

/// Sync the stored character-screen roll totals into the corresponding text nodes.
pub fn sync_character_screen_roll_result_texts(
    bridge: Res<CharacterScreenRollBridge>,
    mut texts: ParamSet<(
        Query<(&AttributeRollResultText, &mut Text)>,
        Query<(&SkillRollResultText, &mut Text)>,
    )>,
) {
    if !bridge.is_changed() {
        return;
    }

    for (marker, mut text) in texts.p0().iter_mut() {
        let value = bridge.last_attribute_totals.get(&marker.attribute);
        **text = value.map(|v| format!("Last: {}", v)).unwrap_or_default();
    }

    for (marker, mut text) in texts.p1().iter_mut() {
        let value = bridge.last_skill_totals.get(&marker.skill);
        **text = value.map(|v| format!("Last: {}", v)).unwrap_or_default();
    }
}

// ============================================================================
// Character Sheet Dice Settings Modal
// ============================================================================

/// Handle clicking the character sheet settings (gear) button.
pub fn handle_character_sheet_settings_button_click(
    mut click_events: MessageReader<IconButtonClickEvent>,
    button_query: Query<(), With<CharacterSheetSettingsButton>>,
    ui_state: Res<UiState>,
    mut settings_state: ResMut<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    if ui_state.active_tab != AppTab::CharacterSheet {
        return;
    }

    for event in click_events.read() {
        if button_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.show_modal = true;
        settings_state.modal_kind = ActiveModalKind::CharacterSheetDiceSettings;
        settings_state.character_sheet_editing_die = settings_state.settings.character_sheet_default_die;
    }
}

/// Spawn/despawn the character sheet dice settings modal.
pub fn manage_character_sheet_settings_modal(
    mut commands: Commands,
    settings_state: Res<SettingsState>,
    theme: Res<MaterialTheme>,
    modal_query: Query<Entity, With<CharacterSheetSettingsModalOverlay>>,
    children_query: Query<&Children>,
) {
    if !settings_state.is_changed() {
        return;
    }

    let should_show = settings_state.show_modal
        && settings_state.modal_kind == ActiveModalKind::CharacterSheetDiceSettings;

    if should_show {
        if modal_query.is_empty() {
            spawn_character_sheet_settings_modal(&mut commands, &theme, &settings_state);
        }
    } else {
        for entity in modal_query.iter() {
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_character_sheet_settings_modal(
    commands: &mut Commands,
    theme: &MaterialTheme,
    settings_state: &SettingsState,
) {
    let options = [
        DiceTypeSetting::D4,
        DiceTypeSetting::D6,
        DiceTypeSetting::D8,
        DiceTypeSetting::D10,
        DiceTypeSetting::D12,
        DiceTypeSetting::D20,
    ];

    let selected_index = options
        .iter()
        .position(|d| *d == settings_state.character_sheet_editing_die)
        .unwrap_or(5);

    let select_options: Vec<SelectOption> = options
        .iter()
        .map(|d| SelectOption::new(d.label()).value(d.label()))
        .collect();

    let dialog_entity = commands
        .spawn((
            DialogBuilder::new()
                .title("Character Sheet Settings")
                .open()
                .modal(true)
                .build(theme),
            CharacterSheetSettingsModal,
        ))
        .id();

    let scrim_entity = commands
        .spawn((
            create_dialog_scrim_for(theme, dialog_entity, true),
            CharacterSheetSettingsModalOverlay,
        ))
        .id();

    commands.entity(scrim_entity).add_child(dialog_entity);

    commands.entity(dialog_entity).with_children(|dialog| {
        dialog
            .spawn(Node {
                width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                min_height: Val::Px(260.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|content| {
                content.spawn((
                    Text::new("Character Sheet Settings"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                ));

                content.spawn((
                    Text::new("Default die for character-sheet rolls"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));

                // Die type select
                content.spawn(Node::default()).with_children(|slot| {
                    let builder = SelectBuilder::new(select_options)
                        .outlined()
                        .label("Default die")
                        .selected(selected_index)
                        .width(Val::Px(210.0));
                    slot.spawn_select_with(theme, builder);
                });

                // Spacer
                content.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });

                // Buttons row
                content
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(10.0),
                        width: Val::Percent(100.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        // Cancel
                        buttons
                            .spawn(Node {
                                width: Val::Px(100.0),
                                height: Val::Px(36.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                slot.spawn((
                                    MaterialButtonBuilder::new("Cancel")
                                        .outlined()
                                        .build(theme),
                                    CharacterSheetSettingsCancelButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Cancel"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(theme.primary),
                                        ButtonLabel,
                                    ));
                                });
                            });

                        // Save
                        buttons
                            .spawn(Node {
                                width: Val::Px(100.0),
                                height: Val::Px(36.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                slot.spawn((
                                    MaterialButtonBuilder::new("Save").filled().build(theme),
                                    CharacterSheetSettingsSaveButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Save"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(theme.on_primary),
                                        ButtonLabel,
                                    ));
                                });
                            });
                    });
            });
    });
}

/// Handle selection changes in the character sheet settings modal.
pub fn handle_character_sheet_die_type_select_change(
    mut events: MessageReader<SelectChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == ActiveModalKind::CharacterSheetDiceSettings)
    {
        return;
    }

    let options = [
        DiceTypeSetting::D4,
        DiceTypeSetting::D6,
        DiceTypeSetting::D8,
        DiceTypeSetting::D10,
        DiceTypeSetting::D12,
        DiceTypeSetting::D20,
    ];

    for event in events.read() {
        if let Some(setting) = options.get(event.index).copied() {
            settings_state.character_sheet_editing_die = setting;
        }
    }
}

/// Handle Save click for character sheet settings.
pub fn handle_character_sheet_settings_save_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    button_query: Query<(), With<CharacterSheetSettingsSaveButton>>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == ActiveModalKind::CharacterSheetDiceSettings)
    {
        return;
    }

    for event in click_events.read() {
        if button_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.settings.character_sheet_default_die = settings_state.character_sheet_editing_die;
        let _ = settings_state.settings.save();
        settings_state.show_modal = false;
        settings_state.modal_kind = ActiveModalKind::None;
    }
}

/// Handle Cancel click for character sheet settings.
pub fn handle_character_sheet_settings_cancel_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    button_query: Query<(), With<CharacterSheetSettingsCancelButton>>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == ActiveModalKind::CharacterSheetDiceSettings)
    {
        return;
    }

    for event in click_events.read() {
        if button_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.show_modal = false;
        settings_state.modal_kind = ActiveModalKind::None;
    }
}

/// Handle expertise toggle clicks
pub fn handle_expertise_toggle(
    interaction_query: Query<(&Interaction, &ExpertiseCheckbox), Changed<Interaction>>,
    mut character_data: ResMut<CharacterData>,
    settings_state: Res<SettingsState>,
) {
    if settings_state.show_modal {
        return;
    }

    for (interaction, checkbox) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(sheet) = &mut character_data.sheet {
                if let Some(skill) = sheet.skills.get_mut(&checkbox.skill_name) {
                    // Toggle expertise: None/Some(false) -> Some(true), Some(true) -> Some(false)
                    skill.expertise = Some(!skill.expertise.unwrap_or(false));
                    character_data.is_modified = true;
                }
            }
        }
    }
}

// ============================================================================
// Rebuild Systems
// ============================================================================

/// Rebuild character list panel when character list changes
pub fn rebuild_character_list_on_change(
    mut commands: Commands,
    character_manager: Res<CharacterManager>,
    character_data: Res<CharacterData>,
    icon_assets: Res<IconAssets>,
    icon_font: Res<MaterialIconFont>,
    theme: Option<Res<MaterialTheme>>,
    screen_root: Query<Entity, With<CharacterScreenRoot>>,
    list_panel: Query<Entity, With<CharacterListPanel>>,
    children_query: Query<&Children>,
) {
    if !character_manager.is_changed() {
        return;
    }

    let Some(root) = screen_root.iter().next() else {
        return;
    };

    fn despawn_recursive(commands: &mut Commands, entity: Entity, children_query: &Query<&Children>) {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                despawn_recursive(commands, child, children_query);
            }
        }
        commands.entity(entity).despawn();
    }

    // Despawn existing panel (and its subtree)
    for entity in list_panel.iter() {
        despawn_recursive(&mut commands, entity, &children_query);
    }

    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let icon_font = icon_font.0.clone();

    // Rebuild the panel immediately so the left-side list stays in sync with the DB.
    commands.entity(root).with_children(|parent| {
        spawn_character_list_panel(
            parent,
            &character_manager,
            &character_data,
            &icon_assets,
            icon_font,
            &theme,
        );
    });
}

/// Rebuild character panel when data changes
pub fn rebuild_character_panel_on_change(
    mut commands: Commands,
    character_manager: Res<CharacterManager>,
    character_data: Res<CharacterData>,
    edit_state: Res<GroupEditState>,
    adding_state: Res<AddingEntryState>,
    icon_assets: Res<IconAssets>,
    icon_font: Res<MaterialIconFont>,
    theme: Option<Res<MaterialTheme>>,
    screen_root: Query<Entity, With<CharacterScreenRoot>>,
    stats_panel: Query<Entity, With<CharacterStatsPanel>>,
    children_query: Query<&Children>,
) {
    if !character_manager.is_changed()
        && !character_data.is_changed()
        && !edit_state.is_changed()
        && !adding_state.is_changed()
    {
        return;
    }

    let Some(root) = screen_root.iter().next() else {
        return;
    };

    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let icon_font = icon_font.0.clone();

    fn despawn_recursive(commands: &mut Commands, entity: Entity, children_query: &Query<&Children>) {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                despawn_recursive(commands, child, children_query);
            }
        }
        commands.entity(entity).despawn();
    }

    // Remove the old stats panel subtree
    for entity in stats_panel.iter() {
        despawn_recursive(&mut commands, entity, &children_query);
    }

    // Recreate the stats panel subtree
    commands.entity(root).with_children(|parent| {
        crate::dice3d::systems::character_screen::tabs::spawn_tabbed_content_panel(
            parent,
            &character_data,
            &character_manager,
            &edit_state,
            &adding_state,
            &icon_assets,
            icon_font,
            &theme,
        );
    });
}

/// Refresh character display when switching characters
pub fn refresh_character_display(
    character_manager: Res<CharacterManager>,
    character_data: Res<CharacterData>,
    // UI refresh logic
) {
    if !character_manager.is_changed() && !character_data.is_changed() {
        return;
    }

    // Refresh logic would update all displayed values
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the current value of a field from character data
fn get_field_value(character_data: &CharacterData, field: &EditingField) -> String {
    let Some(sheet) = &character_data.sheet else {
        return String::new();
    };

    match field {
        EditingField::CharacterName => sheet.character.name.clone(),
        EditingField::CharacterClass => sheet.character.class.clone(),
        EditingField::CharacterRace => sheet.character.race.clone(),
        EditingField::CharacterLevel => sheet.character.level.to_string(),
        EditingField::AttributeStrength => sheet.attributes.strength.to_string(),
        EditingField::AttributeDexterity => sheet.attributes.dexterity.to_string(),
        EditingField::AttributeConstitution => sheet.attributes.constitution.to_string(),
        EditingField::AttributeIntelligence => sheet.attributes.intelligence.to_string(),
        EditingField::AttributeWisdom => sheet.attributes.wisdom.to_string(),
        EditingField::AttributeCharisma => sheet.attributes.charisma.to_string(),
        EditingField::ArmorClass => sheet.combat.armor_class.to_string(),
        EditingField::Initiative => format_modifier(sheet.combat.initiative),
        EditingField::Speed => sheet.combat.speed.to_string(),
        EditingField::HitPointsCurrent => sheet.combat.hit_points.as_ref().map(|hp| hp.current.to_string()).unwrap_or_default(),
        EditingField::HitPointsMaximum => sheet.combat.hit_points.as_ref().map(|hp| hp.maximum.to_string()).unwrap_or_default(),
        EditingField::ProficiencyBonus => format!("+{}", sheet.proficiency_bonus),
        EditingField::Skill(name) => sheet.skills.get(name).map(|s| format_modifier(s.modifier)).unwrap_or_default(),
        EditingField::SavingThrow(name) => sheet.saving_throws.get(name).map(|s| format_modifier(s.modifier)).unwrap_or_default(),
        EditingField::CustomBasicInfo(name) => sheet.custom_basic_info.get(name).cloned().unwrap_or_default(),
        EditingField::CustomAttribute(name) => sheet.custom_attributes.get(name).map(|v| v.to_string()).unwrap_or_default(),
        EditingField::CustomCombat(name) => sheet.custom_combat.get(name).cloned().unwrap_or_default(),
        EditingField::SkillLabel(name) | EditingField::SavingThrowLabel(name) 
        | EditingField::CustomBasicInfoLabel(name) | EditingField::CustomAttributeLabel(name) 
        | EditingField::CustomCombatLabel(name) => name.clone(),
    }
}

/// Apply a new value to a field
fn apply_field_value(
    character_data: &mut CharacterData,
    text_input: &mut TextInputState,
    field: &EditingField,
    value: &str,
) -> bool {
    let value = value.trim();
    let before = get_field_value(character_data, field);

    {
        let Some(sheet) = &mut character_data.sheet else {
            return false;
        };

        match field {
            EditingField::CharacterName => sheet.character.name = value.to_string(),
            EditingField::CharacterClass => sheet.character.class = value.to_string(),
            EditingField::CharacterRace => sheet.character.race = value.to_string(),
            EditingField::CharacterLevel => {
                if let Ok(level) = value.parse() {
                    sheet.character.level = level;
                }
            }
        EditingField::AttributeStrength => {
            if let Ok(v) = value.parse() {
                sheet.attributes.strength = v;
                sheet.modifiers.strength = Attributes::calculate_modifier(v);
            }
        }
        EditingField::AttributeDexterity => {
            if let Ok(v) = value.parse() {
                sheet.attributes.dexterity = v;
                sheet.modifiers.dexterity = Attributes::calculate_modifier(v);
            }
        }
        EditingField::AttributeConstitution => {
            if let Ok(v) = value.parse() {
                sheet.attributes.constitution = v;
                sheet.modifiers.constitution = Attributes::calculate_modifier(v);
            }
        }
        EditingField::AttributeIntelligence => {
            if let Ok(v) = value.parse() {
                sheet.attributes.intelligence = v;
                sheet.modifiers.intelligence = Attributes::calculate_modifier(v);
            }
        }
        EditingField::AttributeWisdom => {
            if let Ok(v) = value.parse() {
                sheet.attributes.wisdom = v;
                sheet.modifiers.wisdom = Attributes::calculate_modifier(v);
            }
        }
        EditingField::AttributeCharisma => {
            if let Ok(v) = value.parse() {
                sheet.attributes.charisma = v;
                sheet.modifiers.charisma = Attributes::calculate_modifier(v);
            }
        }
        EditingField::ArmorClass => {
            if let Ok(v) = value.parse() {
                sheet.combat.armor_class = v;
            }
        }
        EditingField::Initiative => {
            if let Ok(v) = parse_modifier(value) {
                sheet.combat.initiative = v;
            }
        }
        EditingField::Speed => {
            if let Ok(v) = value.trim_end_matches(" ft").parse() {
                sheet.combat.speed = v;
            }
        }
        EditingField::HitPointsCurrent => {
            if let Some(hp) = &mut sheet.combat.hit_points {
                if let Ok(v) = value.parse() {
                    hp.current = v;
                }
            }
        }
        EditingField::HitPointsMaximum => {
            if let Some(hp) = &mut sheet.combat.hit_points {
                if let Ok(v) = value.parse() {
                    hp.maximum = v;
                }
            }
        }
        EditingField::ProficiencyBonus => {
            if let Ok(v) = parse_modifier(value) {
                sheet.proficiency_bonus = v;
            }
        }
        EditingField::Skill(name) => {
            if let Some(skill) = sheet.skills.get_mut(name) {
                if let Ok(v) = parse_modifier(value) {
                    skill.modifier = v;
                }
            }
        }
        EditingField::SavingThrow(name) => {
            if let Some(save) = sheet.saving_throws.get_mut(name) {
                if let Ok(v) = parse_modifier(value) {
                    save.modifier = v;
                }
            }
        }
        EditingField::CustomBasicInfo(name) => {
            sheet.custom_basic_info.insert(name.clone(), value.to_string());
        }
        EditingField::CustomAttribute(name) => {
            if let Ok(v) = value.parse() {
                sheet.custom_attributes.insert(name.clone(), v);
            }
        }
            EditingField::CustomCombat(name) => {
                sheet.custom_combat.insert(name.clone(), value.to_string());
            }
            _ => {} // Label fields handled separately
        }
    }

    let after = get_field_value(character_data, field);
    let changed = before != after;
    if changed {
        character_data.is_modified = true;
        text_input.modified_fields.insert(field.clone());
    }

    changed
}

/// Format a modifier value with + prefix for positive numbers
fn format_modifier(value: i32) -> String {
    if value >= 0 {
        format!("+{}", value)
    } else {
        value.to_string()
    }
}

/// Parse a modifier string (handles +/- prefix)
fn parse_modifier(value: &str) -> Result<i32, std::num::ParseIntError> {
    let trimmed = value.trim().trim_start_matches('+');
    trimmed.parse()
}

// ============================================================================
// DnD Info Screen Setup
// ============================================================================

/// Setup the DnD info screen
pub fn setup_dnd_info_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(48.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(MD3_SURFACE),
            Visibility::Hidden,
            DndInfoScreenRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(16.0),
                        ..default()
                    },
                    InfoScrollContent,
                ))
                .with_children(|content| {
                    // Title
                    content.spawn((
                        Text::new("DnD Game Rolls: How Rolls Work (App Guide)"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "This tab documents the exact roll behaviors supported by the app (GUI + CLI) and how they map to common D&D 5e mechanics.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    // -----------------------------------------------------------------
                    // Core Concepts
                    // -----------------------------------------------------------------
                    content.spawn((
                        Text::new("Dice Roller Tab (3D)"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "• Click in the dice box to throw the current dice set.\n• Press R to reset dice positions (no roll).\n• Results panel shows the final values once dice settle (and includes your modifier/label when applicable).\n• Panels like Results / Quick Rolls / Command History can be dragged (positions persist via settings).",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    content.spawn((
                        Text::new("Command Input (GUI)"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "The Command box supports a small, app-specific command format (whitespace-separated):\n• Dice: d20, 2d6, d8, 3d10, etc.\n• Options:\n  - --dice / -d <NdX> (same as writing the dice directly)\n  - --modifier / -m <number> (adds a flat bonus/penalty)\n  - --checkon <name> (pulls the modifier from your Character Sheet by skill/ability/save name)\n\nExamples you can paste:\n• d20\n• 2d6 --modifier 3\n• d20 --checkon stealth\n• d20 --checkon dex --modifier 2\n\nNotes:\n• For multi-word skills, use the Character Sheet's internal name (e.g., sleightOfHand, animalHandling).\n• The GUI command input does NOT use the shorthand '1d20+5' style. Use '--modifier 5' instead.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    content.spawn((
                        Text::new("Command History"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "• The Command History panel shows past commands.\n• Click a history entry to select it and reroll that same command.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    content.spawn((
                        Text::new("Quick Rolls (Dice View)"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "Quick Rolls are one-click rolls powered by your Character Sheet:\n• Skills: rolls 1 die + the skill modifier.\n• Ability Checks: rolls 1 die + the ability modifier.\n• Saving Throws: rolls 1 die + the saving throw modifier.\n\nApp-specific setting: Quick Rolls die type is configurable (Dice Settings → Quick Rolls die type). In standard D&D 5e, checks/saves/attacks are typically a d20 — if you change this, Quick Rolls will no longer match standard rules.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    // -----------------------------------------------------------------
                    // Ability Checks & Skill Checks
                    // -----------------------------------------------------------------
                    content.spawn((
                        Text::new("D&D 5e Mechanics Refresher"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "Core pattern: roll + modifier, then compare.\n• Ability / Skill check: d20 + modifier vs DC\n• Saving throw: d20 + save modifier vs save DC\n• Attack roll: d20 + attack bonus vs AC\n• Damage: roll the damage dice separately (e.g., 1d8+3)",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    // -----------------------------------------------------------------
                    // Saving Throws
                    // -----------------------------------------------------------------
                    content.spawn((
                        Text::new(
                            "Saving throws: d20 + the relevant save modifier vs the effect DC.\n• Concentration checks are Constitution saves: DC 10 or half the damage taken (whichever is higher).\n• Spell save DC reminder (typical 5e): 8 + proficiency bonus + spellcasting ability modifier.\n• Death saves: at 0 HP, roll a d20 at the start of your turn (usually no modifiers). 10+ success, 9- failure; 3 successes stabilize; 3 failures die; natural 20 regains 1 HP; natural 1 counts as two failures.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    // -----------------------------------------------------------------
                    // Attack Rolls & Damage
                    // -----------------------------------------------------------------
                    content.spawn((
                        Text::new(
                            "Criticals reminder (5e core): a natural 20 on an attack roll is a critical hit (roll extra damage dice). A natural 1 on an attack roll is an automatic miss.\nFor ability checks and saving throws, special natural 20/1 outcomes depend on your table rules.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    // -----------------------------------------------------------------
                    // Advantage / Disadvantage
                    // -----------------------------------------------------------------
                    content.spawn((
                        Text::new("Advantage & Disadvantage"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "Advantage/disadvantage is a d20 rule: roll two d20s and keep the higher (adv) or lower (dis).\nApp note: the GUI dice roller does not currently have a built-in advantage toggle; roll twice manually or use the CLI mode flags.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    // -----------------------------------------------------------------
                    // Initiative
                    // -----------------------------------------------------------------
                    content.spawn((
                        Text::new("Initiative"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "Initiative is typically d20 + Dexterity modifier (plus any other initiative bonuses). In this app: roll a d20 (or use d20 --checkon dex) and then add any extra initiative bonuses manually (see Character Sheet → Combat → Initiative).",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));

                    // -----------------------------------------------------------------
                    // How This App Handles Rolls
                    // -----------------------------------------------------------------
                    content.spawn((
                        Text::new("CLI Mode (Optional)"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE),
                    ));

                    content.spawn((
                        Text::new(
                            "If you run the app in CLI mode, you can use dedicated commands for common rolls (and advantage/disadvantage flags):\n• dndgamerolls --cli skill stealth\n• dndgamerolls --cli save dex\n• dndgamerolls --cli --dice 2d6 --modifier 3\n\nThe CLI is more feature-complete than the GUI command parser.",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(MD3_ON_SURFACE_VARIANT),
                    ));
                });
        });
}

/// Initialize the character manager and database
pub fn init_character_manager(mut commands: Commands) {
    // Initialize the database
    let db = match CharacterDatabase::open() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open character database: {}. Using in-memory database.", e);
            CharacterDatabase::open_in_memory().expect("Failed to create in-memory database")
        }
    };

    // Get character list from database
    let characters = db.list_characters().unwrap_or_default();

    commands.insert_resource(db);

    commands.insert_resource(CharacterManager {
        characters: characters.clone(),
        current_character_id: None,
        list_version: 0,
        available_characters: vec![],
        current_character_path: None,
    });

    commands.insert_resource(TextInputState::default());
    commands.insert_resource(GroupEditState::default());
    commands.insert_resource(AddingEntryState::default());
}
