//! Input handling systems
//!
//! This module contains systems for keyboard input, dice rolling controls,
//! command input, and command history management.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::dice3d::throw_control::ThrowControlState;
use crate::dice3d::types::*;
use bevy_material_ui::prelude::{ButtonClickEvent, MaterialTextField, TextFieldSubmitEvent};

use super::dice_box_controls::start_container_shake;

use super::setup::{calculate_dice_position, spawn_die};

#[derive(bevy::ecs::system::SystemParam)]
pub struct CommandInputParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub db: Res<'w, CharacterDatabase>,
    pub settings_state: Res<'w, crate::dice3d::types::SettingsState>,
    pub command_history: ResMut<'w, CommandHistory>,
    pub dice_config: ResMut<'w, DiceConfig>,
    pub dice_results: ResMut<'w, DiceResults>,
    pub roll_state: ResMut<'w, RollState>,
    pub character_data: Res<'w, CharacterData>,
    pub ui_state: Res<'w, UiState>,

    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub dice_query: Query<'w, 's, Entity, With<Die>>,

    pub shake_state: Res<'w, ShakeState>,
    pub shake_config: Res<'w, ContainerShakeConfig>,
    pub shake_anim: ResMut<'w, ContainerShakeAnimation>,
    pub container_query: Query<'w, 's, (Entity, &'static Transform), With<DiceBox>>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct CommandHistoryRerollParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub db: Res<'w, CharacterDatabase>,
    pub ui_state: Res<'w, UiState>,
    pub settings_state: Res<'w, crate::dice3d::types::SettingsState>,
    pub command_history: ResMut<'w, CommandHistory>,
    pub dice_config: ResMut<'w, DiceConfig>,
    pub dice_results: ResMut<'w, DiceResults>,
    pub roll_state: ResMut<'w, RollState>,
    pub character_data: Res<'w, CharacterData>,

    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub dice_query: Query<'w, 's, Entity, With<Die>>,

    pub shake_state: Res<'w, ShakeState>,
    pub shake_config: Res<'w, ContainerShakeConfig>,
    pub shake_anim: ResMut<'w, ContainerShakeAnimation>,
    pub container_query: Query<'w, 's, (Entity, &'static Transform), With<DiceBox>>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct QuickRollParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub db: Res<'w, CharacterDatabase>,
    pub dice_config: ResMut<'w, DiceConfig>,
    pub character_data: Res<'w, CharacterData>,
    pub roll_state: ResMut<'w, RollState>,
    pub dice_results: ResMut<'w, DiceResults>,
    pub command_history: ResMut<'w, CommandHistory>,
    pub throw_state: Res<'w, ThrowControlState>,
    pub settings_state: Res<'w, SettingsState>,

    pub shake_state: Res<'w, ShakeState>,
    pub shake_config: Res<'w, ContainerShakeConfig>,
    pub shake_anim: ResMut<'w, ContainerShakeAnimation>,
    pub container_query: Query<'w, 's, (Entity, &'static Transform), With<DiceBox>>,

    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub dice_query: Query<'w, 's, Entity, With<Die>>,
}

/// Handle keyboard input for rolling and resetting dice
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    ui_state: Res<UiState>,
    settings_state: Res<crate::dice3d::types::SettingsState>,
    mut roll_state: ResMut<RollState>,
    mut dice_results: ResMut<DiceResults>,
    mut dice_query: Query<(&mut Transform, &mut Velocity), (With<Die>, Without<DiceBox>)>,
    dice_config: Res<DiceConfig>,
    command_field: Query<&MaterialTextField, With<CommandInputField>>,
    throw_state: Res<ThrowControlState>,

    shake_state: Res<ShakeState>,
    shake_config: Res<ContainerShakeConfig>,
    mut shake_anim: ResMut<ContainerShakeAnimation>,
    container_query: Query<(Entity, &Transform), With<DiceBox>>,
) {
    if ui_state.active_tab != AppTab::DiceRoller {
        return;
    }

    // Modal dialog open: block interactions with the game world.
    if settings_state.show_modal {
        return;
    }

    // Don't process game inputs when the command field is focused
    let command_focused = command_field
        .iter()
        .any(|field| field.focused && !field.disabled);
    if command_focused {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) && throw_state.mouse_over_box && !roll_state.rolling {
        roll_state.rolling = true;
        dice_results.results.clear();

        let mut rng = rand::thread_rng();
        let num_dice = dice_config.dice_to_roll.len();

        let use_shake = settings_state.settings.default_roll_uses_shake;

        // Get base throw velocity from mouse-controlled throw state
        let base_velocity = throw_state.calculate_throw_velocity();

        for (i, (mut transform, mut velocity)) in dice_query.iter_mut().enumerate() {
            let position = calculate_dice_position(i, num_dice);
            // Add slight randomness to starting position
            // (Keep it inside the box: the ceiling is at ~1.5.)
            transform.translation = position
                + Vec3::new(
                    rng.gen_range(-0.3..0.3),
                    rng.gen_range(0.0..0.3),
                    rng.gen_range(-0.3..0.3),
                );
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
            );

            if use_shake {
                // No initial impulse; the shake animation will provide motion.
                velocity.linvel = Vec3::ZERO;
                velocity.angvel = Vec3::ZERO;
            } else {
                // Use mouse-controlled throw direction with slight randomness
                velocity.linvel = base_velocity
                    + Vec3::new(
                        rng.gen_range(-0.5..0.5),
                        rng.gen_range(-0.3..0.0),
                        rng.gen_range(-0.5..0.5),
                    );
                velocity.angvel = throw_state.calculate_angular_velocity(&mut rng);
            }
        }

        if use_shake {
            let _started = start_container_shake(
                &shake_state,
                &shake_config,
                &mut shake_anim,
                &container_query,
            );
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        roll_state.rolling = false;
        dice_results.results.clear();

        let num_dice = dice_config.dice_to_roll.len();

        for (i, (mut transform, mut velocity)) in dice_query.iter_mut().enumerate() {
            let mut pos = calculate_dice_position(i, num_dice);
            pos.y = 0.3; // Rest on floor
            transform.translation = pos;
            transform.rotation = Quat::IDENTITY;
            velocity.linvel = Vec3::ZERO;
            velocity.angvel = Vec3::ZERO;
        }
    }
}

/// Handle command input from the user
pub fn handle_command_input(
    mut params: CommandInputParams,
    mut submit_events: MessageReader<TextFieldSubmitEvent>,
    mut command_field_query: Query<(Entity, &mut MaterialTextField), With<CommandInputField>>,
) {
    if params.ui_state.active_tab != AppTab::DiceRoller {
        return;
    }

    // Modal dialog open: block interactions with the game world.
    if params.settings_state.show_modal {
        return;
    }

    let command_field_entity = command_field_query
        .iter()
        .next()
        .map(|(e, _field)| e)
        .unwrap_or(Entity::PLACEHOLDER);

    // Handle submit from the Material text field (Enter)
    for ev in submit_events.read() {
        if ev.entity != command_field_entity {
            continue;
        }

        let cmd = ev.value.trim().to_string();
        if cmd.is_empty() {
            continue;
        }

        // Parse and apply the command
        if let Some(new_config) = parse_command(&cmd, &params.character_data) {
            // Add to command history (only unique commands)
            params.command_history.add_command(cmd.clone());
            let _ = params
                .db
                .save_command_history(&params.command_history.commands);

            // Remove old dice
            for entity in params.dice_query.iter() {
                params.commands.entity(entity).despawn();
            }

            // Update config
            *params.dice_config = new_config;
            params.dice_results.results.clear();

            let use_shake = params.settings_state.settings.default_roll_uses_shake;

            // Spawn new dice
            let mut spawned: Vec<Entity> = Vec::new();
            for (i, die_type) in params.dice_config.dice_to_roll.iter().enumerate() {
                let position = calculate_dice_position(i, params.dice_config.dice_to_roll.len());
                let e = spawn_die(
                    &mut params.commands,
                    &mut params.meshes,
                    &mut params.materials,
                    *die_type,
                    position,
                );
                spawned.push(e);
            }

            if use_shake {
                for e in spawned {
                    params.commands.entity(e).insert(Velocity {
                        linvel: Vec3::ZERO,
                        angvel: Vec3::ZERO,
                    });
                }

                let _started = start_container_shake(
                    &params.shake_state,
                    &params.shake_config,
                    &mut params.shake_anim,
                    &params.container_query,
                );
            }

            // Start rolling immediately
            params.roll_state.rolling = true;
        }

        // Clear + blur the field after submit.
        // Also disable auto-focus so game hotkeys (e.g. R to reset) won't immediately
        // re-activate the command input on the next keypress.
        if let Ok((_, mut field)) = command_field_query.get_mut(command_field_entity) {
            field.value.clear();
            field.has_content = false;
            field.focused = false;
            field.auto_focus = false;
        }
    }
}

/// Handle clicks on command history items (reroll selected command)
pub fn handle_command_history_item_clicks(
    mut click_events: MessageReader<ButtonClickEvent>,
    item_query: Query<&CommandHistoryItem>,
    mut params: CommandHistoryRerollParams,
) {
    if params.ui_state.active_tab != AppTab::DiceRoller {
        return;
    }

    // Modal dialog open: block interactions with the game world.
    if params.settings_state.show_modal {
        return;
    }

    for ev in click_events.read() {
        let Ok(item) = item_query.get(ev.entity) else {
            continue;
        };

        let Some(cmd) = params.command_history.commands.get(item.index).cloned() else {
            continue;
        };

        params.command_history.selected_index = Some(item.index);

        if let Some(new_config) = parse_command(&cmd, &params.character_data) {
            // Remove old dice
            for entity in params.dice_query.iter() {
                params.commands.entity(entity).despawn();
            }

            // Update config
            *params.dice_config = new_config;
            params.dice_results.results.clear();

            let use_shake = params.settings_state.settings.default_roll_uses_shake;

            // Spawn new dice
            let mut spawned: Vec<Entity> = Vec::new();
            for (i, die_type) in params.dice_config.dice_to_roll.iter().enumerate() {
                let position = calculate_dice_position(i, params.dice_config.dice_to_roll.len());
                let e = spawn_die(
                    &mut params.commands,
                    &mut params.meshes,
                    &mut params.materials,
                    *die_type,
                    position,
                );
                spawned.push(e);
            }

            if use_shake {
                for e in spawned {
                    params.commands.entity(e).insert(Velocity {
                        linvel: Vec3::ZERO,
                        angvel: Vec3::ZERO,
                    });
                }

                let _started = start_container_shake(
                    &params.shake_state,
                    &params.shake_config,
                    &mut params.shake_anim,
                    &params.container_query,
                );
            }

            // Start rolling immediately
            params.roll_state.rolling = true;
        }
    }
}

/// Parse a command string into a DiceConfig
fn parse_command(cmd: &str, character_data: &CharacterData) -> Option<DiceConfig> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut dice_to_roll = Vec::new();
    let mut modifier = 0i32;
    let mut modifier_name = String::new();
    let mut checkon: Option<String> = None;

    let mut i = 0;
    while i < parts.len() {
        let part = parts[i];

        if part == "--dice" || part == "-d" {
            if i + 1 < parts.len() {
                i += 1;
                if let Some((count, die_type)) = parse_dice_str(parts[i]) {
                    for _ in 0..count {
                        dice_to_roll.push(die_type);
                    }
                }
            }
        } else if part == "--checkon" {
            if i + 1 < parts.len() {
                i += 1;
                checkon = Some(parts[i].to_string());
            }
        } else if part == "--modifier" || part == "-m" {
            if i + 1 < parts.len() {
                i += 1;
                if let Ok(m) = parts[i].parse::<i32>() {
                    modifier += m;
                }
            }
        } else if part.contains('d') && !part.starts_with('-') {
            // Direct dice notation like "2d6"
            if let Some((count, die_type)) = parse_dice_str(part) {
                for _ in 0..count {
                    dice_to_roll.push(die_type);
                }
            }
        }

        i += 1;
    }

    // Apply checkon modifier (skill / ability / saving throw) similar to the CLI.
    if let Some(check) = checkon {
        let check_lower = check.to_lowercase();

        if let Some(skill_mod) = character_data.get_skill_modifier(&check_lower) {
            modifier += skill_mod;
            modifier_name = check;
        } else if let Some(ability_mod) = character_data.get_ability_modifier(&check_lower) {
            modifier += ability_mod;
            modifier_name = format!("{} check", check);
        } else if let Some(save_mod) = character_data.get_saving_throw_modifier(&check_lower) {
            modifier += save_mod;
            modifier_name = format!("{} save", check);
        } else {
            // Unknown label: keep the name for display, but don't change the modifier.
            modifier_name = check;
        }
    }

    // Default to 1d20 if no dice specified.
    if dice_to_roll.is_empty() {
        dice_to_roll.push(DiceType::D20);
    }

    Some(DiceConfig {
        dice_to_roll,
        modifier,
        modifier_name,
    })
}

