//! Input handling systems
//!
//! This module contains systems for keyboard input, dice rolling controls,
//! command input, and command history management.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::dice3d::types::*;

use super::setup::{calculate_dice_position, spawn_die};

/// Handle keyboard input for rolling and resetting dice
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut roll_state: ResMut<RollState>,
    mut dice_results: ResMut<DiceResults>,
    mut dice_query: Query<(&mut Transform, &mut Velocity), With<Die>>,
    dice_config: Res<DiceConfig>,
    command_input: Res<CommandInput>,
) {
    // Don't process game inputs when command input is active
    if command_input.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) && !roll_state.rolling {
        roll_state.rolling = true;
        dice_results.results.clear();

        let mut rng = rand::thread_rng();
        let num_dice = dice_config.dice_to_roll.len();

        for (i, (mut transform, mut velocity)) in dice_query.iter_mut().enumerate() {
            let position = calculate_dice_position(i, num_dice);
            // Add slight randomness to starting position
            transform.translation = position
                + Vec3::new(
                    rng.gen_range(-0.3..0.3),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(-0.3..0.3),
                );
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
            );

            // Throw dice with more energy so they bounce around
            velocity.linvel = Vec3::new(
                rng.gen_range(-3.0..3.0),
                rng.gen_range(-2.0..0.0), // Throw downward
                rng.gen_range(-3.0..3.0),
            );
            velocity.angvel = Vec3::new(
                rng.gen_range(-20.0..20.0),
                rng.gen_range(-20.0..20.0),
                rng.gen_range(-20.0..20.0),
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
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut command_input: ResMut<CommandInput>,
    mut command_history: ResMut<CommandHistory>,
    mut input_text_query: Query<&mut Text, With<CommandInputText>>,
    mut history_text_query: Query<&mut Text, (With<CommandHistoryList>, Without<CommandInputText>)>,
    mut dice_config: ResMut<DiceConfig>,
    mut dice_results: ResMut<DiceResults>,
    mut roll_state: ResMut<RollState>,
    character_data: Res<CharacterData>,
    // For respawning dice
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dice_query: Query<Entity, With<Die>>,
) {
    // Handle number keys 1-9 to reroll from history (when not in input mode)
    if !command_input.active {
        let history_keys = [
            (KeyCode::Digit1, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
            (KeyCode::Digit4, 3),
            (KeyCode::Digit5, 4),
            (KeyCode::Digit6, 5),
            (KeyCode::Digit7, 6),
            (KeyCode::Digit8, 7),
            (KeyCode::Digit9, 8),
        ];

        for (key, index) in history_keys {
            if keyboard.just_pressed(key) {
                if let Some(cmd) = command_history.commands.get(index).cloned() {
                    // Execute the command from history
                    if let Some(new_config) = parse_command(&cmd, &character_data) {
                        // Remove old dice
                        for entity in dice_query.iter() {
                            commands.entity(entity).despawn_recursive();
                        }

                        // Update config
                        *dice_config = new_config;
                        dice_results.results.clear();
                        roll_state.rolling = false;

                        // Spawn new dice
                        for (i, die_type) in dice_config.dice_to_roll.iter().enumerate() {
                            let position =
                                calculate_dice_position(i, dice_config.dice_to_roll.len());
                            spawn_die(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                *die_type,
                                position,
                            );
                        }
                    }
                    return;
                }
            }
        }
    }

    // Toggle command input with / or Enter when not active
    if !command_input.active
        && (keyboard.just_pressed(KeyCode::Slash) || keyboard.just_pressed(KeyCode::Enter))
    {
        command_input.active = true;
        command_input.text.clear();
        for mut text in input_text_query.iter_mut() {
            text.sections[0].value = "> ".to_string();
            text.sections[0].style.color = Color::srgb(1.0, 1.0, 0.5);
        }
        return;
    }

    if !command_input.active {
        return;
    }

    // Handle escape to cancel
    if keyboard.just_pressed(KeyCode::Escape) {
        command_input.active = false;
        command_input.text.clear();
        for mut text in input_text_query.iter_mut() {
            text.sections[0].value =
                "> Type command: --dice 2d6 --checkon stealth  |  Press 1-9 to reroll from history"
                    .to_string();
            text.sections[0].style.color = Color::srgba(0.7, 0.7, 0.7, 0.8);
        }
        return;
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        command_input.text.pop();
        for mut text in input_text_query.iter_mut() {
            text.sections[0].value = format!("> {}_", command_input.text);
        }
        return;
    }

    // Handle enter to submit
    if keyboard.just_pressed(KeyCode::Enter) {
        let cmd = command_input.text.clone();
        command_input.active = false;
        command_input.text.clear();

        // Parse and apply the command
        if let Some(new_config) = parse_command(&cmd, &character_data) {
            // Add to command history (only unique commands)
            command_history.add_command(cmd.clone());

            // Update history display
            update_history_display(&command_history, &mut history_text_query);

            // Remove old dice
            for entity in dice_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Update config
            *dice_config = new_config;
            dice_results.results.clear();
            roll_state.rolling = false;

            // Spawn new dice
            for (i, die_type) in dice_config.dice_to_roll.iter().enumerate() {
                let position = calculate_dice_position(i, dice_config.dice_to_roll.len());
                spawn_die(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    *die_type,
                    position,
                );
            }
        }

        for mut text in input_text_query.iter_mut() {
            text.sections[0].value =
                "> Type command: --dice 2d6 --checkon stealth  |  Press 1-9 to reroll from history"
                    .to_string();
            text.sections[0].style.color = Color::srgba(0.7, 0.7, 0.7, 0.8);
        }
        return;
    }

    // Handle character input
    for event in char_events.read() {
        if event.state == bevy::input::ButtonState::Pressed {
            // Map key codes to characters
            let c = match event.key_code {
                KeyCode::Space => ' ',
                KeyCode::Minus => '-',
                KeyCode::Equal => '=',
                KeyCode::Digit0 => '0',
                KeyCode::Digit1 => '1',
                KeyCode::Digit2 => '2',
                KeyCode::Digit3 => '3',
                KeyCode::Digit4 => '4',
                KeyCode::Digit5 => '5',
                KeyCode::Digit6 => '6',
                KeyCode::Digit7 => '7',
                KeyCode::Digit8 => '8',
                KeyCode::Digit9 => '9',
                KeyCode::KeyA => 'a',
                KeyCode::KeyB => 'b',
                KeyCode::KeyC => 'c',
                KeyCode::KeyD => 'd',
                KeyCode::KeyE => 'e',
                KeyCode::KeyF => 'f',
                KeyCode::KeyG => 'g',
                KeyCode::KeyH => 'h',
                KeyCode::KeyI => 'i',
                KeyCode::KeyJ => 'j',
                KeyCode::KeyK => 'k',
                KeyCode::KeyL => 'l',
                KeyCode::KeyM => 'm',
                KeyCode::KeyN => 'n',
                KeyCode::KeyO => 'o',
                KeyCode::KeyP => 'p',
                KeyCode::KeyQ => 'q',
                KeyCode::KeyR => 'r',
                KeyCode::KeyS => 's',
                KeyCode::KeyT => 't',
                KeyCode::KeyU => 'u',
                KeyCode::KeyV => 'v',
                KeyCode::KeyW => 'w',
                KeyCode::KeyX => 'x',
                KeyCode::KeyY => 'y',
                KeyCode::KeyZ => 'z',
                _ => continue,
            };
            command_input.text.push(c);
        }
    }

    // Update display
    for mut text in input_text_query.iter_mut() {
        text.sections[0].value = format!("> {}_", command_input.text);
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

    // Apply checkon modifier
    if let Some(check) = checkon {
        let check_lower = check.to_lowercase();

        if let Some(skill_mod) = character_data.get_skill_modifier(&check_lower) {
            modifier += skill_mod;
            modifier_name = check.clone();
        } else if let Some(ability_mod) = character_data.get_ability_modifier(&check_lower) {
            modifier += ability_mod;
            modifier_name = format!("{} check", check);
        } else if let Some(save_mod) = character_data.get_saving_throw_modifier(&check_lower) {
            modifier += save_mod;
            modifier_name = format!("{} save", check);
        } else {
            modifier_name = check;
        }
    }

    // Default to 1d20 if no dice specified
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

/// Update the command history display
fn update_history_display(
    history: &CommandHistory,
    history_text_query: &mut Query<
        &mut Text,
        (With<CommandHistoryList>, Without<CommandInputText>),
    >,
) {
    let mut history_text = String::from("Command History:\n");

    if history.commands.is_empty() {
        history_text.push_str("(no commands yet)");
    } else {
        for (i, cmd) in history.commands.iter().enumerate().take(9) {
            history_text.push_str(&format!("[{}] {}\n", i + 1, cmd));
        }
    }

    for mut text in history_text_query.iter_mut() {
        text.sections[0].value = history_text.clone();
    }
}
