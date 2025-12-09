//! DnD Game Rolls CLI
//!
//! A command-line D&D dice roller with character sheet support.

use clap::{Parser, Subcommand};
use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// DnD Game Rolls - CLI dice roller
#[derive(Parser)]
#[command(name = "dndrolls")]
#[command(
    author,
    version,
    about = "DnD Game Rolls - A command-line D&D dice roller"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the character stats JSON file
    #[arg(short = 'f', long = "file", default_value = "dnd_stats.json")]
    character_file: PathBuf,

    /// Dice to roll (e.g., "2d6", "1d20", "d8"). Can specify multiple.
    #[arg(short, long, value_parser = parse_dice_arg)]
    dice: Option<Vec<(usize, DiceType)>>,

    /// Check to apply modifier for (skill, ability, or save name)
    #[arg(long)]
    checkon: Option<String>,

    /// Custom modifier to add to the roll
    #[arg(short, long, default_value = "0")]
    modifier: i32,

    /// Roll with advantage (roll twice, take higher)
    #[arg(short, long)]
    advantage: bool,

    /// Roll with disadvantage (roll twice, take lower)
    #[arg(short = 'D', long)]
    disadvantage: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Roll a strength check
    #[command(visible_alias = "str")]
    Strength,

    /// Roll a dexterity check
    #[command(visible_alias = "dex")]
    Dexterity,

    /// Roll a constitution check
    #[command(visible_alias = "con")]
    Constitution,

    /// Roll an intelligence check
    #[command(visible_alias = "int")]
    Intelligence,

    /// Roll a wisdom check
    #[command(visible_alias = "wis")]
    Wisdom,

    /// Roll a charisma check
    #[command(visible_alias = "cha")]
    Charisma,

    /// Roll an initiative check (Dexterity based)
    Initiative,

    /// Roll a skill check
    Skill {
        /// Skill name (e.g., stealth, perception, acrobatics)
        name: String,
    },

    /// Roll a saving throw
    Save {
        /// Ability name (str, dex, con, int, wis, cha)
        ability: String,
    },

    /// Roll an attack
    Attack {
        /// Weapon name
        weapon: String,
    },

    /// Display character stats
    Stats,
}

// ============================================================================
// Dice Types
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
}

impl DiceType {
    pub fn max_value(&self) -> u32 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DiceType::D4 => "D4",
            DiceType::D6 => "D6",
            DiceType::D8 => "D8",
            DiceType::D10 => "D10",
            DiceType::D12 => "D12",
            DiceType::D20 => "D20",
        }
    }

    pub fn parse(s: &str) -> Option<DiceType> {
        match s.to_lowercase().as_str() {
            "d4" => Some(DiceType::D4),
            "d6" => Some(DiceType::D6),
            "d8" => Some(DiceType::D8),
            "d10" => Some(DiceType::D10),
            "d12" => Some(DiceType::D12),
            "d20" => Some(DiceType::D20),
            _ => None,
        }
    }
}

fn parse_dice_arg(s: &str) -> Result<(usize, DiceType), String> {
    let s = s.to_lowercase();

    let (count_str, die_str) = if s.starts_with('d') {
        ("1", s.as_str())
    } else if let Some(pos) = s.find('d') {
        (&s[..pos], &s[pos..])
    } else {
        return Err(format!(
            "Invalid dice format: {}. Use format like '2d6' or 'd20'",
            s
        ));
    };

    let count: usize = count_str
        .parse()
        .map_err(|_| format!("Invalid count: {}", count_str))?;
    let die_type = DiceType::parse(die_str).ok_or_else(|| {
        format!(
            "Unknown die type: {}. Valid: d4, d6, d8, d10, d12, d20",
            die_str
        )
    })?;

    Ok((count, die_type))
}

// ============================================================================
// Character Types
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
struct Character {
    character: CharacterInfo,
    attributes: Attributes,
    modifiers: Modifiers,
    combat: Combat,
    #[serde(rename = "proficiencyBonus")]
    proficiency_bonus: i32,
    #[serde(rename = "savingThrows")]
    saving_throws: SavingThrows,
    skills: Skills,
    equipment: Equipment,
}

