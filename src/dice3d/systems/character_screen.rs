//! Character screen UI system
//!
//! This module contains systems for displaying and editing character sheets,
//! tab navigation, and character file management.

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::dice3d::types::*;

// ============================================================================
// Colors and Styling Constants
// ============================================================================

const TAB_ACTIVE_BG: Color = Color::srgb(0.2, 0.4, 0.6);
const TAB_INACTIVE_BG: Color = Color::srgb(0.15, 0.15, 0.2);
const TAB_HOVER_BG: Color = Color::srgb(0.25, 0.35, 0.45);
const PANEL_BG: Color = Color::srgb(0.1, 0.1, 0.15);
const GROUP_BG: Color = Color::srgb(0.12, 0.12, 0.18);
const FIELD_BG: Color = Color::srgb(0.08, 0.08, 0.12);
const FIELD_MODIFIED_BG: Color = Color::srgb(0.2, 0.15, 0.08); // Orange tint for modified fields
const BUTTON_BG: Color = Color::srgb(0.2, 0.5, 0.3);
#[allow(dead_code)]
const BUTTON_HOVER: Color = Color::srgb(0.25, 0.6, 0.35);
const TEXT_PRIMARY: Color = Color::WHITE;
const TEXT_SECONDARY: Color = Color::srgb(0.7, 0.7, 0.7);
const TEXT_MUTED: Color = Color::srgb(0.5, 0.5, 0.5);
const PROFICIENT_COLOR: Color = Color::srgb(0.3, 0.7, 0.3);

// Icon constants (simple Unicode symbols that render reliably)
const ICON_EDIT: &str = "✎";
const ICON_CHECK: &str = "✓";
const ICON_CANCEL: &str = "✕";
const ICON_DELETE: &str = "✕";

// Icon button colors
const ICON_BUTTON_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.0); // Transparent
const ICON_BUTTON_ACTIVE: Color = Color::srgb(0.3, 0.5, 0.4);

// ============================================================================
// Tab Bar Setup
// ============================================================================

/// Setup the tab bar UI (called once on startup)
pub fn setup_tab_bar(mut commands: Commands, icon_assets: Res<IconAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    height: Val::Px(40.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.08, 0.08, 0.1)),
                z_index: ZIndex::Global(100),
                ..default()
            },
            TabBar,
        ))
        .with_children(|parent| {
            // Dice Roller Tab
            spawn_tab_button(
                parent,
                &icon_assets,
                "Dice Roller",
                IconType::Dice,
                AppTab::DiceRoller,
                true,
            );
            // Character Sheet Tab
            spawn_tab_button(
                parent,
                &icon_assets,
                "Character",
                IconType::Character,
                AppTab::CharacterSheet,
                false,
            );
            // DnD Info Tab
            spawn_tab_button(
                parent,
                &icon_assets,
                "DnD Rolling Info",
                IconType::Info,
                AppTab::DndInfo,
                false,
            );
            // Contributors Tab
            spawn_tab_button(
                parent,
                &icon_assets,
                "Contributors",
                IconType::Character,
                AppTab::Contributors,
                false,
            );
        });
}

fn spawn_tab_button(
    parent: &mut ChildBuilder,
    icon_assets: &IconAssets,
    text: &str,
    icon_type: IconType,
    tab: AppTab,
    is_active: bool,
) {
    let icon_handle = icon_assets.icons.get(&icon_type).cloned();

    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                background_color: BackgroundColor(if is_active {
                    TAB_ACTIVE_BG
                } else {
                    TAB_INACTIVE_BG
                }),
                border_color: BorderColor(if is_active {
                    Color::srgb(0.4, 0.6, 0.8)
                } else {
                    Color::srgb(0.2, 0.2, 0.3)
                }),
                ..default()
            },
            TabButton { tab },
        ))
        .with_children(|button| {
            // Icon
            if let Some(handle) = icon_handle {
                button.spawn(ImageBundle {
                    image: UiImage::new(handle),
                    style: Style {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                });
            }
            // Text
            button.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 16.0,
                    color: if is_active {
                        TEXT_PRIMARY
                    } else {
                        TEXT_SECONDARY
                    },
                    ..default()
                },
            ));
        });
}

// ============================================================================
// Character Screen Setup
// ============================================================================

/// Setup the character screen UI (spawned when switching to character tab)
pub fn setup_character_screen(
    mut commands: Commands,
    character_data: Res<CharacterData>,
    character_manager: Res<CharacterManager>,
    edit_state: Res<GroupEditState>,
    adding_state: Res<AddingEntryState>,
    icon_assets: Res<IconAssets>,
) {
    // Root container (hidden by default, shown when tab is active)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(45.0), // Below tab bar
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                background_color: BackgroundColor(PANEL_BG),
                visibility: Visibility::Hidden,
                ..default()
            },
            CharacterScreenRoot,
        ))
        .with_children(|parent| {
            // Left panel - Character list
            spawn_character_list_panel(parent, &character_manager, &character_data, &icon_assets);

            // Right panel - Character stats
            spawn_character_stats_panel(
                parent,
                &character_data,
                &edit_state,
                &adding_state,
                &icon_assets,
            );
        });
}

fn spawn_character_list_panel(
    parent: &mut ChildBuilder,
    character_manager: &CharacterManager,
    character_data: &CharacterData,
    icon_assets: &IconAssets,
) {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(250.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(5.0),
                    border: UiRect::right(Val::Px(2.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
                border_color: BorderColor(Color::srgb(0.2, 0.2, 0.3)),
                ..default()
            },
            CharacterListPanel,
        ))
        .with_children(|panel| {
            // Header row with "Characters" title and Roll All button
            panel
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|header| {
                    header.spawn(TextBundle::from_section(
                        "Characters",
                        TextStyle {
                            font_size: 18.0,
                            color: TEXT_PRIMARY,
                            ..default()
                        },
                    ));

                    // Roll All Stats button (large dice icon)
                    header
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(36.0),
                                    height: Val::Px(36.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::srgb(0.6, 0.3, 0.1)),
                                ..default()
                            },
                            RollAllStatsButton,
                        ))
                        .with_children(|btn| {
                            if let Some(handle) = dice_icon.clone() {
                                btn.spawn(ImageBundle {
                                    image: UiImage::new(handle),
                                    style: Style {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                    ..default()
                                });
                            } else {
                                btn.spawn(TextBundle::from_section(
                                    "🎲",
                                    TextStyle {
                                        font_size: 20.0,
                                        color: TEXT_PRIMARY,
                                        ..default()
                                    },
                                ));
                            }
                        });
                });

            // New Character button
            panel
                .spawn((
                    ButtonBundle {
                        style: Style {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            margin: UiRect::vertical(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(BUTTON_BG),
                        ..default()
                    },
                    NewCharacterButton,
                ))
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "+ New Character",
                        TextStyle {
                            font_size: 14.0,
                            color: TEXT_PRIMARY,
                            ..default()
                        },
                    ));
                });

            // Character list items
            for (i, char_entry) in character_manager.characters.iter().enumerate() {
                let is_current = character_manager
                    .current_character_id
                    .map(|id| id == char_entry.id)
                    .unwrap_or(false);
                let display_name = if is_current && character_data.is_modified {
                    format!("{}*", char_entry.name)
                } else {
                    char_entry.name.clone()
                };
                let base_name = char_entry.name.clone();

                panel
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(if is_current {
                                Color::srgb(0.2, 0.3, 0.4) // Highlighted for selected character
                            } else {
                                FIELD_BG
                            }),
                            ..default()
                        },
                        CharacterListItem { index: i },
                    ))
                    .with_children(|item| {
                        item.spawn((
                            TextBundle::from_section(
                                display_name,
                                TextStyle {
                                    font_size: 14.0,
                                    color: TEXT_PRIMARY,
                                    ..default()
                                },
                            ),
                            CharacterListItemText {
                                index: i,
                                base_name,
                            },
                        ));
                    });
            }
        });
}

fn spawn_character_stats_panel(
    parent: &mut ChildBuilder,
    character_data: &CharacterData,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
) {
    // Outer container with fixed size and clipping
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                ..default()
            },
            CharacterStatsPanel,
        ))
        .with_children(|container| {
            // Inner scrollable content
            container
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(15.0),
                            // Add padding with extra at bottom to ensure last item isn't clipped
                            padding: UiRect {
                                left: Val::Px(15.0),
                                right: Val::Px(15.0),
                                top: Val::Px(15.0),
                                bottom: Val::Px(50.0), // Extra padding at bottom
                            },
                            ..default()
                        },
                        ..default()
                    },
                    ScrollableContent,
                ))
                .with_children(|panel| {
                    if let Some(sheet) = &character_data.sheet {
                        // Header with save button
                        spawn_header_row(panel, sheet, character_data.is_modified, icon_assets);

                        // Stats layout - two columns
                        panel
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(20.0),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|columns| {
                                // Left column
                                columns
                                    .spawn(NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Column,
                                            min_width: Val::Px(300.0),
                                            flex_grow: 1.0,
                                            row_gap: Val::Px(15.0),
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .with_children(|col| {
                                        spawn_basic_info_group(
                                            col,
                                            sheet,
                                            edit_state,
                                            adding_state,
                                            icon_assets,
                                        );
                                        spawn_attributes_group(
                                            col,
                                            sheet,
                                            edit_state,
                                            adding_state,
                                            icon_assets,
                                        );
                                        spawn_combat_group(
                                            col,
                                            sheet,
                                            edit_state,
                                            adding_state,
                                            icon_assets,
                                        );
                                    });

                                // Right column
                                columns
                                    .spawn(NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Column,
                                            min_width: Val::Px(300.0),
                                            flex_grow: 1.0,
                                            row_gap: Val::Px(15.0),
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .with_children(|col| {
                                        spawn_saving_throws_group(
                                            col,
                                            sheet,
                                            edit_state,
                                            adding_state,
                                            icon_assets,
                                        );
                                        spawn_skills_group(
                                            col,
                                            sheet,
                                            edit_state,
                                            adding_state,
                                            icon_assets,
                                        );
                                    });
                            });
                    } else {
                        // No character loaded - show centered "Create First Character" button
                        panel
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_grow: 1.0,
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    min_height: Val::Px(300.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|center| {
                                // Create First Character button
                                center
                                    .spawn((
                                        ButtonBundle {
                                            style: Style {
                                                padding: UiRect::axes(Val::Px(24.0), Val::Px(16.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            background_color: BackgroundColor(BUTTON_BG),
                                            ..default()
                                        },
                                        NewCharacterButton,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            "+ Create First Character",
                                            TextStyle {
                                                font_size: 20.0,
                                                color: TEXT_PRIMARY,
                                                ..default()
                                            },
                                        ));
                                    });
                            });
                    }
                });
        });
}

fn spawn_header_row(
    parent: &mut ChildBuilder,
    sheet: &CharacterSheet,
    is_modified: bool,
    icon_assets: &IconAssets,
) {
    let save_icon = icon_assets.icons.get(&IconType::Save).cloned();

    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            // Character name as title
            let title = format!(
                "{} {}",
                sheet.character.name,
                if is_modified { "*" } else { "" }
            );
            row.spawn(TextBundle::from_section(
                title,
                TextStyle {
                    font_size: 24.0,
                    color: TEXT_PRIMARY,
                    ..default()
                },
            ));

            // Save button
            row.spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    background_color: BackgroundColor(if is_modified {
                        BUTTON_BG
                    } else {
                        Color::srgb(0.3, 0.3, 0.35)
                    }),
                    ..default()
                },
                SaveButton,
            ))
            .with_children(|btn| {
                // Icon
                if let Some(handle) = save_icon {
                    btn.spawn(ImageBundle {
                        image: UiImage::new(handle),
                        style: Style {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        ..default()
                    });
                }
                btn.spawn(TextBundle::from_section(
                    "Save",
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_PRIMARY,
                        ..default()
                    },
                ));
            });
        });
}

/// Spawn a group header with title and edit button
fn spawn_group_header(
    parent: &mut ChildBuilder,
    title: &str,
    group_type: GroupType,
    edit_state: &GroupEditState,
    icon_assets: &IconAssets,
) {
    let is_editing = edit_state.editing_groups.contains(&group_type);
    let edit_icon = if is_editing {
        icon_assets.icons.get(&IconType::Check).cloned()
    } else {
        icon_assets.icons.get(&IconType::Edit).cloned()
    };

    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                ..default()
            },
            ..default()
        })
        .with_children(|header| {
            // Title
            header.spawn(TextBundle::from_section(
                title,
                TextStyle {
                    font_size: 16.0,
                    color: TEXT_PRIMARY,
                    ..default()
                },
            ));

            // Edit button
            header
                .spawn((
                    ButtonBundle {
                        style: Style {
                            padding: UiRect::all(Val::Px(4.0)),
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(if is_editing {
                            ICON_BUTTON_ACTIVE
                        } else {
                            ICON_BUTTON_BG
                        }),
                        ..default()
                    },
                    GroupEditButton {
                        group_type: group_type.clone(),
                    },
                ))
                .with_children(|btn| {
                    if let Some(handle) = edit_icon {
                        btn.spawn(ImageBundle {
                            image: UiImage::new(handle),
                            style: Style {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                ..default()
                            },
                            ..default()
                        });
                    } else {
                        btn.spawn(TextBundle::from_section(
                            if is_editing { ICON_CHECK } else { ICON_EDIT },
                            TextStyle {
                                font_size: 14.0,
                                color: if is_editing {
                                    PROFICIENT_COLOR
                                } else {
                                    TEXT_MUTED
                                },
                                ..default()
                            },
                        ));
                    }
                });
        });
}

