# Screenshots

This directory contains screenshots and examples of the DnD Game Rolls application.

## 3D Dice Simulator Preview

![3D Dice Results](dice3d/dice3d-results.png)

## UI Screenshots

### Dice Roller

| Dice Roller Tab | Cup Container |
|:--------------:|:-------------:|
| ![Dice Roller Tab](diceroller_tab.png) | ![Dice Roller Cup](dice_roller_cup.png) |

| Fine Control Settings | No Limit on Dice |
|:--------------------:|:----------------:|
| ![Fine Control Settings](dice_roller_settings_fine_control.png) | ![No Limit on Dice](no_limit_on_dice.png) |

### Character Sheet

![Character Setup](character_setup_character.png)

## Directory Structure

```
screenshots/
├── basic_play.mp4                 # Short gameplay clip
├── character_setup_character.png  # Character sheet / setup
├── diceroller_tab.png             # Main dice roller tab
├── dice_roller_cup.png            # Cup container mode
├── dice_roller_settings_fine_control.png # Fine control settings UI
├── no_limit_on_dice.png           # Stress test / many dice
├── cli/                           # CLI dice roller examples
│   ├── README.md
│   ├── stats.txt                  # Character stats display
│   ├── skill-stealth.txt          # Stealth check (Expertise)
│   ├── skill-perception-advantage.txt
│   ├── ability-dex.txt            # Dexterity ability check
│   ├── save-dex.txt               # DEX saving throw
│   └── attack-dagger.txt          # Dagger attack roll
│
└── dice3d/                        # 3D dice simulator examples
    ├── README.md
    ├── basic-roll.md              # Basic d20 roll
    ├── skill-check.md             # Roll with skill modifier
    ├── dice3d-basic.png           # Default view
    ├── dice3d-rolling.png         # Dice in motion
    ├── dice3d-results.png         # Results display
    ├── dice3d-command-input.png   # Command input mode
    └── dice3d-history.png         # Command history
```

## Quick Links

- [CLI Examples](cli/README.md) - Text output samples
- [3D Simulator Examples](dice3d/README.md) - Visual screenshots and usage

## Screenshot Checklist

### CLI (text examples included ✓)
- [x] `stats.txt` - Character stats
- [x] `skill-stealth.txt` - Stealth check
- [x] `skill-perception-advantage.txt` - Advantage roll
- [x] `ability-dex.txt` - Ability check
- [x] `save-dex.txt` - Saving throw
- [x] `attack-dagger.txt` - Attack roll

### dice3d (need PNG screenshots)
- [x] `dice3d-rolling.png` - Dice in motion
- [x] `dice3d-results.png` - Settled with results
- [x] `dice3d-command-input.png` - Command input UI
- [x] `dice3d-history.png` - Command history panel
