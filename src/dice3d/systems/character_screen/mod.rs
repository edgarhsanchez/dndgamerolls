//! Character screen UI module
//!
//! This module contains systems for displaying and editing character sheets,
//! using Material Design 3 components.
//!
//! ## Structure
//! - `mod.rs` - Main module with shared constants, types, and re-exports
//! - `tab_bar.rs` - App-level tab bar (Dice Roller, Character, DnD Info, Contributors)
//! - `character_list.rs` - Character list panel (left side)
//! - `tabs/` - Character sheet content (wrapping group card layout)
//!   - `mod.rs` - Wrap layout container
//!   - `basic_info.rs` - Basic character info (name, class, race, etc.)
//!   - `attributes.rs` - Ability scores and modifiers
//!   - `combat.rs` - Combat stats (HP, AC, initiative, etc.)
//!   - `saving_throws.rs` - Saving throw proficiencies
//!   - `skills.rs` - Skills and proficiencies
//! - `components.rs` - Shared UI components (stat fields, group headers, etc.)
//! - `handlers.rs` - Input and event handlers

use bevy::prelude::*;

// Submodules
mod character_list;
mod components;
mod handlers;
mod tab_bar;
mod tabs;

// ============================================================================
// Material Design 3 Color Tokens (dark theme defaults)
//
// These are provided as simple constants for legacy UI code that still expects
// MD3_* tokens. Newer UI should prefer `MaterialTheme` directly.
// ============================================================================

pub const MD3_PRIMARY: Color = Color::srgb(0.82, 0.71, 1.0); // theme.primary
pub const MD3_ON_PRIMARY: Color = Color::srgb(0.25, 0.09, 0.46); // theme.on_primary

pub const MD3_SECONDARY: Color = Color::srgb(0.80, 0.78, 0.90); // theme.secondary
pub const MD3_TERTIARY: Color = Color::srgb(0.94, 0.73, 0.78); // theme.tertiary

pub const MD3_SURFACE: Color = Color::srgb(0.08, 0.07, 0.09); // theme.surface
pub const MD3_SURFACE_CONTAINER: Color = Color::srgb(0.13, 0.12, 0.14); // theme.surface_container
pub const MD3_SURFACE_CONTAINER_HIGH: Color = Color::srgb(0.17, 0.16, 0.18); // theme.surface_container_high

pub const MD3_ON_SURFACE: Color = Color::srgb(0.90, 0.87, 0.92); // theme.on_surface
pub const MD3_ON_SURFACE_VARIANT: Color = Color::srgb(0.78, 0.74, 0.82); // theme.on_surface_variant

pub const MD3_OUTLINE: Color = Color::srgb(0.58, 0.55, 0.62); // theme.outline
pub const MD3_OUTLINE_VARIANT: Color = Color::srgb(0.29, 0.27, 0.32); // theme.outline_variant

pub const MD3_ERROR: Color = Color::srgb(1.0, 0.71, 0.68); // theme.error
pub const MD3_SUCCESS: Color = Color::srgb(0.30, 0.70, 0.30);

// ============================================================================
// Public re-exports (used by `dice3d::systems`)
// ============================================================================

pub use character_list::{
	spawn_character_list_panel,
	handle_character_list_clicks,
	handle_new_character_click,
	handle_roll_all_stats_click,
	update_character_list_modified_indicator,
};
pub use components::*;
pub use handlers::{
	handle_stat_field_click, handle_label_click, handle_text_input, update_editing_display,
	handle_group_edit_toggle, handle_group_add_click, handle_new_entry_confirm,
	handle_new_entry_cancel, handle_new_entry_input, update_new_entry_input_display,
	handle_delete_click, handle_expertise_toggle, handle_roll_attribute_click, handle_roll_skill_click,
	handle_save_click, update_save_button_appearance,
	handle_scroll_input,
	ensure_buttons_have_interaction,
	rebuild_character_list_on_change, rebuild_character_panel_on_change, refresh_character_display,
	setup_dnd_info_screen,
	init_character_manager,
	record_character_screen_roll_on_settle, sync_character_screen_roll_result_texts,
	handle_character_sheet_settings_button_click,
	manage_character_sheet_settings_modal,
	handle_character_sheet_die_type_select_change,
	handle_character_sheet_settings_save_click,
	handle_character_sheet_settings_cancel_click,
};
pub use tab_bar::{setup_tab_bar, handle_tab_clicks, update_tab_styles, update_tab_visibility};
pub use tabs::setup_character_screen;