#[derive(Debug, Deserialize, Serialize)]
struct CharacterInfo {
    name: String,
    #[serde(rename = "alterEgo")]
    alter_ego: Option<String>,
    class: String,
    subclass: Option<String>,
    race: String,
    level: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Attributes {
    strength: i32,
    dexterity: i32,
    constitution: i32,
    intelligence: i32,
    wisdom: i32,
    charisma: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Modifiers {
    strength: i32,
    dexterity: i32,
    constitution: i32,
    intelligence: i32,
    wisdom: i32,
    charisma: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Combat {
    #[serde(rename = "armorClass")]
    armor_class: i32,
    initiative: i32,
    #[serde(rename = "hitPoints")]
    hit_points: HitPoints,
}

#[derive(Debug, Deserialize, Serialize)]
struct HitPoints {
    current: i32,
    maximum: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct SavingThrows {
    strength: SavingThrow,
    dexterity: SavingThrow,
    constitution: SavingThrow,
    intelligence: SavingThrow,
    wisdom: SavingThrow,
    charisma: SavingThrow,
}

#[derive(Debug, Deserialize, Serialize)]
struct SavingThrow {
    proficient: bool,
    modifier: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Skills {
    acrobatics: Skill,
    #[serde(rename = "animalHandling")]
    animal_handling: Skill,
    arcana: Skill,
    athletics: Skill,
    deception: Skill,
    history: Skill,
    insight: Skill,
    intimidation: Skill,
    investigation: Skill,
    medicine: Skill,
    nature: Skill,
    perception: Skill,
    performance: Skill,
    persuasion: Skill,
    religion: Skill,
    #[serde(rename = "sleightOfHand")]
    sleight_of_hand: Skill,
    stealth: Skill,
    survival: Skill,
}

#[derive(Debug, Deserialize, Serialize)]
struct Skill {
    proficient: bool,
    modifier: i32,
    #[serde(default)]
    expertise: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Equipment {
    weapons: Vec<Weapon>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Weapon {
    name: String,
    #[serde(rename = "attackBonus")]
    attack_bonus: i32,
    damage: String,
    #[serde(rename = "damageType")]
    damage_type: String,
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let cli = Cli::parse();

    // If using --dice or --checkon, do a direct roll
    if cli.dice.is_some() || cli.checkon.is_some() {
        run_dice_roll(&cli);
        return;
    }

    // Handle subcommands
    if let Some(command) = &cli.command {
        let character = match load_character(&cli.character_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "{} Failed to load character file '{}': {}",
                    "Error:".red().bold(),
                    cli.character_file.display(),
                    e
                );
                std::process::exit(1);
            }
        };

        match command {
            Commands::Strength => {
                let modifier = character.modifiers.strength;
                roll_ability_check("Strength", modifier, cli.advantage, cli.disadvantage);
            }
            Commands::Dexterity => {
                let modifier = character.modifiers.dexterity;
                roll_ability_check("Dexterity", modifier, cli.advantage, cli.disadvantage);
            }
            Commands::Constitution => {
                let modifier = character.modifiers.constitution;
                roll_ability_check("Constitution", modifier, cli.advantage, cli.disadvantage);
            }
            Commands::Intelligence => {
                let modifier = character.modifiers.intelligence;
                roll_ability_check("Intelligence", modifier, cli.advantage, cli.disadvantage);
            }
            Commands::Wisdom => {
                let modifier = character.modifiers.wisdom;
                roll_ability_check("Wisdom", modifier, cli.advantage, cli.disadvantage);
            }
            Commands::Charisma => {
                let modifier = character.modifiers.charisma;
                roll_ability_check("Charisma", modifier, cli.advantage, cli.disadvantage);
            }
            Commands::Initiative => {
                let modifier = character.combat.initiative;
                roll_ability_check("Initiative", modifier, cli.advantage, cli.disadvantage);
            }
            Commands::Skill { name } => {
                if let Some((skill_name, skill)) = get_skill_by_name(&character.skills, name) {
                    let proficiency_str = if skill.expertise {
                        " (Expertise)"
                    } else if skill.proficient {
                        " (Proficient)"
                    } else {
                        ""
                    };
                    roll_ability_check(
                        &format!("{}{}", skill_name, proficiency_str),
                        skill.modifier,
                        cli.advantage,
                        cli.disadvantage,
                    );
                } else {
                    eprintln!("{} Unknown skill '{}'", "Error:".red().bold(), name);
                    eprintln!("Available skills: acrobatics, animal handling, arcana, athletics,");
                    eprintln!("  deception, history, insight, intimidation, investigation,");
                    eprintln!("  medicine, nature, perception, performance, persuasion,");
                    eprintln!("  religion, sleight of hand, stealth, survival");
                    std::process::exit(1);
                }
            }
            Commands::Save { ability } => {
                let ability_lower = ability.to_lowercase();
                let (save_name, save) = match ability_lower.as_str() {
                    "str" | "strength" => ("Strength", &character.saving_throws.strength),
                    "dex" | "dexterity" => ("Dexterity", &character.saving_throws.dexterity),
                    "con" | "constitution" => {
                        ("Constitution", &character.saving_throws.constitution)
                    }
                    "int" | "intelligence" => {
                        ("Intelligence", &character.saving_throws.intelligence)
                    }
                    "wis" | "wisdom" => ("Wisdom", &character.saving_throws.wisdom),
                    "cha" | "charisma" => ("Charisma", &character.saving_throws.charisma),
                    _ => {
                        eprintln!("{} Unknown ability '{}'", "Error:".red().bold(), ability);
                        eprintln!("Use: str, dex, con, int, wis, cha");
                        std::process::exit(1);
                    }
                };
                let proficiency_str = if save.proficient { " (Proficient)" } else { "" };
                roll_ability_check(
                    &format!("{} Save{}", save_name, proficiency_str),
                    save.modifier,
                    cli.advantage,
                    cli.disadvantage,
                );
            }
            Commands::Attack { weapon } => {
                let weapon_lower = weapon.to_lowercase();
                if let Some(wpn) = character
                    .equipment
                    .weapons
                    .iter()
                    .find(|w| w.name.to_lowercase() == weapon_lower)
                {
                    roll_attack(wpn, cli.advantage, cli.disadvantage);
                } else {
                    eprintln!("{} Weapon '{}' not found", "Error:".red().bold(), weapon);
                    eprintln!("Available weapons:");
                    for wpn in &character.equipment.weapons {
                        eprintln!("  - {}", wpn.name);
                    }
                    std::process::exit(1);
                }
            }
            Commands::Stats => {
                display_stats(&character);
            }
        }
    } else {
        // No command or dice specified - show help
        eprintln!("{} No command specified", "Error:".red().bold());
        eprintln!("Use --help to see available commands");
        eprintln!("\nExamples:");
        eprintln!("  dndrolls --dice 2d6");
        eprintln!("  dndrolls --dice d20 --checkon stealth");
        eprintln!("  dndrolls skill stealth");
        eprintln!("  dndrolls stats");
        std::process::exit(1);
    }
}

// ============================================================================
// Dice Rolling Functions
// ============================================================================

fn run_dice_roll(cli: &Cli) {
    let mut total_modifier = cli.modifier;
    let mut modifier_name = String::new();
    let mut dice_to_roll: Vec<DiceType> = Vec::new();

    // Parse dice
    if let Some(dice_args) = &cli.dice {
        for (count, die_type) in dice_args {
            for _ in 0..*count {
                dice_to_roll.push(*die_type);
            }
        }
    }

    // Apply checkon modifier from character file
    if let Some(check) = &cli.checkon {
        if let Ok(character) = load_character(&cli.character_file) {
            let check_lower = check.to_lowercase();

            if let Some((_, skill)) = get_skill_by_name(&character.skills, &check_lower) {
                total_modifier += skill.modifier;
                modifier_name = check.clone();
            } else {
                // Try ability modifiers
                let ability_mod = match check_lower.as_str() {
                    "str" | "strength" => Some(character.modifiers.strength),
                    "dex" | "dexterity" => Some(character.modifiers.dexterity),
                    "con" | "constitution" => Some(character.modifiers.constitution),
                    "int" | "intelligence" => Some(character.modifiers.intelligence),
                    "wis" | "wisdom" => Some(character.modifiers.wisdom),
                    "cha" | "charisma" => Some(character.modifiers.charisma),
                    _ => None,
                };

                if let Some(mod_val) = ability_mod {
                    total_modifier += mod_val;
                    modifier_name = format!("{} check", check);
                } else {
                    modifier_name = check.clone();
                    eprintln!("Warning: '{}' not found in character sheet", check);
                }
            }
        } else {
            modifier_name = check.clone();
            eprintln!("Warning: Could not load character file for modifier lookup");
        }
    }

    // Default to 1d20 if no dice specified
    if dice_to_roll.is_empty() {
        dice_to_roll.push(DiceType::D20);
    }

    // Roll the dice
    let mut rng = rand::thread_rng();
    let mut results: Vec<(DiceType, u32)> = Vec::new();
    let mut total: i32 = 0;

    for die in &dice_to_roll {
        let roll = rng.gen_range(1..=die.max_value());
        results.push((*die, roll));
        total += roll as i32;
    }

    // Handle advantage/disadvantage for d20 rolls
    if dice_to_roll.len() == 1 && dice_to_roll[0] == DiceType::D20 {
        if cli.advantage && !cli.disadvantage {
            let roll2 = rng.gen_range(1..=20);
            let roll1 = results[0].1;
            let used = roll1.max(roll2);
            let dropped = roll1.min(roll2);
            results[0].1 = used;
            total = used as i32;

            println!("\n{}", "═══════════════════════════════════════".cyan());
            println!(
                "{} {} {}",
                "Rolling:".bold().white(),
                modifier_name.yellow().bold(),
                "(Advantage)".green()
            );
            println!(
                "{} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", used).bright_green().bold(),
                format!("[{}]", dropped).dimmed()
            );
        } else if cli.disadvantage && !cli.advantage {
            let roll2 = rng.gen_range(1..=20);
            let roll1 = results[0].1;
            let used = roll1.min(roll2);
            let dropped = roll1.max(roll2);
            results[0].1 = used;
            total = used as i32;

            println!("\n{}", "═══════════════════════════════════════".cyan());
            println!(
                "{} {} {}",
                "Rolling:".bold().white(),
                modifier_name.yellow().bold(),
                "(Disadvantage)".red()
            );
            println!(
                "{} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", used).bright_red().bold(),
                format!("[{}]", dropped).dimmed()
            );
        } else {
            print_normal_roll(&results, &modifier_name);
        }
    } else {
        print_normal_roll(&results, &modifier_name);
    }

    // Print modifier and total
    if total_modifier != 0 {
        let modifier_str = if total_modifier >= 0 {
            format!("+{}", total_modifier).cyan()
        } else {
            format!("{}", total_modifier).cyan()
        };
        println!("{} {}", "Modifier:".bold().white(), modifier_str);
    }

    let final_total = total + total_modifier;
    let d20_roll = if dice_to_roll.len() == 1 && dice_to_roll[0] == DiceType::D20 {
        Some(results[0].1)
    } else {
        None
    };

    let total_color = match d20_roll {
        Some(20) => format!("{}", final_total).bright_green().bold(),
        Some(1) => format!("{}", final_total).bright_red().bold(),
        _ if final_total >= 20 => format!("{}", final_total).green().bold(),
        _ if final_total >= 15 => format!("{}", final_total).white().bold(),
        _ if final_total >= 10 => format!("{}", final_total).yellow(),
        _ => format!("{}", final_total).red(),
    };

    println!("{} {}", "Total:".bold().white(), total_color);

    if let Some(20) = d20_roll {
        println!("{}", "NATURAL 20! CRITICAL SUCCESS!".bright_green().bold());
    } else if let Some(1) = d20_roll {
        println!("{}", "NATURAL 1! CRITICAL FAILURE!".bright_red().bold());
    }

    println!("{}", "═══════════════════════════════════════".cyan());
}

fn print_normal_roll(results: &[(DiceType, u32)], modifier_name: &str) {
    println!("\n{}", "═══════════════════════════════════════".cyan());
    if !modifier_name.is_empty() {
        println!(
            "{} {}",
            "Rolling:".bold().white(),
            modifier_name.yellow().bold()
        );
    } else {
        let dice_str: Vec<String> = results.iter().map(|(d, _)| d.name().to_string()).collect();
        println!(
            "{} {}",
            "Rolling:".bold().white(),
            dice_str.join(", ").yellow().bold()
        );
    }

    let rolls_str: Vec<String> = results
        .iter()
        .map(|(d, r)| {
            let roll_color = if d == &DiceType::D20 {
                match r {
                    20 => format!("[{}]", r).bright_green().bold().to_string(),
                    1 => format!("[{}]", r).bright_red().bold().to_string(),
                    _ => format!("[{}]", r).bright_white().bold().to_string(),
                }
            } else {
                format!("[{}]", r).bright_white().bold().to_string()
            };
            format!("{}: {}", d.name(), roll_color)
        })
        .collect();
    println!("{} {}", "Dice:".bold().white(), rolls_str.join(", "));
}

fn roll_d20() -> i32 {
    rand::thread_rng().gen_range(1..=20)
}

fn roll_with_advantage_disadvantage(advantage: bool, disadvantage: bool) -> (i32, Option<i32>) {
    if advantage && disadvantage {
        (roll_d20(), None)
    } else if advantage {
        let roll1 = roll_d20();
        let roll2 = roll_d20();
        (roll1.max(roll2), Some(roll1.min(roll2)))
    } else if disadvantage {
        let roll1 = roll_d20();
        let roll2 = roll_d20();
        (roll1.min(roll2), Some(roll1.max(roll2)))
    } else {
        (roll_d20(), None)
    }
}

fn roll_ability_check(name: &str, modifier: i32, advantage: bool, disadvantage: bool) {
    let (dice_roll, dropped_roll) = roll_with_advantage_disadvantage(advantage, disadvantage);
    let total = dice_roll + modifier;
    display_roll_result(
        name,
        dice_roll,
        modifier,
        total,
        dropped_roll,
        advantage,
        disadvantage,
    );
}

fn roll_attack(weapon: &Weapon, advantage: bool, disadvantage: bool) {
    let (dice_roll, dropped_roll) = roll_with_advantage_disadvantage(advantage, disadvantage);
    let total = dice_roll + weapon.attack_bonus;

    println!("\n{}", "═══════════════════════════════════════".cyan());
    println!("{} Attack", weapon.name.bold().yellow());

    if let Some(dropped) = dropped_roll {
        if advantage {
            println!(
                "{} {} {} (dropped {})",
                "Attack Roll:".bold().white(),
                format!("[{}]", dice_roll).bright_green().bold(),
                "(advantage)".green(),
                format!("[{}]", dropped).dimmed()
            );
        } else if disadvantage {
            println!(
                "{} {} {} (dropped {})",
                "Attack Roll:".bold().white(),
                format!("[{}]", dice_roll).bright_red().bold(),
                "(disadvantage)".red(),
                format!("[{}]", dropped).dimmed()
            );
        }
    } else {
        let dice_color = match dice_roll {
            20 => format!("[{}]", dice_roll).bright_green().bold(),
            1 => format!("[{}]", dice_roll).bright_red().bold(),
            _ => format!("[{}]", dice_roll).bright_white().bold(),
        };
        println!("{} {}", "Attack Roll:".bold().white(), dice_color);
    }

    let modifier_str = if weapon.attack_bonus >= 0 {
        format!("+{}", weapon.attack_bonus).cyan()
    } else {
        format!("{}", weapon.attack_bonus).cyan()
    };
    println!("{} {}", "Attack Bonus:".bold().white(), modifier_str);

    let total_color = if dice_roll == 20 {
        format!("{}", total).bright_green().bold()
    } else if dice_roll == 1 {
        format!("{}", total).bright_red().bold()
    } else {
        format!("{}", total).white().bold()
    };
    println!("{} {}", "Total:".bold().white(), total_color);

    if dice_roll == 20 {
        println!("{}", "CRITICAL HIT!".bright_green().bold());
    } else if dice_roll == 1 {
        println!("{}", "CRITICAL MISS!".bright_red().bold());
    }

    println!(
        "{} {} ({})",
        "Damage:".bold().white(),
        weapon.damage.yellow(),
        weapon.damage_type.dimmed()
    );
    println!("{}", "═══════════════════════════════════════".cyan());
}

fn display_roll_result(
    roll_type: &str,
    dice_roll: i32,
    modifier: i32,
    total: i32,
    dropped_roll: Option<i32>,
    advantage: bool,
    disadvantage: bool,
) {
    println!("\n{}", "═══════════════════════════════════════".cyan());
    println!(
        "{} {}",
        "Rolling:".bold().white(),
        roll_type.bold().yellow()
    );

    if let Some(dropped) = dropped_roll {
        if advantage {
            println!(
                "{} {} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", dice_roll).bright_green().bold(),
                "(advantage)".green(),
                format!("[{}]", dropped).dimmed()
            );
        } else if disadvantage {
            println!(
                "{} {} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", dice_roll).bright_red().bold(),
                "(disadvantage)".red(),
                format!("[{}]", dropped).dimmed()
            );
        }
    } else {
        let dice_color = match dice_roll {
            20 => format!("[{}]", dice_roll).bright_green().bold(),
            1 => format!("[{}]", dice_roll).bright_red().bold(),
            _ => format!("[{}]", dice_roll).bright_white().bold(),
        };
        println!("{} {}", "Dice:".bold().white(), dice_color);
    }

    let modifier_str = if modifier >= 0 {
        format!("+{}", modifier).cyan()
    } else {
        format!("{}", modifier).cyan()
    };
    println!("{} {}", "Modifier:".bold().white(), modifier_str);

    let total_color = if dice_roll == 20 {
        format!("{}", total).bright_green().bold()
    } else if dice_roll == 1 {
        format!("{}", total).bright_red().bold()
    } else if total >= 20 {
        format!("{}", total).green().bold()
    } else if total >= 15 {
        format!("{}", total).white().bold()
    } else if total >= 10 {
        format!("{}", total).yellow()
    } else {
        format!("{}", total).red()
    };

    println!("{} {}", "Total:".bold().white(), total_color);

    if dice_roll == 20 {
        println!("{}", "NATURAL 20! CRITICAL SUCCESS!".bright_green().bold());
    } else if dice_roll == 1 {
        println!("{}", "NATURAL 1! CRITICAL FAILURE!".bright_red().bold());
    }

    println!("{}", "═══════════════════════════════════════".cyan());
}

// ============================================================================
// Character Functions
// ============================================================================

fn load_character(path: &PathBuf) -> Result<Character, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let character: Character = serde_json::from_str(&content)?;
    Ok(character)
}

