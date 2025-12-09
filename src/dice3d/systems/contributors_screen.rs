//! Contributors screen system
//!
//! This module displays GitHub contributors loaded from contributors.json

use super::avatar_loader::AvatarImage;
use crate::dice3d::types::{
    ContributorCard, ContributorsData, ContributorsList, ContributorsScreenRoot, ContributorsState,
    IconAssets, IconType,
};
use bevy::prelude::*;

/// Initialize contributors data
pub fn init_contributors(mut commands: Commands) {
    let data = ContributorsData::load();
    commands.insert_resource(ContributorsState { data, loaded: true });
}

/// Setup the Contributors screen
pub fn setup_contributors_screen(
    mut commands: Commands,
    contributors_state: Res<ContributorsState>,
    icon_assets: Res<IconAssets>,
) {
    let header_color = Color::srgb(0.9, 0.8, 0.4);
    let text_color = Color::srgb(0.85, 0.85, 0.9);
    let subtext_color = Color::srgb(0.6, 0.6, 0.7);
    let card_bg = Color::srgb(0.12, 0.12, 0.18);
    let accent_color = Color::srgb(0.4, 0.7, 1.0);

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
            ContributorsScreenRoot,
        ))
        .with_children(|parent| {
            // Header section
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|header| {
                    // Title with icon
                    header
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(12.0),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|row| {
                            // Icon
                            if let Some(icon_handle) = icon_assets.icons.get(&IconType::Character) {
                                row.spawn(ImageBundle {
                                    image: UiImage::new(icon_handle.clone()),
                                    style: Style {
                                        width: Val::Px(36.0),
                                        height: Val::Px(36.0),
                                        ..default()
                                    },
                                    ..default()
                                });
                            }

                            row.spawn(TextBundle::from_section(
                                "Contributors",
                                TextStyle {
                                    font_size: 32.0,
                                    color: header_color,
                                    ..default()
                                },
                            ));
                        });

                    // Subtitle with repo info
                    header.spawn(TextBundle::from_section(
                        format!("Thank you to everyone who has contributed to {}", contributors_state.data.repository),
                        TextStyle {
                            font_size: 16.0,
                            color: subtext_color,
                            ..default()
                        },
                    ));

                    // Last updated
                    header.spawn(
                        TextBundle::from_section(
                            format!("Last updated: {}", contributors_state.data.last_updated),
                            TextStyle {
                                font_size: 12.0,
                                color: Color::srgb(0.5, 0.5, 0.55),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(5.0)),
                            ..default()
                        }),
                    );

                    // Version info
                    header.spawn(
                        TextBundle::from_section(
                            format!("Version {}", env!("CARGO_PKG_VERSION")),
                            TextStyle {
                                font_size: 14.0,
                                color: accent_color,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        }),
                    );
                });

            // Divider
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(10.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.3, 0.3, 0.35)),
                ..default()
            });

            // Scrollable contributors list
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::clip_y(),
                            ..default()
                        },
                        ..default()
                    },
                    ContributorsList,
                ))
                .with_children(|list| {
                    // Contributors grid/list
                    list.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::Center,
                            row_gap: Val::Px(15.0),
                            column_gap: Val::Px(15.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|grid| {
                        // Spawn a card for each contributor
                        for (index, contributor) in contributors_state.data.contributors.iter().enumerate() {
                            spawn_contributor_card(
                                grid,
                                index,
                                &contributor.login,
                                contributor.display_name(),
                                &contributor.avatar_url,
                                contributor.contributions,
                                contributor.role.as_deref(),
                                card_bg,
                                text_color,
                                subtext_color,
                                accent_color,
                            );
                        }

                        // If no contributors, show message
                        if contributors_state.data.contributors.is_empty() {
                            grid.spawn(TextBundle::from_section(
                                "No contributors data available.\nContributors will be updated at release time.",
                                TextStyle {
                                    font_size: 16.0,
                                    color: subtext_color,
                                    ..default()
                                },
                            ));
                        }
                    });
                });

            // Footer with GitHub link and copyright
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::top(Val::Px(20.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|footer| {
                    footer.spawn(TextBundle::from_section(
                        "Want to contribute? Visit github.com/edgarhsanchez/dndgamerolls",
                        TextStyle {
                            font_size: 14.0,
                            color: accent_color,
                            ..default()
                        },
                    ));

                    // Copyright notice
                    footer.spawn(TextBundle::from_section(
                        "Copyright (c) 2025 Edgar Sanchez. All rights reserved.",
                        TextStyle {
                            font_size: 12.0,
                            color: subtext_color,
                            ..default()
                        },
                    ));
                });
        });
}

/// Spawn a contributor card
fn spawn_contributor_card(
    parent: &mut ChildBuilder,
    index: usize,
    login: &str,
    display_name: &str,
    avatar_url: &str,
    contributions: u32,
    role: Option<&str>,
    card_bg: Color,
    text_color: Color,
    subtext_color: Color,
    accent_color: Color,
) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    min_height: Val::Px(120.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(15.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: BackgroundColor(card_bg),
                border_color: BorderColor(Color::srgb(0.25, 0.25, 0.3)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            ContributorCard { index },
        ))
        .with_children(|card| {
            // Avatar container (circle)
            card.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: BackgroundColor(accent_color.with_alpha(0.3)),
                border_radius: BorderRadius::all(Val::Px(32.0)),
                ..default()
            })
            .with_children(|avatar_container| {
                if !avatar_url.is_empty() {
                    // Spawn image that will be loaded from URL
                    avatar_container.spawn((
                        ImageBundle {
                            style: Style {
                                width: Val::Px(64.0),
                                height: Val::Px(64.0),
                                ..default()
                            },
                            // Start with transparent, will be replaced when loaded
                            background_color: BackgroundColor(Color::NONE),
                            ..default()
                        },
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
                    avatar_container.spawn(TextBundle::from_section(
                        initial,
                        TextStyle {
                            font_size: 28.0,
                            color: accent_color,
                            ..default()
                        },
                    ));
                }
            });

            // Display name
            card.spawn(TextBundle::from_section(
                display_name,
                TextStyle {
                    font_size: 16.0,
                    color: text_color,
                    ..default()
                },
            ));

            // Username (if different from display name)
            if display_name != login {
                card.spawn(TextBundle::from_section(
                    format!("@{}", login),
                    TextStyle {
                        font_size: 12.0,
                        color: subtext_color,
                        ..default()
                    },
                ));
            }

            // Role badge (if any)
            if let Some(role_text) = role {
                card.spawn(
                    TextBundle::from_section(
                        role_text,
                        TextStyle {
                            font_size: 11.0,
                            color: Color::srgb(0.4, 0.8, 0.4),
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::top(Val::Px(5.0)),
                        ..default()
                    }),
                );
            }

            // Contributions count
            card.spawn(
                TextBundle::from_section(
                    format!("{} contributions", contributions),
                    TextStyle {
                        font_size: 12.0,
                        color: subtext_color,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                }),
            );
        });
}
