# DnD Game Rolls

[![CI](https://github.com/edgarhsanchez/dndgamerolls/actions/workflows/ci.yml/badge.svg)](https://github.com/edgarhsanchez/dndgamerolls/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/dndgamerolls.svg)](https://crates.io/crates/dndgamerolls)
[![License](https://img.shields.io/crates/l/dndgamerolls.svg)](https://github.com/edgarhsanchez/dndgamerolls/blob/main/LICENSE)

A D&D 5e dice roller with both **CLI** and **3D visualization** in a single binary!  
Powered by Bevy game engine with real physics simulation.

## Screenshots

### 3D Dice Simulator

| Rolling Dice | Results Display |
|:------------:|:---------------:|
| ![Dice Rolling](screenshots/dice3d/dice3d-rolling.png) | ![Dice Results](screenshots/dice3d/dice3d-results.png) |

| Command Input | Command History |
|:-------------:|:---------------:|
| ![Command Input](screenshots/dice3d/dice3d-command-input.png) | ![Command History](screenshots/dice3d/dice3d-history.png) |

---

## Installation

### From crates.io (Recommended)

```bash
cargo install dndgamerolls
```

### From Source

```bash
git clone https://github.com/edgarhsanchez/dndgamerolls.git
cd dndgamerolls
cargo build --release
```

The executable will be at: `target/release/dndgamerolls.exe`

---

## Usage

### 3D Mode (Default)

Simply run without `--cli` to launch the 3D dice simulator:

```bash
# Launch 3D simulator with default d20
dndgamerolls

# Launch with specific dice
dndgamerolls --dice 2d6 --dice 1d20

# Launch with skill modifier applied
dndgamerolls --dice 1d20 --checkon stealth
```

### CLI Mode

Use `--cli` for headless command-line rolling:

```bash
# Roll dice with skill modifier (no GUI)
dndgamerolls --cli --dice 2d10 --checkon perception

# Roll with advantage
dndgamerolls --cli --dice 1d20 --checkon stealth --advantage

# Traditional subcommands also work
dndgamerolls skill stealth
dndgamerolls --advantage skill perception
dndgamerolls attack dagger
dndgamerolls stats
```

### Quick Examples

```bash
# View character stats
dndgamerolls stats

# Skill check with expertise
dndgamerolls skill stealth
# 🎲 Stealth Check (Expertise)
# Roll: 15 + 9 = 24

# Skill check with advantage
dndgamerolls --advantage skill perception

# Attack roll with damage
dndgamerolls attack dagger

# CLI mode: Roll 2d10 + perception modifier
dndgamerolls --cli --dice 2d10 --checkon perception
```

For more examples, see the [screenshots directory](screenshots/README.md).

---

## 3D Dice Simulator

A visual 3D dice rolling experience with physics simulation.

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

### Command Input Mode

Press `/` or `Enter` to open command input, then type commands like:
- `--dice 2d6 --checkon stealth` - Roll 2d6 with stealth modifier
- `1d20 --checkon perception` - Roll d20 with perception modifier
- `--dice 1d20 --dice 1d8 --modifier 3` - Roll multiple dice with bonus

Press **1-9** to quickly reroll from command history shown on the right.

### Features
- 🎲 All standard D&D dice types (D4, D6, D8, D10, D12, D20)
- ⚡ Real-time physics simulation with Rapier3D
- 🎨 Crystal-themed translucent dice
- 📦 Glass dice box with realistic bouncing
- 🎯 Automatic result detection when dice stop
- 💡 Dynamic lighting and shadows
- 📝 Command history for quick rerolls

---

## CLI Mode

The CLI mode provides quick command-line dice rolling without the 3D GUI.

### Traditional Subcommands

```bash
# Ability Checks
dndgamerolls strength      # or str
dndgamerolls dexterity     # or dex

# Skill Checks
dndgamerolls skill stealth
dndgamerolls skill perception

# Saving Throws
dndgamerolls save dex
dndgamerolls save wisdom

# Attack Rolls
dndgamerolls attack shortsword
dndgamerolls attack dagger

# View Character Stats
dndgamerolls stats
```

### Advanced CLI Mode

Use `--cli` with `--dice` and `--checkon` for flexible dice rolling:

```bash
# Roll 2d10 with perception modifier
dndgamerolls --cli --dice 2d10 --checkon perception

# Roll 1d20 with stealth and advantage
dndgamerolls --cli --dice 1d20 --checkon stealth --advantage

# Roll multiple dice types
dndgamerolls --cli --dice 1d20 --dice 2d6 --modifier 5

# Add custom modifier
dndgamerolls --cli --dice 3d8 --modifier 10
```

### Advantage/Disadvantage

Add `--advantage` or `--disadvantage` before the subcommand:

```bash
dndgamerolls --advantage skill stealth
dndgamerolls --disadvantage attack shortsword
dndgamerolls --cli --dice 1d20 --checkon perception --advantage
```

#### Custom Character File
By default, the tool looks for `dnd_stats.json` in the current directory. You can specify a different file:

```bash
dndgamerolls -f path/to/character.json intelligence
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

## Building for Release

```bash
cargo build --release
```

Copy the executable to a directory in your PATH for easy access:

```powershell
# Windows
copy target\release\dndgamerolls.exe C:\Users\YourName\.cargo\bin\

# Or install directly from source
cargo install --path .
```

Now you can run `dndgamerolls` from anywhere!

## License

See [LICENSE](LICENSE) for details.
