//! Character sheet tabs module
//!
//! This module contains the tabbed interface for the character sheet,
//! using Material Design 3 tabs for navigation between different sections.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use super::*;
use crate::dice3d::types::*;

// Submodules for each tab
mod attributes;
mod basic_info;
mod combat;
mod saving_throws;
mod skills;

// Re-export tab content builders
pub use attributes::spawn_attributes_content;
pub use basic_info::spawn_basic_info_content;
pub use combat::spawn_combat_content;
pub use saving_throws::spawn_saving_throws_content;
pub use skills::spawn_skills_content;

// ============================================================================
// Character Sheet Tab Container
// ============================================================================

/// Setup the character screen with Material Design tabs
pub fn setup_character_screen(
    mut commands: Commands,
    character_data: Res<CharacterData>,
    character_manager: Res<CharacterManager>,
    edit_state: Res<GroupEditState>,
    adding_state: Res<AddingEntryState>,
    icon_assets: Res<IconAssets>,
    icon_font: Res<bevy_material_ui::prelude::MaterialIconFont>,
    theme: Option<Res<MaterialTheme>>,
) {
    let theme = theme.map(|t| t.clone()).unwrap_or_default();

    // Root container (hidden by default, shown when tab is active)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(48.0), // Below app tab bar
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(MD3_SURFACE),
            Visibility::Hidden,
            CharacterScreenRoot,
        ))
        .with_children(|parent| {
            // Left panel - Character list
            spawn_character_list_panel(
                parent,
                &character_manager,
                &character_data,
                &icon_assets,
                icon_font.0.clone(),
                &theme,
            );

            // Right panel - Tabbed character sheet content
            spawn_tabbed_content_panel(
                parent,
                &character_data,
                &edit_state,
                &adding_state,
                &icon_assets,
                icon_font.0.clone(),
                &theme,
            );
        });

    // Initialize selected tab resource
    commands.insert_resource(SelectedCharacterSheetTab::default());
}

/// Spawn the right panel with Material Design tabs
pub(crate) fn spawn_tabbed_content_panel(
    parent: &mut ChildSpawnerCommands,
    character_data: &CharacterData,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            CharacterStatsPanel,
        ))
        .with_children(|container| {
            // Material Tabs bar
            spawn_character_sheet_tabs(container, theme);

            // Tab content area
            spawn_tab_content_area(
                container,
                character_data,
                edit_state,
                adding_state,
                icon_assets,
                icon_font,
                theme,
            );
        });
}

/// Spawn the Material Design tabs bar for character sheet sections
fn spawn_character_sheet_tabs(parent: &mut ChildSpawnerCommands, theme: &MaterialTheme) {
    parent
        .spawn((
            MaterialTabs::new()
                .with_variant(TabVariant::Secondary)
                .selected(0),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(TAB_HEIGHT_SECONDARY),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(MD3_SURFACE_CONTAINER),
            BorderColor::from(MD3_OUTLINE_VARIANT),
            CharacterSheetTabBar, // Marker component
        ))
        .with_children(|tabs| {
            for (index, tab) in CharacterSheetTab::all().iter().enumerate() {
                spawn_sheet_tab_button(tabs, *tab, index, index == 0, theme);
            }
        });
}

/// Spawn a single tab button using bevy_material_ui MaterialTab
fn spawn_sheet_tab_button(
    parent: &mut ChildSpawnerCommands,
    tab: CharacterSheetTab,
    index: usize,
    is_selected: bool,
    theme: &MaterialTheme,
) {
    parent
        .spawn((
            MaterialTab::new(index, tab.label()).selected(is_selected),
            Button,
            Node {
                flex_grow: 1.0,
                max_width: Val::Px(120.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            CharacterSheetTabButton { tab },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(tab.label()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if is_selected {
                    theme.primary
                } else {
                    theme.on_surface_variant
                }),
                CharacterSheetTabText,
            ));
        });
}

/// Spawn the scrollable content area for tab content
fn spawn_tab_content_area(
    parent: &mut ChildSpawnerCommands,
    character_data: &CharacterData,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    parent
        .spawn((Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip_y(),
            ..default()
        },))
        .with_children(|container| {
            // Scrollable inner content
            container
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(16.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        ..default()
                    },
                    ScrollableContent,
                ))
                .with_children(|content| {
                    if let Some(sheet) = &character_data.sheet {
                        // Spawn header with save button
                        spawn_header_row(
                            content,
                            sheet,
                            character_data.is_modified,
                            icon_assets,
                            theme,
                        );

                        // Spawn all tab contents (visibility controlled by selected tab)
                        spawn_all_tab_contents(
                            content,
                            sheet,
                            edit_state,
                            adding_state,
                            icon_assets,
                            icon_font,
                            theme,
                        );
                    } else {
                        // No character loaded - show create button
                        spawn_no_character_message(content);
                    }
                });
        });
}

/// Spawn header row with character name and save button
fn spawn_header_row(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    _is_modified: bool,
    icon_assets: &IconAssets,
    theme: &MaterialTheme,
) {
    let save_icon = icon_assets.icons.get(&IconType::Save).cloned();

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect::bottom(Val::Px(8.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            BorderColor::from(MD3_OUTLINE_VARIANT),
        ))
        .with_children(|header| {
            // Character name
            header.spawn((
                Text::new(&sheet.character.name),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE),
            ));

            // Save button
            // Use a Material button so `ButtonClickEvent` + disabling behavior are consistent.
            // Styling/disabled state will be handled by `update_save_button_appearance`.
            header
                .spawn((
                    MaterialButtonBuilder::new("Save")
                        .filled_tonal()
                        .build(theme),
                    SaveButton,
                ))
                .insert(Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|btn| {
                    if let Some(handle) = save_icon {
                        btn.spawn((
                            ImageNode::new(handle),
                            Node {
                                width: Val::Px(18.0),
                                height: Val::Px(18.0),
                                ..default()
                            },
                        ));
                    }
                    btn.spawn((
                        bevy_material_ui::button::ButtonLabel,
                        Text::new("Save"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.on_surface),
                    ));
                });
        });
}

