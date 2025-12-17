use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::prelude::*;

pub mod colors;
pub mod dice;
pub mod layout;
pub mod shake_curve;

/// Creates a per-tab scrollable content panel.
///
/// Runs `build` inside the tab's scrollable content root.
pub fn spawn_scrollable_tab_content(
    parent: &mut ChildSpawnerCommands,
    tabs_entity: Entity,
    tab_index: usize,
    visible: bool,
    build: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    // Panel for TabContent (fills available space).
    let mut panel = parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            min_width: Val::Px(0.0),
            min_height: Val::Px(0.0),
            display: if visible {
                Display::Flex
            } else {
                Display::None
            },
            flex_direction: FlexDirection::Column,
            ..default()
        },
        TabContent::new(tab_index, tabs_entity),
    ));

    let panel_entity = panel.id();

    panel.with_children(|tab_root| {
        // Scroll container provides wheel scrolling + scrollbars.
        tab_root
            .spawn((
                ScrollContainer::vertical(),
                ScrollPosition::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(|scroll| {
                // Actual content root (child of ScrollContent wrapper created by plugin).
                scroll
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        min_width: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        // Leave a little gutter so the vertical scrollbar doesn't overlap content.
                        padding: UiRect {
                            left: Val::Px(2.0),
                            right: Val::Px(18.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                        },
                        ..default()
                    })
                    .with_children(build);
            });
    });

    panel_entity
}
