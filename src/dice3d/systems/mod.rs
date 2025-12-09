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

mod avatar_loader;
mod camera;
pub mod character_screen;
mod contributors_screen;
mod dice;
mod input;
pub mod rendering;
mod settings;
mod setup;

// Re-export all public systems
pub use avatar_loader::{
    process_avatar_loads, request_avatars, update_avatar_images, AvatarImage, AvatarLoader,
};
pub use camera::{handle_zoom_slider, rotate_camera};
pub use character_screen::{
    handle_character_list_clicks, handle_delete_click, handle_expertise_toggle,
    handle_group_add_click, handle_group_edit_toggle, handle_label_click,
    handle_new_character_click, handle_new_entry_cancel, handle_new_entry_confirm,
    handle_new_entry_input, handle_roll_all_stats_click, handle_roll_attribute_click,
    handle_save_click, handle_scroll_input, handle_stat_field_click, handle_tab_clicks,
    handle_text_input, init_character_manager, rebuild_character_list_on_change,
    rebuild_character_panel_on_change, refresh_character_display, setup_character_screen,
    setup_dnd_info_screen, setup_tab_bar, update_character_list_modified_indicator,
    update_editing_display, update_new_entry_input_display, update_save_button_appearance,
    update_tab_styles, update_tab_visibility,
};
pub use contributors_screen::{init_contributors, setup_contributors_screen};
pub use dice::{check_dice_settled, update_results_display};
pub use input::{handle_command_input, handle_input, handle_quick_roll_clicks};
pub use settings::{
    apply_initial_settings, handle_color_slider_drag, handle_color_text_input,
    handle_settings_button_click, handle_settings_button_hover, handle_settings_cancel_click,
    handle_settings_ok_click, handle_slider_drag_continuous, handle_slider_release,
    manage_settings_modal, spawn_settings_button, update_color_ui,
};
pub use setup::{calculate_dice_position, rebuild_quick_roll_panel, setup, spawn_die};
