//! Dice box hover highlight material and components.

use bevy::color::LinearRgba;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

pub const DICE_BOX_HIGHLIGHT_SHADER: &str = "shaders/dice_box_highlight.wgsl";

/// Marker for the dice box floor mesh entity (the clickable/hoverable area).
#[derive(Component)]
pub struct DiceBoxFloor;

/// The extension applied to `StandardMaterial` that adds a hover highlight.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct DiceBoxHighlightExtension {
    /// We start at binding slot 100 to avoid conflicts with `StandardMaterial`.
    #[uniform(100)]
    pub params: DiceBoxHighlightParams,
}

impl MaterialExtension for DiceBoxHighlightExtension {
    fn fragment_shader() -> ShaderRef {
        DICE_BOX_HIGHLIGHT_SHADER.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        DICE_BOX_HIGHLIGHT_SHADER.into()
    }
}

/// GPU parameters for the hover highlight.
#[derive(Clone, Copy, Debug, Default, ShaderType, Reflect)]
pub struct DiceBoxHighlightParams {
    /// Highlight color in linear space.
    pub highlight_color: Vec4,
    /// 1.0 when hovered, 0.0 otherwise.
    pub hovered: f32,
    /// Highlight intensity.
    pub strength: f32,
    /// Padding for alignment.
    pub _pad: Vec2,
}

impl DiceBoxHighlightParams {
    pub fn set_highlight_color(&mut self, color: Color) {
        self.highlight_color = LinearRgba::from(color).to_vec4();
    }
}

pub type DiceBoxHighlightMaterial = ExtendedMaterial<StandardMaterial, DiceBoxHighlightExtension>;