fn get_skill_by_name<'a>(skills: &'a Skills, name: &str) -> Option<(&'static str, &'a Skill)> {
    let name_lower = name.to_lowercase().replace(' ', "");

    match name_lower.as_str() {
        "acrobatics" => Some(("Acrobatics", &skills.acrobatics)),
        "animalhandling" | "animal" => Some(("Animal Handling", &skills.animal_handling)),
        "arcana" => Some(("Arcana", &skills.arcana)),
        "athletics" => Some(("Athletics", &skills.athletics)),
        "deception" => Some(("Deception", &skills.deception)),
        "history" => Some(("History", &skills.history)),
        "insight" => Some(("Insight", &skills.insight)),
        "intimidation" => Some(("Intimidation", &skills.intimidation)),
        "investigation" => Some(("Investigation", &skills.investigation)),
        "medicine" => Some(("Medicine", &skills.medicine)),
        "nature" => Some(("Nature", &skills.nature)),
        "perception" => Some(("Perception", &skills.perception)),
        "performance" => Some(("Performance", &skills.performance)),
        "persuasion" => Some(("Persuasion", &skills.persuasion)),
        "religion" => Some(("Religion", &skills.religion)),
        "sleightofhand" | "sleight" => Some(("Sleight of Hand", &skills.sleight_of_hand)),
        "stealth" => Some(("Stealth", &skills.stealth)),
        "survival" => Some(("Survival", &skills.survival)),
        _ => None,
    }
}

