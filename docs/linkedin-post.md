# LinkedIn Post - DnD Game Rolls Release

## Post Text

---

🎲 **Excited to announce my new open-source project: DnD Game Rolls!**

I built a D&D 5e dice roller in Rust that includes:

✨ **CLI Dice Roller** - Quick command-line rolls with your character stats
🎮 **3D Dice Simulator** - Visual dice rolling with real physics (Bevy game engine + Rapier3D)

**Key Features:**
• All D&D dice types (d4, d6, d8, d10, d12, d20)
• Advantage/Disadvantage support
• Skill checks, saving throws, attack rolls
• Character stats from JSON file
• Crystal-themed translucent 3D dice
• Command history for quick rerolls

**Tech Stack:** Rust 🦀 | Bevy | Rapier3D Physics | Clap CLI

📦 Install via cargo:
```
cargo install dndgamerolls
```

🔗 GitHub: https://github.com/edgarhsanchez/dndgamerolls
📦 Crates.io: https://crates.io/crates/dndgamerolls

Would love to hear feedback from fellow D&D players and Rustaceans! 🐉

#Rust #GameDev #DnD #OpenSource #Bevy #Programming #DungeonsAndDragons #CLI #3DGraphics

---

## Suggested Images

Attach these screenshots from the repository:
1. `screenshots/dice3d/dice3d-results.png` - Main showcase image
2. `screenshots/dice3d/dice3d-rolling.png` - Action shot of dice bouncing

Optional (newer UI screenshots):
- `screenshots/diceroller_tab.png` - Main dice roller tab
- `screenshots/dice_roller_cup.png` - Cup container mode
- `screenshots/character_setup_character.png` - Character sheet / setup

## Alternative Shorter Version

---

🎲 Just released DnD Game Rolls - a Rust-based D&D dice roller!

Features a 3D physics-based dice simulator built with Bevy game engine, plus a fast CLI for quick rolls.

📦 `cargo install dndgamerolls`
🔗 https://github.com/edgarhsanchez/dndgamerolls

#Rust #DnD #GameDev #OpenSource

---

## Twitter Automation

Twitter posts are automated via GitHub Actions when a new release is published to crates.io.

### Required GitHub Secrets

Add these secrets to your repository (Settings → Secrets → Actions):

| Secret | Description |
|--------|-------------|
| `TWITTER_API_KEY` | Twitter API Key (Consumer Key) |
| `TWITTER_API_SECRET` | Twitter API Secret (Consumer Secret) |
| `TWITTER_ACCESS_TOKEN` | OAuth Access Token |
| `TWITTER_ACCESS_TOKEN_SECRET` | OAuth Access Token Secret |
| `TWITTER_BEARER_TOKEN` | Bearer Token (optional, for v2 API) |

Get these from: https://developer.twitter.com/en/portal/dashboard