/// Spawn a plus button at the bottom of a group, or an input field if currently adding
fn spawn_group_add_button(
    parent: &mut ChildBuilder,
    group_type: GroupType,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
) {
    // Check if we're currently adding to this group
    let is_adding = adding_state.adding_to.as_ref() == Some(&group_type);
    let check_icon = icon_assets.icons.get(&IconType::Check).cloned();
    let cancel_icon = icon_assets.icons.get(&IconType::Cancel).cloned();
    let add_icon = icon_assets.icons.get(&IconType::Add).cloned();

    if is_adding {
        // Show input field for new entry name
        parent
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.15, 0.2, 0.35, 0.9)),
                border_color: BorderColor(Color::srgb(0.3, 0.5, 0.7)),
                ..default()
            })
            .with_children(|row| {
                // Name input display area
                row.spawn((
                    NodeBundle {
                        style: Style {
                            flex_grow: 1.0,
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            min_height: Val::Px(24.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                        ..default()
                    },
                    NewEntryInput {
                        group_type: group_type.clone(),
                    },
                ))
                .with_children(|input| {
                    input.spawn(TextBundle::from_section(
                        if adding_state.new_entry_name.is_empty() {
                            "Name...".to_string()
                        } else {
                            format!("{}|", adding_state.new_entry_name)
                        },
                        TextStyle {
                            font_size: 14.0,
                            color: if adding_state.new_entry_name.is_empty() {
                                TEXT_MUTED
                            } else {
                                Color::srgb(0.9, 0.9, 0.5)
                            },
                            ..default()
                        },
                    ));
                });

                // Colon separator
                row.spawn(TextBundle::from_section(
                    ":",
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_SECONDARY,
                        ..default()
                    },
                ));

                // Value input display area
                row.spawn((
                    NodeBundle {
                        style: Style {
                            flex_grow: 1.0,
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            min_height: Val::Px(24.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
                        ..default()
                    },
                    NewEntryValueInput {
                        group_type: group_type.clone(),
                    },
                ))
                .with_children(|input| {
                    input.spawn(TextBundle::from_section(
                        if adding_state.new_entry_value.is_empty() {
                            "Value...".to_string()
                        } else {
                            adding_state.new_entry_value.clone()
                        },
                        TextStyle {
                            font_size: 14.0,
                            color: if adding_state.new_entry_value.is_empty() {
                                TEXT_MUTED
                            } else {
                                TEXT_PRIMARY
                            },
                            ..default()
                        },
                    ));
                });

                // Confirm button
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
                        ..default()
                    },
                    NewEntryConfirmButton {
                        group_type: group_type.clone(),
                    },
                ))
                .with_children(|btn| {
                    if let Some(handle) = check_icon.clone() {
                        btn.spawn(ImageBundle {
                            image: UiImage::new(handle),
                            style: Style {
                                width: Val::Px(18.0),
                                height: Val::Px(18.0),
                                ..default()
                            },
                            ..default()
                        });
                    } else {
                        btn.spawn(TextBundle::from_section(
                            ICON_CHECK,
                            TextStyle {
                                font_size: 14.0,
                                color: Color::srgb(0.5, 1.0, 0.5),
                                ..default()
                            },
                        ));
                    }
                });

                // Cancel button
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                        ..default()
                    },
                    NewEntryCancelButton {
                        group_type: group_type.clone(),
                    },
                ))
                .with_children(|btn| {
                    if let Some(handle) = cancel_icon {
                        btn.spawn(ImageBundle {
                            image: UiImage::new(handle),
                            style: Style {
                                width: Val::Px(18.0),
                                height: Val::Px(18.0),
                                ..default()
                            },
                            ..default()
                        });
                    } else {
                        btn.spawn(TextBundle::from_section(
                            ICON_CANCEL,
                            TextStyle {
                                font_size: 14.0,
                                color: Color::srgb(1.0, 0.5, 0.5),
                                ..default()
                            },
                        ));
                    }
                });
            });
    } else {
        // Show regular add button
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.15, 0.35, 0.15, 0.8)),
                    border_color: BorderColor(Color::srgb(0.4, 0.7, 0.4)),
                    ..default()
                },
                GroupAddButton { group_type },
            ))
            .with_children(|btn| {
                if let Some(handle) = add_icon {
                    btn.spawn(ImageBundle {
                        image: UiImage::new(handle),
                        style: Style {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        ..default()
                    });
                }
                btn.spawn(TextBundle::from_section(
                    "Add New",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::srgb(0.5, 0.9, 0.5),
                        ..default()
                    },
                ));
            });
    }
}

/// Spawn a delete button for an entry (shown in edit mode)
fn spawn_delete_button(
    parent: &mut ChildBuilder,
    group_type: GroupType,
    entry_id: &str,
    icon_assets: &IconAssets,
) {
    let delete_icon = icon_assets.icons.get(&IconType::Delete).cloned();

    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::left(Val::Px(4.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.5, 0.2, 0.2, 0.8)),
                ..default()
            },
            DeleteEntryButton {
                group_type,
                entry_id: entry_id.to_string(),
            },
        ))
        .with_children(|btn| {
            if let Some(handle) = delete_icon {
                btn.spawn(ImageBundle {
                    image: UiImage::new(handle),
                    style: Style {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    ..default()
                });
            } else {
                btn.spawn(TextBundle::from_section(
                    ICON_DELETE,
                    TextStyle {
                        font_size: 12.0,
                        color: Color::srgb(1.0, 0.5, 0.5),
                        ..default()
                    },
                ));
            }
        });
}

/// Spawn a row for a custom field (editable name and value with delete button)
fn spawn_custom_field_row(
    parent: &mut ChildBuilder,
    field_name: &str,
    field_value: &str,
    group_type: GroupType,
    is_editing: bool,
    icon_assets: &IconAssets,
) {
    // Determine the editing fields based on group type
    let (label_field, value_field) = match &group_type {
        GroupType::BasicInfo => (
            EditingField::CustomBasicInfoLabel(field_name.to_string()),
            EditingField::CustomBasicInfo(field_name.to_string()),
        ),
        GroupType::Attributes => (
            EditingField::CustomAttributeLabel(field_name.to_string()),
            EditingField::CustomAttribute(field_name.to_string()),
        ),
        GroupType::Combat => (
            EditingField::CustomCombatLabel(field_name.to_string()),
            EditingField::CustomCombat(field_name.to_string()),
        ),
        _ => return, // Skills/Saves are handled differently
    };

    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            // Editable field name (clickable label)
            if is_editing {
                row.spawn((
                    ButtonBundle {
                        style: Style {
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.5)),
                        ..default()
                    },
                    EditableLabelButton {
                        field: label_field.clone(),
                        current_name: field_name.to_string(),
                    },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        TextBundle::from_section(
                            field_name,
                            TextStyle {
                                font_size: 14.0,
                                color: TEXT_SECONDARY,
                                ..default()
                            },
                        ),
                        EditableLabelText { field: label_field },
                    ));
                });
            } else {
                // Non-editing: just text
                row.spawn(TextBundle::from_section(
                    field_name,
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_SECONDARY,
                        ..default()
                    },
                ));
            }

            // Value and delete button container
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|right| {
                // Editable value field (clickable)
                let is_numeric = matches!(&group_type, GroupType::Attributes);
                right
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                min_width: Val::Px(40.0),
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(FIELD_BG),
                            ..default()
                        },
                        StatField {
                            field: value_field.clone(),
                            is_numeric,
                        },
                    ))
                    .with_children(|field| {
                        field.spawn((
                            TextBundle::from_section(
                                field_value,
                                TextStyle {
                                    font_size: 14.0,
                                    color: TEXT_PRIMARY,
                                    ..default()
                                },
                            ),
                            StatFieldValue { field: value_field },
                        ));
                    });

                // Delete button (shown when in edit mode)
                if is_editing {
                    spawn_delete_button(right, group_type, field_name, icon_assets);
                }
            });
        });
}

fn spawn_basic_info_group(
    parent: &mut ChildBuilder,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
) {
    let group_type = GroupType::BasicInfo;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                background_color: BackgroundColor(GROUP_BG),
                ..default()
            },
            StatGroup {
                name: "Basic Info".to_string(),
                group_type: group_type.clone(),
            },
        ))
        .with_children(|group| {
            spawn_group_header(
                group,
                "Basic Info",
                group_type.clone(),
                edit_state,
                icon_assets,
            );

            spawn_stat_field(
                group,
                "Name",
                &sheet.character.name,
                EditingField::CharacterName,
                false,
                is_editing,
                Some(group_type.clone()),
                Some("name"),
                icon_assets,
            );
            spawn_stat_field(
                group,
                "Class",
                &sheet.character.class,
                EditingField::CharacterClass,
                false,
                is_editing,
                Some(group_type.clone()),
                Some("class"),
                icon_assets,
            );
            spawn_stat_field(
                group,
                "Race",
                &sheet.character.race,
                EditingField::CharacterRace,
                false,
                is_editing,
                Some(group_type.clone()),
                Some("race"),
                icon_assets,
            );
            spawn_stat_field(
                group,
                "Level",
                &sheet.character.level.to_string(),
                EditingField::CharacterLevel,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("level"),
                icon_assets,
            );

            if let Some(subclass) = &sheet.character.subclass {
                spawn_readonly_field(group, "Subclass", subclass);
            }
            if let Some(background) = &sheet.character.background {
                spawn_readonly_field(group, "Background", background);
            }
            if let Some(alignment) = &sheet.character.alignment {
                spawn_readonly_field(group, "Alignment", alignment);
            }

            // Render custom basic info fields
            for (field_name, field_value) in sheet.custom_basic_info.iter() {
                spawn_custom_field_row(
                    group,
                    field_name,
                    field_value,
                    GroupType::BasicInfo,
                    is_editing,
                    icon_assets,
                );
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(group, group_type, adding_state, icon_assets);
            }
        });
}

fn spawn_attributes_group(
    parent: &mut ChildBuilder,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
) {
    let group_type = GroupType::Attributes;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                background_color: BackgroundColor(GROUP_BG),
                ..default()
            },
            StatGroup {
                name: "Attributes".to_string(),
                group_type: group_type.clone(),
            },
        ))
        .with_children(|group| {
            spawn_group_header(
                group,
                "Attributes",
                group_type.clone(),
                edit_state,
                icon_assets,
            );

            let attrs = [
                (
                    "Strength",
                    sheet.attributes.strength,
                    sheet.modifiers.strength,
                    EditingField::AttributeStrength,
                ),
                (
                    "Dexterity",
                    sheet.attributes.dexterity,
                    sheet.modifiers.dexterity,
                    EditingField::AttributeDexterity,
                ),
                (
                    "Constitution",
                    sheet.attributes.constitution,
                    sheet.modifiers.constitution,
                    EditingField::AttributeConstitution,
                ),
                (
                    "Intelligence",
                    sheet.attributes.intelligence,
                    sheet.modifiers.intelligence,
                    EditingField::AttributeIntelligence,
                ),
                (
                    "Wisdom",
                    sheet.attributes.wisdom,
                    sheet.modifiers.wisdom,
                    EditingField::AttributeWisdom,
                ),
                (
                    "Charisma",
                    sheet.attributes.charisma,
                    sheet.modifiers.charisma,
                    EditingField::AttributeCharisma,
                ),
            ];

            for (name, score, modifier, field) in attrs {
                spawn_attribute_row(group, name, score, modifier, field, is_editing, icon_assets);
            }

            // Render custom attributes
            for (attr_name, attr_score) in sheet.custom_attributes.iter() {
                let modifier = Attributes::calculate_modifier(*attr_score);
                spawn_custom_attribute_row(
                    group,
                    attr_name,
                    *attr_score,
                    modifier,
                    is_editing,
                    icon_assets,
                );
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(group, group_type, adding_state, icon_assets);
            }
        });
}

fn spawn_attribute_row(
    parent: &mut ChildBuilder,
    name: &str,
    score: i32,
    modifier: i32,
    field: EditingField,
    is_editing: bool,
    icon_assets: &IconAssets,
) {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();
    let attr_name = name.to_string();

    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            // Left side: attribute name with dice button
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|name_row| {
                // Small dice roll button
                name_row
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.5, 0.25, 0.1)),
                            ..default()
                        },
                        RollAttributeButton {
                            attribute: attr_name,
                        },
                    ))
                    .with_children(|btn| {
                        if let Some(handle) = dice_icon {
                            btn.spawn(ImageBundle {
                                image: UiImage::new(handle),
                                style: Style {
                                    width: Val::Px(14.0),
                                    height: Val::Px(14.0),
                                    ..default()
                                },
                                ..default()
                            });
                        } else {
                            btn.spawn(TextBundle::from_section(
                                "🎲",
                                TextStyle {
                                    font_size: 12.0,
                                    color: TEXT_PRIMARY,
                                    ..default()
                                },
                            ));
                        }
                    });

                // Attribute name
                name_row.spawn(TextBundle::from_section(
                    name,
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_SECONDARY,
                        ..default()
                    },
                ));
            });

            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|values| {
                // Score (editable, dimmed when in edit mode)
                values
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                min_width: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(if is_editing {
                                Color::srgba(0.12, 0.12, 0.15, 0.5)
                            } else {
                                FIELD_BG
                            }),
                            ..default()
                        },
                        StatField {
                            field: field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .with_children(|field_node| {
                        field_node.spawn((
                            TextBundle::from_section(
                                score.to_string(),
                                TextStyle {
                                    font_size: 14.0,
                                    color: if is_editing { TEXT_MUTED } else { TEXT_PRIMARY },
                                    ..default()
                                },
                            ),
                            StatFieldValue { field },
                        ));
                    });

                // Modifier (calculated, readonly)
                let mod_str = if modifier >= 0 {
                    format!("+{}", modifier)
                } else {
                    modifier.to_string()
                };
                values.spawn(TextBundle::from_section(
                    format!("({})", mod_str),
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_MUTED,
                        ..default()
                    },
                ));
            });

            // Delete button (shown when in edit mode)
            if is_editing {
                spawn_delete_button(row, GroupType::Attributes, name, icon_assets);
            }
        });
}

/// Spawn a row for a custom attribute (with delete button)
fn spawn_custom_attribute_row(
    parent: &mut ChildBuilder,
    name: &str,
    score: i32,
    modifier: i32,
    is_editing: bool,
    icon_assets: &IconAssets,
) {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();
    let attr_name = name.to_string();
    let label_field = EditingField::CustomAttributeLabel(name.to_string());
    let value_field = EditingField::CustomAttribute(name.to_string());

    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            // Left side: attribute name with dice button
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|name_row| {
                // Small dice roll button
                name_row
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.5, 0.25, 0.1)),
                            ..default()
                        },
                        RollAttributeButton {
                            attribute: attr_name,
                        },
                    ))
                    .with_children(|btn| {
                        if let Some(handle) = dice_icon {
                            btn.spawn(ImageBundle {
                                image: UiImage::new(handle),
                                style: Style {
                                    width: Val::Px(14.0),
                                    height: Val::Px(14.0),
                                    ..default()
                                },
                                ..default()
                            });
                        } else {
                            btn.spawn(TextBundle::from_section(
                                "🎲",
                                TextStyle {
                                    font_size: 12.0,
                                    color: TEXT_PRIMARY,
                                    ..default()
                                },
                            ));
                        }
                    });

                // Attribute name - editable when in edit mode
                if is_editing {
                    name_row
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::srgba(
                                    0.2, 0.2, 0.25, 0.5,
                                )),
                                ..default()
                            },
                            EditableLabelButton {
                                field: label_field.clone(),
                                current_name: name.to_string(),
                            },
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                TextBundle::from_section(
                                    name,
                                    TextStyle {
                                        font_size: 14.0,
                                        color: TEXT_SECONDARY,
                                        ..default()
                                    },
                                ),
                                EditableLabelText { field: label_field },
                            ));
                        });
                } else {
                    name_row.spawn(TextBundle::from_section(
                        name,
                        TextStyle {
                            font_size: 14.0,
                            color: TEXT_SECONDARY,
                            ..default()
                        },
                    ));
                }
            });

            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|values| {
                // Score display - clickable to edit
                values
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                min_width: Val::Px(30.0),
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(FIELD_BG),
                            ..default()
                        },
                        StatField {
                            field: value_field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .with_children(|field| {
                        field.spawn((
                            TextBundle::from_section(
                                score.to_string(),
                                TextStyle {
                                    font_size: 14.0,
                                    color: TEXT_PRIMARY,
                                    ..default()
                                },
                            ),
                            StatFieldValue { field: value_field },
                        ));
                    });

                // Modifier (calculated)
                let mod_str = if modifier >= 0 {
                    format!("+{}", modifier)
                } else {
                    modifier.to_string()
                };
                values.spawn(TextBundle::from_section(
                    format!("({})", mod_str),
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_MUTED,
                        ..default()
                    },
                ));

                // Delete button (shown when in edit mode)
                if is_editing {
                    spawn_delete_button(values, GroupType::Attributes, name, icon_assets);
                }
            });
        });
}

