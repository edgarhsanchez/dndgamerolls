//! Dice state and result systems
//!
//! This module contains systems for checking dice settlement,
//! determining dice results, and updating the results display.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::dice3d::types::*;

/// System to check if dice have settled and determine results
pub fn check_dice_settled(
    mut roll_state: ResMut<RollState>,
    mut dice_results: ResMut<DiceResults>,
    dice_query: Query<(&Die, &Velocity, &Transform)>,
    time: Res<Time>,
) {
    if !roll_state.rolling {
        return;
    }

    let all_settled = dice_query
        .iter()
        .all(|(_, vel, _)| vel.linvel.length() < 0.1 && vel.angvel.length() < 0.1);

    if all_settled {
        roll_state.settle_timer += time.delta_seconds();

        if roll_state.settle_timer > 0.5 {
            roll_state.rolling = false;
            roll_state.settle_timer = 0.0;

            dice_results.results.clear();
            for (die, _, transform) in dice_query.iter() {
                let result = determine_dice_result(die, transform);
                dice_results.results.push((die.die_type, result));
            }
        }
    } else {
        roll_state.settle_timer = 0.0;
    }
}

/// Determine the upward-facing value of a die based on its rotation
fn determine_dice_result(die: &Die, transform: &Transform) -> u32 {
    let up = Vec3::Y;
    let mut best_match = 1;
    let mut best_dot = -2.0_f32;

    for (normal, value) in &die.face_normals {
        let world_normal = transform.rotation * *normal;
        let dot = world_normal.dot(up);

        if dot > best_dot {
            best_dot = dot;
            best_match = *value;
        }
    }

    best_match
}

/// System to update the results display text
pub fn update_results_display(
    dice_results: Res<DiceResults>,
    roll_state: Res<RollState>,
    dice_config: Res<DiceConfig>,
    character_data: Res<CharacterData>,
    mut text_query: Query<&mut Text, With<ResultsText>>,
) {
    for mut text in text_query.iter_mut() {
        // Character info header
        let char_info = if let Some(sheet) = &character_data.sheet {
            format!(
                "{} - {} {} (Level {})\n",
                sheet.character.name,
                sheet.character.race,
                sheet.character.class,
                sheet.character.level
            )
        } else {
            String::from("No character loaded\n")
        };

        if roll_state.rolling {
            text.sections[0].value = format!("{}Rolling...", char_info);
        } else if dice_results.results.is_empty() {
            let modifier_info = format_modifier_info(&dice_config);
            text.sections[0].value = format!(
                "{}{}\nPress SPACE to roll dice\nPress R to reset",
                char_info, modifier_info
            );
        } else {
            let mut result_text = format!("{}Results:\n", char_info);
            let mut total = 0i32;

            // Group results by die type using BTreeMap for stable ordering
            let mut grouped: std::collections::BTreeMap<u32, (DiceType, Vec<u32>)> =
                std::collections::BTreeMap::new();
            for (die_type, value) in &dice_results.results {
                // Key by max_value for consistent ordering (D4=4, D6=6, etc.)
                let key = die_type.max_value();
                grouped
                    .entry(key)
                    .or_insert_with(|| (*die_type, Vec::new()))
                    .1
                    .push(*value);
            }

            // Sort values within each group for consistent display
            for (_die_type, values) in grouped.values_mut() {
                values.sort();
            }

            for (die_type, values) in grouped.values() {
                let sum: u32 = values.iter().sum();
                total += sum as i32;
                if values.len() == 1 {
                    result_text.push_str(&format!("{}: {}\n", die_type.name(), values[0]));
                } else {
                    let values_str: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    result_text.push_str(&format!(
                        "{}x{}: {} = {}\n",
                        values.len(),
                        die_type.name(),
                        values_str.join(" + "),
                        sum
                    ));
                }
            }

            // Apply modifier
            let modifier = dice_config.modifier;
            let final_total = total + modifier;

            if modifier != 0 {
                let sign = if modifier >= 0 { "+" } else { "" };
                let mod_name = if !dice_config.modifier_name.is_empty() {
                    format!(" ({})", dice_config.modifier_name)
                } else {
                    String::new()
                };
                result_text.push_str(&format!(
                    "\nDice Total: {}\nModifier{}: {}{}\n\nFINAL TOTAL: {}",
                    total, mod_name, sign, modifier, final_total
                ));
            } else {
                result_text.push_str(&format!("\nTOTAL: {}", total));
            }

            result_text.push_str("\n\nPress SPACE to roll again\nPress R to reset");
            text.sections[0].value = result_text;
        }
    }
}

/// Format modifier information for display
fn format_modifier_info(dice_config: &DiceConfig) -> String {
    if !dice_config.modifier_name.is_empty() {
        let sign = if dice_config.modifier >= 0 { "+" } else { "" };
        format!(
            "Modifier: {} ({}{})\n",
            dice_config.modifier_name, sign, dice_config.modifier
        )
    } else if dice_config.modifier != 0 {
        let sign = if dice_config.modifier >= 0 { "+" } else { "" };
        format!("Modifier: {}{}\n", sign, dice_config.modifier)
    } else {
        String::new()
    }
}
