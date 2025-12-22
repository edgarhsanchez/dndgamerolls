//! App-level tab bar (Dice Roller, Character, DnD Info, Contributors)
//!
//! This module handles the main navigation tabs at the top of the application
//! using bevy_material_ui's MaterialTabs and MaterialTab components.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use crate::dice3d::types::*;
use super::*;

// ============================================================================
// Tab Bar Setup
// ============================================================================

/// Get the AppTab for a given index
fn app_tab_from_index(index: usize) -> AppTab {
    match index {
        0 => AppTab::DiceRoller,
        1 => AppTab::CharacterSheet,
        2 => AppTab::DndInfo,
        3 => AppTab::Contributors,
        _ => AppTab::DiceRoller,
    }
}

/// Get the index for a given AppTab
fn index_from_app_tab(tab: AppTab) -> usize {
    match tab {
        AppTab::DiceRoller => 0,
        AppTab::CharacterSheet => 1,
        AppTab::DndInfo => 2,
        AppTab::Contributors => 3,
    }
}

/// Setup the tab bar UI (called once on startup)
pub fn setup_tab_bar(
    mut commands: Commands,
    icon_assets: Res<IconAssets>,
    theme: Option<Res<MaterialTheme>>,
) {
    let theme = theme.map(|t| t.clone()).unwrap_or_default();

    commands
        .spawn((
            MaterialTabs::new()
                .with_variant(TabVariant::Primary)
                .selected(0),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(TAB_HEIGHT_SECONDARY),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
            BackgroundColor(MD3_SURFACE),
            ZIndex(100),
            TabBar,
            AppTabBar, // Marker to identify this as the app-level tab bar
        ))
        .with_children(|parent| {
            // Dice Roller Tab
            spawn_app_tab(parent, &icon_assets, &theme, "Dice Roller", IconType::Dice, 0, true);
            // Character Sheet Tab
            spawn_app_tab(parent, &icon_assets, &theme, "Character", IconType::Character, 1, false);
            // DnD Info Tab
            spawn_app_tab(parent, &icon_assets, &theme, "DnD Info", IconType::Info, 2, false);
            // Contributors Tab
            spawn_app_tab(parent, &icon_assets, &theme, "Contributors", IconType::Character, 3, false);
        });
}

fn spawn_app_tab(
    parent: &mut ChildSpawnerCommands,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
    label: &str,
    icon_type: IconType,
    index: usize,
    is_selected: bool,
) {
    let icon_handle = icon_assets.icons.get(&icon_type).cloned();
    let tab = app_tab_from_index(index);

    parent
        .spawn((
            MaterialTab::new(index, label).selected(is_selected),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            TabButton { tab },
            AppTabButton { index },
        ))
        .with_children(|button| {
            // Icon
            if let Some(handle) = icon_handle {
                button.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                ));
            }
            // Text
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if is_selected {
                    theme.primary
                } else {
                    theme.on_surface_variant
                }),
                AppTabText,
            ));
        });
}

// ============================================================================
// Tab Change Event Handling
// ============================================================================

/// Handle tab change events from bevy_material_ui
pub fn handle_tab_clicks(
    mut tab_events: MessageReader<TabChangeEvent>,
    mut ui_state: ResMut<UiState>,
    app_tab_query: Query<&AppTabButton>,
) {
    for event in tab_events.read() {
        // Check if this is an app-level tab by looking up the entity
        if let Ok(app_tab) = app_tab_query.get(event.tab_entity) {
            ui_state.active_tab = app_tab_from_index(app_tab.index);
        }
    }
}

/// Update tab button styles based on active tab (syncs with bevy_material_ui)
pub fn update_tab_styles(
    ui_state: Res<UiState>,
    mut tabs_query: Query<&mut MaterialTabs, With<AppTabBar>>,
    mut tab_query: Query<(&AppTabButton, &mut MaterialTab)>,
    mut text_query: Query<&mut TextColor, With<AppTabText>>,
    theme: Option<Res<MaterialTheme>>,
) {
    if !ui_state.is_changed() {
        return;
    }

    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let selected_index = index_from_app_tab(ui_state.active_tab);

    // Update the MaterialTabs selected index
    for mut tabs in tabs_query.iter_mut() {
        tabs.selected = selected_index;
    }

    // Update individual tab selected states
    for (app_tab, mut material_tab) in tab_query.iter_mut() {
        material_tab.selected = app_tab.index == selected_index;
    }

    // Update text colors
    for mut text_color in text_query.iter_mut() {
        // Note: This is a simplified update; ideally each text would know its parent tab
        text_color.0 = theme.on_surface_variant;
    }
}

/// Update visibility of screens based on active tab
#[allow(clippy::type_complexity)]
pub fn update_tab_visibility(
    ui_state: Res<UiState>,
    mut dice_roller_query: Query<
        &mut Visibility,
        (
            With<DiceRollerRoot>,
            Without<CharacterScreenRoot>,
            Without<DndInfoScreenRoot>,
            Without<ContributorsScreenRoot>,
        ),
    >,
    mut character_screen: Query<
        &mut Visibility,
        (
            With<CharacterScreenRoot>,
            Without<DiceRollerRoot>,
            Without<DndInfoScreenRoot>,
            Without<ContributorsScreenRoot>,
        ),
    >,
    mut dnd_info_screen: Query<
        &mut Visibility,
        (
            With<DndInfoScreenRoot>,
            Without<DiceRollerRoot>,
            Without<CharacterScreenRoot>,
            Without<ContributorsScreenRoot>,
        ),
    >,
    mut contributors_screen: Query<
        &mut Visibility,
        (
            With<ContributorsScreenRoot>,
            Without<DiceRollerRoot>,
            Without<CharacterScreenRoot>,
            Without<DndInfoScreenRoot>,
        ),
    >,
) {
    if !ui_state.is_changed() {
        return;
    }

    // Dice roller visibility
    for mut visibility in dice_roller_query.iter_mut() {
        *visibility = if ui_state.active_tab == AppTab::DiceRoller {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Character screen visibility
    for mut visibility in character_screen.iter_mut() {
        *visibility = if ui_state.active_tab == AppTab::CharacterSheet {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // DnD Info screen visibility
    for mut visibility in dnd_info_screen.iter_mut() {
        *visibility = if ui_state.active_tab == AppTab::DndInfo {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Contributors screen visibility
    for mut visibility in contributors_screen.iter_mut() {
        *visibility = if ui_state.active_tab == AppTab::Contributors {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