fn spawn_combat_group(
    parent: &mut ChildBuilder,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
) {
    let group_type = GroupType::Combat;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                background_color: BackgroundColor(GROUP_BG),
                ..default()
            },
            StatGroup {
                name: "Combat".to_string(),
                group_type: group_type.clone(),
            },
        ))
        .with_children(|group| {
            spawn_group_header(group, "Combat", group_type.clone(), edit_state, icon_assets);

            spawn_stat_field(
                group,
                "Armor Class",
                &sheet.combat.armor_class.to_string(),
                EditingField::ArmorClass,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("armor_class"),
                icon_assets,
            );
            spawn_stat_field(
                group,
                "Initiative",
                &format!(
                    "{}{}",
                    if sheet.combat.initiative >= 0 {
                        "+"
                    } else {
                        ""
                    },
                    sheet.combat.initiative
                ),
                EditingField::Initiative,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("initiative"),
                icon_assets,
            );
            spawn_stat_field(
                group,
                "Speed",
                &format!("{} ft", sheet.combat.speed),
                EditingField::Speed,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("speed"),
                icon_assets,
            );
            spawn_stat_field(
                group,
                "Proficiency",
                &format!("+{}", sheet.proficiency_bonus),
                EditingField::ProficiencyBonus,
                true,
                is_editing,
                Some(group_type.clone()),
                Some("proficiency_bonus"),
                icon_assets,
            );

            if let Some(hp) = &sheet.combat.hit_points {
                spawn_hp_field(group, hp, is_editing);
            }

            // Render custom combat stats
            for (stat_name, stat_value) in sheet.custom_combat.iter() {
                spawn_custom_field_row(
                    group,
                    stat_name,
                    stat_value,
                    GroupType::Combat,
                    is_editing,
                    icon_assets,
                );
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(group, group_type, adding_state, icon_assets);
            }
        });
}

fn spawn_hp_field(parent: &mut ChildBuilder, hp: &HitPoints, is_editing: bool) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            row.spawn(TextBundle::from_section(
                "Hit Points",
                TextStyle {
                    font_size: 14.0,
                    color: TEXT_SECONDARY,
                    ..default()
                },
            ));

            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|values| {
                // Current HP (dimmed when in edit mode)
                let hp_current_field = EditingField::HitPointsCurrent;
                values
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                min_width: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(if is_editing {
                                Color::srgba(0.12, 0.12, 0.15, 0.5)
                            } else {
                                FIELD_BG
                            }),
                            ..default()
                        },
                        StatField {
                            field: hp_current_field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .with_children(|field| {
                        field.spawn((
                            TextBundle::from_section(
                                hp.current.to_string(),
                                TextStyle {
                                    font_size: 14.0,
                                    color: if is_editing {
                                        TEXT_MUTED
                                    } else if hp.current < hp.maximum / 2 {
                                        Color::srgb(0.9, 0.4, 0.4)
                                    } else {
                                        TEXT_PRIMARY
                                    },
                                    ..default()
                                },
                            ),
                            StatFieldValue {
                                field: hp_current_field,
                            },
                        ));
                    });

                values.spawn(TextBundle::from_section(
                    "/",
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_MUTED,
                        ..default()
                    },
                ));

                // Max HP (dimmed when in edit mode)
                let hp_max_field = EditingField::HitPointsMaximum;
                values
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                min_width: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(if is_editing {
                                Color::srgba(0.12, 0.12, 0.15, 0.5)
                            } else {
                                FIELD_BG
                            }),
                            ..default()
                        },
                        StatField {
                            field: hp_max_field.clone(),
                            is_numeric: true,
                        },
                    ))
                    .with_children(|field| {
                        field.spawn((
                            TextBundle::from_section(
                                hp.maximum.to_string(),
                                TextStyle {
                                    font_size: 14.0,
                                    color: if is_editing { TEXT_MUTED } else { TEXT_PRIMARY },
                                    ..default()
                                },
                            ),
                            StatFieldValue {
                                field: hp_max_field,
                            },
                        ));
                    });

                if hp.temporary > 0 {
                    values.spawn(TextBundle::from_section(
                        format!("(+{} temp)", hp.temporary),
                        TextStyle {
                            font_size: 12.0,
                            color: Color::srgb(0.4, 0.7, 0.9),
                            ..default()
                        },
                    ));
                }
            });
        });
}

fn spawn_saving_throws_group(
    parent: &mut ChildBuilder,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
) {
    let group_type = GroupType::SavingThrows;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                background_color: BackgroundColor(GROUP_BG),
                ..default()
            },
            StatGroup {
                name: "Saving Throws".to_string(),
                group_type: group_type.clone(),
            },
        ))
        .with_children(|group| {
            spawn_group_header(
                group,
                "Saving Throws",
                group_type.clone(),
                edit_state,
                icon_assets,
            );

            let abilities = [
                "strength",
                "dexterity",
                "constitution",
                "intelligence",
                "wisdom",
                "charisma",
            ];
            for ability in abilities {
                if let Some(save) = sheet.saving_throws.get(ability) {
                    spawn_saving_throw_row(group, ability, save, is_editing, icon_assets);
                }
            }

            // Also show any custom saving throws
            for (save_name, save) in sheet.saving_throws.iter() {
                if !abilities.contains(&save_name.as_str()) {
                    spawn_saving_throw_row(group, save_name, save, is_editing, icon_assets);
                }
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(group, group_type, adding_state, icon_assets);
            }
        });
}

fn spawn_saving_throw_row(
    parent: &mut ChildBuilder,
    ability: &str,
    save: &SavingThrow,
    is_editing: bool,
    icon_assets: &IconAssets,
) {
    let ability_owned = ability.to_string();

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
            SavingThrowRow {
                ability: ability_owned.clone(),
            },
        ))
        .with_children(|row| {
            // Proficiency indicator and name
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|left| {
                // Proficiency indicator (non-clickable, checked if modifier != 0)
                let has_value = save.modifier != 0;
                left.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(14.0),
                            height: Val::Px(14.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(if has_value {
                            PROFICIENT_COLOR
                        } else {
                            Color::NONE
                        }),
                        border_color: BorderColor(TEXT_MUTED),
                        ..default()
                    },
                    ProficiencyCheckbox {
                        target: ProficiencyTarget::SavingThrow(ability_owned.clone()),
                    },
                ));

                // Ability name (capitalized) - editable when in edit mode
                let display_name = format!(
                    "{}{}",
                    ability.chars().next().unwrap().to_uppercase(),
                    &ability[1..3]
                );
                let has_value = save.modifier != 0;
                let label_field = EditingField::SavingThrowLabel(ability_owned.clone());

                if is_editing {
                    // Clickable label when in edit mode
                    left.spawn((
                        ButtonBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.5)),
                            border_color: BorderColor(Color::srgb(0.3, 0.3, 0.4)),
                            ..default()
                        },
                        EditableLabelButton {
                            field: label_field.clone(),
                            current_name: ability_owned.clone(),
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            TextBundle::from_section(
                                display_name.clone(),
                                TextStyle {
                                    font_size: 14.0,
                                    color: if has_value {
                                        PROFICIENT_COLOR
                                    } else {
                                        TEXT_SECONDARY
                                    },
                                    ..default()
                                },
                            ),
                            EditableLabelText { field: label_field },
                        ));
                    });
                } else {
                    // Static text when not in edit mode
                    left.spawn(TextBundle::from_section(
                        display_name,
                        TextStyle {
                            font_size: 14.0,
                            color: if has_value {
                                PROFICIENT_COLOR
                            } else {
                                TEXT_SECONDARY
                            },
                            ..default()
                        },
                    ));
                }
            });

            // Modifier - editable field (dimmed when in edit mode)
            let mod_str = if save.modifier >= 0 {
                format!("+{}", save.modifier)
            } else {
                save.modifier.to_string()
            };
            let field = EditingField::SavingThrow(ability_owned.clone());
            let field_clone = field.clone();
            row.spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                        min_width: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(if is_editing {
                        Color::srgba(0.12, 0.12, 0.15, 0.5) // Dimmed when editing labels
                    } else {
                        FIELD_BG
                    }),
                    ..default()
                },
                StatField {
                    field,
                    is_numeric: true,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    TextBundle::from_section(
                        mod_str,
                        TextStyle {
                            font_size: 14.0,
                            color: if is_editing { TEXT_MUTED } else { TEXT_PRIMARY },
                            ..default()
                        },
                    ),
                    StatFieldValue { field: field_clone },
                ));
            });

            // Delete button (shown when in edit mode)
            if is_editing {
                spawn_delete_button(row, GroupType::SavingThrows, &ability_owned, icon_assets);
            }
        });
}

fn spawn_skills_group(
    parent: &mut ChildBuilder,
    sheet: &CharacterSheet,
    edit_state: &GroupEditState,
    adding_state: &AddingEntryState,
    icon_assets: &IconAssets,
) {
    let group_type = GroupType::Skills;
    let is_editing = edit_state.editing_groups.contains(&group_type);

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(4.0),
                    // No max_height - allow dynamic growth
                    ..default()
                },
                background_color: BackgroundColor(GROUP_BG),
                ..default()
            },
            StatGroup {
                name: "Skills".to_string(),
                group_type: group_type.clone(),
            },
        ))
        .with_children(|group| {
            spawn_group_header(group, "Skills", group_type.clone(), edit_state, icon_assets);

            // Sort skills alphabetically
            let mut skills: Vec<_> = sheet.skills.iter().collect();
            skills.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

            for (skill_name, skill) in skills {
                spawn_skill_row(group, skill_name, skill, is_editing, icon_assets);
            }

            // Add button (shown when editing)
            if is_editing {
                spawn_group_add_button(group, group_type, adding_state, icon_assets);
            }
        });
}

fn spawn_skill_row(
    parent: &mut ChildBuilder,
    skill_name: &str,
    skill: &Skill,
    is_editing: bool,
    icon_assets: &IconAssets,
) {
    // Convert camelCase to Title Case
    let display_name = skill_name
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().next().unwrap()
            } else if c.is_uppercase() {
                format!(" {}", c).chars().next().unwrap()
            } else {
                c
            }
        })
        .collect::<String>();

    let skill_name_owned = skill_name.to_string();

    parent
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
            SkillRow {
                skill_name: skill_name_owned.clone(),
            },
        ))
        .with_children(|row| {
            // Left side: proficiency checkbox, name
            row.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|left| {
                // Proficiency indicator (non-clickable, checked if modifier != 0)
                let has_value = skill.modifier != 0;
                left.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(14.0),
                            height: Val::Px(14.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(if has_value {
                            PROFICIENT_COLOR
                        } else {
                            Color::NONE
                        }),
                        border_color: BorderColor(TEXT_MUTED),
                        ..default()
                    },
                    ProficiencyCheckbox {
                        target: ProficiencyTarget::Skill(skill_name_owned.clone()),
                    },
                ));

                // Skill name - editable when in edit mode
                let has_value = skill.modifier != 0;
                let label_field = EditingField::SkillLabel(skill_name_owned.clone());

                if is_editing {
                    // Clickable label when in edit mode
                    left.spawn((
                        ButtonBundle {
                            style: Style {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.5)),
                            border_color: BorderColor(Color::srgb(0.3, 0.3, 0.4)),
                            ..default()
                        },
                        EditableLabelButton {
                            field: label_field.clone(),
                            current_name: skill_name_owned.clone(),
                        },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            TextBundle::from_section(
                                display_name.clone(),
                                TextStyle {
                                    font_size: 13.0,
                                    color: if has_value {
                                        PROFICIENT_COLOR
                                    } else {
                                        TEXT_SECONDARY
                                    },
                                    ..default()
                                },
                            ),
                            EditableLabelText { field: label_field },
                        ));
                    });
                } else {
                    // Static text when not in edit mode
                    left.spawn(TextBundle::from_section(
                        display_name,
                        TextStyle {
                            font_size: 13.0,
                            color: if has_value {
                                PROFICIENT_COLOR
                            } else {
                                TEXT_SECONDARY
                            },
                            ..default()
                        },
                    ));
                }
            });

            // Modifier - editable field (dimmed when in edit mode)
            let mod_str = if skill.modifier >= 0 {
                format!("+{}", skill.modifier)
            } else {
                skill.modifier.to_string()
            };
            let field = EditingField::Skill(skill_name_owned.clone());
            let field_clone = field.clone();
            row.spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                        min_width: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(if is_editing {
                        Color::srgba(0.12, 0.12, 0.15, 0.5) // Dimmed when editing labels
                    } else {
                        FIELD_BG
                    }),
                    ..default()
                },
                StatField {
                    field,
                    is_numeric: true,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    TextBundle::from_section(
                        mod_str,
                        TextStyle {
                            font_size: 13.0,
                            color: if is_editing { TEXT_MUTED } else { TEXT_PRIMARY },
                            ..default()
                        },
                    ),
                    StatFieldValue { field: field_clone },
                ));
            });

            // Delete button (shown when in edit mode)
            if is_editing {
                spawn_delete_button(row, GroupType::Skills, &skill_name_owned, icon_assets);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn spawn_stat_field(
    parent: &mut ChildBuilder,
    label: &str,
    value: &str,
    field: EditingField,
    is_numeric: bool,
    is_editing: bool,
    group_type: Option<GroupType>,
    entry_id: Option<&str>,
    icon_assets: &IconAssets,
) {
    let field_clone = field.clone();
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            row.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 14.0,
                    color: TEXT_SECONDARY,
                    ..default()
                },
            ));

            // Use ButtonBundle for clickable interaction (dimmed when in edit mode)
            row.spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        min_width: Val::Px(80.0),
                        ..default()
                    },
                    background_color: BackgroundColor(if is_editing {
                        Color::srgba(0.12, 0.12, 0.15, 0.5)
                    } else {
                        FIELD_BG
                    }),
                    ..default()
                },
                StatField { field, is_numeric },
            ))
            .with_children(|field_node| {
                field_node.spawn((
                    TextBundle::from_section(
                        value,
                        TextStyle {
                            font_size: 14.0,
                            color: if is_editing { TEXT_MUTED } else { TEXT_PRIMARY },
                            ..default()
                        },
                    ),
                    StatFieldValue { field: field_clone },
                ));
            });

            // Delete button (shown when in edit mode)
            if is_editing {
                if let (Some(gt), Some(eid)) = (group_type, entry_id) {
                    spawn_delete_button(row, gt, eid, icon_assets);
                }
            }
        });
}

