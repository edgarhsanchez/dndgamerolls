# DnD Game Rolls CLI

A lightweight command-line D&D dice roller with character sheet support.

## Installation

```bash
cargo install dndgamerolls-cli
```

## Usage

### Basic Dice Rolling

```bash
# Roll a d20
dndrolls --dice d20

# Roll 2d6
dndrolls --dice 2d6

# Roll multiple dice
dndrolls --dice 2d6 --dice d8

# Roll with advantage
dndrolls --dice d20 --advantage

# Roll with disadvantage
dndrolls --dice d20 --disadvantage
```

### Character-Based Rolls

By default, the CLI loads character data from the local SurrealDB database (`characters.surrealdb`) used by the 3D app.

You can also provide a JSON file as one-off input (see example format below):

```bash
# Roll a skill check
dndrolls skill stealth

# Roll an ability check
dndrolls str
dndrolls dex

# Roll a saving throw
dndrolls save wis

# Roll an attack
dndrolls attack "longsword"

# Display character stats
dndrolls stats

# Roll with modifier from character sheet
dndrolls --dice d20 --checkon perception
```

### Options

- `-d, --dice <DICE>` - Dice to roll (e.g., "2d6", "1d20", "d8")
- `-f, --file <FILE>` - Path to character stats JSON file (optional; if omitted, loads from SQLite)
- `-f, --file <FILE>` - Path to character stats JSON file (optional; if omitted, loads from local SurrealDB)
- `--character <NAME>` - Select a character by name from the local database (ignored when --file is provided)
- `--character-id <ID>` - Select a character by id from the local database (ignored when --file is provided)
- `--checkon <NAME>` - Apply modifier from skill, ability, or save
- `-m, --modifier <NUM>` - Custom modifier to add
- `-a, --advantage` - Roll with advantage
- `-D, --disadvantage` - Roll with disadvantage

## Example Character File

```json
{
  "character": {
    "name": "Thorin",
    "class": "Fighter",
    "race": "Dwarf",
    "level": 5
  },
  "attributes": {
    "strength": 16,
    "dexterity": 14,
    "constitution": 15,
    "intelligence": 10,
    "wisdom": 12,
    "charisma": 8
  },
  "modifiers": {
    "strength": 3,
    "dexterity": 2,
    "constitution": 2,
    "intelligence": 0,
    "wisdom": 1,
    "charisma": -1
  },
  "combat": {
    "armorClass": 18,
    "initiative": 2,
    "hitPoints": { "current": 44, "maximum": 44 }
  },
  "proficiencyBonus": 3,
  "savingThrows": {
    "strength": { "proficient": true, "modifier": 6 },
    "dexterity": { "proficient": false, "modifier": 2 },
    "constitution": { "proficient": true, "modifier": 5 },
    "intelligence": { "proficient": false, "modifier": 0 },
    "wisdom": { "proficient": false, "modifier": 1 },
    "charisma": { "proficient": false, "modifier": -1 }
  },
  "skills": {
    "acrobatics": { "proficient": false, "modifier": 2 },
    "animalHandling": { "proficient": false, "modifier": 1 },
    "arcana": { "proficient": false, "modifier": 0 },
    "athletics": { "proficient": true, "modifier": 6 },
    "deception": { "proficient": false, "modifier": -1 },
    "history": { "proficient": false, "modifier": 0 },
    "insight": { "proficient": false, "modifier": 1 },
    "intimidation": { "proficient": true, "modifier": 2 },
    "investigation": { "proficient": false, "modifier": 0 },
    "medicine": { "proficient": false, "modifier": 1 },
    "nature": { "proficient": false, "modifier": 0 },
    "perception": { "proficient": true, "modifier": 4 },
    "performance": { "proficient": false, "modifier": -1 },
    "persuasion": { "proficient": false, "modifier": -1 },
    "religion": { "proficient": false, "modifier": 0 },
    "sleightOfHand": { "proficient": false, "modifier": 2 },
    "stealth": { "proficient": false, "modifier": 2 },
    "survival": { "proficient": true, "modifier": 4 }
  },
  "equipment": {
    "weapons": [
      {
        "name": "Longsword",
        "attackBonus": 6,
        "damage": "1d8+3",
        "damageType": "slashing"
      }
    ]
  }
}
```

## License

MIT License - Copyright (c) 2025 Edgar Sanchez