/// Spawn all tab content containers (visibility toggled based on selected tab)
fn spawn_all_tab_contents(
    parent: &mut ChildSpawnerCommands,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
    icon_font: Handle<Font>,
    theme: &MaterialTheme,
) {
    // Basic Info tab content
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                ..default()
            },
            CharacterSheetTabContent {
                tab: CharacterSheetTab::BasicInfo,
            },
        ))
        .with_children(|content| {
            spawn_basic_info_content(
                content,
                sheet,
                edit_state,
                adding_state,
                icon_assets,
                icon_font.clone(),
                theme,
            );
        });

    // Attributes tab content
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: Display::None, // Hidden by default
                ..default()
            },
            CharacterSheetTabContent {
                tab: CharacterSheetTab::Attributes,
            },
        ))
        .with_children(|content| {
            spawn_attributes_content(
                content,
                sheet,
                edit_state,
                adding_state,
                icon_assets,
                icon_font.clone(),
                theme,
            );
        });

    // Combat tab content
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: Display::None,
                ..default()
            },
            CharacterSheetTabContent {
                tab: CharacterSheetTab::Combat,
            },
        ))
        .with_children(|content| {
            spawn_combat_content(
                content,
                sheet,
                edit_state,
                adding_state,
                icon_assets,
                icon_font.clone(),
                theme,
            );
        });

    // Saving Throws tab content
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: Display::None,
                ..default()
            },
            CharacterSheetTabContent {
                tab: CharacterSheetTab::SavingThrows,
            },
        ))
        .with_children(|content| {
            spawn_saving_throws_content(
                content,
                sheet,
                edit_state,
                adding_state,
                icon_assets,
                icon_font.clone(),
                theme,
            );
        });

    // Skills tab content
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: Display::None,
                ..default()
            },
            CharacterSheetTabContent {
                tab: CharacterSheetTab::Skills,
            },
        ))
        .with_children(|content| {
            spawn_skills_content(
                content,
                sheet,
                edit_state,
                adding_state,
                icon_assets,
                icon_font.clone(),
                theme,
            );
        });
}

/// Spawn the "no character" message with create button
fn spawn_no_character_message(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            min_height: Val::Px(300.0),
            ..default()
        })
        .with_children(|center| {
            // Message
            center.spawn((
                Text::new("No character loaded"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(MD3_ON_SURFACE_VARIANT),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Create button
            center
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(MD3_PRIMARY),
                    BorderRadius::all(Val::Px(12.0)),
                    NewCharacterButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("+ Create First Character"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(MD3_ON_PRIMARY),
                    ));
                });
        });
}

// ============================================================================
// Tab Switching Handlers
// ============================================================================

/// Handle tab change events from bevy_material_ui for character sheet tabs
pub fn handle_sheet_tab_clicks(
    mut tab_events: MessageReader<TabChangeEvent>,
    mut selected_tab: ResMut<SelectedCharacterSheetTab>,
    sheet_tab_query: Query<&CharacterSheetTabButton>,
) {
    for event in tab_events.read() {
        // Check if this is a character sheet tab by looking up the entity
        if let Ok(sheet_tab) = sheet_tab_query.get(event.tab_entity) {
            selected_tab.current = sheet_tab.tab;
        }
    }
}

/// Update tab button visuals based on selection (syncs with bevy_material_ui)
pub fn update_sheet_tab_styles(
    selected_tab: Res<SelectedCharacterSheetTab>,
    mut tabs_query: Query<&mut MaterialTabs, With<CharacterSheetTabBar>>,
    mut tab_query: Query<(&CharacterSheetTabButton, &mut MaterialTab)>,
    mut text_query: Query<&mut TextColor, With<CharacterSheetTabText>>,
    theme: Option<Res<MaterialTheme>>,
) {
    if !selected_tab.is_changed() {
        return;
    }

    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let selected_index = CharacterSheetTab::all()
        .iter()
        .position(|&t| t == selected_tab.current)
        .unwrap_or(0);

    // Update the MaterialTabs selected index
    for mut tabs in tabs_query.iter_mut() {
        tabs.selected = selected_index;
    }

    // Update individual tab selected states
    for (sheet_tab, mut material_tab) in tab_query.iter_mut() {
        material_tab.selected = sheet_tab.tab == selected_tab.current;
    }

    // Update text colors - simplified approach
    for mut text_color in text_query.iter_mut() {
        text_color.0 = theme.on_surface_variant;
    }
}

/// Update tab content visibility based on selected tab
pub fn update_sheet_tab_visibility(
    selected_tab: Res<SelectedCharacterSheetTab>,
    mut content_query: Query<(&CharacterSheetTabContent, &mut Node), Without<ScrollableContent>>,
    mut scrollable_query: Query<
        &mut Node,
        (With<ScrollableContent>, Without<CharacterSheetTabContent>),
    >,
) {
    if !selected_tab.is_changed() {
        return;
    }

    for (content, mut node) in content_query.iter_mut() {
        node.display = if content.tab == selected_tab.current {
            Display::Flex
        } else {
            Display::None
        };
    }

    // When switching tabs, reset scroll so the newly selected tab isn't hidden
    // due to a previous tab's scroll offset.
    for mut node in scrollable_query.iter_mut() {
        node.top = Val::Px(0.0);
    }
}