fn spawn_readonly_field(parent: &mut ChildBuilder, label: &str, value: &str) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            row.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 14.0,
                    color: TEXT_SECONDARY,
                    ..default()
                },
            ));

            row.spawn(TextBundle::from_section(
                value,
                TextStyle {
                    font_size: 14.0,
                    color: TEXT_MUTED,
                    ..default()
                },
            ));
        });
}

// ============================================================================
// Tab Switching Systems
// ============================================================================

/// Handle tab button clicks
pub fn handle_tab_clicks(
    mut interaction_query: Query<
        (
            &Interaction,
            &TabButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut ui_state: ResMut<UiState>,
) {
    let mut new_active_tab = None;

    for (interaction, tab_button, mut bg, mut border) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                new_active_tab = Some(tab_button.tab);
                // Immediately style the pressed tab as active
                *bg = BackgroundColor(TAB_ACTIVE_BG);
                *border = BorderColor(Color::srgb(0.4, 0.6, 0.8));
            }
            Interaction::Hovered => {
                if ui_state.active_tab != tab_button.tab {
                    *bg = BackgroundColor(TAB_HOVER_BG);
                }
            }
            Interaction::None => {
                // Reset to appropriate state based on whether it's active
                if ui_state.active_tab == tab_button.tab {
                    *bg = BackgroundColor(TAB_ACTIVE_BG);
                    *border = BorderColor(Color::srgb(0.4, 0.6, 0.8));
                } else {
                    *bg = BackgroundColor(TAB_INACTIVE_BG);
                    *border = BorderColor(Color::srgb(0.2, 0.2, 0.3));
                }
            }
        }
    }

    // Update the active tab if one was pressed
    if let Some(tab) = new_active_tab {
        ui_state.active_tab = tab;
    }
}

/// System to update tab button styles when active tab changes
pub fn update_tab_styles(
    ui_state: Res<UiState>,
    mut tab_buttons: Query<(&TabButton, &mut BackgroundColor, &mut BorderColor)>,
) {
    if !ui_state.is_changed() {
        return;
    }

    // Update all tab buttons based on which one is active
    for (tab_button, mut bg, mut border) in tab_buttons.iter_mut() {
        if tab_button.tab == ui_state.active_tab {
            *bg = BackgroundColor(TAB_ACTIVE_BG);
            *border = BorderColor(Color::srgb(0.4, 0.6, 0.8));
        } else {
            *bg = BackgroundColor(TAB_INACTIVE_BG);
            *border = BorderColor(Color::srgb(0.2, 0.2, 0.3));
        }
    }
}

/// Update visibility based on active tab
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
    mut char_screen_query: Query<
        &mut Visibility,
        (
            With<CharacterScreenRoot>,
            Without<DiceRollerRoot>,
            Without<DndInfoScreenRoot>,
            Without<ContributorsScreenRoot>,
        ),
    >,
    mut info_screen_query: Query<
        &mut Visibility,
        (
            With<DndInfoScreenRoot>,
            Without<DiceRollerRoot>,
            Without<CharacterScreenRoot>,
            Without<ContributorsScreenRoot>,
        ),
    >,
    mut contributors_screen_query: Query<
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

    for mut vis in dice_roller_query.iter_mut() {
        *vis = if ui_state.active_tab == AppTab::DiceRoller {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in char_screen_query.iter_mut() {
        *vis = if ui_state.active_tab == AppTab::CharacterSheet {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in info_screen_query.iter_mut() {
        *vis = if ui_state.active_tab == AppTab::DndInfo {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in contributors_screen_query.iter_mut() {
        *vis = if ui_state.active_tab == AppTab::Contributors {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// ============================================================================
// Mouse Wheel Scrolling System
// ============================================================================

/// Handle mouse wheel scrolling for the character stats panel
pub fn handle_scroll_input(
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut scrollable_query: Query<(&mut Style, &Node, &Parent), With<ScrollableContent>>,
    parent_query: Query<&Node>,
    ui_state: Res<UiState>,
    mut info_scroll_query: Query<
        (&mut Style, &Node, &Parent),
        (With<InfoScrollContent>, Without<ScrollableContent>),
    >,
) {
    // Only scroll when on character sheet or info tab
    if ui_state.active_tab != AppTab::CharacterSheet && ui_state.active_tab != AppTab::DndInfo {
        return;
    }

    let scroll_speed = 30.0;
    let mut scroll_delta = 0.0;

    for event in mouse_wheel.read() {
        scroll_delta += event.y * scroll_speed;
    }

    if scroll_delta == 0.0 {
        return;
    }

    // Handle character sheet scrolling
    if ui_state.active_tab == AppTab::CharacterSheet {
        for (mut style, node, parent) in scrollable_query.iter_mut() {
            if let Ok(parent_node) = parent_query.get(parent.get()) {
                let content_height = node.size().y;
                let container_height = parent_node.size().y;
                let max_scroll = (content_height - container_height).max(0.0);

                let current_top = match style.top {
                    Val::Px(px) => px,
                    _ => 0.0,
                };

                let new_top = (current_top + scroll_delta).clamp(-max_scroll, 0.0);
                style.top = Val::Px(new_top);
            }
        }
    }

    // Handle info screen scrolling
    if ui_state.active_tab == AppTab::DndInfo {
        for (mut style, node, parent) in info_scroll_query.iter_mut() {
            if let Ok(parent_node) = parent_query.get(parent.get()) {
                let content_height = node.size().y;
                let container_height = parent_node.size().y;
                let max_scroll = (content_height - container_height).max(0.0);

                let current_top = match style.top {
                    Val::Px(px) => px,
                    _ => 0.0,
                };

                let new_top = (current_top + scroll_delta).clamp(-max_scroll, 0.0);
                style.top = Val::Px(new_top);
            }
        }
    }
}

// ============================================================================
// Field Editing Systems
// ============================================================================

/// Handle clicking on stat fields to start editing
pub fn handle_stat_field_click(
    interaction_query: Query<(&Interaction, &StatField), Changed<Interaction>>,
    mut text_input: ResMut<TextInputState>,
    character_data: Res<CharacterData>,
    edit_state: Res<GroupEditState>,
) {
    for (interaction, stat_field) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Don't allow editing values while the group is in edit mode
            // Determine which group this field belongs to
            let field_group = match &stat_field.field {
                EditingField::CharacterName
                | EditingField::CharacterClass
                | EditingField::CharacterRace
                | EditingField::CharacterLevel
                | EditingField::CustomBasicInfo(_)
                | EditingField::CustomBasicInfoLabel(_) => Some(GroupType::BasicInfo),
                EditingField::AttributeStrength
                | EditingField::AttributeDexterity
                | EditingField::AttributeConstitution
                | EditingField::AttributeIntelligence
                | EditingField::AttributeWisdom
                | EditingField::AttributeCharisma
                | EditingField::CustomAttribute(_)
                | EditingField::CustomAttributeLabel(_) => Some(GroupType::Attributes),
                EditingField::ArmorClass
                | EditingField::Initiative
                | EditingField::Speed
                | EditingField::ProficiencyBonus
                | EditingField::HitPointsCurrent
                | EditingField::HitPointsMaximum
                | EditingField::CustomCombat(_)
                | EditingField::CustomCombatLabel(_) => Some(GroupType::Combat),
                EditingField::SavingThrow(_) | EditingField::SavingThrowLabel(_) => {
                    Some(GroupType::SavingThrows)
                }
                EditingField::Skill(_) | EditingField::SkillLabel(_) => Some(GroupType::Skills),
            };

            // Skip if this field's group is in edit mode
            if let Some(group) = field_group {
                if edit_state.editing_groups.contains(&group) {
                    continue;
                }
            }

            // Start editing this field
            let current_value = get_field_value(&character_data, &stat_field.field);
            text_input.active_field = Some(stat_field.field.clone());
            text_input.current_text = current_value;
            text_input.cursor_position = text_input.current_text.len();
        }
    }
}

/// Handle clicking on editable labels (skill/save names) to start renaming
pub fn handle_label_click(
    interaction_query: Query<(&Interaction, &EditableLabelButton), Changed<Interaction>>,
    mut text_input: ResMut<TextInputState>,
) {
    for (interaction, label_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Start editing this label
            text_input.active_field = Some(label_button.field.clone());
            text_input.current_text = label_button.current_name.clone();
            text_input.cursor_position = text_input.current_text.len();
        }
    }
}

/// Get the current value of a field from character data
fn get_field_value(character_data: &CharacterData, field: &EditingField) -> String {
    let Some(sheet) = &character_data.sheet else {
        return String::new();
    };

    match field {
        EditingField::CharacterName => sheet.character.name.clone(),
        EditingField::CharacterClass => sheet.character.class.clone(),
        EditingField::CharacterRace => sheet.character.race.clone(),
        EditingField::CharacterLevel => sheet.character.level.to_string(),
        EditingField::AttributeStrength => sheet.attributes.strength.to_string(),
        EditingField::AttributeDexterity => sheet.attributes.dexterity.to_string(),
        EditingField::AttributeConstitution => sheet.attributes.constitution.to_string(),
        EditingField::AttributeIntelligence => sheet.attributes.intelligence.to_string(),
        EditingField::AttributeWisdom => sheet.attributes.wisdom.to_string(),
        EditingField::AttributeCharisma => sheet.attributes.charisma.to_string(),
        EditingField::ArmorClass => sheet.combat.armor_class.to_string(),
        EditingField::Initiative => format_modifier(sheet.combat.initiative),
        EditingField::Speed => sheet.combat.speed.to_string(),
        EditingField::ProficiencyBonus => format_modifier(sheet.proficiency_bonus),
        EditingField::HitPointsCurrent => sheet
            .combat
            .hit_points
            .as_ref()
            .map(|hp| hp.current.to_string())
            .unwrap_or_default(),
        EditingField::HitPointsMaximum => sheet
            .combat
            .hit_points
            .as_ref()
            .map(|hp| hp.maximum.to_string())
            .unwrap_or_default(),
        EditingField::Skill(name) => sheet
            .skills
            .get(name)
            .map(|s| format_modifier(s.modifier))
            .unwrap_or_default(),
        EditingField::SavingThrow(name) => sheet
            .saving_throws
            .get(name)
            .map(|s| format_modifier(s.modifier))
            .unwrap_or_default(),
        // Label fields return the current name
        EditingField::SkillLabel(name) => name.clone(),
        EditingField::SavingThrowLabel(name) => name.clone(),
        // Custom fields
        EditingField::CustomBasicInfo(name) => sheet
            .custom_basic_info
            .get(name)
            .cloned()
            .unwrap_or_default(),
        EditingField::CustomBasicInfoLabel(name) => name.clone(),
        EditingField::CustomAttribute(name) => sheet
            .custom_attributes
            .get(name)
            .map(|v| v.to_string())
            .unwrap_or_default(),
        EditingField::CustomAttributeLabel(name) => name.clone(),
        EditingField::CustomCombat(name) => {
            sheet.custom_combat.get(name).cloned().unwrap_or_default()
        }
        EditingField::CustomCombatLabel(name) => name.clone(),
    }
}

/// Format a modifier with + or - sign (e.g., +5 or -2)
fn format_modifier(value: i32) -> String {
    if value >= 0 {
        format!("+{}", value)
    } else {
        value.to_string() // Negative numbers already have the minus sign
    }
}

/// Parse a modifier string that may have a leading + or - sign
fn parse_modifier(value: &str) -> Option<i32> {
    let trimmed = value.trim();
    // Handle explicit + sign
    if let Some(rest) = trimmed.strip_prefix('+') {
        rest.parse().ok()
    } else {
        trimmed.parse().ok()
    }
}

/// Get the ordered list of all editable fields for tab navigation
fn get_tab_order(character_data: &CharacterData) -> Vec<EditingField> {
    let mut fields = vec![
        // Basic info (top section, left to right / top to bottom)
        EditingField::CharacterName,
        EditingField::CharacterClass,
        EditingField::CharacterRace,
        EditingField::CharacterLevel,
        // Combat stats (right side, top to bottom)
        EditingField::ArmorClass,
        EditingField::Initiative,
        EditingField::Speed,
        EditingField::ProficiencyBonus,
        EditingField::HitPointsCurrent,
        EditingField::HitPointsMaximum,
        // Attributes (left column, top to bottom)
        EditingField::AttributeStrength,
        EditingField::AttributeDexterity,
        EditingField::AttributeConstitution,
        EditingField::AttributeIntelligence,
        EditingField::AttributeWisdom,
        EditingField::AttributeCharisma,
    ];

    // Add custom and dynamic fields
    if let Some(sheet) = &character_data.sheet {
        // Add custom basic info fields (alphabetically sorted)
        let mut custom_basic: Vec<_> = sheet.custom_basic_info.keys().collect();
        custom_basic.sort();
        for name in custom_basic {
            fields.push(EditingField::CustomBasicInfo(name.clone()));
        }

        // Add custom attributes (alphabetically sorted)
        let mut custom_attrs: Vec<_> = sheet.custom_attributes.keys().collect();
        custom_attrs.sort();
        for name in custom_attrs {
            fields.push(EditingField::CustomAttribute(name.clone()));
        }

        // Add custom combat fields (alphabetically sorted)
        let mut custom_combat: Vec<_> = sheet.custom_combat.keys().collect();
        custom_combat.sort();
        for name in custom_combat {
            fields.push(EditingField::CustomCombat(name.clone()));
        }

        // Add saving throws (alphabetically sorted)
        let mut saves: Vec<_> = sheet.saving_throws.keys().collect();
        saves.sort();
        for save in saves {
            fields.push(EditingField::SavingThrow(save.clone()));
        }

        // Add skills (alphabetically sorted)
        let mut skills: Vec<_> = sheet.skills.keys().collect();
        skills.sort();
        for skill in skills {
            fields.push(EditingField::Skill(skill.clone()));
        }
    }

    fields
}

/// Handle keyboard input for text editing
pub fn handle_text_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut text_input: ResMut<TextInputState>,
    mut character_data: ResMut<CharacterData>,
    ui_state: Res<UiState>,
) {
    // Only process when on character sheet
    if ui_state.active_tab != AppTab::CharacterSheet {
        return;
    }

    // Handle Tab/Shift+Tab for field navigation (works even without active field)
    if keyboard.just_pressed(KeyCode::Tab) {
        let tab_order = get_tab_order(&character_data);
        if tab_order.is_empty() {
            return;
        }

        let shift_held =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        // Apply current changes if a field was being edited
        if let Some(active_field) = text_input.active_field.clone() {
            let current_text = text_input.current_text.clone();
            apply_field_value(
                &mut character_data,
                &mut text_input,
                &active_field,
                &current_text,
            );
        }

        // Find current position and move to next/previous
        let current_idx = text_input
            .active_field
            .as_ref()
            .and_then(|f| tab_order.iter().position(|x| x == f));

        let new_idx = match current_idx {
            Some(idx) => {
                if shift_held {
                    if idx == 0 {
                        tab_order.len() - 1
                    } else {
                        idx - 1
                    }
                } else {
                    (idx + 1) % tab_order.len()
                }
            }
            None => {
                if shift_held {
                    tab_order.len() - 1
                } else {
                    0
                }
            }
        };

        let new_field = tab_order[new_idx].clone();
        let new_value = get_field_value(&character_data, &new_field);
        text_input.active_field = Some(new_field);
        text_input.current_text = new_value.clone();
        text_input.cursor_position = new_value.len();
        return;
    }

    let Some(active_field) = text_input.active_field.clone() else {
        return;
    };

    // Handle escape to cancel editing
    if keyboard.just_pressed(KeyCode::Escape) {
        text_input.active_field = None;
        text_input.current_text.clear();
        return;
    }

    // Handle enter to confirm editing
    if keyboard.just_pressed(KeyCode::Enter) {
        let current_text = text_input.current_text.clone();
        apply_field_value(
            &mut character_data,
            &mut text_input,
            &active_field,
            &current_text,
        );
        text_input.active_field = None;
        text_input.current_text.clear();
        return;
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) && !text_input.current_text.is_empty() {
        text_input.current_text.pop();
        return;
    }

    // Check if shift is pressed for uppercase letters
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Handle character input - collect chars first to avoid borrow issues
    let mut chars_to_add: Vec<char> = Vec::new();
    for event in char_events.read() {
        if event.state.is_pressed() {
            if let Some(key_code) = event.key_code.to_char(shift_pressed) {
                // For name fields, allow alphanumeric and space (replace special with _)
                let c = match &active_field {
                    EditingField::CharacterName => {
                        if key_code.is_alphanumeric() || key_code == ' ' {
                            key_code
                        } else {
                            '_'
                        }
                    }
                    // For plain numeric fields (no +/- sign display), only allow digits and minus
                    EditingField::CharacterLevel
                    | EditingField::AttributeStrength
                    | EditingField::AttributeDexterity
                    | EditingField::AttributeConstitution
                    | EditingField::AttributeIntelligence
                    | EditingField::AttributeWisdom
                    | EditingField::AttributeCharisma
                    | EditingField::ArmorClass
                    | EditingField::Speed
                    | EditingField::HitPointsCurrent
                    | EditingField::HitPointsMaximum => {
                        if key_code.is_ascii_digit() || key_code == '-' {
                            key_code
                        } else {
                            continue;
                        }
                    }
                    // For modifier fields, allow digits, minus, and plus
                    EditingField::Initiative
                    | EditingField::ProficiencyBonus
                    | EditingField::Skill(_)
                    | EditingField::SavingThrow(_) => {
                        if key_code.is_ascii_digit() || key_code == '-' || key_code == '+' {
                            key_code
                        } else {
                            continue;
                        }
                    }
                    _ => key_code,
                };
                chars_to_add.push(c);
            }
        }
    }

    for c in chars_to_add {
        text_input.current_text.push(c);
    }
}

/// Helper trait to convert KeyCode to char
trait KeyCodeToChar {
    fn to_char(&self, shift_pressed: bool) -> Option<char>;
}

impl KeyCodeToChar for KeyCode {
    fn to_char(&self, shift_pressed: bool) -> Option<char> {
        let base_char = match self {
            KeyCode::KeyA => Some('a'),
            KeyCode::KeyB => Some('b'),
            KeyCode::KeyC => Some('c'),
            KeyCode::KeyD => Some('d'),
            KeyCode::KeyE => Some('e'),
            KeyCode::KeyF => Some('f'),
            KeyCode::KeyG => Some('g'),
            KeyCode::KeyH => Some('h'),
            KeyCode::KeyI => Some('i'),
            KeyCode::KeyJ => Some('j'),
            KeyCode::KeyK => Some('k'),
            KeyCode::KeyL => Some('l'),
            KeyCode::KeyM => Some('m'),
            KeyCode::KeyN => Some('n'),
            KeyCode::KeyO => Some('o'),
            KeyCode::KeyP => Some('p'),
            KeyCode::KeyQ => Some('q'),
            KeyCode::KeyR => Some('r'),
            KeyCode::KeyS => Some('s'),
            KeyCode::KeyT => Some('t'),
            KeyCode::KeyU => Some('u'),
            KeyCode::KeyV => Some('v'),
            KeyCode::KeyW => Some('w'),
            KeyCode::KeyX => Some('x'),
            KeyCode::KeyY => Some('y'),
            KeyCode::KeyZ => Some('z'),
            KeyCode::Digit0 => Some('0'),
            KeyCode::Digit1 => Some('1'),
            KeyCode::Digit2 => Some('2'),
            KeyCode::Digit3 => Some('3'),
            KeyCode::Digit4 => Some('4'),
            KeyCode::Digit5 => Some('5'),
            KeyCode::Digit6 => Some('6'),
            KeyCode::Digit7 => Some('7'),
            KeyCode::Digit8 => Some('8'),
            KeyCode::Digit9 => Some('9'),
            KeyCode::Space => Some(' '),
            KeyCode::Minus => Some('-'),
            KeyCode::Equal => Some('+'), // Shift+= gives +, but we'll treat = as +
            KeyCode::NumpadAdd => Some('+'),
            KeyCode::NumpadSubtract => Some('-'),
            _ => None,
        };

        // Apply shift for uppercase letters
        base_char.map(|c| {
            if shift_pressed && c.is_ascii_lowercase() {
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
    }
}

/// Apply a value to a field in the character data
/// Only marks as modified if the value actually changed
fn apply_field_value(
    character_data: &mut CharacterData,
    text_input: &mut TextInputState,
    field: &EditingField,
    value: &str,
) {
    let Some(sheet) = &mut character_data.sheet else {
        return;
    };

    // Trim whitespace from both ends
    let value = value.trim();

    // Track if we actually made a change
    let mut changed = false;

    match field {
        EditingField::CharacterName => {
            let new_value = CharacterManager::sanitize_name(value);
            if sheet.character.name != new_value {
                sheet.character.name = new_value;
                changed = true;
            }
        }
        EditingField::CharacterClass => {
            let new_value = value.to_string();
            if sheet.character.class != new_value {
                sheet.character.class = new_value;
                changed = true;
            }
        }
        EditingField::CharacterRace => {
            let new_value = value.to_string();
            if sheet.character.race != new_value {
                sheet.character.race = new_value;
                changed = true;
            }
        }
        EditingField::CharacterLevel => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.character.level != v {
                    sheet.character.level = v;
                    changed = true;
                }
            }
        }
        EditingField::AttributeStrength => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.attributes.strength != v {
                    sheet.attributes.strength = v;
                    sheet.modifiers.strength = Attributes::calculate_modifier(v);
                    changed = true;
                }
            }
        }
        EditingField::AttributeDexterity => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.attributes.dexterity != v {
                    sheet.attributes.dexterity = v;
                    sheet.modifiers.dexterity = Attributes::calculate_modifier(v);
                    changed = true;
                }
            }
        }
        EditingField::AttributeConstitution => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.attributes.constitution != v {
                    sheet.attributes.constitution = v;
                    sheet.modifiers.constitution = Attributes::calculate_modifier(v);
                    changed = true;
                }
            }
        }
        EditingField::AttributeIntelligence => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.attributes.intelligence != v {
                    sheet.attributes.intelligence = v;
                    sheet.modifiers.intelligence = Attributes::calculate_modifier(v);
                    changed = true;
                }
            }
        }
        EditingField::AttributeWisdom => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.attributes.wisdom != v {
                    sheet.attributes.wisdom = v;
                    sheet.modifiers.wisdom = Attributes::calculate_modifier(v);
                    changed = true;
                }
            }
        }
        EditingField::AttributeCharisma => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.attributes.charisma != v {
                    sheet.attributes.charisma = v;
                    sheet.modifiers.charisma = Attributes::calculate_modifier(v);
                    changed = true;
                }
            }
        }
        EditingField::ArmorClass => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.combat.armor_class != v {
                    sheet.combat.armor_class = v;
                    changed = true;
                }
            }
        }
        EditingField::Initiative => {
            if let Some(v) = parse_modifier(value) {
                if sheet.combat.initiative != v {
                    sheet.combat.initiative = v;
                    changed = true;
                }
            }
        }
        EditingField::Speed => {
            if let Ok(v) = value.parse::<i32>() {
                if sheet.combat.speed != v {
                    sheet.combat.speed = v;
                    changed = true;
                }
            }
        }
        EditingField::ProficiencyBonus => {
            if let Some(v) = parse_modifier(value) {
                if sheet.proficiency_bonus != v {
                    sheet.proficiency_bonus = v;
                    changed = true;
                }
            }
        }
        EditingField::HitPointsCurrent => {
            if let Ok(v) = value.parse::<i32>() {
                if let Some(hp) = &mut sheet.combat.hit_points {
                    if hp.current != v {
                        hp.current = v;
                        changed = true;
                    }
                }
            }
        }
        EditingField::HitPointsMaximum => {
            if let Ok(v) = value.parse::<i32>() {
                if let Some(hp) = &mut sheet.combat.hit_points {
                    if hp.maximum != v {
                        hp.maximum = v;
                        changed = true;
                    }
                }
            }
        }
        EditingField::Skill(name) => {
            if let Some(v) = parse_modifier(value) {
                if let Some(skill) = sheet.skills.get_mut(name) {
                    if skill.modifier != v {
                        skill.modifier = v;
                        changed = true;
                    }
                }
            }
        }
        EditingField::SavingThrow(name) => {
            if let Some(v) = parse_modifier(value) {
                if let Some(save) = sheet.saving_throws.get_mut(name) {
                    if save.modifier != v {
                        save.modifier = v;
                        changed = true;
                    }
                }
            }
        }
        EditingField::SkillLabel(old_name) => {
            let new_name = CharacterManager::sanitize_name(value);
            if !new_name.is_empty() && &new_name != old_name {
                // Rename a skill - remove old key and insert with new key
                if let Some(skill_data) = sheet.skills.remove(old_name) {
                    sheet.skills.insert(new_name, skill_data);
                    changed = true;
                }
            }
        }
        EditingField::SavingThrowLabel(old_name) => {
            let new_name = CharacterManager::sanitize_name(value);
            if !new_name.is_empty() && &new_name != old_name {
                // Rename a saving throw - remove old key and insert with new key
                if let Some(save_data) = sheet.saving_throws.remove(old_name) {
                    sheet.saving_throws.insert(new_name, save_data);
                    changed = true;
                }
            }
        }
        // Custom fields
        EditingField::CustomBasicInfo(name) => {
            let new_value = value.to_string();
            if let Some(current) = sheet.custom_basic_info.get(name) {
                if current != &new_value {
                    sheet.custom_basic_info.insert(name.clone(), new_value);
                    changed = true;
                }
            }
        }
        EditingField::CustomBasicInfoLabel(old_name) => {
            let new_name = CharacterManager::sanitize_name(value);
            if !new_name.is_empty() && &new_name != old_name {
                if let Some(val) = sheet.custom_basic_info.remove(old_name) {
                    sheet.custom_basic_info.insert(new_name, val);
                    changed = true;
                }
            }
        }
        EditingField::CustomAttribute(name) => {
            if let Ok(v) = value.parse::<i32>() {
                if let Some(current) = sheet.custom_attributes.get(name) {
                    if current != &v {
                        sheet.custom_attributes.insert(name.clone(), v);
                        changed = true;
                    }
                }
            }
        }
        EditingField::CustomAttributeLabel(old_name) => {
            let new_name = CharacterManager::sanitize_name(value);
            if !new_name.is_empty() && &new_name != old_name {
                if let Some(val) = sheet.custom_attributes.remove(old_name) {
                    sheet.custom_attributes.insert(new_name, val);
                    changed = true;
                }
            }
        }
        EditingField::CustomCombat(name) => {
            let new_value = value.to_string();
            if let Some(current) = sheet.custom_combat.get(name) {
                if current != &new_value {
                    sheet.custom_combat.insert(name.clone(), new_value);
                    changed = true;
                }
            }
        }
        EditingField::CustomCombatLabel(old_name) => {
            let new_name = CharacterManager::sanitize_name(value);
            if !new_name.is_empty() && &new_name != old_name {
                if let Some(val) = sheet.custom_combat.remove(old_name) {
                    sheet.custom_combat.insert(new_name, val);
                    changed = true;
                }
            }
        }
    }

    // Only mark as modified if something actually changed
    if changed {
        character_data.is_modified = true;
        character_data.needs_refresh = true;
        text_input.modified_fields.insert(field.clone());
    }
}

