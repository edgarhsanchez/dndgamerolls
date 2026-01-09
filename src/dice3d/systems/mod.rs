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
//! - `character_screen`: Character sheet UI and tab navigation
//! - `settings`: Settings UI and persistence
//! - `contributors_screen`: GitHub contributors display
//! - `avatar_loader`: Async loading of profile images from URLs
//! - `dice_designer_screen`: Dice customization interface
//! - `dice_preview`: 3D dice preview component for designer

mod avatar_loader;
mod box_highlight;
mod camera;
pub mod character_screen;
mod collision_sfx;
pub mod color_picker;
mod container_centering;
mod contributors_screen;
mod dice;
mod dice_designer_screen;
mod dice_face_labels;
pub mod dice_box_controls;
pub mod dice_box_lid_animations;
pub mod dice_fx;
mod dice_preview;
mod gltf_colliders;
mod gltf_spawn_points;
mod input;
pub mod rendering;
mod select_theme_preview;
mod settings;
pub mod settings_tabs;
mod setup;
mod slider_group;
mod theme_refresh;

// Re-export all public systems
pub use avatar_loader::*;
pub use box_highlight::*;
pub use camera::*;
pub use character_screen::*;
pub use collision_sfx::*;
pub use container_centering::*;
pub use contributors_screen::*;
pub use dice::*;
pub use dice_designer_screen::*;
pub use dice_face_labels::*;
pub use dice_box_controls::*;
pub use dice_box_lid_animations::*;
pub use dice_fx::*;
pub use dice_preview::*;
pub use gltf_colliders::*;
pub use gltf_spawn_points::*;
pub use input::*;
pub use select_theme_preview::*;
pub use settings::*;
pub use setup::*;
pub use slider_group::*;
pub use theme_refresh::*;
