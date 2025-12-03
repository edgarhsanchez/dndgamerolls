# 3D Dice Simulator Examples

This folder contains examples of the dice3d simulator.

## Screenshots to Add

Please take screenshots of the following scenarios and save them here:

| Filename | Description | How to Capture |
|----------|-------------|----------------|
| `rolling.png` | Dice in motion | Press SPACE, screenshot while dice are bouncing |
| `results.png` | Dice settled with results | Wait for dice to stop, results shown top-left |
| `command-input.png` | Command input active | Press `/` to activate, type a command |
| `history.png` | Command history visible | Execute several commands, history shows on right |
| `multi-dice.png` | Multiple dice types | Run with `--dice 1d20 --dice 2d6 --dice 1d8` |
| `skill-check.png` | Skill check with modifier | Use `--checkon stealth` to show modifier |

## Example Commands

```bash
# Default roll (1d20)
dice3d

# Multiple dice
dice3d --dice 2d6 --dice 1d20

# With skill modifier
dice3d --dice 1d20 --checkon stealth

# Custom dice combo
dice3d --dice 1d20 --dice 1d8 --modifier 5
```

## In-App Commands

Once running, press `/` or `Enter` to open command input:
- `--dice 2d6 --checkon stealth`
- `1d20 --checkon perception`
- `--dice 1d20 --dice 1d8 --modifier 3`

Press 1-9 to reroll from history.
