use clap::{Parser, Subcommand};
use colored::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gameroll")]
#[command(about = "D&D dice roller using character stats", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the character stats JSON file
    #[arg(short, long, default_value = "dnd_stats.json")]
    file: PathBuf,

    /// Roll with advantage (roll twice, take higher)
    #[arg(short, long)]
    advantage: bool,

    /// Roll with disadvantage (roll twice, take lower)
    #[arg(short, long)]
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

fn roll_d20() -> i32 {
    rand::thread_rng().gen_range(1..=20)
}

fn roll_with_advantage_disadvantage(advantage: bool, disadvantage: bool) -> (i32, Option<i32>) {
    if advantage && disadvantage {
        // Cancel out - roll normally
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
        println!(
            "{}",
            "🎉 NATURAL 20! CRITICAL SUCCESS! 🎉".bright_green().bold()
        );
    } else if dice_roll == 1 {
        println!(
            "{}",
            "💀 NATURAL 1! CRITICAL FAILURE! 💀".bright_red().bold()
        );
    }

    println!("{}", "═══════════════════════════════════════".cyan());
}

fn load_character(path: &PathBuf) -> Result<Character, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let character: Character = serde_json::from_str(&content)?;
    Ok(character)
}

fn get_skill_by_name<'a>(skills: &'a Skills, name: &str) -> Option<(&'static str, &'a Skill)> {
    let name_lower = name.to_lowercase().replace(" ", "");

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
    println!("{} {}", "Class:".bold().white(), class_display.cyan());
    println!("{} {}", "Race:".bold().white(), info.race.cyan());
    println!(
        "{} {}",
        "Level:".bold().white(),
        info.level.to_string().yellow()
    );

    println!("\n{}", "ABILITIES".bold().yellow());
    println!(
        "{} {} ({})",
        "STR:".bold(),
        character.attributes.strength,
        format_modifier(character.modifiers.strength)
    );
    println!(
        "{} {} ({})",
        "DEX:".bold(),
        character.attributes.dexterity,
        format_modifier(character.modifiers.dexterity)
    );
    println!(
        "{} {} ({})",
        "CON:".bold(),
        character.attributes.constitution,
        format_modifier(character.modifiers.constitution)
    );
    println!(
        "{} {} ({})",
        "INT:".bold(),
        character.attributes.intelligence,
        format_modifier(character.modifiers.intelligence)
    );
    println!(
        "{} {} ({})",
        "WIS:".bold(),
        character.attributes.wisdom,
        format_modifier(character.modifiers.wisdom)
    );
    println!(
        "{} {} ({})",
        "CHA:".bold(),
        character.attributes.charisma,
        format_modifier(character.modifiers.charisma)
    );

    println!("\n{}", "COMBAT".bold().yellow());
    println!(
        "{} {}",
        "AC:".bold(),
        character.combat.armor_class.to_string().green()
    );
    println!(
        "{} {}/{}",
        "HP:".bold(),
        character.combat.hit_points.current.to_string().red(),
        character.combat.hit_points.maximum.to_string().red()
    );
    println!(
        "{} {}",
        "Initiative:".bold(),
        format_modifier(character.combat.initiative)
    );
    println!(
        "{} {}",
        "Proficiency Bonus:".bold(),
        format_modifier(character.proficiency_bonus)
    );

    println!("{}", "═══════════════════════════════════════".cyan());
}

fn format_modifier(modifier: i32) -> String {
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    }
}