/// Parse a dice string like "2d6" into a count and die type
fn parse_dice_str(s: &str) -> Option<(usize, DiceType)> {
    let s = s.to_lowercase();

    let (count_str, die_str) = if s.starts_with('d') {
        ("1", s.as_str())
    } else if let Some(pos) = s.find('d') {
        (&s[..pos], &s[pos..])
    } else {
        return None;
    };

    let count: usize = count_str.parse().ok()?;
    let die_type = DiceType::parse(die_str)?;

    Some((count, die_type))
}

/// Handle quick roll button clicks
pub fn handle_quick_roll_clicks(
    mut click_events: MessageReader<ButtonClickEvent>,
    quick_roll_query: Query<&QuickRollButton>,
    mut params: QuickRollParams,
) {
    if params.settings_state.show_modal {
        return;
    }

    for event in click_events.read() {
        let Ok(quick_roll) = quick_roll_query.get(event.entity) else {
            continue;
        };
        // Get the modifier based on roll type
        let (modifier, modifier_name) = match &quick_roll.roll_type {
            QuickRollType::Skill(skill_name) => {
                let mod_val = params
                    .character_data
                    .get_skill_modifier(skill_name)
                    .unwrap_or(0);
                // Format skill name nicely - capitalize first letter
                let display_name = skill_name
                    .chars()
                    .enumerate()
                    .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                    .collect::<String>();
                (mod_val, display_name)
            }
            QuickRollType::AbilityCheck(ability_name) => {
                let mod_val = params
                    .character_data
                    .get_ability_modifier(ability_name)
                    .unwrap_or(0);
                let display_name = format!("{} check", capitalize_first(ability_name));
                (mod_val, display_name)
            }
            QuickRollType::SavingThrow(ability_name) => {
                let mod_val = params
                    .character_data
                    .get_saving_throw_modifier(ability_name)
                    .unwrap_or(0);
                let display_name = format!("{} save", capitalize_first(ability_name));
                (mod_val, display_name)
            }
        };

        let die_type = params
            .settings_state
            .settings
            .quick_roll_default_die
            .to_dice_type();

        // Remove old dice (Quick Rolls always uses exactly one die)
        for entity in params.dice_query.iter() {
            params.commands.entity(entity).despawn();
        }

        // Update dice config
        params.dice_config.dice_to_roll.clear();
        params.dice_config.dice_to_roll.push(die_type);
        params.dice_config.modifier = modifier;
        params.dice_config.modifier_name = modifier_name.clone();

        // Add to command history
        let sign = if modifier >= 0 { "+" } else { "" };
        params.command_history.add_command(format!(
            "1d{} --checkon {} ({}{})",
            die_type.max_value(),
            modifier_name,
            sign,
            modifier
        ));
        let _ = params
            .db
            .save_command_history(&params.command_history.commands);

        // Trigger the roll
        params.roll_state.rolling = true;
        params.dice_results.results.clear();

        // Spawn the single die and override its transform/velocity using throw control.
        let die_entity = spawn_die(
            &mut params.commands,
            &mut params.meshes,
            &mut params.materials,
            die_type,
            calculate_dice_position(0, 1),
        );

        let use_shake = params.settings_state.settings.default_roll_uses_shake;

        let mut rng = rand::thread_rng();
        let base_velocity = params.throw_state.calculate_throw_velocity();

        let transform = Transform::from_translation(Vec3::new(
            rng.gen_range(-0.5..0.5),
            1.0,
            rng.gen_range(-0.5..0.5),
        ))
        .with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            rng.gen_range(0.0..std::f32::consts::TAU),
            rng.gen_range(0.0..std::f32::consts::TAU),
            rng.gen_range(0.0..std::f32::consts::TAU),
        ))
        .with_scale(Vec3::splat(die_type.scale()));

        let velocity = if use_shake {
            Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::ZERO,
            }
        } else {
            Velocity {
                linvel: base_velocity
                    + Vec3::new(
                        rng.gen_range(-0.5..0.5),
                        rng.gen_range(-0.3..0.0),
                        rng.gen_range(-0.5..0.5),
                    ),
                angvel: params.throw_state.calculate_angular_velocity(&mut rng),
            }
        };

        params
            .commands
            .entity(die_entity)
            .insert((transform, velocity));

        if use_shake {
            let _started = start_container_shake(
                &params.shake_state,
                &params.shake_config,
                &mut params.shake_anim,
                &params.container_query,
            );
        }
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}
