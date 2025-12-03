# CLI Examples

This folder contains examples of the CLI dice roller output.

## Files

- `stats.txt` - Character stats display
- `skill-stealth.txt` - Stealth skill check (with Expertise)
- `skill-perception-advantage.txt` - Perception with advantage
- `ability-dex.txt` - Dexterity ability check
- `save-dex.txt` - Dexterity saving throw
- `attack-dagger.txt` - Dagger attack roll

## How to Reproduce

```bash
# Character stats
dndgamerolls stats

# Skill checks
dndgamerolls skill stealth
dndgamerolls --advantage skill perception

# Ability checks
dndgamerolls dex

# Saving throws
dndgamerolls save dex

# Attacks
dndgamerolls attack dagger
```
