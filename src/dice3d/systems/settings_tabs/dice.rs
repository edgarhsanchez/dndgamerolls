use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy_material_ui::prelude::*;

pub fn build_dice_tab(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    select_options: Vec<SelectOption>,
    selected_index: usize,
) {
    parent.spawn((
        Text::new("Quick Rolls"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn((
        Text::new("Default die for Quick Rolls"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme.on_surface_variant),
    ));

    parent.spawn(Node::default()).with_children(|slot| {
        let builder = SelectBuilder::new(select_options)
            .outlined()
            .label("Quick roll die")
            .selected(selected_index)
            .width(Val::Px(210.0));
        slot.spawn_select_with(theme, builder);
    });
}