fn main() {
    let cli = Cli::parse();

    let character = match load_character(&cli.file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{} Failed to load character file: {}",
                "Error:".red().bold(),
                e
            );
            eprintln!(
                "Make sure '{}' exists in the current directory",
                cli.file.display()
            );
            std::process::exit(1);
        }
    };

    match &cli.command {
        Some(Commands::Strength) => {
            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = character.modifiers.strength;
            let total = dice_roll + modifier;
            display_roll_result(
                "Strength Check",
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Dexterity) => {
            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = character.modifiers.dexterity;
            let total = dice_roll + modifier;
            display_roll_result(
                "Dexterity Check",
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Constitution) => {
            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = character.modifiers.constitution;
            let total = dice_roll + modifier;
            display_roll_result(
                "Constitution Check",
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Intelligence) => {
            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = character.modifiers.intelligence;
            let total = dice_roll + modifier;
            display_roll_result(
                "Intelligence Check",
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Wisdom) => {
            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = character.modifiers.wisdom;
            let total = dice_roll + modifier;
            display_roll_result(
                "Wisdom Check",
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Charisma) => {
            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = character.modifiers.charisma;
            let total = dice_roll + modifier;
            display_roll_result(
                "Charisma Check",
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Initiative) => {
            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = character.combat.initiative;
            let total = dice_roll + modifier;
            display_roll_result(
                "Initiative",
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Skill { name }) => {
            if let Some((skill_name, skill)) = get_skill_by_name(&character.skills, name) {
                let (dice_roll, dropped) =
                    roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
                let modifier = skill.modifier;
                let total = dice_roll + modifier;
                let expertise_note = if skill.expertise { " (Expertise)" } else { "" };
                display_roll_result(
                    &format!("{} Check{}", skill_name, expertise_note),
                    dice_roll,
                    modifier,
                    total,
                    dropped,
                    cli.advantage,
                    cli.disadvantage,
                );
            } else {
                eprintln!("{} Skill '{}' not found", "Error:".red().bold(), name);
                eprintln!("Available skills: acrobatics, animal-handling, arcana, athletics, deception, history, insight, intimidation, investigation, medicine, nature, perception, performance, persuasion, religion, sleight-of-hand, stealth, survival");
                std::process::exit(1);
            }
        }
        Some(Commands::Save { ability }) => {
            let ability_lower = ability.to_lowercase();
            let (save_name, save) = match ability_lower.as_str() {
                "str" | "strength" => ("Strength", &character.saving_throws.strength),
                "dex" | "dexterity" => ("Dexterity", &character.saving_throws.dexterity),
                "con" | "constitution" => ("Constitution", &character.saving_throws.constitution),
                "int" | "intelligence" => ("Intelligence", &character.saving_throws.intelligence),
                "wis" | "wisdom" => ("Wisdom", &character.saving_throws.wisdom),
                "cha" | "charisma" => ("Charisma", &character.saving_throws.charisma),
                _ => {
                    eprintln!("{} Unknown ability: {}", "Error:".red().bold(), ability);
                    std::process::exit(1);
                }
            };

            let (dice_roll, dropped) =
                roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
            let modifier = save.modifier;
            let total = dice_roll + modifier;
            display_roll_result(
                &format!("{} Saving Throw", save_name),
                dice_roll,
                modifier,
                total,
                dropped,
                cli.advantage,
                cli.disadvantage,
            );
        }
        Some(Commands::Attack { weapon }) => {
            let weapon_lower = weapon.to_lowercase();
            if let Some(wpn) = character
                .equipment
                .weapons
                .iter()
                .find(|w| w.name.to_lowercase().contains(&weapon_lower))
            {
                let (dice_roll, dropped) =
                    roll_with_advantage_disadvantage(cli.advantage, cli.disadvantage);
                let modifier = wpn.attack_bonus;
                let total = dice_roll + modifier;
                display_roll_result(
                    &format!("{} Attack", wpn.name),
                    dice_roll,
                    modifier,
                    total,
                    dropped,
                    cli.advantage,
                    cli.disadvantage,
                );

                if dice_roll == 20 {
                    println!(
                        "\n{}",
                        "Roll damage dice twice for critical hit!"
                            .bright_green()
                            .bold()
                    );
                }
                println!("{} {}", "Damage:".bold().white(), wpn.damage.yellow());
            } else {
                eprintln!("{} Weapon '{}' not found", "Error:".red().bold(), weapon);
                eprintln!("Available weapons:");
                for wpn in &character.equipment.weapons {
                    eprintln!("  - {}", wpn.name);
                }
                std::process::exit(1);
            }
        }
        Some(Commands::Stats) => {
            display_stats(&character);
        }
        None => {
            eprintln!("{} No command specified", "Error:".red().bold());
            eprintln!("Use --help to see available commands");
            std::process::exit(1);
        }
    }
}
