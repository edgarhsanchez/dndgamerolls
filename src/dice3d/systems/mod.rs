//! Systems module for dice3d
//!
//! This module contains all the Bevy systems for the 3D dice roller,
//! organized into submodules by functionality:
//!
//! - `setup`: Scene initialization (camera, lights, dice box, dice, UI)
//! - `camera`: Camera rotation and zoom controls
//! - `dice`: Dice settlement detection and result determination
//! - `input`: Keyboard input handling and command parsing
//! - `rendering`: Number mesh generation for dice labels

mod camera;
mod dice;
mod input;
pub mod rendering;
mod setup;

// Re-export all public systems
pub use camera::{handle_zoom_slider, rotate_camera};
pub use dice::{check_dice_settled, update_results_display};
pub use input::{handle_command_input, handle_input};
pub use setup::{calculate_dice_position, setup, spawn_die};