/// Update the display of fields being edited
pub fn update_editing_display(
    text_input: Res<TextInputState>,
    character_data: Res<CharacterData>,
    mut text_query: Query<(&mut Text, &StatFieldValue)>,
    mut field_query: Query<(&StatField, &mut BackgroundColor)>,
    mut label_text_query: Query<(&mut Text, &EditableLabelText), Without<StatFieldValue>>,
    mut label_button_query: Query<(&EditableLabelButton, &mut BackgroundColor), Without<StatField>>,
) {
    if !text_input.is_changed() {
        return;
    }

    // Update field background colors based on active editing
    for (stat_field, mut bg) in field_query.iter_mut() {
        if Some(&stat_field.field) == text_input.active_field.as_ref() {
            // Highlight the active field (blue tint)
            *bg = BackgroundColor(Color::srgb(0.2, 0.25, 0.35));
        } else if text_input.modified_fields.contains(&stat_field.field) {
            // Highlight modified fields (orange tint)
            *bg = BackgroundColor(FIELD_MODIFIED_BG);
        } else {
            // Reset to default
            *bg = BackgroundColor(FIELD_BG);
        }
    }

    // Update editable label button backgrounds
    for (label_button, mut bg) in label_button_query.iter_mut() {
        if Some(&label_button.field) == text_input.active_field.as_ref() {
            // Highlight the active label (blue tint)
            *bg = BackgroundColor(Color::srgb(0.2, 0.25, 0.35));
        } else {
            // Reset to default
            *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.5));
        }
    }

    // Update text display for all stat fields
    for (mut text, field_value) in text_query.iter_mut() {
        if Some(&field_value.field) == text_input.active_field.as_ref() {
            // Show current input with cursor for active field
            let display = format!("{}|", text_input.current_text);
            if let Some(section) = text.sections.first_mut() {
                section.value = display;
                section.style.color = Color::srgb(0.9, 0.9, 0.5); // Highlight editing
            }
        } else {
            // Restore original value for inactive fields (removes cursor)
            let value = get_field_value(&character_data, &field_value.field);
            if let Some(section) = text.sections.first_mut() {
                // Only update if the current value has a cursor (ends with |)
                if section.value.ends_with('|') || section.style.color != TEXT_PRIMARY {
                    section.value = value;
                    section.style.color = TEXT_PRIMARY;
                }
            }
        }
    }

    // Update text display for editable labels
    for (mut text, label_text) in label_text_query.iter_mut() {
        if Some(&label_text.field) == text_input.active_field.as_ref() {
            // Show current input with cursor for active label
            let display = format!("{}|", text_input.current_text);
            if let Some(section) = text.sections.first_mut() {
                section.value = display;
                section.style.color = Color::srgb(0.9, 0.9, 0.5); // Highlight editing
            }
        }
        // Note: Labels get their value from the current_name in EditableLabelButton,
        // which is set when the label is spawned. After renaming, the UI needs to rebuild.
    }
}

