# Screenshots

This directory contains screenshots and examples of the DnD Game Rolls application.

## Directory Structure

```
screenshots/
├── cli/                    # CLI dice roller examples
│   ├── README.md
│   ├── stats.txt           # Character stats display
│   ├── skill-stealth.txt   # Stealth check (Expertise)
│   ├── skill-perception-advantage.txt
│   ├── ability-dex.txt     # Dexterity ability check
│   ├── save-dex.txt        # DEX saving throw
│   └── attack-dagger.txt   # Dagger attack roll
│
└── dice3d/                 # 3D dice simulator examples
    ├── README.md
    ├── basic-roll.md       # Basic d20 roll
    ├── skill-check.md      # Roll with skill modifier
    ├── rolling.png         # (add screenshot)
    ├── results.png         # (add screenshot)
    └── command-input.png   # (add screenshot)
```

## Adding Screenshots

### For dice3d (graphical)
1. Run `cargo run --bin dice3d`
2. Use Windows Snipping Tool (Win + Shift + S)
3. Save as PNG in `screenshots/dice3d/`

### For CLI
The text examples are already captured in the `cli/` folder.
To create image screenshots:
1. Run the command in Windows Terminal
2. Screenshot the terminal output
3. Save as PNG in `screenshots/cli/`

## Screenshot Checklist

### CLI (text examples included ✓)
- [x] `stats.txt` - Character stats
- [x] `skill-stealth.txt` - Stealth check
- [x] `skill-perception-advantage.txt` - Advantage roll
- [x] `ability-dex.txt` - Ability check
- [x] `save-dex.txt` - Saving throw
- [x] `attack-dagger.txt` - Attack roll

### dice3d (need PNG screenshots)
- [ ] `rolling.png` - Dice in motion
- [ ] `results.png` - Settled with results
- [ ] `command-input.png` - Command input UI
- [ ] `history.png` - Command history panel
- [ ] `multi-dice.png` - Multiple dice types