fn display_stats(character: &Character) {
    let info = &character.character;
    println!("\n{}", "═══════════════════════════════════════".cyan());
    println!("{}", "CHARACTER STATS".bold().yellow());
    println!("{}", "═══════════════════════════════════════".cyan());

    let name_display = if let Some(alter_ego) = &info.alter_ego {
        format!("{} ({})", info.name, alter_ego)
    } else {
        info.name.clone()
    };
    println!("{} {}", "Name:".bold().white(), name_display.green());

    let class_display = if let Some(subclass) = &info.subclass {
        format!("{} ({})", info.class, subclass)
    } else {
        info.class.clone()
    };
    println!(
        "{} Level {} {} {}",
        "Class:".bold().white(),
        info.level,
        class_display.cyan(),
        info.race.yellow()
    );

    println!("\n{}", "ATTRIBUTES".bold().yellow());
    println!(
        "  {} {} ({:+})",
        "STR:".bold(),
        character.attributes.strength,
        character.modifiers.strength
    );
    println!(
        "  {} {} ({:+})",
        "DEX:".bold(),
        character.attributes.dexterity,
        character.modifiers.dexterity
    );
    println!(
        "  {} {} ({:+})",
        "CON:".bold(),
        character.attributes.constitution,
        character.modifiers.constitution
    );
    println!(
        "  {} {} ({:+})",
        "INT:".bold(),
        character.attributes.intelligence,
        character.modifiers.intelligence
    );
    println!(
        "  {} {} ({:+})",
        "WIS:".bold(),
        character.attributes.wisdom,
        character.modifiers.wisdom
    );
    println!(
        "  {} {} ({:+})",
        "CHA:".bold(),
        character.attributes.charisma,
        character.modifiers.charisma
    );

    println!("\n{}", "COMBAT".bold().yellow());
    println!("  {} {}", "AC:".bold(), character.combat.armor_class);
    println!(
        "  {} {:+}",
        "Initiative:".bold(),
        character.combat.initiative
    );
    println!(
        "  {} {}/{}",
        "HP:".bold(),
        character.combat.hit_points.current,
        character.combat.hit_points.maximum
    );
    println!(
        "  {} {:+}",
        "Proficiency Bonus:".bold(),
        character.proficiency_bonus
    );

    println!("\n{}", "WEAPONS".bold().yellow());
    for weapon in &character.equipment.weapons {
        println!(
            "  {} {:+} to hit, {} {}",
            weapon.name.bold(),
            weapon.attack_bonus,
            weapon.damage,
            weapon.damage_type.dimmed()
        );
    }

    println!("{}", "═══════════════════════════════════════".cyan());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_d20_in_range() {
        for _ in 0..100 {
            let roll = roll_d20();
            assert!(roll >= 1 && roll <= 20, "Roll {} out of range", roll);
        }
    }

    #[test]
    fn test_advantage_disadvantage_cancel() {
        for _ in 0..10 {
            let (_, dropped) = roll_with_advantage_disadvantage(true, true);
            assert!(dropped.is_none(), "Should cancel out with no dropped roll");
        }
    }

    #[test]
    fn test_normal_roll_no_dropped() {
        for _ in 0..10 {
            let (_, dropped) = roll_with_advantage_disadvantage(false, false);
            assert!(dropped.is_none(), "Normal roll should have no dropped roll");
        }
    }

    #[test]
    fn test_advantage_has_dropped_roll() {
        for _ in 0..10 {
            let (used, dropped) = roll_with_advantage_disadvantage(true, false);
            assert!(dropped.is_some(), "Advantage should have dropped roll");
            let dropped_val = dropped.unwrap();
            assert!(used >= dropped_val, "Advantage should use higher roll");
        }
    }

    #[test]
    fn test_disadvantage_has_dropped_roll() {
        for _ in 0..10 {
            let (used, dropped) = roll_with_advantage_disadvantage(false, true);
            assert!(dropped.is_some(), "Disadvantage should have dropped roll");
            let dropped_val = dropped.unwrap();
            assert!(used <= dropped_val, "Disadvantage should use lower roll");
        }
    }

    #[test]
    fn test_parse_dice_arg() {
        assert_eq!(parse_dice_arg("d20").unwrap(), (1, DiceType::D20));
        assert_eq!(parse_dice_arg("2d6").unwrap(), (2, DiceType::D6));
        assert_eq!(parse_dice_arg("1d8").unwrap(), (1, DiceType::D8));
        assert_eq!(parse_dice_arg("D20").unwrap(), (1, DiceType::D20));
        assert!(parse_dice_arg("invalid").is_err());
        assert!(parse_dice_arg("2d100").is_err());
    }
}