// ============================================================================
// Expertise Toggle System
// ============================================================================

/// Get the base ability for a skill
fn get_skill_ability(skill: &str) -> &'static str {
    match skill.to_lowercase().as_str() {
        "acrobatics" | "sleightofhand" | "stealth" => "dexterity",
        "athletics" => "strength",
        "arcana" | "history" | "investigation" | "nature" | "religion" => "intelligence",
        "animalhandling" | "insight" | "medicine" | "perception" | "survival" => "wisdom",
        "deception" | "intimidation" | "performance" | "persuasion" => "charisma",
        _ => "strength",
    }
}

/// Handle expertise checkbox clicks
pub fn handle_expertise_toggle(
    interaction_query: Query<(&Interaction, &ExpertiseCheckbox), Changed<Interaction>>,
    mut character_data: ResMut<CharacterData>,
) {
    for (interaction, checkbox) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let Some(sheet) = &mut character_data.sheet else {
                continue;
            };

            if let Some(skill) = sheet.skills.get_mut(&checkbox.skill_name) {
                // Can only have expertise if proficient
                if skill.proficient {
                    let current = skill.expertise.unwrap_or(false);
                    skill.expertise = Some(!current);

                    // Recalculate modifier
                    let base_ability = get_skill_ability(&checkbox.skill_name);
                    let ability_mod = match base_ability {
                        "dexterity" => sheet.modifiers.dexterity,
                        "strength" => sheet.modifiers.strength,
                        "constitution" => sheet.modifiers.constitution,
                        "intelligence" => sheet.modifiers.intelligence,
                        "wisdom" => sheet.modifiers.wisdom,
                        "charisma" => sheet.modifiers.charisma,
                        _ => 0,
                    };
                    let prof_bonus = sheet.proficiency_bonus;
                    let expertise_bonus = if skill.expertise.unwrap_or(false) {
                        sheet.proficiency_bonus
                    } else {
                        0
                    };
                    skill.modifier = ability_mod + prof_bonus + expertise_bonus;
                    character_data.is_modified = true;
                }
            }
        }
    }
}

// ============================================================================
// Group Edit Mode System
// ============================================================================

/// Handle group edit button clicks to toggle edit mode
pub fn handle_group_edit_toggle(
    interaction_query: Query<(&Interaction, &GroupEditButton), Changed<Interaction>>,
    mut edit_state: ResMut<GroupEditState>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if edit_state.editing_groups.contains(&button.group_type) {
                edit_state.editing_groups.remove(&button.group_type);
            } else {
                edit_state.editing_groups.insert(button.group_type.clone());
            }
        }
    }
}

/// Handle group add button clicks to start adding a new entry (shows input field)
pub fn handle_group_add_click(
    interaction_query: Query<(&Interaction, &GroupAddButton), Changed<Interaction>>,
    mut adding_state: ResMut<AddingEntryState>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Set the adding state to show input field for this group
            adding_state.adding_to = Some(button.group_type.clone());
            adding_state.new_entry_name.clear();
        }
    }
}

/// Handle new entry confirm button clicks
pub fn handle_new_entry_confirm(
    interaction_query: Query<(&Interaction, &NewEntryConfirmButton), Changed<Interaction>>,
    mut adding_state: ResMut<AddingEntryState>,
    mut character_data: ResMut<CharacterData>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Trim whitespace from both fields
            let entry_name = adding_state.new_entry_name.trim().to_string();
            let entry_value = adding_state.new_entry_value.trim().to_string();

            // Only add if we have a name
            if entry_name.is_empty() {
                continue;
            }

            let group_type = button.group_type.clone();

            // Check if we have a sheet
            if character_data.sheet.is_none() {
                continue;
            }

            let mut added = false;

            match group_type {
                GroupType::Skills => {
                    if let Some(sheet) = &mut character_data.sheet {
                        // Parse value as modifier, default to 0
                        let modifier = entry_value.parse::<i32>().unwrap_or(0);
                        sheet.skills.insert(
                            entry_name.clone(),
                            Skill {
                                modifier,
                                proficient: false,
                                expertise: None,
                                proficiency_type: None,
                            },
                        );
                        added = true;
                    }
                }
                GroupType::SavingThrows => {
                    if let Some(sheet) = &mut character_data.sheet {
                        // Parse value as modifier, default to 0
                        let modifier = entry_value.parse::<i32>().unwrap_or(0);
                        sheet.saving_throws.insert(
                            entry_name.clone(),
                            SavingThrow {
                                modifier,
                                proficient: false,
                            },
                        );
                        added = true;
                    }
                }
                GroupType::BasicInfo => {
                    if let Some(sheet) = &mut character_data.sheet {
                        // Use the entered value
                        sheet
                            .custom_basic_info
                            .insert(entry_name.clone(), entry_value);
                        added = true;
                    }
                }
                GroupType::Attributes => {
                    if let Some(sheet) = &mut character_data.sheet {
                        // Parse value as score, default to 10
                        let score = entry_value.parse::<i32>().unwrap_or(10);
                        sheet.custom_attributes.insert(entry_name.clone(), score);
                        added = true;
                    }
                }
                GroupType::Combat => {
                    if let Some(sheet) = &mut character_data.sheet {
                        // Use the entered value
                        sheet.custom_combat.insert(entry_name.clone(), entry_value);
                        added = true;
                    }
                }
            }

            if added {
                character_data.is_modified = true;
            }

            // Clear the adding state
            adding_state.adding_to = None;
            adding_state.new_entry_name.clear();
            adding_state.new_entry_value.clear();
            adding_state.value_focused = false;
        }
    }
}

/// Handle new entry cancel button clicks
pub fn handle_new_entry_cancel(
    interaction_query: Query<(&Interaction, &NewEntryCancelButton), Changed<Interaction>>,
    mut adding_state: ResMut<AddingEntryState>,
) {
    for (interaction, _button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Clear the adding state
            adding_state.adding_to = None;
            adding_state.new_entry_name.clear();
            adding_state.new_entry_value.clear();
            adding_state.value_focused = false;
        }
    }
}

/// Handle text input for new entry name and value
pub fn handle_new_entry_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut adding_state: ResMut<AddingEntryState>,
) {
    // Only process input if we're adding an entry
    if adding_state.adding_to.is_none() {
        return;
    }

    for event in keyboard_events.read() {
        // Only process key press events
        if !event.state.is_pressed() {
            continue;
        }

        match event.key_code {
            KeyCode::Tab => {
                // Toggle between name and value field
                adding_state.value_focused = !adding_state.value_focused;
            }
            KeyCode::Backspace => {
                if adding_state.value_focused {
                    adding_state.new_entry_value.pop();
                } else {
                    adding_state.new_entry_name.pop();
                }
            }
            KeyCode::Enter => {
                // If we have a name, submit (confirm will handle it)
                // For now, we'll just clear if empty
                if adding_state.new_entry_name.is_empty() {
                    adding_state.adding_to = None;
                    adding_state.value_focused = false;
                }
            }
            KeyCode::Escape => {
                adding_state.adding_to = None;
                adding_state.new_entry_name.clear();
                adding_state.new_entry_value.clear();
                adding_state.value_focused = false;
            }
            _ => {
                // Handle text input from logical_key
                if let bevy::input::keyboard::Key::Character(ref s) = event.logical_key {
                    for c in s.chars() {
                        // Accept alphanumeric and some special characters
                        if c.is_alphanumeric()
                            || c == ' '
                            || c == '-'
                            || c == '_'
                            || c == '.'
                            || c == '+'
                        {
                            if adding_state.value_focused {
                                adding_state.new_entry_value.push(c);
                            } else {
                                adding_state.new_entry_name.push(c);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Update the new entry input text display (name and value fields)
pub fn update_new_entry_input_display(
    adding_state: Res<AddingEntryState>,
    name_input_query: Query<&Children, With<NewEntryInput>>,
    value_input_query: Query<&Children, With<NewEntryValueInput>>,
    mut text_query: Query<&mut Text>,
) {
    // Only update if adding state has changed
    if !adding_state.is_changed() {
        return;
    }

    // Update name field
    for children in name_input_query.iter() {
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if let Some(section) = text.sections.first_mut() {
                    if adding_state.new_entry_name.is_empty() {
                        section.value = "Name...".to_string();
                        section.style.color = TEXT_MUTED;
                    } else if adding_state.value_focused {
                        // Name field not focused, no cursor
                        section.value = adding_state.new_entry_name.clone();
                        section.style.color = TEXT_PRIMARY;
                    } else {
                        // Name field focused, show cursor
                        section.value = format!("{}|", adding_state.new_entry_name);
                        section.style.color = Color::srgb(0.9, 0.9, 0.5);
                    }
                }
            }
        }
    }

    // Update value field
    for children in value_input_query.iter() {
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if let Some(section) = text.sections.first_mut() {
                    if adding_state.new_entry_value.is_empty() {
                        section.value = "Value...".to_string();
                        section.style.color = TEXT_MUTED;
                    } else if !adding_state.value_focused {
                        // Value field not focused, no cursor
                        section.value = adding_state.new_entry_value.clone();
                        section.style.color = TEXT_PRIMARY;
                    } else {
                        // Value field focused, show cursor
                        section.value = format!("{}|", adding_state.new_entry_value);
                        section.style.color = Color::srgb(0.9, 0.9, 0.5);
                    }
                }
            }
        }
    }
}

/// Handle delete entry button clicks
pub fn handle_delete_click(
    interaction_query: Query<(&Interaction, &DeleteEntryButton), Changed<Interaction>>,
    mut character_data: ResMut<CharacterData>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let Some(sheet) = &mut character_data.sheet else {
                continue;
            };

            let mut deleted = false;

            match &button.group_type {
                GroupType::Skills => {
                    sheet.skills.remove(&button.entry_id);
                    deleted = true;
                }
                GroupType::SavingThrows => {
                    sheet.saving_throws.remove(&button.entry_id);
                    deleted = true;
                }
                GroupType::Attributes => {
                    // Try to delete from custom attributes (standard 6 cannot be deleted)
                    if sheet.custom_attributes.remove(&button.entry_id).is_some() {
                        deleted = true;
                    }
                }
                GroupType::Combat => {
                    // Try to delete from custom combat stats
                    if sheet.custom_combat.remove(&button.entry_id).is_some() {
                        deleted = true;
                    }
                }
                GroupType::BasicInfo => {
                    // Try to delete from custom basic info
                    if sheet.custom_basic_info.remove(&button.entry_id).is_some() {
                        deleted = true;
                    }
                }
            }

            if deleted {
                character_data.is_modified = true;
            }
        }
    }
}

// ============================================================================
// Character List and Save Systems
// ============================================================================

/// Handle character list item clicks
pub fn handle_character_list_clicks(
    interaction_query: Query<(&Interaction, &CharacterListItem), Changed<Interaction>>,
    mut character_manager: ResMut<CharacterManager>,
    mut character_data: ResMut<CharacterData>,
    db: Res<CharacterDatabase>,
) {
    for (interaction, list_item) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(char_entry) = character_manager.characters.get(list_item.index) {
                // Load from database using the ID
                match CharacterData::load_from_database(&db, char_entry.id) {
                    Ok(data) => {
                        *character_data = data;
                        character_manager.current_character_id = Some(char_entry.id);
                    }
                    Err(e) => {
                        eprintln!("Failed to load character '{}': {}", char_entry.name, e);
                    }
                }
            }
        }
    }
}

/// Handle new character button click
pub fn handle_new_character_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NewCharacterButton>)>,
    mut character_data: ResMut<CharacterData>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            *character_data = CharacterData::create_new();
        }
    }
}

/// Handle roll all stats button click - rolls d20 for all attributes
pub fn handle_roll_all_stats_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RollAllStatsButton>)>,
    mut character_data: ResMut<CharacterData>,
    mut text_input: ResMut<TextInputState>,
) {
    use rand::Rng;

    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Clear any active editing field to allow the refresh to happen
            if text_input.active_field.is_some() {
                text_input.active_field = None;
                text_input.current_text.clear();
            }
            if let Some(ref mut sheet) = character_data.sheet {
                let mut rng = rand::thread_rng();

                // Roll d20 for each core attribute
                sheet.attributes.strength = rng.gen_range(1..=20);
                sheet.attributes.dexterity = rng.gen_range(1..=20);
                sheet.attributes.constitution = rng.gen_range(1..=20);
                sheet.attributes.intelligence = rng.gen_range(1..=20);
                sheet.attributes.wisdom = rng.gen_range(1..=20);
                sheet.attributes.charisma = rng.gen_range(1..=20);

                // Roll d20 for each custom attribute
                for (_name, score) in sheet.custom_attributes.iter_mut() {
                    *score = rng.gen_range(1..=20);
                }

                // Recalculate modifiers
                sheet.modifiers.strength =
                    Attributes::calculate_modifier(sheet.attributes.strength);
                sheet.modifiers.dexterity =
                    Attributes::calculate_modifier(sheet.attributes.dexterity);
                sheet.modifiers.constitution =
                    Attributes::calculate_modifier(sheet.attributes.constitution);
                sheet.modifiers.intelligence =
                    Attributes::calculate_modifier(sheet.attributes.intelligence);
                sheet.modifiers.wisdom = Attributes::calculate_modifier(sheet.attributes.wisdom);
                sheet.modifiers.charisma =
                    Attributes::calculate_modifier(sheet.attributes.charisma);

                // Update derived stats
                sheet.combat.armor_class = 10 + sheet.modifiers.dexterity;
                sheet.combat.initiative = sheet.modifiers.dexterity;

                // Update hit points based on new constitution
                if let Some(ref mut hp) = sheet.combat.hit_points {
                    let base_hp = 10; // Fighter's d10 at level 1
                    hp.maximum = (base_hp + sheet.modifiers.constitution).max(1);
                    hp.current = hp.maximum;
                }

                // Update saving throw modifiers
                if let Some(save) = sheet.saving_throws.get_mut("strength") {
                    save.modifier = sheet.modifiers.strength;
                }
                if let Some(save) = sheet.saving_throws.get_mut("dexterity") {
                    save.modifier = sheet.modifiers.dexterity;
                }
                if let Some(save) = sheet.saving_throws.get_mut("constitution") {
                    save.modifier = sheet.modifiers.constitution;
                }
                if let Some(save) = sheet.saving_throws.get_mut("intelligence") {
                    save.modifier = sheet.modifiers.intelligence;
                }
                if let Some(save) = sheet.saving_throws.get_mut("wisdom") {
                    save.modifier = sheet.modifiers.wisdom;
                }
                if let Some(save) = sheet.saving_throws.get_mut("charisma") {
                    save.modifier = sheet.modifiers.charisma;
                }

                character_data.is_modified = true;
                character_data.needs_refresh = true;
            }
        }
    }
}

