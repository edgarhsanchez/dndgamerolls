# GameRoll - D&D Dice Roller

A command-line dice roller for D&D 5e that uses your character stats from a JSON file.  
Includes a **3D Dice Simulator** powered by Bevy game engine!

## Installation

### From crates.io (Recommended)

```bash
# Install both CLI and 3D simulator
cargo install gameroll
```

### From Microsoft Store (Windows)

[![Get it from Microsoft Store](https://get.microsoft.com/images/en-us%20dark.svg)](https://apps.microsoft.com/store/detail/STORE_ID)

*Coming soon!*

### From Source

```bash
git clone https://github.com/edgarhsanchez/dndgamerolls.git
cd dndgamerolls
cargo build --release
```

The executables will be at:
- `target/release/gameroll.exe` - CLI dice roller
- `target/release/dice3d.exe` - 3D dice simulator

### From GitHub Releases

Download pre-built binaries from the [Releases](https://github.com/edgarhsanchez/dndgamerolls/releases) page.

---

## 3D Dice Simulator (dice3d)

A visual 3D dice rolling experience with physics simulation.

### Running the 3D Simulator
```bash
cargo run --bin dice3d
```

### Controls

| Key | Action |
|-----|--------|
| **SPACE** | Roll a complete set of dice (D4, D6, D8, D10, D12, D20) |
| **1** | Spawn D4 (tetrahedron) |
| **2** | Spawn D6 (cube) |
| **3** | Spawn D8 (octahedron) |
| **4** | Spawn D10 (pentagonal trapezohedron) |
| **5** | Spawn D12 (dodecahedron) |
| **6** | Spawn D20 (icosahedron) |
| **R** | Clear all dice |
| **W/A/S/D** | Move camera |
| **Q/E** | Rotate camera |

### Features
- 🎲 All standard D&D dice types (D4, D6, D8, D10, D12, D20)
- ⚡ Real-time physics simulation with Rapier3D
- 🎨 Color-coded dice for easy identification
- 📦 Wooden dice box with realistic bouncing
- 🎯 Automatic result detection when dice stop
- 💡 Dynamic lighting and shadows

### Custom 3D Models

You can add your own dice models in the `assets/models/` folder. See `assets/models/README.md` for detailed instructions on:
- Supported formats (GLTF/GLB recommended)
- Model requirements and scale
- Converting FBX to GLTF
- Creating dice in Blender

---

## CLI Dice Roller (gameroll)

## Usage

### Ability Checks
Roll ability checks using the full name or abbreviation:

```bash
gameroll strength      # or gameroll str
gameroll dexterity     # or gameroll dex
gameroll constitution  # or gameroll con
gameroll intelligence  # or gameroll int
gameroll wisdom        # or gameroll wis
gameroll charisma      # or gameroll cha
```

### Initiative
```bash
gameroll initiative
```

### Skill Checks
```bash
gameroll skill stealth
gameroll skill perception
gameroll skill acrobatics
gameroll skill investigation
```

### Saving Throws
```bash
gameroll save dex
gameroll save int
gameroll save wisdom
```

### Attack Rolls
```bash
gameroll attack shortsword
gameroll attack shortbow
gameroll attack dagger
```

### Advantage/Disadvantage
Add `-a` for advantage or `-d` for disadvantage:

```bash
gameroll dex -a              # Roll with advantage
gameroll skill stealth -d    # Roll with disadvantage
gameroll attack shortsword -a
```

### View Character Stats
```bash
gameroll stats
```

### Custom Character File
By default, the tool looks for `dnd_stats.json` in the current directory. You can specify a different file:

```bash
gameroll -f path/to/character.json intelligence
```

## Features

- 🎲 D&D 5e compliant rolls (d20 + modifier)
- ✨ Advantage/Disadvantage support
- 🎨 Colored output with critical success/failure highlighting
- 📊 Displays both dice roll and final total
- 🎯 Automatic modifier calculation from character stats
- 🗡️ Attack rolls with weapon stats
- 💾 Loads character data from JSON file
- 🎭 Shows expertise on relevant skills

## D&D Rules Implemented

- **Natural 20**: Critical success (highlighted in green)
- **Natural 1**: Critical failure (highlighted in red)
- **Advantage**: Roll twice, take the higher result
- **Disadvantage**: Roll twice, take the lower result
- **Proficiency Bonus**: Automatically applied to proficient saves/skills
- **Expertise**: Double proficiency bonus (shown in character stats)

## Character File Format

The tool expects a JSON file with your character stats. See `dnd_stats.json` for the full structure.

## Examples

```bash
# Strawberry Picker (Blackberry) rolls stealth with expertise
gameroll skill stealth
# Output: d20 roll + 9 modifier (with expertise)

# Roll intelligence check with advantage
gameroll int -a
# Output: Rolls twice, takes higher + 1 modifier

# Attack with shortsword
gameroll attack shortsword
# Output: d20 + 7 attack bonus, shows 1d6+5 damage

# Check initiative
gameroll initiative
# Output: d20 + 5 (Dexterity modifier)
```

## Building for Release

```bash
cargo build --release
```

Copy the executable to a directory in your PATH for easy access:

```powershell
copy target\release\gameroll.exe C:\Users\YourName\.cargo\bin\
```

Now you can run `gameroll` from anywhere!
