# 3D Dice Simulator Examples

This folder contains screenshots and examples of the dice3d simulator.

## Screenshots

### Basic View
The dice box ready for rolling.

![Basic View](dice3d-basic.png)

### Dice Rolling
Dice in motion with physics simulation.

![Dice Rolling](dice3d-rolling.png)

### Results Display
Dice settled with results shown in the top-left corner.

![Results](dice3d-results.png)

### Command Input
Press `/` or `Enter` to open the command input field at the bottom.

![Command Input](dice3d-command-input.png)

### Command History
Previously executed commands appear on the right side. Press 1-9 to reroll.

![Command History](dice3d-history.png)

---

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
