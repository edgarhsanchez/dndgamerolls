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
mod box_highlight;
mod camera;
pub mod character_screen;
mod contributors_screen;
mod dice;
pub mod dice_box_controls;
mod gltf_colliders;
mod gltf_spawn_points;
mod input;
pub mod rendering;
mod settings;
mod settings_tabs;
mod select_theme_preview;
mod setup;
mod slider_group;
mod theme_refresh;

// Re-export all public systems
pub use avatar_loader::{
    process_avatar_loads, request_avatars, update_avatar_images, AvatarImage, AvatarLoader,
};
pub use box_highlight::update_dice_box_highlight;
pub use camera::{handle_zoom_slider_changes, rotate_camera};
pub use character_screen::{
    // UI fixups
    ensure_buttons_have_interaction,
    // Legacy SQLite -> SurrealDB conversion
    finalize_sqlite_conversion_if_done,
    handle_character_list_clicks,
    handle_character_sheet_die_type_select_change,
    // Character sheet dice settings modal
    handle_character_sheet_settings_button_click,
    handle_character_sheet_settings_cancel_click,
    handle_character_sheet_settings_save_click,
    handle_delete_click,
    handle_expertise_toggle,
    handle_group_add_click,
    handle_group_edit_toggle,
    handle_label_click,
    handle_new_character_click,
    handle_new_entry_cancel,
    handle_new_entry_confirm,
    handle_new_entry_input,
    handle_roll_all_stats_click,
    handle_roll_attribute_click,
    handle_roll_skill_click,
    // Save handling
    handle_save_click,
    // Scroll handling
    handle_scroll_input,
    handle_sqlite_conversion_no_click,
    handle_sqlite_conversion_ok_click,
    handle_sqlite_conversion_yes_click,
    // Editing handlers
    handle_stat_field_click,
    handle_tab_clicks,
    handle_text_input,
    // Init
    init_character_manager,
    manage_character_sheet_settings_modal,
    // Rebuild systems
    rebuild_character_list_on_change,
    rebuild_character_panel_on_change,
    // Character sheet dice -> dice roller bridge
    record_character_screen_roll_on_settle,
    refresh_character_display,
    run_sqlite_conversion_step,
    // Character screen setup
    setup_character_screen,
    // DnD info screen
    setup_dnd_info_screen,
    // Tab bar systems
    setup_tab_bar,
    // Character list systems
    spawn_character_list_panel,
    start_sqlite_conversion_if_needed,
    sync_character_screen_roll_result_texts,
    update_character_list_modified_indicator,
    update_editing_display,
    update_new_entry_input_display,
    update_save_button_appearance,
    update_sqlite_conversion_dialog_ui,
    update_tab_styles,
    update_tab_visibility,
    MD3_ERROR,
    MD3_ON_PRIMARY,
    MD3_ON_SURFACE,
    MD3_ON_SURFACE_VARIANT,
    MD3_OUTLINE,
    // Theme colors (for other modules that may need them)
    MD3_PRIMARY,
    MD3_SUCCESS,
    MD3_SURFACE,
    MD3_SURFACE_CONTAINER,
};
pub use contributors_screen::{init_contributors, setup_contributors_screen};
pub use dice::{check_dice_settled, update_results_display};
pub use dice_box_controls::{
    animate_container_shake, handle_dice_box_rotate_click, handle_dice_box_shake_box_click,
    handle_dice_box_toggle_container_click, handle_shake_slider_changes,
    sync_dice_container_mode_text, sync_dice_container_toggle_icon,
};
pub use gltf_colliders::{
    apply_crystal_material_to_container_models, spawn_colliders_from_gltf_guides,
};
pub use gltf_spawn_points::{
    apply_spawn_points_to_dice_when_ready, collect_dice_spawn_points_from_gltf, DiceSpawnPoints,
    DiceSpawnPointsApplied,
};
pub use input::{
    handle_command_history_item_clicks, handle_command_input, handle_input,
    handle_quick_roll_clicks,
};
pub use settings::{
    apply_initial_settings, apply_initial_shake_config, autosave_and_apply_shake_config,
    drag_shake_curve_bezier_handle, drag_shake_curve_point, handle_color_slider_changes,
    handle_color_text_input, handle_default_roll_uses_shake_switch_change,
    handle_quick_roll_die_type_select_change, handle_theme_seed_select_change,
    handle_settings_button_click,
    handle_settings_cancel_click, handle_settings_ok_click, handle_settings_reset_layout_click,
    handle_shake_curve_bezier_handle_press, handle_shake_curve_chip_clicks,
    handle_shake_curve_graph_click_to_add_point, handle_shake_curve_point_press,
    handle_shake_duration_text_input, manage_settings_modal, persist_settings_to_db,
    load_settings_state_from_db, spawn_settings_button, sync_shake_curve_chip_ui,
    sync_shake_curve_graph_ui, update_color_ui,
};
pub use setup::{
    calculate_dice_position, rebuild_command_history_panel, rebuild_quick_roll_panel, setup,
    spawn_die,
};
pub use slider_group::handle_slider_group_drag;
pub use select_theme_preview::tint_recent_theme_dropdown_items;
pub use theme_refresh::refresh_scrollbar_colors_on_theme_change;
