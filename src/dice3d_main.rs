use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use clap::Parser;

use dndgamerolls::dice3d::{
    check_dice_settled, handle_command_input, handle_input, rotate_camera, setup,
    update_results_display, CharacterData, CommandHistory, CommandInput, DiceConfig, DiceResults,
    DiceType, RollState,
};

/// 3D Dice Roller for D&D
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dice to roll (e.g., "2d6", "1d20", "d8"). Can specify multiple.
    #[arg(short, long, value_parser = parse_dice_arg)]
    dice: Option<Vec<(usize, DiceType)>>,

    /// Path to character JSON file
    #[arg(short, long, default_value = "dnd_stats.json")]
    character: String,

    /// Check to apply modifier for (skill, ability, or save name)
    /// Examples: "perception", "stealth", "dex", "wisdom save"
    #[arg(long)]
    checkon: Option<String>,

    /// Skill check to roll (adds 1d20 + skill modifier)
    #[arg(short, long)]
    skill: Option<String>,

    /// Ability check to roll (adds 1d20 + ability modifier)
    #[arg(short, long)]
    ability: Option<String>,

    /// Saving throw to roll (adds 1d20 + save modifier)
    #[arg(long)]
    save: Option<String>,

    /// Custom modifier to add to the roll
    #[arg(short, long, default_value = "0")]
    modifier: i32,
}

fn parse_dice_arg(s: &str) -> Result<(usize, DiceType), String> {
    let s = s.to_lowercase();
    
    // Handle formats: "2d6", "d20", "1d8"
    let (count_str, die_str) = if s.starts_with('d') {
        ("1", s.as_str())
    } else if let Some(pos) = s.find('d') {
        (&s[..pos], &s[pos..])
    } else {
        return Err(format!("Invalid dice format: {}. Use format like '2d6' or 'd20'", s));
    };

    let count: usize = count_str.parse().map_err(|_| format!("Invalid count: {}", count_str))?;
    let die_type = DiceType::from_str(die_str)
        .ok_or_else(|| format!("Unknown die type: {}. Valid: d4, d6, d8, d10, d12, d20", die_str))?;

    Ok((count, die_type))
}

fn main() {
    let args = Args::parse();

    // Load character data
    let character_data = CharacterData::load_from_file(&args.character);

    // Build dice configuration
    let mut dice_to_roll = Vec::new();
    let mut modifier = args.modifier;
    let mut modifier_name = String::new();

    // Handle --checkon: apply modifier to custom dice
    if let Some(check) = &args.checkon {
        // Try to find the modifier from skill, ability, or save
        let check_lower = check.to_lowercase();
        
        if let Some(skill_mod) = character_data.get_skill_modifier(&check_lower) {
            modifier += skill_mod;
            modifier_name = format!("{}", check);
        } else if let Some(ability_mod) = character_data.get_ability_modifier(&check_lower) {
            modifier += ability_mod;
            modifier_name = format!("{} check", check);
        } else if check_lower.ends_with(" save") || check_lower.ends_with(" saving") {
            // Handle "dex save", "wisdom saving", etc.
            let save_name = check_lower
                .trim_end_matches(" save")
                .trim_end_matches(" saving")
                .trim();
            if let Some(save_mod) = character_data.get_saving_throw_modifier(save_name) {
                modifier += save_mod;
                modifier_name = format!("{} save", save_name);
            } else {
                modifier_name = check.clone();
                eprintln!("Warning: '{}' not found in character sheet", check);
            }
        } else if let Some(save_mod) = character_data.get_saving_throw_modifier(&check_lower) {
            // Try as a saving throw directly
            modifier += save_mod;
            modifier_name = format!("{} save", check);
        } else {
            modifier_name = check.clone();
            eprintln!("Warning: '{}' not found in character sheet", check);
        }
        
        // Add dice from --dice arguments
        if let Some(dice_args) = &args.dice {
            for (count, die_type) in dice_args {
                for _ in 0..*count {
                    dice_to_roll.push(*die_type);
                }
            }
        }
    }
    // Legacy behavior: --skill, --ability, --save add 1d20 automatically
    else if let Some(skill) = &args.skill {
        dice_to_roll.push(DiceType::D20);
        if let Some(skill_mod) = character_data.get_skill_modifier(skill) {
            modifier += skill_mod;
            modifier_name = format!("{} skill", skill);
        } else {
            modifier_name = skill.clone();
            eprintln!("Warning: Skill '{}' not found in character sheet", skill);
        }
    } else if let Some(ability) = &args.ability {
        dice_to_roll.push(DiceType::D20);
        if let Some(ability_mod) = character_data.get_ability_modifier(ability) {
            modifier += ability_mod;
            modifier_name = format!("{} check", ability);
        } else {
            modifier_name = ability.clone();
            eprintln!("Warning: Ability '{}' not found in character sheet", ability);
        }
    } else if let Some(save) = &args.save {
        dice_to_roll.push(DiceType::D20);
        if let Some(save_mod) = character_data.get_saving_throw_modifier(save) {
            modifier += save_mod;
            modifier_name = format!("{} save", save);
        } else {
            modifier_name = save.clone();
            eprintln!("Warning: Saving throw '{}' not found in character sheet", save);
        }
    } else if let Some(dice_args) = &args.dice {
        // Parse dice arguments
        for (count, die_type) in dice_args {
            for _ in 0..*count {
                dice_to_roll.push(*die_type);
            }
        }
    }

    // Default to 1d20 if nothing specified
    if dice_to_roll.is_empty() {
        dice_to_roll.push(DiceType::D20);
    }

    // Print what we're rolling
    println!("Rolling: {:?}", dice_to_roll.iter().map(|d| d.name()).collect::<Vec<_>>());
    if modifier != 0 {
        let sign = if modifier >= 0 { "+" } else { "" };
        println!("Modifier: {}{} ({})", sign, modifier, modifier_name);
    }

    let dice_config = DiceConfig {
        dice_to_roll,
        modifier,
        modifier_name,
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "D&D Dice Roller 3D".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(dice_config)
        .insert_resource(character_data)
        .insert_resource(DiceResults::default())
        .insert_resource(RollState::default())
        .insert_resource(CommandInput::default())
        .insert_resource(CommandHistory::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                check_dice_settled,
                update_results_display,
                handle_input,
                handle_command_input,
                rotate_camera,
            ),
        )
        .run();
}
