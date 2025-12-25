//! Dice special-effects

use bevy::prelude::*;
#[derive(Resource, Default, Clone, Copy)]
pub struct DiceFxRollingTracker {
    pub was_rolling: bool,
}

pub struct DiceFxPlugin;

impl Plugin for DiceFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<crate::dice3d::types::DiceRollCompletedEvent>()
            .init_resource::<DiceFxRollingTracker>()
            .add_systems(Startup, crate::dice3d::init_dice_hanabi_fx_assets)
            // Clear effects on roll start, apply on settle (effects persist until next roll).
            .add_systems(
                Update,
                crate::dice3d::clear_dice_fx_on_roll_start
                    .after(crate::dice3d::handle_input)
                    .after(crate::dice3d::handle_command_input)
                    .after(crate::dice3d::handle_quick_roll_clicks),
            )
            .add_systems(
                Update,
                crate::dice3d::apply_dice_fx_from_roll_complete.after(crate::dice3d::check_dice_settled),
            )
            .add_systems(Update, crate::dice3d::update_electricity_wander);
    }
}
