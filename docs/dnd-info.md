---
layout: default
title: DnD Info
---

# DnD Info (How Rolls Work)

This page is the same content used by the in-app **DnD Info** tab.

---

## DnD Game Rolls: How Rolls Work (App Guide)

This tab documents the exact roll behaviors supported by the app (GUI + CLI) and how they map to common D&D 5e mechanics.

## Dice Roller Tab (3D)

- Click in the dice box to throw the current dice set.
- Press `R` to reset dice positions (no roll).
- Results panel shows the final values once dice settle (and includes your modifier/label when applicable).
- Panels like Results / Quick Rolls / Command History can be dragged (positions persist via settings).

## Command Input (GUI)

The Command box supports a small, app-specific command format (whitespace-separated):

- Dice: `d20`, `2d6`, `d8`, `3d10`, etc.
- Options:
  - `--dice` / `-d <NdX>` (same as writing the dice directly)
  - `--modifier` / `-m <number>` (adds a flat bonus/penalty)
  - `--checkon <name>` (pulls the modifier from your Character Sheet by skill/ability/save name)

### Examples

```text
d20
2d6 --modifier 3
d20 --checkon stealth
d20 --checkon dex --modifier 2
```

Notes:

- For multi-word skills, use the Character Sheet's internal name (e.g., `sleightOfHand`, `animalHandling`).
- The GUI command input does NOT use the shorthand `1d20+5` style. Use `--modifier 5` instead.

## Command History

- The Command History panel shows past commands.
- Click a history entry to select it and reroll that same command.

## Quick Rolls (Dice View)

Quick Rolls are one-click rolls powered by your Character Sheet:

- Skills: rolls 1 die + the skill modifier.
- Ability Checks: rolls 1 die + the ability modifier.
- Saving Throws: rolls 1 die + the saving throw modifier.

App-specific setting: Quick Rolls die type is configurable (Dice Settings → Quick Rolls die type). In standard D&D 5e, checks/saves/attacks are typically a d20 — if you change this, Quick Rolls will no longer match standard rules.

## D&D 5e Mechanics Refresher

Core pattern: roll + modifier, then compare.

- Ability / Skill check: `d20 + modifier` vs DC
- Saving throw: `d20 + save modifier` vs save DC
- Attack roll: `d20 + attack bonus` vs AC
- Damage: roll the damage dice separately (e.g., `1d8+3`)

Saving throws: `d20 + the relevant save modifier` vs the effect DC.

- Concentration checks are Constitution saves: DC 10 or half the damage taken (whichever is higher).
- Spell save DC reminder (typical 5e): `8 + proficiency bonus + spellcasting ability modifier`.
- Death saves: at 0 HP, roll a d20 at the start of your turn (usually no modifiers).
  - 10+ success, 9- failure
  - 3 successes stabilize; 3 failures die
  - natural 20 regains 1 HP
  - natural 1 counts as two failures

Criticals reminder (5e core):

- A natural 20 on an attack roll is a critical hit (roll extra damage dice).
- A natural 1 on an attack roll is an automatic miss.

For ability checks and saving throws, special natural 20/1 outcomes depend on your table rules.

## Advantage & Disadvantage

Advantage/disadvantage is a d20 rule: roll two d20s and keep the higher (adv) or lower (dis).

App note: the GUI dice roller does not currently have a built-in advantage toggle; roll twice manually or use the CLI mode flags.

## Initiative

Initiative is typically `d20 + Dexterity modifier` (plus any other initiative bonuses).

In this app: roll a d20 (or use `d20 --checkon dex`) and then add any extra initiative bonuses manually (see Character Sheet → Combat → Initiative).

## CLI Mode (Optional)

If you run the app in CLI mode, you can use dedicated commands for common rolls (and advantage/disadvantage flags):

```text
dndgamerolls --cli skill stealth
dndgamerolls --cli save dex
dndgamerolls --cli --dice 2d6 --modifier 3
```

The CLI is more feature-complete than the GUI command parser.

---

### Icons in Markdown (optional)

You can embed Material icons using this syntax:

- `:icon(zoom_in):`
- `:icon(zoom_out):`
- `:icon(swap_horiz):`

This uses the same Material icon font as the rest of the UI.
