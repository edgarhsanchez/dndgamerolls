//! Types for the Dice Designer tab
//!
//! This module contains all types, components, and resources for the dice face
//! customization interface. Users can select dice types and assign custom textures
//! (color, depth, normal) to each face.

use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use super::dice::DiceType;

// ============================================================================
// Dice Designer State
// ============================================================================

/// Resource tracking the current state of the dice designer
#[derive(Resource, Debug, Clone)]
pub struct DiceDesignerState {
    /// Currently selected dice type in the list
    pub selected_dice: DiceType,
    /// Currently selected face value for the selected dice
    /// None means editing all faces at once (atlas mode)
    pub selected_face: Option<u32>,
    /// Per-dice-type face texture configurations
    pub dice_configs: HashMap<DiceType, DiceTextureConfig>,
    /// Whether the 3D preview should auto-rotate
    pub auto_rotate: bool,
    /// Current rotation of the preview dice (euler angles in radians)
    pub preview_rotation: Vec3,
    /// Whether we're currently dragging to rotate
    pub is_dragging_rotation: bool,
    /// Which axis is being dragged (if any)
    pub active_drag_axis: Option<RotationAxis>,
}

impl Default for DiceDesignerState {
    fn default() -> Self {
        let mut dice_configs = HashMap::new();
        for die_type in DiceType::all() {
            dice_configs.insert(die_type, DiceTextureConfig::new(die_type));
        }

        Self {
            // Default to the first item in the list for a predictable UX.
            selected_dice: DiceType::D4,
            selected_face: None,
            dice_configs,
            auto_rotate: true,
            preview_rotation: Vec3::ZERO,
            is_dragging_rotation: false,
            active_drag_axis: None,
        }
    }
}

/// Configuration for a single dice type's face textures
#[derive(Debug, Clone)]
pub struct DiceTextureConfig {
    pub die_type: DiceType,
    /// Per-face texture paths. Key is face value (1-20), value is the texture set.
    pub face_textures: HashMap<u32, FaceTextureSet>,
    /// Default/fallback texture set used when a face doesn't have custom textures
    pub default_textures: FaceTextureSet,
}

impl DiceTextureConfig {
    pub fn new(die_type: DiceType) -> Self {
        let face_count = die_type.face_count();
        let mut face_textures = HashMap::new();

        // Initialize empty texture sets for each face
        for face_value in 1..=face_count {
            face_textures.insert(face_value, FaceTextureSet::default());
        }

        Self {
            die_type,
            face_textures,
            default_textures: FaceTextureSet::default(),
        }
    }
}

/// Texture paths for a single die face (or default)
#[derive(Debug, Clone, Default)]
pub struct FaceTextureSet {
    /// Path to the color/albedo texture (PNG)
    pub color_path: Option<PathBuf>,
    /// Path to the depth/height map texture (PNG)
    pub depth_path: Option<PathBuf>,
    /// Path to the normal map texture (PNG)
    pub normal_path: Option<PathBuf>,
}

// ============================================================================
// UI Component Markers
// ============================================================================

/// Marker for the dice designer screen root
#[derive(Component)]
pub struct DiceDesignerScreenRoot;

/// Marker for the left panel containing the dice type list
#[derive(Component)]
pub struct DiceDesignerListPanel;

/// Marker for a dice type list item button
#[derive(Component)]
pub struct DiceDesignerListItem {
    pub die_type: DiceType,
}

/// Marker for the right panel containing settings
#[derive(Component)]
pub struct DiceDesignerSettingsPanel;

/// Marker for the 3D dice preview container
#[derive(Component)]
pub struct DicePreviewContainer;

/// Marker for the 3D dice preview camera
#[derive(Component)]
pub struct DicePreviewCamera;

/// Marker for the dice entity in the preview
#[derive(Component)]
pub struct DicePreviewDie;

/// Marker for a 3D rotation ring around the preview die
#[derive(Component)]
pub struct DicePreviewRotationRing {
    pub axis: RotationAxis,
}

/// Marker for the rotation gizmo container
#[derive(Component)]
pub struct DicePreviewGizmo;

/// Marker for individual gizmo axis handles
#[derive(Component)]
pub struct DicePreviewGizmoAxis {
    pub axis: RotationAxis,
}

/// Which rotation axis a gizmo handle controls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationAxis {
    X,
    Y,
    Z,
}

/// Marker for the texture settings section
#[derive(Component)]
pub struct DiceDesignerTextureSection;

/// Marker for a texture file input row
#[derive(Component)]
pub struct TextureFileInput {
    pub texture_type: TextureType,
    pub face_value: Option<u32>, // None = default/all faces
}

/// Type of texture being configured
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureType {
    Color,
    Depth,
    Normal,
}

/// Marker for texture preview image
#[derive(Component)]
pub struct TexturePreviewImage {
    pub texture_type: TextureType,
    pub face_value: Option<u32>,
}

/// Marker for the face selector dropdown/list
#[derive(Component)]
pub struct FaceSelectorDropdown;

/// Marker for a face selector item
#[derive(Component)]
pub struct FaceSelectorItem {
    pub face_value: Option<u32>, // None = "All faces" option
}

// ============================================================================
// Events
// ============================================================================

/// Event fired when a dice type is selected in the list
#[derive(Event, Debug, Clone)]
pub struct DiceTypeSelectedEvent {
    pub die_type: DiceType,
}

/// Event fired when a face is selected for editing
#[derive(Event, Debug, Clone)]
pub struct FaceSelectedEvent {
    pub face_value: Option<u32>,
}

/// Event fired when a texture file is selected
#[derive(Event, Debug, Clone)]
pub struct TextureFileSelectedEvent {
    pub texture_type: TextureType,
    pub face_value: Option<u32>,
    pub path: PathBuf,
}

/// Event fired when the preview rotation changes (from gizmo interaction)
#[derive(Event, Debug, Clone)]
pub struct PreviewRotationChangedEvent {
    pub rotation: Vec3,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl DiceType {
    /// Get all dice types in display order
    pub fn all() -> Vec<DiceType> {
        vec![
            DiceType::D4,
            DiceType::D6,
            DiceType::D8,
            DiceType::D10,
            DiceType::D12,
            DiceType::D20,
        ]
    }

    /// Get the number of faces for this dice type
    pub fn face_count(&self) -> u32 {
        match self {
            DiceType::D4 => 4,
            DiceType::D6 => 6,
            DiceType::D8 => 8,
            DiceType::D10 => 10,
            DiceType::D12 => 12,
            DiceType::D20 => 20,
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            DiceType::D4 => "D4 (Tetrahedron)",
            DiceType::D6 => "D6 (Cube)",
            DiceType::D8 => "D8 (Octahedron)",
            DiceType::D10 => "D10 (Trapezohedron)",
            DiceType::D12 => "D12 (Dodecahedron)",
            DiceType::D20 => "D20 (Icosahedron)",
        }
    }
}

impl TextureType {
    pub fn display_name(&self) -> &'static str {
        match self {
            TextureType::Color => "Color Map",
            TextureType::Depth => "Depth Map",
            TextureType::Normal => "Normal Map",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TextureType::Color => "The visible color/albedo texture (PNG)",
            TextureType::Depth => "Height map for parallax effect - white=surface, black=deep (PNG)",
            TextureType::Normal => "Normal map for surface detail lighting (PNG)",
        }
    }
}
