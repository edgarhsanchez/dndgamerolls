@echo off
echo Starting D&D 3D Dice Roller...
cd /d "%~dp0"
cargo run --bin dice3d --release