/// Handle individual attribute dice roll button click
pub fn handle_roll_attribute_click(
    interaction_query: Query<(&Interaction, &RollAttributeButton), Changed<Interaction>>,
    mut character_data: ResMut<CharacterData>,
    mut text_input: ResMut<TextInputState>,
) {
    use rand::Rng;

    for (interaction, roll_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Clear any active editing field to allow the refresh to happen
            if text_input.active_field.is_some() {
                text_input.active_field = None;
                text_input.current_text.clear();
            }

            if let Some(ref mut sheet) = character_data.sheet {
                let mut rng = rand::thread_rng();
                let new_value = rng.gen_range(1..=20);
                let attr_lower = roll_btn.attribute.to_lowercase();

                // Update the appropriate attribute
                match attr_lower.as_str() {
                    "strength" => {
                        sheet.attributes.strength = new_value;
                        sheet.modifiers.strength = Attributes::calculate_modifier(new_value);
                        if let Some(save) = sheet.saving_throws.get_mut("strength") {
                            save.modifier = sheet.modifiers.strength;
                        }
                    }
                    "dexterity" => {
                        sheet.attributes.dexterity = new_value;
                        sheet.modifiers.dexterity = Attributes::calculate_modifier(new_value);
                        sheet.combat.armor_class = 10 + sheet.modifiers.dexterity;
                        sheet.combat.initiative = sheet.modifiers.dexterity;
                        if let Some(save) = sheet.saving_throws.get_mut("dexterity") {
                            save.modifier = sheet.modifiers.dexterity;
                        }
                    }
                    "constitution" => {
                        sheet.attributes.constitution = new_value;
                        sheet.modifiers.constitution = Attributes::calculate_modifier(new_value);
                        if let Some(ref mut hp) = sheet.combat.hit_points {
                            let base_hp = 10;
                            hp.maximum = (base_hp + sheet.modifiers.constitution).max(1);
                            hp.current = hp.maximum;
                        }
                        if let Some(save) = sheet.saving_throws.get_mut("constitution") {
                            save.modifier = sheet.modifiers.constitution;
                        }
                    }
                    "intelligence" => {
                        sheet.attributes.intelligence = new_value;
                        sheet.modifiers.intelligence = Attributes::calculate_modifier(new_value);
                        if let Some(save) = sheet.saving_throws.get_mut("intelligence") {
                            save.modifier = sheet.modifiers.intelligence;
                        }
                    }
                    "wisdom" => {
                        sheet.attributes.wisdom = new_value;
                        sheet.modifiers.wisdom = Attributes::calculate_modifier(new_value);
                        if let Some(save) = sheet.saving_throws.get_mut("wisdom") {
                            save.modifier = sheet.modifiers.wisdom;
                        }
                    }
                    "charisma" => {
                        sheet.attributes.charisma = new_value;
                        sheet.modifiers.charisma = Attributes::calculate_modifier(new_value);
                        if let Some(save) = sheet.saving_throws.get_mut("charisma") {
                            save.modifier = sheet.modifiers.charisma;
                        }
                    }
                    // Custom attribute
                    _ => {
                        if let Some(score) = sheet.custom_attributes.get_mut(&roll_btn.attribute) {
                            *score = new_value;
                        }
                    }
                }

                character_data.is_modified = true;
                character_data.needs_refresh = true;
            }
        }
    }
}

/// Handle save button click
pub fn handle_save_click(
    interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SaveButton>),
    >,
    mut character_data: ResMut<CharacterData>,
    mut character_manager: ResMut<CharacterManager>,
    mut text_input: ResMut<TextInputState>,
    db: Res<CharacterDatabase>,
) {
    for (interaction, _bg) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Apply any pending text input before saving
            if let Some(active_field) = text_input.active_field.clone() {
                let current_text = text_input.current_text.clone();
                apply_field_value(
                    &mut character_data,
                    &mut text_input,
                    &active_field,
                    &current_text,
                );
            }
            // Clear the active field so we exit edit mode
            text_input.active_field = None;
            text_input.current_text.clear();

            match character_data.save_to_database(&db) {
                Ok(_) => {
                    println!(
                        "Character saved successfully (id={:?})",
                        character_data.character_id
                    );
                    // Clear modified fields tracking
                    text_input.modified_fields.clear();
                    // Refresh the character manager from database
                    character_manager.characters = db.list_characters().unwrap_or_default();
                    character_manager.current_character_id = character_data.character_id;
                    // Increment version to trigger UI rebuild (handles renames)
                    character_manager.list_version = character_manager.list_version.wrapping_add(1);
                }
                Err(e) => {
                    eprintln!("Failed to save character: {}", e);
                }
            }
        }
    }
}

/// Update save button appearance based on modified state
pub fn update_save_button_appearance(
    character_data: Res<CharacterData>,
    mut save_button_query: Query<&mut BackgroundColor, With<SaveButton>>,
) {
    if !character_data.is_changed() {
        return;
    }

    for mut bg in save_button_query.iter_mut() {
        *bg = BackgroundColor(if character_data.is_modified {
            BUTTON_BG
        } else {
            Color::srgb(0.3, 0.3, 0.35)
        });
    }
}

// ============================================================================
// UI Refresh System
// ============================================================================

/// Refresh the character stats display when character data changes
pub fn refresh_character_display(
    mut character_data: ResMut<CharacterData>,
    text_input: Res<TextInputState>,
    mut stat_value_query: Query<(&mut Text, &StatFieldValue)>,
    mut proficiency_query: Query<(&ProficiencyCheckbox, &mut BackgroundColor)>,
) {
    // Skip refresh if actively editing a field
    if text_input.active_field.is_some() {
        return;
    }

    // Refresh when needs_refresh flag is set OR when text input changed (stopped editing)
    let needs = character_data.needs_refresh;
    let input_changed = text_input.is_changed();
    let should_refresh = needs || input_changed;

    if !should_refresh {
        return;
    }

    // Clear the refresh flag
    character_data.needs_refresh = false;

    // Update all stat field values
    for (mut text, field_value) in stat_value_query.iter_mut() {
        let value = get_field_value(&character_data, &field_value.field);
        if let Some(section) = text.sections.first_mut() {
            section.value = value;
            section.style.color = TEXT_PRIMARY;
        }
    }

    // Update proficiency checkbox colors based on modifier value (checked if != 0)
    if let Some(sheet) = &character_data.sheet {
        for (checkbox, mut bg) in proficiency_query.iter_mut() {
            let has_value = match &checkbox.target {
                ProficiencyTarget::Skill(name) => {
                    if let Some(skill) = sheet.skills.get(name) {
                        skill.modifier != 0
                    } else {
                        false
                    }
                }
                ProficiencyTarget::SavingThrow(name) => {
                    if let Some(save) = sheet.saving_throws.get(name) {
                        save.modifier != 0
                    } else {
                        false
                    }
                }
            };

            *bg = BackgroundColor(if has_value {
                PROFICIENT_COLOR
            } else {
                Color::NONE
            });
        }
    }
}

/// Update the character list to show asterisk on unsaved changes
pub fn update_character_list_modified_indicator(
    character_data: Res<CharacterData>,
    character_manager: Res<CharacterManager>,
    mut text_query: Query<(&mut Text, &CharacterListItemText)>,
) {
    if !character_data.is_changed() {
        return;
    }

    for (mut text, item_text) in text_query.iter_mut() {
        // Check if this is the currently selected character
        let is_current = character_manager
            .characters
            .get(item_text.index)
            .map(|char_entry| {
                character_manager
                    .current_character_id
                    .map(|id| id == char_entry.id)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        let display_name = if is_current && character_data.is_modified {
            format!("{}*", item_text.base_name)
        } else {
            item_text.base_name.clone()
        };

        if let Some(section) = text.sections.first_mut() {
            section.value = display_name;
        }
    }
}

/// Rebuild the character stats panel when a different character is loaded or edit state changes
#[allow(clippy::too_many_arguments)]
pub fn rebuild_character_panel_on_change(
    mut commands: Commands,
    character_data: Res<CharacterData>,
    edit_state: Res<GroupEditState>,
    adding_state: Res<AddingEntryState>,
    icon_assets: Res<IconAssets>,
    stats_panel_query: Query<Entity, With<CharacterStatsPanel>>,
    screen_root_query: Query<Entity, With<CharacterScreenRoot>>,
    mut last_character_id: Local<Option<i64>>,
    mut last_edit_state: Local<std::collections::HashSet<GroupType>>,
    mut last_skills_count: Local<usize>,
    mut last_saves_count: Local<usize>,
    mut last_adding_state: Local<Option<GroupType>>,
) {
    // Check if the character ID has changed (meaning a different character was loaded)
    // OR if the edit state has changed
    // OR if the number of skills/saves has changed (items added/removed)
    // OR if the adding state has changed
    // NOTE: We do NOT rebuild when only is_modified changes - that's handled by update_save_button_appearance
    let current_id = character_data.character_id;
    let id_changed = *last_character_id != current_id;

    let edit_changed = *last_edit_state != edit_state.editing_groups;
    let adding_changed = *last_adding_state != adding_state.adding_to;

    let current_skills_count = character_data
        .sheet
        .as_ref()
        .map(|s| s.skills.len())
        .unwrap_or(0);
    let current_saves_count = character_data
        .sheet
        .as_ref()
        .map(|s| s.saving_throws.len())
        .unwrap_or(0);
    let items_changed =
        *last_skills_count != current_skills_count || *last_saves_count != current_saves_count;

    if !id_changed && !edit_changed && !items_changed && !adding_changed {
        return;
    }

    *last_character_id = current_id;
    *last_edit_state = edit_state.editing_groups.clone();
    *last_adding_state = adding_state.adding_to.clone();
    *last_skills_count = current_skills_count;
    *last_saves_count = current_saves_count;

    // Get the screen root to attach the new panel to
    let Ok(screen_root) = screen_root_query.get_single() else {
        return;
    };

    // Despawn the old stats panel
    for entity in stats_panel_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Spawn a new stats panel as a child of the screen root
    let edit_state_ref = &*edit_state;
    let adding_state_ref = &*adding_state;
    let icon_assets_ref = &*icon_assets;
    let new_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                ..default()
            },
            CharacterStatsPanel,
        ))
        .with_children(|container| {
            // Inner scrollable content
            container
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(15.0),
                            // Add padding with extra at bottom to ensure last item isn't clipped
                            padding: UiRect {
                                left: Val::Px(15.0),
                                right: Val::Px(15.0),
                                top: Val::Px(15.0),
                                bottom: Val::Px(50.0), // Extra padding at bottom
                            },
                            ..default()
                        },
                        ..default()
                    },
                    ScrollableContent,
                ))
                .with_children(|panel| {
                    if let Some(sheet) = &character_data.sheet {
                        spawn_header_row(panel, sheet, character_data.is_modified, icon_assets_ref);

                        panel
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(20.0),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|columns| {
                                columns
                                    .spawn(NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Column,
                                            min_width: Val::Px(300.0),
                                            flex_grow: 1.0,
                                            row_gap: Val::Px(15.0),
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .with_children(|col| {
                                        spawn_basic_info_group(
                                            col,
                                            sheet,
                                            edit_state_ref,
                                            adding_state_ref,
                                            icon_assets_ref,
                                        );
                                        spawn_attributes_group(
                                            col,
                                            sheet,
                                            edit_state_ref,
                                            adding_state_ref,
                                            icon_assets_ref,
                                        );
                                        spawn_combat_group(
                                            col,
                                            sheet,
                                            edit_state_ref,
                                            adding_state_ref,
                                            icon_assets_ref,
                                        );
                                    });

                                columns
                                    .spawn(NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Column,
                                            min_width: Val::Px(300.0),
                                            flex_grow: 1.0,
                                            row_gap: Val::Px(15.0),
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .with_children(|col| {
                                        spawn_saving_throws_group(
                                            col,
                                            sheet,
                                            edit_state_ref,
                                            adding_state_ref,
                                            icon_assets_ref,
                                        );
                                        spawn_skills_group(
                                            col,
                                            sheet,
                                            edit_state_ref,
                                            adding_state_ref,
                                            icon_assets_ref,
                                        );
                                    });
                            });
                    } else {
                        // No character loaded - show centered "Create First Character" button
                        panel
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_grow: 1.0,
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    min_height: Val::Px(300.0),
                                    ..default()
                                },
                                ..default()
                            })
                            .with_children(|center| {
                                // Create First Character button
                                center
                                    .spawn((
                                        ButtonBundle {
                                            style: Style {
                                                padding: UiRect::axes(Val::Px(24.0), Val::Px(16.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            background_color: BackgroundColor(BUTTON_BG),
                                            ..default()
                                        },
                                        NewCharacterButton,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            "+ Create First Character",
                                            TextStyle {
                                                font_size: 20.0,
                                                color: TEXT_PRIMARY,
                                                ..default()
                                            },
                                        ));
                                    });
                            });
                    }
                });
        })
        .id();

    // Add the new panel as a child of the screen root
    commands.entity(screen_root).add_child(new_panel);
}

/// Rebuild the character list panel when the character manager changes
#[allow(clippy::too_many_arguments)]
pub fn rebuild_character_list_on_change(
    mut commands: Commands,
    character_manager: Res<CharacterManager>,
    character_data: Res<CharacterData>,
    icon_assets: Res<IconAssets>,
    list_panel_query: Query<Entity, With<CharacterListPanel>>,
    screen_root_query: Query<Entity, With<CharacterScreenRoot>>,
    mut last_character_count: Local<usize>,
    mut last_current_id: Local<Option<i64>>,
    mut last_list_version: Local<u32>,
) {
    // Check if the number of characters, current ID, or list version changed
    let current_count = character_manager.characters.len();
    let current_id = character_manager.current_character_id;
    let current_version = character_manager.list_version;

    if *last_character_count == current_count
        && *last_current_id == current_id
        && *last_list_version == current_version
    {
        return;
    }

    *last_character_count = current_count;
    *last_current_id = current_id;
    *last_list_version = current_version;

    // Get the screen root
    let Ok(screen_root) = screen_root_query.get_single() else {
        return;
    };

    // Despawn the old list panel
    for entity in list_panel_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Spawn a new list panel
    let new_panel = spawn_character_list_panel_entity(
        &mut commands,
        &character_manager,
        &character_data,
        &icon_assets,
    );

    // Add as first child of screen root
    commands
        .entity(screen_root)
        .insert_children(0, &[new_panel]);
}

