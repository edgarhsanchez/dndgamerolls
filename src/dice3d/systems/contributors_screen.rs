//! Contributors screen system
//!
//! This module displays GitHub contributors loaded from the bundled contributors asset.

use super::avatar_loader::AvatarImage;
use crate::dice3d::types::{
    ContributorCard, ContributorsData, ContributorsList, ContributorsScreenRoot, ContributorsState,
};
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_material_ui::icons::MaterialIconFont;
use bevy_material_ui::prelude::*;

/// Initialize contributors data
pub fn init_contributors(mut commands: Commands) {
    let data = ContributorsData::load();
    commands.insert_resource(ContributorsState { data, loaded: true });
}

/// Setup the Contributors screen
pub fn setup_contributors_screen(
    mut commands: Commands,
    contributors_state: Res<ContributorsState>,
    icon_font: Res<MaterialIconFont>,
    theme: Option<Res<MaterialTheme>>,
) {
    let theme = theme.map(|t| t.clone()).unwrap_or_default();

    // Root container (hidden by default)
    commands
        .spawn((
            Node {
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
            BackgroundColor(theme.surface),
            Visibility::Hidden,
            ContributorsScreenRoot,
        ))
        .with_children(|parent| {
            // Header section
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|header| {
                    // Title with icon
                    header
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // Icon
                            let icon = MaterialIcon::from_name("groups")
                                .or_else(|| MaterialIcon::from_name("group"))
                                .unwrap_or_else(MaterialIcon::info);
                            row.spawn((
                                Text::new(icon.as_str()),
                                TextFont {
                                    font: icon_font.0.clone(),
                                    font_size: 36.0,
                                    ..default()
                                },
                                TextColor(theme.primary),
                            ));

                            row.spawn((
                                Text::new("Contributors"),
                                TextFont {
                                    font_size: 32.0,
                                    ..default()
                                },
                                TextColor(theme.on_surface),
                            ));
                        });

                    // Subtitle with repo info
                    header.spawn((
                        Text::new(format!("Thank you to everyone who has contributed to {}", contributors_state.data.repository)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));

                    // Last updated
                    header.spawn((
                        Text::new(format!("Last updated: {}", contributors_state.data.last_updated)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant.with_alpha(0.85)),
                        Node {
                            margin: UiRect::top(Val::Px(5.0)),
                            ..default()
                        },
                    ));

                    // Version info
                    header.spawn((
                        Text::new(format!("Version {}", env!("CARGO_PKG_VERSION"))),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.primary),
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                });

            // Divider
            parent
                .spawn(Node {
                    margin: UiRect::vertical(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|divider| {
                    divider.spawn(horizontal_divider(&theme));
                });

            // Scrollable contributors list
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    ContributorsList,
                ))
                .with_children(|list| {
                    // Contributors grid/list
                    list.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(15.0),
                        column_gap: Val::Px(15.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    })
                    .with_children(|grid| {
                        // Spawn a card for each contributor
                        for (index, contributor) in contributors_state.data.contributors.iter().enumerate() {
                            spawn_contributor_card(
                                grid,
                                &theme,
                                index,
                                &contributor.login,
                                contributor.display_name(),
                                &contributor.avatar_url,
                                contributor.contributions,
                                contributor.role.as_deref(),
                            );
                        }

                        // If no contributors, show message
                        if contributors_state.data.contributors.is_empty() {
                            grid.spawn((
                                Text::new("No contributors data available.\nContributors will be updated at release time."),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(theme.on_surface_variant),
                            ));
                        }
                    });
                });

            // Footer with GitHub link and copyright
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::top(Val::Px(20.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|footer| {
                    footer.spawn((
                        Text::new("Want to contribute? Visit github.com/edgarhsanchez/dndgamerolls"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(theme.primary),
                    ));

                    // Copyright notice
                    footer.spawn((
                        Text::new("Copyright (c) 2025 Edgar Sanchez. All rights reserved."),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));
                });
        });
}

/// Spawn a contributor card
fn spawn_contributor_card(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    index: usize,
    login: &str,
    display_name: &str,
    avatar_url: &str,
    contributions: u32,
    role: Option<&str>,
) {
    parent
        .spawn((
            CardBuilder::new()
                .outlined()
                .width(Val::Px(200.0))
                .padding(15.0)
                .build(theme),
            ContributorCard { index },
        ))
        .with_children(|card| {
            // Inner layout node (avoids duplicate `Node` in the card bundle)
            card.spawn(Node {
                min_height: Val::Px(120.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|content| {
                // Avatar container (circle)
                content
                    .spawn((
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(10.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(theme.secondary_container),
                        BorderRadius::all(Val::Px(32.0)),
                    ))
                    .with_children(|avatar_container| {
                        if !avatar_url.is_empty() {
                            // Spawn image that will be loaded from URL
                            avatar_container.spawn((
                                ImageNode::default(),
                                Node {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    ..default()
                                },
                                // Start with transparent, will be replaced when loaded
                                BackgroundColor(Color::NONE),
                                AvatarImage {
                                    url: avatar_url.to_string(),
                                    loaded: false,
                                    failed: false,
                                },
                            ));
                        } else {
                            // No avatar URL, just show initial
                            let initial = display_name
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_uppercase()
                                .to_string();
                            avatar_container.spawn((
                                Text::new(initial),
                                TextFont {
                                    font_size: 28.0,
                                    ..default()
                                },
                                TextColor(theme.on_secondary_container),
                            ));
                        }
                    });

                // Display name
                content.spawn((
                    Text::new(display_name),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                ));

                // Username (if different from display name)
                if display_name != login {
                    content.spawn((
                        Text::new(format!("@{}", login)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));
                }

                // Role badge (if any)
                if let Some(role_text) = role {
                    content.spawn((
                        Text::new(role_text),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(theme.tertiary),
                        Node {
                            margin: UiRect::top(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                }

                // Contributions count
                content.spawn((
                    Text::new(format!("{} contributions", contributions)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                    Node {
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                ));
            });
        });
}
