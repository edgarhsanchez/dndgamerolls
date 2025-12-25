//! Character screen UI module
//!
//! This module contains systems for displaying and editing character sheets,
//! using Material Design 3 components and tabbed navigation.
//!
//! ## Structure
//! - `mod.rs` - Main module with shared constants, types, and re-exports
//! - `tab_bar.rs` - App-level tab bar (Dice Roller, Character, DnD Info, Contributors)
//! - `character_list.rs` - Character list panel (left side)
//! - `tabs/` - Character sheet content tabs
//!   - `mod.rs` - MaterialTabs container and content switching
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
mod conversion_dialog;
mod handlers;
mod tab_bar;
pub mod tabs;

// Re-export submodule contents
pub use character_list::*;
pub use components::*;
pub use conversion_dialog::*;
pub use handlers::*;
pub use tab_bar::*;
pub use tabs::*;

// ============================================================================
// Material Design 3 Theme Colors
// ============================================================================

/// Primary color for accents and active states
pub const MD3_PRIMARY: Color = Color::srgb(0.4, 0.6, 0.9);
/// On-primary color for text/icons on primary backgrounds
pub const MD3_ON_PRIMARY: Color = Color::WHITE;
/// Surface color for cards and panels
pub const MD3_SURFACE: Color = Color::srgb(0.11, 0.11, 0.14);
/// Surface container for nested surfaces
pub const MD3_SURFACE_CONTAINER: Color = Color::srgb(0.14, 0.14, 0.18);
/// Surface container high for elevated surfaces
pub const MD3_SURFACE_CONTAINER_HIGH: Color = Color::srgb(0.17, 0.17, 0.22);
/// On-surface for primary text
pub const MD3_ON_SURFACE: Color = Color::WHITE;
/// On-surface-variant for secondary text
pub const MD3_ON_SURFACE_VARIANT: Color = Color::srgb(0.7, 0.7, 0.75);
/// Outline color for borders
pub const MD3_OUTLINE: Color = Color::srgb(0.3, 0.3, 0.35);
/// Outline variant for subtle borders
pub const MD3_OUTLINE_VARIANT: Color = Color::srgb(0.2, 0.2, 0.25);
/// Secondary color
pub const MD3_SECONDARY: Color = Color::srgb(0.6, 0.7, 0.85);
/// Tertiary color for accents
pub const MD3_TERTIARY: Color = Color::srgb(0.5, 0.8, 0.7);
/// Error color
pub const MD3_ERROR: Color = Color::srgb(0.9, 0.3, 0.3);
/// Success/proficient color
pub const MD3_SUCCESS: Color = Color::srgb(0.3, 0.8, 0.4);
/// Warning/modified color
pub const MD3_WARNING: Color = Color::srgb(0.9, 0.7, 0.3);

// Legacy color aliases for gradual migration
pub const TAB_ACTIVE_BG: Color = MD3_PRIMARY;
pub const TAB_INACTIVE_BG: Color = MD3_SURFACE_CONTAINER;
pub const TAB_HOVER_BG: Color = Color::srgb(0.25, 0.35, 0.55);
pub const PANEL_BG: Color = MD3_SURFACE;
pub const GROUP_BG: Color = MD3_SURFACE_CONTAINER;
pub const FIELD_BG: Color = Color::srgb(0.08, 0.08, 0.12);
pub const FIELD_MODIFIED_BG: Color = Color::srgb(0.2, 0.15, 0.08);
pub const BUTTON_BG: Color = Color::srgb(0.2, 0.5, 0.3);
pub const TEXT_PRIMARY: Color = MD3_ON_SURFACE;
pub const TEXT_SECONDARY: Color = MD3_ON_SURFACE_VARIANT;
pub const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.5);
pub const PROFICIENT_COLOR: Color = MD3_SUCCESS;

// Icon constants (Unicode fallbacks)
pub const ICON_EDIT: &str = "✎";
pub const ICON_CHECK: &str = "✓";
pub const ICON_CANCEL: &str = "✕";
pub const ICON_DELETE: &str = "✕";
pub const ICON_ADD: &str = "+";

// Icon button colors
pub const ICON_BUTTON_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
pub const ICON_BUTTON_ACTIVE: Color = Color::srgb(0.3, 0.5, 0.4);

// ============================================================================
// Character Sheet Tab Types
// ============================================================================

/// Tabs within the character sheet panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CharacterSheetTab {
    #[default]
    BasicInfo,
    Attributes,
    Combat,
    SavingThrows,
    Skills,
}

impl CharacterSheetTab {
    /// Get the display label for this tab
    pub fn label(&self) -> &'static str {
        match self {
            Self::BasicInfo => "Basic Info",
            Self::Attributes => "Attributes",
            Self::Combat => "Combat",
            Self::SavingThrows => "Saves",
            Self::Skills => "Skills",
        }
    }

    /// Get all tabs in order
    pub fn all() -> &'static [Self] {
        &[
            Self::BasicInfo,
            Self::Attributes,
            Self::Combat,
            Self::SavingThrows,
            Self::Skills,
        ]
    }

    /// Get the icon name for this tab (Material Icons)
    pub fn icon(&self) -> &'static str {
        match self {
            Self::BasicInfo => "person",
            Self::Attributes => "fitness_center",
            Self::Combat => "shield",
            Self::SavingThrows => "security",
            Self::Skills => "psychology",
        }
    }
}

/// Marker for the character sheet tab bar container
#[derive(Component)]
pub struct CharacterSheetTabBar;

/// Component to mark a character sheet tab button
#[derive(Component)]
pub struct CharacterSheetTabButton {
    pub tab: CharacterSheetTab,
}

/// Component to mark character sheet tab content
#[derive(Component)]
pub struct CharacterSheetTabContent {
    pub tab: CharacterSheetTab,
}

/// Marker for character sheet tab text (for styling updates)
#[derive(Component)]
pub struct CharacterSheetTabText;

/// Resource to track currently selected character sheet tab
#[derive(Resource, Default)]
pub struct SelectedCharacterSheetTab {
    pub current: CharacterSheetTab,
}