/// Spawn the character list panel as an entity (for rebuilding)
fn spawn_character_list_panel_entity(
    commands: &mut Commands,
    character_manager: &CharacterManager,
    character_data: &CharacterData,
    icon_assets: &IconAssets,
) -> Entity {
    let dice_icon = icon_assets.icons.get(&IconType::Dice).cloned();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(250.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(5.0),
                    border: UiRect::right(Val::Px(2.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
                border_color: BorderColor(Color::srgb(0.2, 0.2, 0.3)),
                ..default()
            },
            CharacterListPanel,
        ))
        .with_children(|panel| {
            // Header row with "Characters" title and Roll All button
            panel
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|header| {
                    header.spawn(TextBundle::from_section(
                        "Characters",
                        TextStyle {
                            font_size: 18.0,
                            color: TEXT_PRIMARY,
                            ..default()
                        },
                    ));

                    // Roll All Stats button (large dice icon)
                    header
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(36.0),
                                    height: Val::Px(36.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::srgb(0.6, 0.3, 0.1)),
                                ..default()
                            },
                            RollAllStatsButton,
                        ))
                        .with_children(|btn| {
                            if let Some(handle) = dice_icon.clone() {
                                btn.spawn(ImageBundle {
                                    image: UiImage::new(handle),
                                    style: Style {
                                        width: Val::Px(24.0),
                                        height: Val::Px(24.0),
                                        ..default()
                                    },
                                    ..default()
                                });
                            } else {
                                btn.spawn(TextBundle::from_section(
                                    "🎲",
                                    TextStyle {
                                        font_size: 20.0,
                                        color: TEXT_PRIMARY,
                                        ..default()
                                    },
                                ));
                            }
                        });
                });

            // New Character button
            panel
                .spawn((
                    ButtonBundle {
                        style: Style {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            margin: UiRect::vertical(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(BUTTON_BG),
                        ..default()
                    },
                    NewCharacterButton,
                ))
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "+ New Character",
                        TextStyle {
                            font_size: 14.0,
                            color: TEXT_PRIMARY,
                            ..default()
                        },
                    ));
                });

            // Character list items
            for (i, char_entry) in character_manager.characters.iter().enumerate() {
                let is_current = character_manager
                    .current_character_id
                    .map(|id| id == char_entry.id)
                    .unwrap_or(false);
                let display_name = if is_current && character_data.is_modified {
                    format!("{}*", char_entry.name)
                } else {
                    char_entry.name.clone()
                };
                let base_name = char_entry.name.clone();

                panel
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(if is_current {
                                Color::srgb(0.2, 0.3, 0.4)
                            } else {
                                FIELD_BG
                            }),
                            ..default()
                        },
                        CharacterListItem { index: i },
                    ))
                    .with_children(|item| {
                        item.spawn((
                            TextBundle::from_section(
                                display_name,
                                TextStyle {
                                    font_size: 14.0,
                                    color: TEXT_PRIMARY,
                                    ..default()
                                },
                            ),
                            CharacterListItemText {
                                index: i,
                                base_name,
                            },
                        ));
                    });
            }
        })
        .id()
}

// ============================================================================
// DnD Info Screen Setup
// ============================================================================

/// Setup the DnD Info screen with helpful documentation
pub fn setup_dnd_info_screen(mut commands: Commands, icon_assets: Res<IconAssets>) {
    // Root container (hidden by default)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(45.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
                visibility: Visibility::Hidden,
                ..default()
            },
            DndInfoScreenRoot,
        ))
        .with_children(|parent| {
            // Scrollable content wrapper
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|scroll| {
                    let icon_assets = &icon_assets;
                    // Inner content with max width for readability
                    scroll
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    max_width: Val::Px(900.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(20.0),
                                    padding: UiRect::all(Val::Px(20.0)),
                                    ..default()
                                },
                                ..default()
                            },
                            InfoScrollContent,
                        ))
                        .with_children(|content| {
                            // Main Title
                            spawn_info_heading_with_icon(content, icon_assets, IconType::Dice, "D&D Dice Rolling Guide", 32.0, Color::srgb(0.9, 0.8, 0.4));

                            spawn_info_divider(content);

                            // How Ability Scores Work Section
                            spawn_info_heading_with_icon(content, icon_assets, IconType::Info, "How Ability Scores Work", 24.0, Color::srgb(0.6, 0.8, 1.0));

                            spawn_info_paragraph(content,
                                "In D&D, each character has six ability scores: Strength (STR), Dexterity (DEX), \
                                Constitution (CON), Intelligence (INT), Wisdom (WIS), and Charisma (CHA). \
                                These scores typically range from 8 to 20 for player characters.");

                            spawn_info_heading(content, "Ability Score → Modifier Formula", 20.0, Color::srgb(0.8, 0.8, 0.9));

                            spawn_info_code_block(content, "Modifier = floor((Score - 10) / 2)");

                            spawn_info_heading(content, "Modifier Reference Table", 18.0, Color::srgb(0.7, 0.7, 0.8));

                            spawn_info_table(content, &[
                                ("Score", "Modifier"),
                                ("1", "-5"),
                                ("2-3", "-4"),
                                ("4-5", "-3"),
                                ("6-7", "-2"),
                                ("8-9", "-1"),
                                ("10-11", "+0"),
                                ("12-13", "+1"),
                                ("14-15", "+2"),
                                ("16-17", "+3"),
                                ("18-19", "+4"),
                                ("20", "+5"),
                            ]);

                            spawn_info_divider(content);

                            // Types of Rolls Section
                            spawn_info_heading_with_icon(content, icon_assets, IconType::Roll, "Types of Dice Rolls", 24.0, Color::srgb(0.6, 0.8, 1.0));

                            // Ability Checks
                            spawn_info_heading(content, "1. Ability Checks", 20.0, Color::srgb(0.5, 0.9, 0.5));
                            spawn_info_paragraph(content,
                                "Used when attempting something uncertain (breaking down a door, climbing a wall).");
                            spawn_info_code_block(content, "Roll: 1d20 + Ability Modifier");
                            spawn_info_example(content, "Example: STR 16 (+3) → Roll 1d20+3");

                            // Skill Checks
                            spawn_info_heading(content, "2. Skill Checks", 20.0, Color::srgb(0.5, 0.9, 0.5));
                            spawn_info_paragraph(content,
                                "Skills are specialized applications of abilities. If proficient, add your proficiency bonus.");
                            spawn_info_code_block(content, "Roll: 1d20 + Ability Modifier + Proficiency Bonus (if proficient)");
                            spawn_info_example(content, "Example: Stealth (DEX 14, Proficient +2) → Roll 1d20+4");

                            spawn_info_paragraph(content, "Common skills and their abilities:");
                            spawn_info_bullet_list(content, &[
                                "STR: Athletics",
                                "DEX: Acrobatics, Sleight of Hand, Stealth",
                                "INT: Arcana, History, Investigation, Nature, Religion",
                                "WIS: Animal Handling, Insight, Medicine, Perception, Survival",
                                "CHA: Deception, Intimidation, Performance, Persuasion",
                            ]);

                            // Saving Throws
                            spawn_info_heading(content, "3. Saving Throws", 20.0, Color::srgb(0.5, 0.9, 0.5));
                            spawn_info_paragraph(content,
                                "Used to resist spells, traps, poisons, and other threats. Each class is proficient in two saving throws.");
                            spawn_info_code_block(content, "Roll: 1d20 + Ability Modifier + Proficiency Bonus (if proficient)");
                            spawn_info_example(content, "Example: DEX save (DEX 14, Proficient +2) → Roll 1d20+4 to dodge a Fireball");

                            // Attack Rolls
                            spawn_info_heading(content, "4. Attack Rolls", 20.0, Color::srgb(0.5, 0.9, 0.5));
                            spawn_info_paragraph(content,
                                "Roll to hit a target. Compare result to target's Armor Class (AC).");
                            spawn_info_code_block(content, "Melee: 1d20 + STR Modifier + Proficiency Bonus\nRanged/Finesse: 1d20 + DEX Modifier + Proficiency Bonus");

                            // Damage Rolls
                            spawn_info_heading(content, "5. Damage Rolls", 20.0, Color::srgb(0.5, 0.9, 0.5));
                            spawn_info_paragraph(content,
                                "When you hit, roll the weapon's damage dice and add your ability modifier.");
                            spawn_info_code_block(content, "Damage: Weapon Dice + Ability Modifier");
                            spawn_info_example(content, "Example: Longsword (STR 16) → 1d8+3 damage");

                            spawn_info_divider(content);

                            // How to Use This App Section
                            spawn_info_heading_with_icon(content, icon_assets, IconType::Settings, "Using This Application", 24.0, Color::srgb(0.6, 0.8, 1.0));

                            spawn_info_heading(content, "Quick Roll Panel", 20.0, Color::srgb(0.9, 0.7, 0.4));
                            spawn_info_paragraph(content,
                                "On the Dice Roller tab, you'll see a Quick Rolls panel on the right showing your character's \
                                abilities, saving throws, and skills. Click any button to instantly roll 1d20 with that modifier applied!");

                            spawn_info_heading(content, "Command Line Rolling", 20.0, Color::srgb(0.9, 0.7, 0.4));
                            spawn_info_paragraph(content,
                                "Press / to enter command mode. Use these commands:");

                            spawn_info_bullet_list(content, &[
                                "--dice 2d6  or  2d6  → Roll specific dice",
                                "--checkon stealth    → Roll 1d20 + Stealth modifier",
                                "--checkon dex        → Roll 1d20 + DEX ability modifier",
                                "--checkon dex save   → Roll 1d20 + DEX saving throw",
                                "--modifier 5  or  -m 5  → Add a flat modifier",
                            ]);

                            spawn_info_example(content, "Examples:\n  2d6 --checkon athletics\n  --dice 1d20 --checkon perception\n  d20 -m 3");

                            spawn_info_heading(content, "Keyboard Shortcuts", 20.0, Color::srgb(0.9, 0.7, 0.4));
                            spawn_info_bullet_list(content, &[
                                "SPACE  → Roll the dice",
                                "R      → Reset dice",
                                "/      → Enter command mode",
                                "1-9    → Re-roll from command history",
                                "ESC    → Cancel command input",
                            ]);

                            spawn_info_heading(content, "Character Management", 20.0, Color::srgb(0.9, 0.7, 0.4));
                            spawn_info_paragraph(content,
                                "Use the Character tab to create and edit characters. Click on any value to edit it. \
                                Your character's modifiers will automatically be used in the Dice Roller. \
                                Click the + button on any group to add custom entries.");

                            spawn_info_heading(content, "Settings", 20.0, Color::srgb(0.9, 0.7, 0.4));
                            spawn_info_paragraph(content,
                                "Click the gear icon on the Dice Roller tab to customize the background color and other settings.");

                            // Footer spacer
                            spawn_info_spacer(content, 40.0);
                        });
                });
        });
}

// Helper functions for building the info screen

fn spawn_info_heading(parent: &mut ChildBuilder, text: &str, font_size: f32, color: Color) {
    parent.spawn(TextBundle::from_section(
        text,
        TextStyle {
            font_size,
            color,
            ..default()
        },
    ));
}

fn spawn_info_heading_with_icon(
    parent: &mut ChildBuilder,
    icon_assets: &IconAssets,
    icon_type: IconType,
    text: &str,
    font_size: f32,
    color: Color,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            // Icon
            if let Some(icon_handle) = icon_assets.icons.get(&icon_type) {
                row.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(font_size),
                        height: Val::Px(font_size),
                        ..default()
                    },
                    image: UiImage::new(icon_handle.clone()),
                    ..default()
                });
            }

            // Text
            row.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size,
                    color,
                    ..default()
                },
            ));
        });
}

fn spawn_info_paragraph(parent: &mut ChildBuilder, text: &str) {
    parent.spawn(
        TextBundle::from_section(
            text,
            TextStyle {
                font_size: 16.0,
                color: Color::srgb(0.85, 0.85, 0.85),
                ..default()
            },
        )
        .with_style(Style {
            max_width: Val::Px(800.0),
            ..default()
        }),
    );
}

fn spawn_info_code_block(parent: &mut ChildBuilder, text: &str) {
    parent
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        })
        .with_children(|code| {
            code.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 15.0,
                    color: Color::srgb(0.4, 0.9, 0.4),
                    ..default()
                },
            ));
        });
}

fn spawn_info_example(parent: &mut ChildBuilder, text: &str) {
    parent
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::vertical(Val::Px(5.0)),
                border: UiRect::left(Val::Px(3.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            border_color: BorderColor(Color::srgb(0.4, 0.6, 0.8)),
            ..default()
        })
        .with_children(|ex| {
            ex.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.7, 0.8, 0.9),
                    ..default()
                },
            ));
        });
}

fn spawn_info_bullet_list(parent: &mut ChildBuilder, items: &[&str]) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::left(Val::Px(20.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|list| {
            for item in items {
                list.spawn(TextBundle::from_section(
                    format!("• {}", item),
                    TextStyle {
                        font_size: 15.0,
                        color: Color::srgb(0.8, 0.8, 0.8),
                        ..default()
                    },
                ));
            }
        });
}

fn spawn_info_table(parent: &mut ChildBuilder, rows: &[(&str, &str)]) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.08, 0.08, 0.1)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        })
        .with_children(|table| {
            for (i, (col1, col2)) in rows.iter().enumerate() {
                let is_header = i == 0;
                let bg_color = if is_header {
                    Color::srgb(0.15, 0.15, 0.2)
                } else if i % 2 == 0 {
                    Color::srgb(0.1, 0.1, 0.12)
                } else {
                    Color::NONE
                };
                let text_color = if is_header {
                    Color::srgb(0.9, 0.8, 0.4)
                } else {
                    Color::srgb(0.8, 0.8, 0.8)
                };

                table
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(bg_color),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn(
                            TextBundle::from_section(
                                *col1,
                                TextStyle {
                                    font_size: 14.0,
                                    color: text_color,
                                    ..default()
                                },
                            )
                            .with_style(Style {
                                width: Val::Px(100.0),
                                ..default()
                            }),
                        );
                        row.spawn(TextBundle::from_section(
                            *col2,
                            TextStyle {
                                font_size: 14.0,
                                color: text_color,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn spawn_info_divider(parent: &mut ChildBuilder) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(15.0)),
            ..default()
        },
        background_color: BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
        ..default()
    });
}

fn spawn_info_spacer(parent: &mut ChildBuilder, height: f32) {
    parent.spawn(NodeBundle {
        style: Style {
            height: Val::Px(height),
            ..default()
        },
        ..default()
    });
}

// ============================================================================
// Initialization System
// ============================================================================

/// Initialize character manager and database by scanning current directory
pub fn init_character_manager(mut commands: Commands) {
    // Initialize the database
    let db = match CharacterDatabase::open() {
        Ok(db) => db,
        Err(e) => {
            eprintln!(
                "Failed to open character database: {}. Using in-memory database.",
                e
            );
            CharacterDatabase::open_in_memory().expect("Failed to create in-memory database")
        }
    };

    // Migrate any existing JSON files to the database
    let current_dir = std::env::current_dir().unwrap_or_default();
    let json_characters = CharacterManager::scan_directory(&current_dir);

    for entry in &json_characters {
        // Check if this character already exists in the database (by name)
        if db.name_exists(&entry.name, None).unwrap_or(false) {
            continue; // Skip, already migrated
        }

        // Try to load and import the JSON file
        match db.import_from_file(&entry.path) {
            Ok(id) => {
                println!(
                    "Migrated character '{}' from JSON to database (id={})",
                    entry.name, id
                );
            }
            Err(e) => {
                eprintln!("Failed to migrate '{}': {}", entry.name, e);
            }
        }
    }

    // Get character list from database
    let characters = db.list_characters().unwrap_or_default();

    commands.insert_resource(db);

    commands.insert_resource(CharacterManager {
        characters: characters.clone(),
        current_character_id: None,
        list_version: 0,
        available_characters: json_characters, // Keep for reference
        current_character_path: None,
    });

    commands.insert_resource(TextInputState::default());
}
