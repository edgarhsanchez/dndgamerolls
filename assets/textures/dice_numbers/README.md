# Dice number atlas (parallax)

This folder contains the generated number atlas textures used for parallax-mapped dice face labels.

Files:
- `atlas_color.png`: RGBA color + alpha mask for digits
- `atlas_depth.png`: grayscale height map (black = highest, white = lowest)
- `atlas_normal.png`: normal map generated from the depth map (linear)

Regenerate:
- Run `cargo run --bin gen_dice_parallax_atlas` from the repo root.
