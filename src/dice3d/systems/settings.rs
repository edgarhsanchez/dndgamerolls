//! Settings UI systems
//!
//! This module contains systems for the settings button, modal, and color picker.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::ui::{ComputedUiTargetCamera, UiGlobalTransform};

use bevy::window::PrimaryWindow;
use bevy_material_ui::prelude::*;
use bevy_material_ui::theme::ThemeMode;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use crate::dice3d::types::*;
use bevy_material_ui::prelude::SwitchChangeEvent;

use crate::dice3d::fx_image_gen::generate_custom_fx_textures;

use super::settings_tabs;

const SETTINGS_DIALOG_WIDTH: f32 = 780.0;
const SETTINGS_DIALOG_HEIGHT: f32 = 720.0;

const DICE_SCALE_PREVIEW_LAYER: u8 = 31;
const DICE_SCALE_PREVIEW_WIDTH: u32 = 360;
const DICE_SCALE_PREVIEW_HEIGHT: u32 = 220;

#[derive(Resource, Clone)]
pub struct DiceScalePreviewRenderTarget {
    pub image: Handle<Image>,
}

#[derive(Resource, Clone)]
pub struct SettingsUiImages {
    pub blank: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct DiceScalePreviewScene {
    pub root: Option<Entity>,
    pub camera: Option<Entity>,
    pub light: Option<Entity>,
    pub d4: Option<Entity>,
    pub d6: Option<Entity>,
    pub d8: Option<Entity>,
    pub d10: Option<Entity>,
    pub d12: Option<Entity>,
    pub d20: Option<Entity>,
}

#[derive(Component, Clone, Copy)]
pub struct DiceScalePreviewDie {
    pub die_type: DiceType,
}

pub fn init_dice_scale_preview_render_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: DICE_SCALE_PREVIEW_WIDTH,
        height: DICE_SCALE_PREVIEW_HEIGHT,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("dice_scale_preview_render_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let handle = images.add(image);
    commands.insert_resource(DiceScalePreviewRenderTarget { image: handle });
    commands.insert_resource(DiceScalePreviewScene::default());
}

pub fn init_settings_ui_images(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("settings_ui_blank_image"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);
    image.data = Some(vec![0, 0, 0, 0]);

    commands.insert_resource(SettingsUiImages {
        blank: images.add(image),
    });
}

pub fn manage_dice_scale_preview_scene(
    mut commands: Commands,
    settings_state: Res<SettingsState>,
    preview_target: Option<Res<DiceScalePreviewRenderTarget>>,
    mut preview_scene: ResMut<DiceScalePreviewScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(preview_target) = preview_target else {
        return;
    };

    let modal_open = settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings;

    // Despawn if the modal is closed.
    if !modal_open {
        if let Some(e) = preview_scene.d4.take() {
            commands.entity(e).despawn();
        }
        if let Some(e) = preview_scene.d6.take() {
            commands.entity(e).despawn();
        }
        if let Some(e) = preview_scene.d8.take() {
            commands.entity(e).despawn();
        }
        if let Some(e) = preview_scene.d10.take() {
            commands.entity(e).despawn();
        }
        if let Some(e) = preview_scene.d12.take() {
            commands.entity(e).despawn();
        }
        if let Some(e) = preview_scene.d20.take() {
            commands.entity(e).despawn();
        }
        if let Some(e) = preview_scene.camera.take() {
            commands.entity(e).despawn();
        }
        if let Some(e) = preview_scene.light.take() {
            commands.entity(e).despawn();
        }
        if let Some(root) = preview_scene.root.take() {
            commands.entity(root).despawn();
        }
        return;
    }

    if preview_scene.root.is_some() {
        return;
    }

    let preview_layer = RenderLayers::layer(DICE_SCALE_PREVIEW_LAYER as usize);

    // Root
    let root = commands
        .spawn((
            Transform::default(),
            Visibility::Visible,
            preview_layer.clone(),
        ))
        .id();

    // Spawn preview content as children of the root.
    let mut camera_id: Option<Entity> = None;
    let mut light_id: Option<Entity> = None;
    let mut d4_id: Option<Entity> = None;
    let mut d6_id: Option<Entity> = None;
    let mut d8_id: Option<Entity> = None;
    let mut d10_id: Option<Entity> = None;
    let mut d12_id: Option<Entity> = None;
    let mut d20_id: Option<Entity> = None;

    commands.entity(root).with_children(|parent| {
        // Light
        let light = parent
            .spawn((
                PointLight {
                    intensity: 2500.0,
                    range: 50.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_xyz(6.0, 8.0, 6.0),
                preview_layer.clone(),
            ))
            .id();
        light_id = Some(light);

        // Camera (render to texture)
        let camera = parent
            .spawn((
                Camera3d::default(),
                Camera {
                    target: RenderTarget::Image(preview_target.image.clone().into()),
                    ..default()
                },
                Transform::from_xyz(0.0, 5.5, 10.5).looking_at(Vec3::new(0.0, 1.3, 0.0), Vec3::Y),
                preview_layer.clone(),
            ))
            .id();
        camera_id = Some(camera);

        let spacing = 2.4;

        let mut spawn_die = |die_type: DiceType, x: f32| -> Entity {
            let (mesh, _collider, _normals) =
                crate::dice3d::meshes::create_die_mesh_and_collider(die_type);
            let mesh_handle = meshes.add(mesh);
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.85, 0.9, 1.0),
                alpha_mode: AlphaMode::Opaque,
                perceptual_roughness: 0.25,
                metallic: 0.0,
                reflectance: 0.2,
                ..default()
            });

            parent
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::from_xyz(x, 1.0, 0.0).with_rotation(Quat::from_euler(
                        EulerRot::XYZ,
                        0.6,
                        0.7,
                        0.0,
                    )),
                    DiceScalePreviewDie { die_type },
                    preview_layer.clone(),
                ))
                .id()
        };

        d4_id = Some(spawn_die(DiceType::D4, -2.5 * spacing));
        d6_id = Some(spawn_die(DiceType::D6, -1.5 * spacing));
        d8_id = Some(spawn_die(DiceType::D8, -0.5 * spacing));
        d10_id = Some(spawn_die(DiceType::D10, 0.5 * spacing));
        d12_id = Some(spawn_die(DiceType::D12, 1.5 * spacing));
        d20_id = Some(spawn_die(DiceType::D20, 2.5 * spacing));
    });

    preview_scene.root = Some(root);
    preview_scene.light = light_id;
    preview_scene.camera = camera_id;
    preview_scene.d4 = d4_id;
    preview_scene.d6 = d6_id;
    preview_scene.d8 = d8_id;
    preview_scene.d10 = d10_id;
    preview_scene.d12 = d12_id;
    preview_scene.d20 = d20_id;
}

pub fn sync_dice_scale_preview_dice(
    settings_state: Res<SettingsState>,
    preview_scene: Res<DiceScalePreviewScene>,
    mut dice_query: Query<(&DiceScalePreviewDie, &mut Transform)>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    if preview_scene.root.is_none() {
        return;
    }

    // Live-preview uses the editing values (not persisted) so users see changes while dragging.
    let scales = &settings_state.editing_dice_scales;
    for (die, mut transform) in dice_query.iter_mut() {
        let s = scales.scale_for(die.die_type) * die.die_type.uniform_size_scale_factor();
        transform.scale = Vec3::splat(s);
    }
}

/// Persist settings changes to SQLite.
///
/// Many UI interactions update settings continuously (dragging panels, curve edits).
/// Instead of writing on every change, systems should set `SettingsState.is_modified = true`.
/// This system flushes once per frame.
pub fn persist_settings_to_db(
    mut settings_state: ResMut<SettingsState>,
    db: Option<Res<CharacterDatabase>>,
) {
    if !settings_state.is_modified {
        return;
    }

    let Some(db) = db else {
        return;
    };

    match settings_state.settings.save_to_db(&db) {
        Ok(()) => settings_state.is_modified = false,
        Err(e) => warn!("Failed to persist settings to SurrealDB: {}", e),
    }
}

/// Load persisted settings after the database resource has been initialized.
///
/// This avoids opening the on-disk datastore twice at startup (which can fail on
/// Windows/MSIX due to file locking or permission constraints).
pub fn load_settings_state_from_db(
    mut settings_state: ResMut<SettingsState>,
    db: Option<Res<CharacterDatabase>>,
    mut theme: ResMut<MaterialTheme>,
) {
    let Some(db) = db else {
        warn!("No CharacterDatabase resource; using default settings");
        return;
    };

    if db.db_path.as_os_str().is_empty() {
        warn!("CharacterDatabase is in-memory; settings will not persist across restarts");
        return;
    }

    match AppSettings::load_from_db(&db) {
        Ok(Some(loaded)) => {
            info!(
                "Loaded settings from SurrealDB at {:?} (background={})",
                db.db_path,
                loaded.background_color.to_hex()
            );

            settings_state.settings = loaded.clone();
            settings_state.is_modified = false;

            settings_state.character_sheet_editing_die = loaded.character_sheet_default_die;
            settings_state.quick_roll_editing_die = loaded.quick_roll_default_die;
            settings_state.default_roll_uses_shake_editing = loaded.default_roll_uses_shake;

            settings_state.editing_color = loaded.background_color.clone();
            settings_state.editing_highlight_color = loaded.dice_box_highlight_color.clone();
            settings_state.editing_shake_config = loaded.shake_config.to_runtime();
            settings_state.last_saved_shake_config = loaded.shake_config.clone();

            // Stage custom dice FX + library into editable state.
            init_custom_dice_fx_editing_from_settings(&mut settings_state);

            settings_state.color_input_text.clear();
            settings_state.highlight_input_text.clear();

            // Apply theme override (if any) before UI setup.
            apply_theme_override(&settings_state.settings, &mut theme);
        }
        Ok(None) => {
            info!(
                "No persisted settings found in SurrealDB at {:?}; using defaults",
                db.db_path
            );
        }
        Err(e) => {
            warn!(
                "Failed to load settings from SurrealDB at {:?}: {}; using defaults",
                db.db_path, e
            );
        }
    }
}

fn init_custom_dice_fx_editing_from_settings(settings_state: &mut SettingsState) {
    // Start from persisted library.
    settings_state.editing_custom_dice_fx_library =
        settings_state.settings.custom_dice_fx_library.clone();
    if settings_state.editing_custom_dice_fx_library.is_empty() {
        settings_state
            .editing_custom_dice_fx_library
            .push(CustomDiceFxSetting::default());
    }

    // Choose the active/selected entry.
    if let Some(active) = settings_state.settings.custom_dice_fx.clone() {
        let idx = active
            .source_image_path
            .as_deref()
            .and_then(|p| {
                settings_state
                    .editing_custom_dice_fx_library
                    .iter()
                    .position(|e| e.source_image_path.as_deref() == Some(p))
            })
            .or_else(|| {
                settings_state
                    .editing_custom_dice_fx_library
                    .iter()
                    .position(|e| e == &active)
            })
            .unwrap_or_else(|| {
                settings_state
                    .editing_custom_dice_fx_library
                    .push(active.clone());
                settings_state.editing_custom_dice_fx_library.len() - 1
            });

        settings_state.selected_custom_dice_fx_library_index = Some(idx);
        settings_state.editing_custom_dice_fx = active;
    } else {
        settings_state.selected_custom_dice_fx_library_index = Some(0);
        settings_state.editing_custom_dice_fx =
            settings_state.editing_custom_dice_fx_library[0].clone();
    }

    settings_state.dice_fx_trigger_value_input_text = settings_state
        .editing_custom_dice_fx
        .trigger_value
        .to_string();
    settings_state.dice_fx_duration_input_text = format!(
        "{:.3}",
        settings_state
            .editing_custom_dice_fx
            .duration_seconds
            .max(0.0)
    );
}

fn apply_theme_override(settings: &AppSettings, theme: &mut MaterialTheme) {
    let mode = theme.mode;

    let default_for_mode = || match mode {
        ThemeMode::Dark => MaterialTheme::dark(),
        ThemeMode::Light => MaterialTheme::light(),
    };

    let Some(seed_hex) = settings.theme_seed_hex.as_deref() else {
        *theme = default_for_mode();
        return;
    };

    let Some(mut parsed) = ColorSetting::parse(seed_hex) else {
        *theme = default_for_mode();
        return;
    };

    parsed.a = 1.0;
    *theme = MaterialTheme::from_seed(parsed.to_color(), mode);
}

/// Spawn the settings (gear) icon button in the dice roller view.
pub fn spawn_settings_button(
    commands: &mut Commands,
    theme: &MaterialTheme,
    icon_font: Handle<Font>,
) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                // Keep below the app-level tab bar which sits at top=0.
                top: Val::Px(TAB_HEIGHT_SECONDARY + 12.0),
                width: Val::Px(ICON_BUTTON_SIZE),
                height: Val::Px(ICON_BUTTON_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(200),
            DiceRollerRoot,
        ))
        .with_children(|slot| {
            slot.spawn((
                IconButtonBuilder::new("settings").standard().build(theme),
                TooltipTrigger::new("Settings").top(),
                SettingsButton,
            ))
            .with_children(|b| {
                let icon = MaterialIcon::from_name("settings").unwrap_or_else(MaterialIcon::search);
                b.spawn((
                    Text::new(icon.as_str()),
                    TextFont {
                        font: icon_font,
                        font_size: ICON_SIZE,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));
            });
        })
        .id()
}

fn spawn_settings_modal(
    commands: &mut Commands,
    theme: &MaterialTheme,
    settings_state: &SettingsState,
    shake_config: &ContainerShakeConfig,
    dice_scale_preview_image: Option<Handle<Image>>,
    blank_image: Option<Handle<Image>>,
) {
    let options = [
        DiceTypeSetting::D4,
        DiceTypeSetting::D6,
        DiceTypeSetting::D8,
        DiceTypeSetting::D10,
        DiceTypeSetting::D12,
        DiceTypeSetting::D20,
    ];

    let selected_index = options
        .iter()
        .position(|d| *d == settings_state.quick_roll_editing_die)
        .unwrap_or(5);

    let select_options: Vec<SelectOption> = options
        .iter()
        .map(|d| SelectOption::new(d.label()).value(d.label()))
        .collect();

    // Custom dialog bundle so we can enforce a fixed size (DialogBuilder clamps width).
    let dialog = MaterialDialog::new()
        .title("Settings")
        .open(true)
        .modal(true);

    let dialog_bg = dialog.surface_color(theme);

    let dialog_entity = commands
        .spawn((
            dialog,
            Node {
                display: Display::None, // synced by DialogPlugin
                position_type: PositionType::Absolute,
                width: Val::Px(SETTINGS_DIALOG_WIDTH),
                height: Val::Px(SETTINGS_DIALOG_HEIGHT),
                min_width: Val::Px(SETTINGS_DIALOG_WIDTH),
                max_width: Val::Px(SETTINGS_DIALOG_WIDTH),
                min_height: Val::Px(SETTINGS_DIALOG_HEIGHT),
                max_height: Val::Px(SETTINGS_DIALOG_HEIGHT),
                padding: UiRect::all(Val::Px(Spacing::EXTRA_LARGE)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(dialog_bg),
            BorderRadius::all(Val::Px(CornerRadius::EXTRA_LARGE)),
            BoxShadow::default(),
            SettingsModal,
        ))
        .id();

    let scrim_entity = commands
        .spawn((
            create_dialog_scrim_for(theme, dialog_entity, true),
            SettingsModalOverlay,
        ))
        .id();

    commands.entity(scrim_entity).add_child(dialog_entity);

    commands.entity(dialog_entity).with_children(|dialog| {
        dialog
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                min_height: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|content| {
                let editing_color = &settings_state.editing_color;
                let editing_highlight_color = &settings_state.editing_highlight_color;

                content.spawn((
                    Text::new("Settings"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                ));

                // Tabs: Dice / Colors / Shake Curve / Layout
                let mut tabs_cmd = content.spawn((
                    MaterialTabs::new()
                        .with_variant(TabVariant::Secondary)
                        .selected(0),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(TAB_HEIGHT_SECONDARY),
                        min_height: Val::Px(TAB_HEIGHT_SECONDARY),
                        max_height: Val::Px(TAB_HEIGHT_SECONDARY),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    BackgroundColor(theme.surface),
                ));
                let tabs_entity = tabs_cmd.id();
                tabs_cmd.with_children(|tabs| {
                    fn spawn_tab_label(
                        t: &mut ChildSpawnerCommands,
                        theme: &MaterialTheme,
                        label: &str,
                    ) {
                        t.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));
                    }

                    tabs.spawn((
                        MaterialTab::new(0, "Dice").selected(true),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            height: Val::Percent(100.0),
                            min_height: Val::Px(TAB_HEIGHT_SECONDARY),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Dice"));

                    tabs.spawn((
                        MaterialTab::new(1, "Colors"),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            height: Val::Percent(100.0),
                            min_height: Val::Px(TAB_HEIGHT_SECONDARY),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Colors"));

                    tabs.spawn((
                        MaterialTab::new(2, "Shake Curve"),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            height: Val::Percent(100.0),
                            min_height: Val::Px(TAB_HEIGHT_SECONDARY),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Shake Curve"));

                    tabs.spawn((
                        MaterialTab::new(3, "Layout"),
                        Button,
                        Node {
                            flex_grow: 1.0,
                            height: Val::Percent(100.0),
                            min_height: Val::Px(TAB_HEIGHT_SECONDARY),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|t| spawn_tab_label(t, theme, "Layout"));
                });

                // Scrollable content area. Each tab is a scroll container.
                content
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(0.0),
                        overflow: Overflow::clip(),
                        ..default()
                    })
                    .with_children(|tab_area| {
                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            0,
                            true,
                            |tab| {
                                settings_tabs::dice::build_dice_tab(
                                    tab,
                                    theme,
                                    select_options.clone(),
                                    selected_index,
                                    settings_state.default_roll_uses_shake_editing,
                                    &settings_state.editing_dice_scales,
                                    dice_scale_preview_image.clone(),
                                    blank_image.clone(),
                                    settings_state,
                                );
                            },
                        );

                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            1,
                            false,
                            |tab| {
                                settings_tabs::colors::build_colors_tab(
                                    tab,
                                    theme,
                                    editing_color,
                                    editing_highlight_color,
                                    &settings_state.theme_seed_input_text,
                                    &settings_state.settings.recent_theme_seeds,
                                );
                            },
                        );

                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            2,
                            false,
                            |tab| {
                                settings_tabs::shake_curve::build_shake_curve_tab(
                                    tab,
                                    theme,
                                    settings_state,
                                    shake_config,
                                );
                            },
                        );

                        settings_tabs::spawn_scrollable_tab_content(
                            tab_area,
                            tabs_entity,
                            3,
                            false,
                            |tab| {
                                settings_tabs::layout::build_layout_tab(tab, theme);
                            },
                        );
                    });

                // Buttons row
                content
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(10.0),
                        width: Val::Percent(100.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        // Cancel button
                        buttons
                            .spawn(Node {
                                width: Val::Px(100.0),
                                height: Val::Px(36.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                slot.spawn((
                                    MaterialButtonBuilder::new("Cancel").outlined().build(theme),
                                    SettingsCancelButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Cancel"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(theme.primary),
                                        ButtonLabel,
                                    ));
                                });
                            });

                        // OK button
                        buttons
                            .spawn(Node {
                                width: Val::Px(100.0),
                                height: Val::Px(36.0),
                                ..default()
                            })
                            .with_children(|slot| {
                                slot.spawn((
                                    MaterialButtonBuilder::new("OK").filled().build(theme),
                                    SettingsOkButton,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("OK"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(theme.on_primary),
                                        ButtonLabel,
                                    ));
                                });
                            });
                    });
            });
    });
}

/// Helper to spawn a color slider row
pub(crate) fn spawn_color_slider(
    parent: &mut ChildSpawnerCommands,
    component: ColorComponent,
    label: &str,
    value: f32,
    track_color: Color,
    theme: &MaterialTheme,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            height: Val::Px(30.0),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(track_color),
            ));

            // Slider
            row.spawn(Node {
                width: Val::Px(180.0),
                height: Val::Px(30.0),
                ..default()
            })
            .with_children(|slot| {
                let slider = MaterialSlider::new(0.0, 1.0)
                    .with_value(value)
                    .track_height(6.0)
                    .thumb_radius(8.0);
                spawn_slider_control_with(slot, theme, slider, ColorSlider { component });
            });

            // Value label
            row.spawn((
                Text::new(format!("{:.2}", value)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
                ColorValueLabel { component },
            ));
        });
}

/// Helper to spawn a dice scale slider row.
pub(crate) fn spawn_dice_scale_slider(
    parent: &mut ChildSpawnerCommands,
    die_type: DiceType,
    label: &str,
    value: f32,
    theme: &MaterialTheme,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            height: Val::Px(30.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));

            row.spawn(Node {
                width: Val::Px(260.0),
                height: Val::Px(30.0),
                ..default()
            })
            .with_children(|slot| {
                let slider =
                    MaterialSlider::new(DiceScaleSettings::MIN_SCALE, DiceScaleSettings::MAX_SCALE)
                        .with_value(
                            value.clamp(DiceScaleSettings::MIN_SCALE, DiceScaleSettings::MAX_SCALE),
                        )
                        .track_height(6.0)
                        .thumb_radius(8.0);
                spawn_slider_control_with(slot, theme, slider, DiceScaleSlider { die_type });
            });

            row.spawn((
                Text::new(format!("{:.2}", value)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
                DiceScaleValueLabel { die_type },
            ));
        });
}

// ============================================================================
// Interaction Systems
// ============================================================================

/// Handle settings button click
pub fn handle_settings_button_click(
    mut click_events: MessageReader<IconButtonClickEvent>,
    button_query: Query<(), With<SettingsButton>>,
    mut settings_state: ResMut<SettingsState>,
    shake_config: Res<ContainerShakeConfig>,
    _theme: Res<MaterialTheme>,
    db: Option<Res<CharacterDatabase>>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in click_events.read() {
        if button_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.show_modal = true;
        settings_state.modal_kind = crate::dice3d::types::ActiveModalKind::DiceRollerSettings;
        settings_state.editing_color = settings_state.settings.background_color.clone();
        settings_state.color_input_text = settings_state.editing_color.to_hex();

        settings_state.editing_highlight_color =
            settings_state.settings.dice_box_highlight_color.clone();
        settings_state.highlight_input_text = settings_state.editing_highlight_color.to_hex();

        settings_state.quick_roll_editing_die = settings_state.settings.quick_roll_default_die;
        settings_state.default_roll_uses_shake_editing =
            settings_state.settings.default_roll_uses_shake;

        settings_state.editing_dice_scales = settings_state.settings.dice_scales.clone();

        settings_state.editing_dice_fx_surface_opacity =
            settings_state.settings.dice_fx_surface_opacity;
        settings_state.editing_dice_fx_plume_height_multiplier =
            settings_state.settings.dice_fx_plume_height_multiplier;
        settings_state.editing_dice_fx_plume_radius_multiplier =
            settings_state.settings.dice_fx_plume_radius_multiplier;

        // Custom dice FX staging.
        init_custom_dice_fx_editing_from_settings(&mut settings_state);

        settings_state.selected_dice_fx_curve_point_id = None;
        settings_state.dragging_dice_fx_curve_point_id = None;
        settings_state.dragging_dice_fx_curve_bezier = None;
        settings_state.dice_fx_curve_edit_mode = ShakeCurveEditMode::None;
        settings_state.dice_fx_curve_add_mask = true;
        settings_state.dice_fx_curve_add_noise = false;
        settings_state.dice_fx_curve_add_ramp = false;
        settings_state.dice_fx_curve_add_opacity = false;
        settings_state.dice_fx_curve_add_plume_height = false;
        settings_state.dice_fx_curve_add_plume_radius = false;

        settings_state.dice_fx_saved_dir_display_text = custom_fx_out_dir(db.as_deref())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        settings_state.dice_fx_preview_source = None;
        settings_state.dice_fx_preview_noise = None;
        settings_state.dice_fx_preview_mask = None;
        settings_state.dice_fx_preview_ramp = None;
        settings_state.dice_fx_preview_source_path = None;
        settings_state.dice_fx_preview_noise_path = None;
        settings_state.dice_fx_preview_mask_path = None;
        settings_state.dice_fx_preview_ramp_path = None;
        refresh_dice_fx_preview_handles_from_paths(&mut settings_state, &mut images);

        // Copy current shake settings into an editable staging area.
        settings_state.editing_shake_config = shake_config.clone();

        // Keep autosave snapshot aligned so opening the modal doesn't immediately rewrite.
        settings_state.last_saved_shake_config = settings_state.settings.shake_config.clone();

        settings_state.shake_duration_input_text = format!(
            "{:.3}",
            settings_state
                .editing_shake_config
                .duration_seconds
                .max(0.0)
        );

        // Theme seed staging.
        settings_state.theme_seed_input_text = settings_state
            .settings
            .theme_seed_hex
            .clone()
            .unwrap_or_default();
        settings_state.editing_theme_seed_override = settings_state
            .settings
            .theme_seed_hex
            .as_deref()
            .and_then(ColorSetting::parse)
            .map(|mut c| {
                c.a = 1.0;
                c
            });
    }
}

/// Persist shake curve changes immediately (every edit) and apply them to runtime.
///
/// The curve editor stores values in normalized percentages (value in [-1..1]).
/// At shake time, the runtime converts to actual movement using:
///   offset = (curve_value * shake_config.distance) * strength
pub fn autosave_and_apply_shake_config(
    mut settings_state: ResMut<SettingsState>,
    mut shake_config: ResMut<ContainerShakeConfig>,
) {
    if !settings_state.show_modal
        || settings_state.modal_kind != crate::dice3d::types::ActiveModalKind::DiceRollerSettings
    {
        return;
    }

    // Convert the editing config into the persisted representation.
    let persisted = ShakeConfigSetting::from_runtime(&settings_state.editing_shake_config);

    if persisted == settings_state.last_saved_shake_config {
        return;
    }

    settings_state.settings.shake_config = persisted;
    settings_state.last_saved_shake_config = settings_state.settings.shake_config.clone();

    // Apply to runtime immediately so the shake feature uses the latest curve without
    // requiring an explicit OK click.
    *shake_config = settings_state.editing_shake_config.clone();

    settings_state.is_modified = true;
}

/// Apply persisted shake config on startup.
pub fn apply_initial_shake_config(
    settings_state: Res<SettingsState>,
    mut shake_config: ResMut<ContainerShakeConfig>,
) {
    *shake_config = settings_state.settings.shake_config.to_runtime();
}

/// Spawn/despawn settings modal based on state
pub fn manage_settings_modal(
    mut commands: Commands,
    settings_state: Res<SettingsState>,
    theme: Res<MaterialTheme>,
    preview_target: Option<Res<DiceScalePreviewRenderTarget>>,
    ui_images: Option<Res<SettingsUiImages>>,
    modal_query: Query<Entity, With<SettingsModalOverlay>>,
) {
    if !settings_state.is_changed() {
        return;
    }

    if settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings
    {
        // Spawn modal if not exists
        if modal_query.is_empty() {
            spawn_settings_modal(
                &mut commands,
                &theme,
                &settings_state,
                &settings_state.editing_shake_config,
                preview_target.map(|r| r.image.clone()),
                ui_images.map(|r| r.blank.clone()),
            );
        }
    } else {
        // Despawn modal
        for entity in modal_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Handle OK button click
pub fn handle_settings_ok_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    ok_query: Query<(), With<SettingsOkButton>>,
    mut settings_state: ResMut<SettingsState>,
    mut clear_color: ResMut<ClearColor>,
    mut shake_config: ResMut<ContainerShakeConfig>,
    mut theme: ResMut<MaterialTheme>,
    db: Option<Res<CharacterDatabase>>,
) {
    for event in click_events.read() {
        if ok_query.get(event.entity).is_err() {
            continue;
        }
        // Apply the editing colors
        settings_state.settings.background_color = settings_state.editing_color.clone();
        settings_state.settings.dice_box_highlight_color =
            settings_state.editing_highlight_color.clone();

        settings_state.settings.quick_roll_default_die = settings_state.quick_roll_editing_die;

        // Apply per-die scale overrides.
        settings_state.settings.dice_scales = settings_state.editing_dice_scales.clone();

        // Apply Dice FX visual parameters.
        settings_state.settings.dice_fx_surface_opacity = settings_state
            .editing_dice_fx_surface_opacity
            .clamp(0.0, 1.0);
        settings_state.settings.dice_fx_plume_height_multiplier = settings_state
            .editing_dice_fx_plume_height_multiplier
            .clamp(0.25, 3.0);
        settings_state.settings.dice_fx_plume_radius_multiplier = settings_state
            .editing_dice_fx_plume_radius_multiplier
            .clamp(0.25, 3.0);

        // Apply roll->effect mapping.
        settings_state.settings.dice_fx_roll_effects =
            settings_state.editing_dice_fx_roll_effects.clone();

        settings_state.settings.default_roll_uses_shake =
            settings_state.default_roll_uses_shake_editing;

        // Persist the currently edited FX back into the selected library slot.
        if let Some(i) = settings_state.selected_custom_dice_fx_library_index {
            if i < settings_state.editing_custom_dice_fx_library.len() {
                settings_state.editing_custom_dice_fx_library[i] =
                    settings_state.editing_custom_dice_fx.clone();
            }
        }

        // Persist library.
        settings_state.settings.custom_dice_fx_library =
            settings_state.editing_custom_dice_fx_library.clone();

        // Apply custom dice FX settings (store as Some when enabled, None when disabled).
        settings_state.settings.custom_dice_fx = if settings_state.editing_custom_dice_fx.enabled {
            Some(settings_state.editing_custom_dice_fx.clone())
        } else {
            None
        };

        // Update the clear color
        clear_color.0 = settings_state.settings.background_color.to_color();

        // Apply shake settings from the editor
        *shake_config = settings_state.editing_shake_config.clone();

        // Theme: persist override if valid.
        let input = settings_state.theme_seed_input_text.trim();
        if input.is_empty() {
            settings_state.settings.theme_seed_hex = None;
        } else if let Some(mut parsed) = ColorSetting::parse(input) {
            parsed.a = 1.0;
            let canonical = parsed.to_hex();
            settings_state.settings.theme_seed_hex = Some(canonical.clone());

            settings_state
                .settings
                .recent_theme_seeds
                .retain(|s| !s.eq_ignore_ascii_case(&canonical));
            settings_state
                .settings
                .recent_theme_seeds
                .insert(0, canonical);

            const MAX_RECENT: usize = 10;
            settings_state
                .settings
                .recent_theme_seeds
                .truncate(MAX_RECENT);
        }

        // Ensure runtime theme matches the persisted selection.
        apply_theme_override(&settings_state.settings, &mut theme);

        settings_state.is_modified = true;

        // Persist immediately so OK behaves like an explicit save.
        // (We still keep `is_modified` as a fallback for the once-per-frame flusher.)
        if let Some(db) = db.as_deref() {
            match settings_state.settings.save_to_db(db) {
                Ok(()) => {
                    settings_state.is_modified = false;
                    info!(
                        "Saved settings to SurrealDB at {:?} (background={})",
                        db.db_path,
                        settings_state.settings.background_color.to_hex()
                    );
                }
                Err(e) => warn!("Failed to persist settings to SurrealDB: {}", e),
            }
        } else {
            warn!("CharacterDatabase resource not available; settings not persisted");
        }

        // Close modal
        settings_state.show_modal = false;
        settings_state.modal_kind = crate::dice3d::types::ActiveModalKind::None;
    }
}

pub fn handle_dice_fx_saved_effect_select_change(
    mut commands: Commands,
    mut events: MessageReader<SelectChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
    selects: Query<&MaterialSelect>,
    modal_query: Query<Entity, With<SettingsModalOverlay>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for ev in events.read() {
        // Filter to our dropdown.
        if let Ok(select) = selects.get(ev.entity) {
            if select.label.as_deref() != Some("Saved dice effects") {
                continue;
            }
        }

        let Some(value) = ev.option.value.clone() else {
            continue;
        };
        let Ok(new_idx) = value.parse::<usize>() else {
            continue;
        };

        // Save current edits into the previously selected entry so switching doesn't lose changes.
        if let Some(prev) = settings_state.selected_custom_dice_fx_library_index {
            if prev < settings_state.editing_custom_dice_fx_library.len() {
                settings_state.editing_custom_dice_fx_library[prev] =
                    settings_state.editing_custom_dice_fx.clone();
            }
        }

        if new_idx >= settings_state.editing_custom_dice_fx_library.len() {
            continue;
        }

        settings_state.selected_custom_dice_fx_library_index = Some(new_idx);
        settings_state.editing_custom_dice_fx =
            settings_state.editing_custom_dice_fx_library[new_idx].clone();

        settings_state.dice_fx_trigger_value_input_text = settings_state
            .editing_custom_dice_fx
            .trigger_value
            .to_string();
        settings_state.dice_fx_duration_input_text = format!(
            "{:.3}",
            settings_state
                .editing_custom_dice_fx
                .duration_seconds
                .max(0.0)
        );

        // Reset curve editor selection/drag state when switching effects.
        settings_state.selected_dice_fx_curve_point_id = None;
        settings_state.dragging_dice_fx_curve_point_id = None;
        settings_state.dragging_dice_fx_curve_bezier = None;

        // Refresh preview base images from disk for the newly selected effect.
        settings_state.dice_fx_preview_source = None;
        settings_state.dice_fx_preview_noise = None;
        settings_state.dice_fx_preview_mask = None;
        settings_state.dice_fx_preview_ramp = None;
        settings_state.dice_fx_preview_last_time_t = -1.0;
        settings_state.dice_fx_preview_source_path = None;
        settings_state.dice_fx_preview_noise_path = None;
        settings_state.dice_fx_preview_mask_path = None;
        settings_state.dice_fx_preview_ramp_path = None;
        refresh_dice_fx_preview_handles_from_paths(&mut settings_state, &mut images);

        // Force a modal rebuild so all widgets (selects/text fields) reflect the new effect.
        for entity in modal_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Handle per-die scale slider changes in the settings modal.
pub fn handle_dice_scale_slider_changes(
    mut events: MessageReader<SliderChangeEvent>,
    slider_query: Query<&DiceScaleSlider>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in events.read() {
        let Ok(slider) = slider_query.get(event.entity) else {
            continue;
        };

        // Allow every die to be sized freely within the global bounds.
        // (Preview and in-scene updates use the editing values; OK persists them.)
        let value = event
            .value
            .clamp(DiceScaleSettings::MIN_SCALE, DiceScaleSettings::MAX_SCALE);
        settings_state
            .editing_dice_scales
            .set_scale_for(slider.die_type, value);
    }
}

/// Handle Dice FX parameter slider changes in the settings modal.
pub fn handle_dice_fx_param_slider_changes(
    mut events: MessageReader<SliderChangeEvent>,
    slider_query: Query<&DiceFxParamSlider>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in events.read() {
        let Ok(slider) = slider_query.get(event.entity) else {
            continue;
        };

        match slider.kind {
            DiceFxParamKind::SurfaceOpacity => {
                settings_state.editing_dice_fx_surface_opacity = event.value.clamp(0.0, 1.0);
            }
            DiceFxParamKind::PlumeHeight => {
                settings_state.editing_dice_fx_plume_height_multiplier =
                    event.value.clamp(0.25, 3.0);
            }
            DiceFxParamKind::PlumeRadius => {
                settings_state.editing_dice_fx_plume_radius_multiplier =
                    event.value.clamp(0.25, 3.0);
            }
        }
    }
}

pub fn handle_dice_fx_preview_time_slider_changes(
    mut events: MessageReader<SliderChangeEvent>,
    slider_query: Query<(), With<DiceFxPreviewTimeSlider>>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in events.read() {
        if slider_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.dice_fx_preview_time_t = event.value.clamp(0.0, 1.0);
    }
}

/// Sync dice scale sliders + value labels from the current editing state.
pub fn update_dice_scale_ui(
    settings_state: Res<SettingsState>,
    mut slider_query: Query<(&DiceScaleSlider, &mut MaterialSlider)>,
    mut label_query: Query<(&DiceScaleValueLabel, &mut Text)>,
) {
    if !settings_state.is_changed() {
        return;
    }

    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for (slider, mut material_slider) in slider_query.iter_mut() {
        material_slider.value = settings_state
            .editing_dice_scales
            .scale_for(slider.die_type);
    }

    for (label, mut text) in label_query.iter_mut() {
        let v = settings_state.editing_dice_scales.scale_for(label.die_type);
        *text = Text::new(format!("{:.2}", v));
    }
}

/// Sync Dice FX parameter sliders + value labels from the current editing state.
pub fn update_dice_fx_param_ui(
    settings_state: Res<SettingsState>,
    mut slider_query: Query<(&DiceFxParamSlider, &mut MaterialSlider)>,
    mut label_query: Query<(&DiceFxParamValueLabel, &mut Text)>,
) {
    if !settings_state.is_changed() {
        return;
    }

    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for (slider, mut material_slider) in slider_query.iter_mut() {
        material_slider.value = match slider.kind {
            DiceFxParamKind::SurfaceOpacity => settings_state
                .editing_dice_fx_surface_opacity
                .clamp(0.0, 1.0),
            DiceFxParamKind::PlumeHeight => settings_state
                .editing_dice_fx_plume_height_multiplier
                .clamp(0.25, 3.0),
            DiceFxParamKind::PlumeRadius => settings_state
                .editing_dice_fx_plume_radius_multiplier
                .clamp(0.25, 3.0),
        };
    }

    for (label, mut text) in label_query.iter_mut() {
        let v = match label.kind {
            DiceFxParamKind::SurfaceOpacity => settings_state.editing_dice_fx_surface_opacity,
            DiceFxParamKind::PlumeHeight => settings_state.editing_dice_fx_plume_height_multiplier,
            DiceFxParamKind::PlumeRadius => settings_state.editing_dice_fx_plume_radius_multiplier,
        };
        *text = Text::new(format!("{:.2}", v));
    }
}

/// Ensure the slider thumb is always inside the slider entity's hit-test area.
///
/// The underlying slider places the thumb centered on the track endpoints.
/// At the exact min/max, that can place part of the thumb outside the slider's
/// clickable node, making it hard to grab. Adding padding equal to the thumb
/// radius keeps the thumb inside the hit-test area.
pub fn fix_dice_scale_slider_thumb_hitbox(
    settings_state: Res<SettingsState>,
    mut sliders: Query<(&MaterialSlider, &mut Node), With<DiceScaleSlider>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for (slider, mut node) in sliders.iter_mut() {
        let r = slider.thumb_radius;
        // Horizontal sliders need left/right padding; vertical sliders need top/bottom.
        // This keeps the thumb entirely inside the slider node.
        match slider.orientation {
            SliderOrientation::Horizontal => {
                node.padding = UiRect {
                    left: Val::Px(r),
                    right: Val::Px(r),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                };
            }
            SliderOrientation::Vertical => {
                node.padding = UiRect {
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(r),
                    bottom: Val::Px(r),
                };
            }
        }
    }
}

/// Apply persisted dice scale settings to any existing dice entities.
///
/// This runs whenever the persisted `settings.dice_scales` value changes (e.g. via OK).
pub fn apply_dice_scale_settings_to_existing_dice(
    settings_state: Res<SettingsState>,
    mut dice_query: Query<(&Die, &mut Transform)>,
    mut last_applied: Local<Option<DiceScaleSettings>>,
) {
    let current = settings_state.settings.dice_scales.clone();

    if last_applied.as_ref() == Some(&current) {
        return;
    }

    for (die, mut transform) in dice_query.iter_mut() {
        let s = current.scale_for(die.die_type) * die.die_type.uniform_size_scale_factor();
        transform.scale = Vec3::splat(s);
    }

    *last_applied = Some(current);
}

/// Live-apply editing dice scale settings while the settings modal is open.
///
/// This makes slider changes immediately visible on any dice already in the scene.
pub fn apply_editing_dice_scales_to_existing_dice_while_open(
    settings_state: Res<SettingsState>,
    mut dice_query: Query<(&Die, &mut Transform)>,
    mut last_applied: Local<Option<DiceScaleSettings>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        *last_applied = None;
        return;
    }

    let current = settings_state.editing_dice_scales.clone();
    if last_applied.as_ref() == Some(&current) {
        return;
    }

    for (die, mut transform) in dice_query.iter_mut() {
        let s = current.scale_for(die.die_type) * die.die_type.uniform_size_scale_factor();
        transform.scale = Vec3::splat(s);
    }

    *last_applied = Some(current);
}

/// Handle switch changes in the dice roller settings modal.
pub fn handle_default_roll_uses_shake_switch_change(
    mut events: MessageReader<SwitchChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
    switch_query: Query<(), With<DefaultRollUsesShakeSwitch>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in events.read() {
        if switch_query.get(event.entity).is_err() {
            continue;
        }

        settings_state.default_roll_uses_shake_editing = event.selected;
    }
}

/// Handle selection changes in the dice roller settings modal (Quick Rolls die).
pub fn handle_quick_roll_die_type_select_change(
    mut commands: Commands,
    mut events: MessageReader<SelectChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
    selects: Query<&MaterialSelect>,
    modal_query: Query<Entity, With<SettingsModalOverlay>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    let options = [
        DiceTypeSetting::D4,
        DiceTypeSetting::D6,
        DiceTypeSetting::D8,
        DiceTypeSetting::D10,
        DiceTypeSetting::D12,
        DiceTypeSetting::D20,
    ];

    for event in events.read() {
        // Ignore selects not related to Quick Rolls.
        if let Ok(select) = selects.get(event.entity) {
            if select.label.as_deref() != Some("Effects die") {
                continue;
            }
        }

        if let Some(setting) = options.get(event.index).copied() {
            if settings_state.quick_roll_editing_die == setting {
                continue;
            }

            settings_state.quick_roll_editing_die = setting;

            // The dice tab contains a per-face effects mapping section whose widgets are built
            // based on the selected die type. Rebuild the modal so the correct controls show
            // immediately without requiring OK + reopening.
            for entity in modal_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handle selection changes in the dice roller settings modal (Dice Roll Effects per-face mapping).
pub fn handle_dice_fx_roll_effect_select_change(
    mut events: MessageReader<SelectChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
    selects: Query<&MaterialSelect>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in events.read() {
        let Ok(select) = selects.get(event.entity) else {
            continue;
        };

        // Only handle our per-face mapping dropdowns.
        let Some(label) = select.label.as_deref() else {
            continue;
        };
        let Some(value_str) = label.strip_prefix("Effect for ") else {
            continue;
        };
        let Ok(value) = value_str.parse::<u32>() else {
            continue;
        };

        let die_type = settings_state.quick_roll_editing_die.to_dice_type();

        let kind = match event.option.value.as_deref() {
            Some("fire") => crate::dice3d::types::DiceFxEffectKind::Fire,
            Some("lightning") => crate::dice3d::types::DiceFxEffectKind::Lightning,
            Some("firework") => crate::dice3d::types::DiceFxEffectKind::Firework,
            Some("explosion") => crate::dice3d::types::DiceFxEffectKind::Explosion,
            _ => crate::dice3d::types::DiceFxEffectKind::None,
        };

        bevy::log::info!(
            "Dice FX mapping set: die={:?} value={} -> {:?}",
            die_type,
            value,
            kind
        );

        settings_state
            .editing_dice_fx_roll_effects
            .set_effect_for(die_type, value, kind);
    }
}

/// Handle selection changes for the theme seed dropdown.
pub fn handle_theme_seed_select_change(
    mut events: MessageReader<SelectChangeEvent>,
    selects: Query<&MaterialSelect>,
    mut settings_state: ResMut<SettingsState>,
    mut theme: ResMut<MaterialTheme>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in events.read() {
        let Ok(select) = selects.get(event.entity) else {
            continue;
        };
        if select.label.as_deref() != Some("Recent themes") {
            continue;
        }

        let chosen = event
            .option
            .value
            .clone()
            .unwrap_or_else(|| event.option.label.clone());
        settings_state.theme_seed_input_text = chosen.clone();

        if let Some(mut parsed) = ColorSetting::parse(chosen.as_str()) {
            parsed.a = 1.0;
            let seed = parsed.to_color();
            settings_state.editing_theme_seed_override = Some(parsed);
            *theme = MaterialTheme::from_seed(seed, theme.mode);
        }
    }
}

// ============================================================================
// Dice FX (Custom Effect) - Settings Modal
// ============================================================================

fn custom_fx_out_dir(db: Option<&CharacterDatabase>) -> Option<PathBuf> {
    let db = db?;
    let data_dir = db.db_path.parent()?;
    Some(data_dir.join("fx").join("custom_dice_effect"))
}

fn load_image_from_disk(path: &Path) -> Result<Image, String> {
    let dyn_img = image::ImageReader::open(path)
        .map_err(|e| format!("Failed to open image {path:?}: {e}"))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess image format {path:?}: {e}"))?
        .decode()
        .map_err(|e| format!("Failed to decode image {path:?}: {e}"))?;

    let rgba = dyn_img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let size = Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);
    image.data = Some(rgba.into_raw());
    Ok(image)
}

fn refresh_dice_fx_preview_handles_from_paths(
    settings_state: &mut SettingsState,
    images: &mut Assets<Image>,
) {
    let cfg = &settings_state.editing_custom_dice_fx;

    if let Some(path) = cfg.source_image_path.as_ref() {
        if settings_state.dice_fx_preview_source_path.as_deref() != Some(path)
            || settings_state.dice_fx_preview_base_source.is_none()
        {
            if let Ok(img) = load_image_from_disk(Path::new(path)) {
                settings_state.dice_fx_preview_base_source = Some(images.add(img.clone()));
                settings_state.dice_fx_preview_source = Some(images.add(img));
                settings_state.dice_fx_preview_source_path = Some(path.clone());
            }
        }
    }

    if let Some(path) = cfg.noise_image_path.as_ref() {
        if settings_state.dice_fx_preview_noise_path.as_deref() != Some(path)
            || settings_state.dice_fx_preview_base_noise.is_none()
        {
            if let Ok(img) = load_image_from_disk(Path::new(path)) {
                settings_state.dice_fx_preview_base_noise = Some(images.add(img.clone()));
                settings_state.dice_fx_preview_noise = Some(images.add(img));
                settings_state.dice_fx_preview_noise_path = Some(path.clone());
            }
        }
    }

    if let Some(path) = cfg.mask_image_path.as_ref() {
        if settings_state.dice_fx_preview_mask_path.as_deref() != Some(path)
            || settings_state.dice_fx_preview_base_mask.is_none()
        {
            if let Ok(img) = load_image_from_disk(Path::new(path)) {
                settings_state.dice_fx_preview_base_mask = Some(images.add(img.clone()));
                settings_state.dice_fx_preview_mask = Some(images.add(img));
                settings_state.dice_fx_preview_mask_path = Some(path.clone());
            }
        }
    }

    if let Some(path) = cfg.ramp_image_path.as_ref() {
        if settings_state.dice_fx_preview_ramp_path.as_deref() != Some(path)
            || settings_state.dice_fx_preview_base_ramp.is_none()
        {
            if let Ok(img) = load_image_from_disk(Path::new(path)) {
                settings_state.dice_fx_preview_base_ramp = Some(images.add(img.clone()));
                settings_state.dice_fx_preview_ramp = Some(images.add(img));
                settings_state.dice_fx_preview_ramp_path = Some(path.clone());
            }
        }
    }
}

fn rgb_to_hsv(rgb: Vec3) -> Vec3 {
    let r = rgb.x;
    let g = rgb.y;
    let b = rgb.z;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let mut h = 0.0;
    if delta > 1e-6 {
        if max == r {
            h = ((g - b) / delta) % 6.0;
        } else if max == g {
            h = ((b - r) / delta) + 2.0;
        } else {
            h = ((r - g) / delta) + 4.0;
        }
        h /= 6.0;
        if h < 0.0 {
            h += 1.0;
        }
    }

    let s = if max <= 1e-6 { 0.0 } else { delta / max };
    let v = max;
    Vec3::new(h, s, v)
}

fn hsv_to_rgb(hsv: Vec3) -> Vec3 {
    let h = hsv.x.fract() * 6.0;
    let s = hsv.y.clamp(0.0, 1.0);
    let v = hsv.z.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (rp, gp, bp) = if (0.0..1.0).contains(&h) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Vec3::new(rp + m, gp + m, bp + m)
}

fn apply_brightness_contrast_gray(v: f32, brightness: f32, contrast: f32) -> f32 {
    ((v - 0.5) * contrast + 0.5 + brightness).clamp(0.0, 1.0)
}

fn apply_preview_effects(
    base: &Image,
    kind: DiceFxPreviewImageKind,
    mask_curve: f32,
    noise_curve: f32,
    ramp_curve: f32,
) -> Option<Vec<u8>> {
    let data = base.data.as_ref()?;
    if data.len() < 4 {
        return None;
    }

    let mut out = data.clone();

    // Map curves (0..1) -> parameters.
    let hue_shift = ramp_curve.clamp(0.0, 1.0);
    let mask_brightness = (mask_curve - 0.5) * 0.35;
    let mask_contrast = 0.75 + 1.75 * mask_curve;
    let noise_contrast = 0.75 + 1.75 * noise_curve;

    match kind {
        DiceFxPreviewImageKind::Ramp => {
            // Hue shift each pixel.
            for px in out.chunks_mut(4) {
                let rgb = Vec3::new(
                    px[0] as f32 / 255.0,
                    px[1] as f32 / 255.0,
                    px[2] as f32 / 255.0,
                );
                let mut hsv = rgb_to_hsv(rgb);
                hsv.x = (hsv.x + hue_shift).fract();
                let rgb2 = hsv_to_rgb(hsv);
                px[0] = (rgb2.x * 255.0).round().clamp(0.0, 255.0) as u8;
                px[1] = (rgb2.y * 255.0).round().clamp(0.0, 255.0) as u8;
                px[2] = (rgb2.z * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
        DiceFxPreviewImageKind::Mask => {
            for px in out.chunks_mut(4) {
                let v = (px[0] as f32) / 255.0;
                let v2 = apply_brightness_contrast_gray(v, mask_brightness, mask_contrast);
                let b = (v2 * 255.0).round().clamp(0.0, 255.0) as u8;
                px[0] = b;
                px[1] = b;
                px[2] = b;
                px[3] = 255;
            }
        }
        DiceFxPreviewImageKind::Noise => {
            for px in out.chunks_mut(4) {
                let v = (px[0] as f32) / 255.0;
                let v2 = apply_brightness_contrast_gray(v, 0.0, noise_contrast);
                let b = (v2 * 255.0).round().clamp(0.0, 255.0) as u8;
                px[0] = b;
                px[1] = b;
                px[2] = b;
                px[3] = 255;
            }
        }
        DiceFxPreviewImageKind::Source => {
            for px in out.chunks_mut(4) {
                let rgb = Vec3::new(
                    px[0] as f32 / 255.0,
                    px[1] as f32 / 255.0,
                    px[2] as f32 / 255.0,
                );
                let mut hsv = rgb_to_hsv(rgb);
                hsv.x = (hsv.x + hue_shift).fract();
                let mut rgb2 = hsv_to_rgb(hsv);
                // Also show mask curve effect as brightness/contrast.
                rgb2.x = apply_brightness_contrast_gray(
                    rgb2.x,
                    mask_brightness * 0.6,
                    0.9 + 0.8 * mask_curve,
                );
                rgb2.y = apply_brightness_contrast_gray(
                    rgb2.y,
                    mask_brightness * 0.6,
                    0.9 + 0.8 * mask_curve,
                );
                rgb2.z = apply_brightness_contrast_gray(
                    rgb2.z,
                    mask_brightness * 0.6,
                    0.9 + 0.8 * mask_curve,
                );
                px[0] = (rgb2.x * 255.0).round().clamp(0.0, 255.0) as u8;
                px[1] = (rgb2.y * 255.0).round().clamp(0.0, 255.0) as u8;
                px[2] = (rgb2.z * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    Some(out)
}

pub fn handle_dice_fx_upload_image_click(
    mut commands: Commands,
    mut click_events: MessageReader<ButtonClickEvent>,
    upload_query: Query<(), With<DiceFxUploadImageButton>>,
    mut settings_state: ResMut<SettingsState>,
    db: Option<Res<CharacterDatabase>>,
    mut images: ResMut<Assets<Image>>,
    modal_query: Query<Entity, With<SettingsModalOverlay>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for ev in click_events.read() {
        if upload_query.get(ev.entity).is_err() {
            continue;
        }

        let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Image",
                &["png", "jpg", "jpeg", "bmp", "tga", "gif", "webp"],
            )
            .pick_file()
        else {
            continue;
        };

        let Some(out_dir) = custom_fx_out_dir(db.as_deref()) else {
            warn!("No CharacterDatabase available; cannot persist custom FX images");
            continue;
        };

        match generate_custom_fx_textures(
            path.as_path(),
            out_dir.as_path(),
            &settings_state.editing_custom_dice_fx,
        ) {
            Ok(gen) => {
                let source_handle = images.add(gen.source_image);
                let noise_handle = images.add(gen.noise_image);
                let ramp_handle = images.add(gen.ramp_image);
                let mask_handle = images.add(gen.mask_image);

                // Persist paths in editing config; the runtime FX system will load these
                // when settings are applied.
                settings_state.editing_custom_dice_fx.enabled = true;
                settings_state.editing_custom_dice_fx.source_image_path =
                    Some(gen.source_path.to_string_lossy().to_string());
                settings_state.editing_custom_dice_fx.noise_image_path =
                    Some(gen.noise_path.to_string_lossy().to_string());
                settings_state.editing_custom_dice_fx.ramp_image_path =
                    Some(gen.ramp_path.to_string_lossy().to_string());
                settings_state.editing_custom_dice_fx.mask_image_path =
                    Some(gen.mask_path.to_string_lossy().to_string());

                // Update preview cache so the settings UI can show the images immediately.
                settings_state.dice_fx_preview_base_source = Some(source_handle);
                settings_state.dice_fx_preview_base_noise = Some(noise_handle);
                settings_state.dice_fx_preview_base_ramp = Some(ramp_handle);
                settings_state.dice_fx_preview_base_mask = Some(mask_handle);

                // Force regeneration of display previews on next sync.
                settings_state.dice_fx_preview_source = None;
                settings_state.dice_fx_preview_noise = None;
                settings_state.dice_fx_preview_ramp = None;
                settings_state.dice_fx_preview_mask = None;
                settings_state.dice_fx_preview_last_time_t = -1.0;

                settings_state.dice_fx_preview_source_path = settings_state
                    .editing_custom_dice_fx
                    .source_image_path
                    .clone();
                settings_state.dice_fx_preview_noise_path = settings_state
                    .editing_custom_dice_fx
                    .noise_image_path
                    .clone();
                settings_state.dice_fx_preview_ramp_path = settings_state
                    .editing_custom_dice_fx
                    .ramp_image_path
                    .clone();
                settings_state.dice_fx_preview_mask_path = settings_state
                    .editing_custom_dice_fx
                    .mask_image_path
                    .clone();

                // Add/update this effect in the editing library and select it.
                let fx = settings_state.editing_custom_dice_fx.clone();
                let idx = fx
                    .source_image_path
                    .as_deref()
                    .and_then(|p| {
                        settings_state
                            .editing_custom_dice_fx_library
                            .iter()
                            .position(|e| e.source_image_path.as_deref() == Some(p))
                    })
                    .unwrap_or_else(|| {
                        settings_state
                            .editing_custom_dice_fx_library
                            .push(fx.clone());
                        settings_state.editing_custom_dice_fx_library.len() - 1
                    });
                settings_state.editing_custom_dice_fx_library[idx] = fx;
                settings_state.selected_custom_dice_fx_library_index = Some(idx);

                // Force a rebuild so the Saved effects dropdown includes the new entry.
                for entity in modal_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            Err(e) => {
                warn!("Failed to generate custom dice FX textures: {e}");
            }
        }
    }
}

pub fn sync_dice_fx_preview_images(
    mut settings_state: ResMut<SettingsState>,
    mut images: ResMut<Assets<Image>>,
    mut preview_nodes: Query<(&DiceFxPreviewImageNode, &mut bevy::ui::widget::ImageNode)>,
    mut time_label: Query<&mut Text, With<DiceFxPreviewTimeLabel>>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    // If the user re-opened the modal, or paths changed via other edits, ensure previews exist.
    refresh_dice_fx_preview_handles_from_paths(&mut settings_state, &mut images);

    // Update the time label.
    let dur = settings_state
        .editing_custom_dice_fx
        .duration_seconds
        .max(0.0);
    let preview_t = settings_state.dice_fx_preview_time_t.clamp(0.0, 1.0);
    let preview_seconds = preview_t * dur;
    if let Some(mut text) = time_label.iter_mut().next() {
        *text = Text::new(format!(
            "t = {:.2}s ({:.0}%)",
            preview_seconds,
            preview_t * 100.0
        ));
    }

    // Regenerate previews if time changed (or if we're missing display handles).
    let time_changed = (settings_state.dice_fx_preview_last_time_t - preview_t).abs() > 0.0005;
    let need_any = settings_state.dice_fx_preview_source.is_none()
        || settings_state.dice_fx_preview_noise.is_none()
        || settings_state.dice_fx_preview_mask.is_none()
        || settings_state.dice_fx_preview_ramp.is_none();

    if time_changed || need_any {
        settings_state.dice_fx_preview_last_time_t = preview_t;
        let u = preview_t;

        let mask_curve =
            sample_fx_curve(&settings_state.editing_custom_dice_fx.curve_points_mask, u)
                .clamp(0.0, 1.0);
        let noise_curve =
            sample_fx_curve(&settings_state.editing_custom_dice_fx.curve_points_noise, u)
                .clamp(0.0, 1.0);
        let ramp_curve =
            sample_fx_curve(&settings_state.editing_custom_dice_fx.curve_points_ramp, u)
                .clamp(0.0, 1.0);

        // Apply to each preview kind.
        let mut update_one =
            |kind: DiceFxPreviewImageKind,
             base_handle: &Option<Handle<Image>>,
             display_handle: &mut Option<Handle<Image>>| {
                let Some(base_handle) = base_handle.as_ref() else {
                    return;
                };
                let Some(base_img) = images.get(base_handle) else {
                    return;
                };
                let Some(new_data) =
                    apply_preview_effects(base_img, kind, mask_curve, noise_curve, ramp_curve)
                else {
                    return;
                };

                if display_handle.is_none() {
                    let mut img = base_img.clone();
                    img.data = Some(new_data);
                    *display_handle = Some(images.add(img));
                    return;
                }

                if let Some(h) = display_handle.as_ref() {
                    if let Some(img) = images.get_mut(h) {
                        img.data = Some(new_data);
                    }
                }
            };

        let base_source = settings_state.dice_fx_preview_base_source.clone();
        let base_noise = settings_state.dice_fx_preview_base_noise.clone();
        let base_mask = settings_state.dice_fx_preview_base_mask.clone();
        let base_ramp = settings_state.dice_fx_preview_base_ramp.clone();

        update_one(
            DiceFxPreviewImageKind::Source,
            &base_source,
            &mut settings_state.dice_fx_preview_source,
        );
        update_one(
            DiceFxPreviewImageKind::Noise,
            &base_noise,
            &mut settings_state.dice_fx_preview_noise,
        );
        update_one(
            DiceFxPreviewImageKind::Mask,
            &base_mask,
            &mut settings_state.dice_fx_preview_mask,
        );
        update_one(
            DiceFxPreviewImageKind::Ramp,
            &base_ramp,
            &mut settings_state.dice_fx_preview_ramp,
        );
    }

    for (tag, mut image_node) in preview_nodes.iter_mut() {
        let desired = match tag.kind {
            DiceFxPreviewImageKind::Source => settings_state.dice_fx_preview_source.as_ref(),
            DiceFxPreviewImageKind::Noise => settings_state.dice_fx_preview_noise.as_ref(),
            DiceFxPreviewImageKind::Mask => settings_state.dice_fx_preview_mask.as_ref(),
            DiceFxPreviewImageKind::Ramp => settings_state.dice_fx_preview_ramp.as_ref(),
        };

        if let Some(handle) = desired {
            image_node.image = handle.clone();
        }
    }
}

pub fn handle_dice_fx_trigger_select_change(
    mut events: MessageReader<SelectChangeEvent>,
    mut settings_state: ResMut<SettingsState>,
    selects: Query<&MaterialSelect>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for ev in events.read() {
        if let Ok(select) = selects.get(ev.entity) {
            if select.label.as_deref() != Some("Custom dice effect trigger") {
                continue;
            }
        }

        let value = ev
            .option
            .value
            .clone()
            .unwrap_or_else(|| ev.option.label.clone());

        settings_state.editing_custom_dice_fx.trigger_kind = match value.as_str() {
            "all_max" => CustomDiceFxTriggerKind::AllMax,
            "any_die_equals" => CustomDiceFxTriggerKind::AnyDieEquals,
            _ => CustomDiceFxTriggerKind::TotalAtLeast,
        };
    }
}

pub fn handle_dice_fx_trigger_value_text_input(
    mut settings_state: ResMut<SettingsState>,
    mut change_events: MessageReader<TextFieldChangeEvent>,
    mut submit_events: MessageReader<TextFieldSubmitEvent>,
    mut field_query: Query<&mut MaterialTextField, With<DiceFxTriggerValueTextInput>>,
) {
    if !settings_state.show_modal {
        return;
    }

    let apply = |value: &str, field: &mut MaterialTextField, state: &mut SettingsState| {
        state.dice_fx_trigger_value_input_text = value.to_string();
        match value.trim().parse::<u32>() {
            Ok(v) if v > 0 => {
                state.editing_custom_dice_fx.trigger_value = v;
                field.error = false;
                field.error_text = None;
            }
            _ => {
                field.error = true;
                field.error_text = Some("Enter a positive integer".to_string());
            }
        }
    };

    for ev in change_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };
        apply(&ev.value, &mut field, &mut settings_state);
    }

    for ev in submit_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };
        apply(&ev.value, &mut field, &mut settings_state);
        if !field.error {
            field.value = settings_state.dice_fx_trigger_value_input_text.clone();
            field.has_content = !field.value.is_empty();
        }
    }
}

pub fn handle_dice_fx_duration_text_input(
    mut settings_state: ResMut<SettingsState>,
    mut change_events: MessageReader<TextFieldChangeEvent>,
    mut submit_events: MessageReader<TextFieldSubmitEvent>,
    mut field_query: Query<&mut MaterialTextField, With<DiceFxDurationTextInput>>,
) {
    if !settings_state.show_modal {
        return;
    }

    let apply = |value: &str, field: &mut MaterialTextField, state: &mut SettingsState| {
        state.dice_fx_duration_input_text = value.to_string();
        match value.trim().parse::<f32>() {
            Ok(mut v) if v.is_finite() && v > 0.0 => {
                v = v.max(0.01);
                state.editing_custom_dice_fx.duration_seconds = v;
                field.error = false;
                field.error_text = None;
            }
            Ok(_) => {
                field.error = true;
                field.error_text = Some("Enter a positive number".to_string());
            }
            Err(_) => {
                field.error = true;
                field.error_text = Some("Enter a number".to_string());
            }
        }
    };

    for ev in change_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };
        apply(&ev.value, &mut field, &mut settings_state);
    }

    for ev in submit_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };
        apply(&ev.value, &mut field, &mut settings_state);
        if !field.error {
            settings_state.dice_fx_duration_input_text = format!(
                "{:.3}",
                settings_state.editing_custom_dice_fx.duration_seconds
            );
            field.value = settings_state.dice_fx_duration_input_text.clone();
            field.has_content = !field.value.is_empty();
        }
    }
}

fn sort_curve_points(points: &mut [ShakeCurvePoint]) {
    points.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(Ordering::Equal));
}

fn find_curve_point_index(points: &[ShakeCurvePoint], id: u64) -> Option<usize> {
    points.iter().position(|p| p.id == id)
}

fn curve_points(cfg: &ContainerShakeConfig, axis: ShakeAxis) -> &[ShakeCurvePoint] {
    match axis {
        ShakeAxis::X => &cfg.curve_points_x,
        ShakeAxis::Y => &cfg.curve_points_y,
        ShakeAxis::Z => &cfg.curve_points_z,
    }
}

fn curve_points_mut(cfg: &mut ContainerShakeConfig, axis: ShakeAxis) -> &mut Vec<ShakeCurvePoint> {
    match axis {
        ShakeAxis::X => &mut cfg.curve_points_x,
        ShakeAxis::Y => &mut cfg.curve_points_y,
        ShakeAxis::Z => &mut cfg.curve_points_z,
    }
}

fn find_curve_point_axis(cfg: &ContainerShakeConfig, id: u64) -> Option<ShakeAxis> {
    if cfg.curve_points_x.iter().any(|p| p.id == id) {
        return Some(ShakeAxis::X);
    }
    if cfg.curve_points_y.iter().any(|p| p.id == id) {
        return Some(ShakeAxis::Y);
    }
    if cfg.curve_points_z.iter().any(|p| p.id == id) {
        return Some(ShakeAxis::Z);
    }
    None
}

fn remove_curve_point_by_id(cfg: &mut ContainerShakeConfig, id: u64) -> bool {
    let Some(axis) = find_curve_point_axis(cfg, id) else {
        return false;
    };
    let points = curve_points_mut(cfg, axis);
    if points.len() <= 2 {
        return false;
    }
    if let Some(i) = find_curve_point_index(points, id) {
        points.remove(i);
        return true;
    }
    false
}

fn window_physical_cursor_to_ui_node_local_logical_px(
    cursor_in_ui_target_physical: Vec2,
    transform: &UiGlobalTransform,
    computed: &ComputedNode,
) -> Option<Vec2> {
    // Hit-testing uses physical pixels, but UI layout values (Val::Px) are in logical UI pixels.
    // Convert the physical-local point into logical-local using the node's inverse scale factor.
    let inv = transform.try_inverse()?;
    let size_physical = computed.size();
    let local_physical = inv.transform_point2(cursor_in_ui_target_physical) + size_physical * 0.5;
    Some(local_physical * computed.inverse_scale_factor())
}

fn ui_target_cursor_physical_px(window: &Window, ui_camera: Option<&Camera>) -> Option<Vec2> {
    let cursor_window = window.physical_cursor_position()?;
    let viewport_min = ui_camera
        .and_then(|c| c.physical_viewport_rect().map(|r| r.min.as_vec2()))
        .unwrap_or(Vec2::ZERO);
    Some(cursor_window - viewport_min)
}

fn shake_curve_t_v_to_local_px(graph_size: Vec2, t: f32, v: f32) -> Option<Vec2> {
    if graph_size.x <= 1.0 || graph_size.y <= 1.0 {
        return None;
    }

    // Keep a small inset so +/-100% is fully visible with point/handle sizes.
    const EDGE_PAD_PX: f32 = 7.0;
    let w = (graph_size.x - 2.0 * EDGE_PAD_PX).max(1.0);
    let h = (graph_size.y - 2.0 * EDGE_PAD_PX).max(1.0);

    let t = t.clamp(0.0, 1.0);
    let v = v.clamp(-1.0, 1.0);
    let x = EDGE_PAD_PX + t * w;
    let y = EDGE_PAD_PX + ((1.0 - v) * 0.5) * h;
    Some(Vec2::new(x, y))
}

fn graph_local_px_to_t_v(graph_size: Vec2, local: Vec2) -> Option<(f32, f32)> {
    if graph_size.x <= 1.0 || graph_size.y <= 1.0 {
        return None;
    }

    const EDGE_PAD_PX: f32 = 7.0;
    let w = (graph_size.x - 2.0 * EDGE_PAD_PX).max(1.0);
    let h = (graph_size.y - 2.0 * EDGE_PAD_PX).max(1.0);

    let lx = (local.x - EDGE_PAD_PX).clamp(0.0, w);
    let ly = (local.y - EDGE_PAD_PX).clamp(0.0, h);

    let t: f32 = (lx / w).clamp(0.0, 1.0);
    let y01: f32 = (ly / h).clamp(0.0, 1.0);
    let v: f32 = (1.0 - 2.0 * y01).clamp(-1.0, 1.0);
    Some((t, v))
}

fn add_curve_point(cfg: &mut ContainerShakeConfig, axis: ShakeAxis, t: f32, value: f32) -> u64 {
    let new_id = cfg.next_curve_point_id;
    cfg.next_curve_point_id += 1;
    curve_points_mut(cfg, axis).push(ShakeCurvePoint {
        id: new_id,
        t: t.clamp(0.0, 1.0),
        value: value.clamp(-1.0, 1.0),
        in_handle: None,
        out_handle: None,
    });
    sort_curve_points(curve_points_mut(cfg, axis));
    new_id
}

fn axis_enabled(settings_state: &SettingsState, axis: ShakeAxis) -> bool {
    match axis {
        ShakeAxis::X => settings_state.shake_curve_add_x,
        ShakeAxis::Y => settings_state.shake_curve_add_y,
        ShakeAxis::Z => settings_state.shake_curve_add_z,
    }
}

fn find_nearest_curve_point_id(
    cfg: &ContainerShakeConfig,
    graph_size: Vec2,
    cursor_local: Vec2,
    settings_state: &SettingsState,
    threshold_px: f32,
) -> Option<u64> {
    let mut best: Option<(u64, f32, u8)> = None;

    // Prefer deterministic axis ordering when points overlap.
    let axis_rank = |axis: ShakeAxis| match axis {
        ShakeAxis::X => 0u8,
        ShakeAxis::Y => 1u8,
        ShakeAxis::Z => 2u8,
    };

    for axis in [ShakeAxis::X, ShakeAxis::Y, ShakeAxis::Z] {
        if !axis_enabled(settings_state, axis) {
            continue;
        }

        for p in curve_points(cfg, axis) {
            let Some(pos) = shake_curve_t_v_to_local_px(graph_size, p.t, p.value) else {
                continue;
            };
            let d = cursor_local.distance(pos);
            if d > threshold_px {
                continue;
            }

            let rank = axis_rank(axis);
            match best {
                None => best = Some((p.id, d, rank)),
                Some((_, best_d, best_rank)) => {
                    if d < best_d || (d == best_d && rank < best_rank) {
                        best = Some((p.id, d, rank));
                    }
                }
            }
        }
    }

    best.map(|(id, _, _)| id)
}

/// Handle chip clicks for shake curve edit mode and axis selection.
pub fn handle_shake_curve_chip_clicks(
    mut click_events: MessageReader<ChipClickEvent>,
    edit_mode_chips: Query<&ShakeCurveEditModeChip>,
    axis_chips: Query<&ShakeCurveAxisChip>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !settings_state.show_modal {
        return;
    }

    for ev in click_events.read() {
        if let Ok(chip) = edit_mode_chips.get(ev.entity) {
            settings_state.shake_curve_edit_mode =
                if settings_state.shake_curve_edit_mode == chip.mode {
                    ShakeCurveEditMode::None
                } else {
                    chip.mode
                };

            // Delete mode cancels any in-progress drag.
            if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
                settings_state.dragging_shake_curve_point_id = None;
                settings_state.dragging_shake_curve_bezier = None;
            }

            continue;
        }

        if let Ok(chip) = axis_chips.get(ev.entity) {
            let axis = chip.axis;
            match chip.axis {
                ShakeAxis::X => {
                    settings_state.shake_curve_add_x = !settings_state.shake_curve_add_x
                }
                ShakeAxis::Y => {
                    settings_state.shake_curve_add_y = !settings_state.shake_curve_add_y
                }
                ShakeAxis::Z => {
                    settings_state.shake_curve_add_z = !settings_state.shake_curve_add_z
                }
            }

            // Never allow the user to disable all axes; that makes the editor feel broken
            // (no points are selectable and Add/Delete may appear to do nothing).
            if !settings_state.shake_curve_add_x
                && !settings_state.shake_curve_add_y
                && !settings_state.shake_curve_add_z
            {
                match axis {
                    ShakeAxis::X => settings_state.shake_curve_add_x = true,
                    ShakeAxis::Y => settings_state.shake_curve_add_y = true,
                    ShakeAxis::Z => settings_state.shake_curve_add_z = true,
                }
            }

            // If the currently-selected point is on a now-disabled axis, deselect it.
            if let Some(selected_id) = settings_state.selected_shake_curve_point_id {
                if let Some(axis) =
                    find_curve_point_axis(&settings_state.editing_shake_config, selected_id)
                {
                    if !axis_enabled(&settings_state, axis) {
                        settings_state.selected_shake_curve_point_id = None;
                        if settings_state.dragging_shake_curve_point_id == Some(selected_id) {
                            settings_state.dragging_shake_curve_point_id = None;
                        }
                        settings_state.dragging_shake_curve_bezier = None;
                    }
                }
            }
        }
    }
}

/// Keep chip selected state in sync with `SettingsState`.
pub fn sync_shake_curve_chip_ui(
    settings_state: Res<SettingsState>,
    mut edit_mode_chips: Query<
        (&ShakeCurveEditModeChip, &mut MaterialChip),
        Without<ShakeCurveAxisChip>,
    >,
    mut axis_chips: Query<
        (&ShakeCurveAxisChip, &mut MaterialChip),
        Without<ShakeCurveEditModeChip>,
    >,
) {
    if !settings_state.show_modal {
        return;
    }

    if !settings_state.is_changed() {
        return;
    }

    for (chip, mut material) in edit_mode_chips.iter_mut() {
        material.selected = settings_state.shake_curve_edit_mode == chip.mode;
    }

    for (chip, mut material) in axis_chips.iter_mut() {
        material.selected = match chip.axis {
            ShakeAxis::X => settings_state.shake_curve_add_x,
            ShakeAxis::Y => settings_state.shake_curve_add_y,
            ShakeAxis::Z => settings_state.shake_curve_add_z,
        };
    }
}

/// Click the graph background to add a point anywhere.
pub fn handle_shake_curve_graph_click_to_add_point(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<ShakeCurveGraphPlotRoot>,
    >,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let mode = settings_state.shake_curve_edit_mode;

    // If we're already dragging something, don't also process a background click.
    if settings_state.dragging_shake_curve_point_id.is_some()
        || settings_state.dragging_shake_curve_bezier.is_some()
    {
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let window = windows.iter().next();

    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    // Use window physical cursor mapping (robust under DPI scaling).
    let cursor_local = if let Some(window) = window {
        let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
        cursor_in_ui_target.and_then(|c| {
            window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed)
        })
    } else {
        None
    };

    let Some(cursor_local) = cursor_local else {
        return;
    };

    // Require the click be inside the plot rect.
    if cursor_local.x < 0.0
        || cursor_local.y < 0.0
        || cursor_local.x > size.x
        || cursor_local.y > size.y
    {
        return;
    }

    let Some((t, v)) = graph_local_px_to_t_v(size, cursor_local) else {
        return;
    };

    // Read chip toggles before mut-borrowing the config.
    let add_x = settings_state.shake_curve_add_x;
    let add_y = settings_state.shake_curve_add_y;
    let add_z = settings_state.shake_curve_add_z;

    match mode {
        ShakeCurveEditMode::Add => {
            // In Add mode, clicking near an existing point selects/drags it instead of creating a new one.
            if let Some(id) = find_nearest_curve_point_id(
                &settings_state.editing_shake_config,
                size,
                cursor_local,
                &settings_state,
                16.0,
            ) {
                settings_state.selected_shake_curve_point_id = Some(id);
                settings_state.dragging_shake_curve_point_id = Some(id);
                return;
            }

            let new_selected: Option<u64> = {
                let cfg = &mut settings_state.editing_shake_config;
                let mut new_selected: Option<u64> = None;
                if add_x {
                    new_selected = Some(add_curve_point(cfg, ShakeAxis::X, t, v));
                }
                if add_y {
                    new_selected = Some(add_curve_point(cfg, ShakeAxis::Y, t, v));
                }
                if add_z {
                    new_selected = Some(add_curve_point(cfg, ShakeAxis::Z, t, v));
                }
                new_selected
            };

            if let Some(id) = new_selected {
                settings_state.selected_shake_curve_point_id = Some(id);
                settings_state.dragging_shake_curve_point_id = Some(id);
            }
        }
        ShakeCurveEditMode::Delete => {
            // Background click deletes nearest point handle.
            let removed_id: Option<u64> = {
                let cfg = &mut settings_state.editing_shake_config;
                let mut best: Option<(u64, f32)> = None;
                let consider = |best: &mut Option<(u64, f32)>, id: u64, dist: f32| match best {
                    None => *best = Some((id, dist)),
                    Some((_, best_dist)) if dist < *best_dist => *best = Some((id, dist)),
                    _ => {}
                };

                // Threshold in px from point center.
                let threshold = 22.0_f32;
                for axis in [ShakeAxis::X, ShakeAxis::Y, ShakeAxis::Z] {
                    let axis_on = match axis {
                        ShakeAxis::X => add_x,
                        ShakeAxis::Y => add_y,
                        ShakeAxis::Z => add_z,
                    };
                    if !axis_on {
                        continue;
                    }
                    for p in curve_points(cfg, axis) {
                        let Some(pos) = shake_curve_t_v_to_local_px(size, p.t, p.value) else {
                            continue;
                        };
                        let d = cursor_local.distance(pos);
                        if d <= threshold {
                            consider(&mut best, p.id, d);
                        }
                    }
                }

                if let Some((id, _)) = best {
                    if remove_curve_point_by_id(cfg, id) {
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(id) = removed_id {
                if settings_state.selected_shake_curve_point_id == Some(id) {
                    settings_state.selected_shake_curve_point_id = None;
                }
                if settings_state.dragging_shake_curve_point_id == Some(id) {
                    settings_state.dragging_shake_curve_point_id = None;
                }
            }
        }
        ShakeCurveEditMode::None => {
            // Select/drag if clicking near a point; otherwise deselect.
            if let Some(id) = find_nearest_curve_point_id(
                &settings_state.editing_shake_config,
                size,
                cursor_local,
                &settings_state,
                16.0,
            ) {
                settings_state.selected_shake_curve_point_id = Some(id);
                settings_state.dragging_shake_curve_point_id = Some(id);
            } else {
                settings_state.selected_shake_curve_point_id = None;
                settings_state.dragging_shake_curve_point_id = None;
                settings_state.dragging_shake_curve_bezier = None;
            }
        }
    }
}

/// Start dragging a Bezier handle when it is pressed.
pub fn handle_shake_curve_bezier_handle_press(
    mut settings_state: ResMut<SettingsState>,
    mut interactions: Query<(&Interaction, &ShakeCurveBezierHandle), Changed<Interaction>>,
) {
    if !settings_state.show_modal {
        return;
    }

    if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    for (interaction, handle) in interactions.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        settings_state.selected_shake_curve_point_id = Some(handle.point_id);
        settings_state.dragging_shake_curve_point_id = None;
        settings_state.dragging_shake_curve_bezier = Some((handle.point_id, handle.kind));
    }
}

/// Drag a Bezier handle for the currently-selected point.
pub fn drag_shake_curve_bezier_handle(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<ShakeCurveGraphPlotRoot>,
    >,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }
    if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        settings_state.dragging_shake_curve_bezier = None;
        return;
    }

    let Some((point_id, kind)) = settings_state.dragging_shake_curve_bezier else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        settings_state.dragging_shake_curve_bezier = None;
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };

    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
    let local = cursor_in_ui_target
        .and_then(|c| window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed));
    let Some(local) = local else {
        return;
    };
    let Some((t, v)) = graph_local_px_to_t_v(size, local) else {
        return;
    };

    let cfg = &mut settings_state.editing_shake_config;
    let Some(axis) = find_curve_point_axis(cfg, point_id) else {
        return;
    };
    let points = curve_points_mut(cfg, axis);
    sort_curve_points(points);
    let Some(i) = find_curve_point_index(points, point_id) else {
        return;
    };

    let pt_t = points[i].t;
    let prev_t = if i > 0 { points[i - 1].t } else { pt_t };
    let next_t = if i + 1 < points.len() {
        points[i + 1].t
    } else {
        pt_t
    };

    let v = v.clamp(-1.0, 1.0);
    let handle_t = match kind {
        ShakeCurveBezierHandleKind::In => t.clamp(prev_t.min(pt_t), pt_t.max(prev_t)),
        ShakeCurveBezierHandleKind::Out => t.clamp(pt_t.min(next_t), next_t.max(pt_t)),
    };

    let handle_pos = Vec2::new(handle_t, v);
    match kind {
        ShakeCurveBezierHandleKind::In => points[i].in_handle = Some(handle_pos),
        ShakeCurveBezierHandleKind::Out => points[i].out_handle = Some(handle_pos),
    }
}

/// Start selecting/dragging curve points when their handle is pressed.
pub fn handle_shake_curve_point_press(
    mut settings_state: ResMut<SettingsState>,
    mut interactions: Query<(&Interaction, &ShakeCurvePointHandle), Changed<Interaction>>,
    graph: Query<&Node, With<ShakeCurveGraphPlotRoot>>,
) {
    if !settings_state.show_modal {
        return;
    }

    // Only active when the graph is visible.
    let Some(node) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    for (interaction, handle) in interactions.iter_mut() {
        if *interaction == Interaction::Pressed {
            if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
                // Deletion is handled on press in delete mode.
                let id = handle.id;
                let removed = {
                    let cfg = &mut settings_state.editing_shake_config;
                    remove_curve_point_by_id(cfg, id)
                };
                if removed {
                    if settings_state.selected_shake_curve_point_id == Some(id) {
                        settings_state.selected_shake_curve_point_id = None;
                    }
                    if settings_state.dragging_shake_curve_point_id == Some(id) {
                        settings_state.dragging_shake_curve_point_id = None;
                    }
                }
            } else {
                settings_state.selected_shake_curve_point_id = Some(handle.id);
                settings_state.dragging_shake_curve_point_id = Some(handle.id);
                settings_state.dragging_shake_curve_bezier = None;
            }
        }
    }
}

/// Drag the selected curve point within the graph bounds.
pub fn drag_shake_curve_point(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<ShakeCurveGraphPlotRoot>,
    >,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }

    if settings_state.shake_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };

    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    if mouse.just_released(MouseButton::Left) {
        settings_state.dragging_shake_curve_point_id = None;
        return;
    }

    let Some(drag_id) = settings_state.dragging_shake_curve_point_id else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        settings_state.dragging_shake_curve_point_id = None;
        return;
    }

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
    let local = cursor_in_ui_target
        .and_then(|c| window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed));
    let Some(local) = local else {
        return;
    };
    let Some((t, v)) = graph_local_px_to_t_v(size, local) else {
        return;
    };

    let cfg = &mut settings_state.editing_shake_config;
    if let Some(axis) = find_curve_point_axis(cfg, drag_id) {
        let points = curve_points_mut(cfg, axis);
        if let Some(i) = find_curve_point_index(points, drag_id) {
            let old_t = points[i].t;
            let old_v = points[i].value;
            points[i].t = t;
            points[i].value = v;

            // Move existing handles along with the point so they stay attached.
            let dt = points[i].t - old_t;
            let dv = points[i].value - old_v;
            if let Some(h) = points[i].in_handle {
                points[i].in_handle = Some(Vec2::new(
                    (h.x + dt).clamp(0.0, 1.0),
                    (h.y + dv).clamp(-1.0, 1.0),
                ));
            }
            if let Some(h) = points[i].out_handle {
                points[i].out_handle = Some(Vec2::new(
                    (h.x + dt).clamp(0.0, 1.0),
                    (h.y + dv).clamp(-1.0, 1.0),
                ));
            }
            sort_curve_points(points);
        }
    }
}

fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let omt = 1.0 - u;
    (omt * omt * omt) * p0
        + (3.0 * omt * omt * u) * p1
        + (3.0 * omt * u * u) * p2
        + (u * u * u) * p3
}

fn cubic_bezier_derivative(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    // d/du of cubic bezier
    let omt = 1.0 - u;
    3.0 * omt * omt * (p1 - p0) + 6.0 * omt * u * (p2 - p1) + 3.0 * u * u * (p3 - p2)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn sample_curve(points: &[ShakeCurvePoint], t: f32) -> f32 {
    if points.is_empty() {
        return 0.0;
    }
    if points.len() == 1 {
        return points[0].value;
    }

    // Non-looping start->finish curve.
    let t = t.clamp(0.0, 1.0);
    let mut points_sorted: std::borrow::Cow<'_, [ShakeCurvePoint]> =
        std::borrow::Cow::Borrowed(points);
    // If points are not sorted (should be), sort a copy.
    if !points.windows(2).all(|w| w[0].t <= w[1].t) {
        let mut tmp = points.to_vec();
        tmp.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(Ordering::Equal));
        points_sorted = std::borrow::Cow::Owned(tmp);
    }
    let points = points_sorted.as_ref();

    if t <= points[0].t {
        return points[0].value;
    }
    if t >= points[points.len() - 1].t {
        return points[points.len() - 1].value;
    }

    for w in points.windows(2) {
        let a = w[0];
        let b = w[1];
        if t >= a.t && t <= b.t {
            let dt = (b.t - a.t).max(0.0001);
            let initial_u = ((t - a.t) / dt).clamp(0.0, 1.0);

            // Resolve control points.
            let mut p1 = a
                .out_handle
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 1.0 / 3.0), a.value));
            let mut p2 = b
                .in_handle
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 2.0 / 3.0), b.value));

            // Clamp handle x within the segment to keep x(u) monotonic-ish.
            p1.x = p1.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p2.x = p2.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p1.y = p1.y.clamp(-1.0, 1.0);
            p2.y = p2.y.clamp(-1.0, 1.0);

            // Invert x(u) to find u such that x(u) == t.
            let mut u = initial_u;
            for _ in 0..8 {
                let x = cubic_bezier(a.t, p1.x, p2.x, b.t, u);
                let dx = cubic_bezier_derivative(a.t, p1.x, p2.x, b.t, u);
                if dx.abs() < 1e-5 {
                    break;
                }
                u = (u - (x - t) / dx).clamp(0.0, 1.0);
            }

            return cubic_bezier(a.value, p1.y, p2.y, b.value, u);
        }
    }

    points[0].value
}

/// Keep graph dots and point handles positioned to match the current curve.
pub fn sync_shake_curve_graph_ui(
    mut commands: Commands,
    theme: Res<MaterialTheme>,
    settings_state: Res<SettingsState>,
    graph: Query<(Entity, &ComputedNode), With<ShakeCurveGraphPlotRoot>>,
    mut dots: Query<
        (&ShakeCurveGraphDot, &mut Node),
        (
            Without<ShakeCurvePointHandle>,
            Without<ShakeCurveBezierHandle>,
        ),
    >,
    mut handles: Query<
        (
            Entity,
            &ShakeCurvePointHandle,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Without<ShakeCurveGraphDot>, Without<ShakeCurveBezierHandle>),
    >,
    mut bezier_handles: Query<
        (
            Entity,
            &ShakeCurveBezierHandle,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Without<ShakeCurvePointHandle>, Without<ShakeCurveGraphDot>),
    >,
    graph_children: Query<&Children>,
) {
    if !settings_state.show_modal {
        return;
    }

    let Some((graph_entity, computed)) = graph.iter().next() else {
        return;
    };

    // UI layout uses logical pixels (Val::Px). `ComputedNode::size()` is in physical pixels.
    // Convert once so point placement math matches cursor mapping under DPI scaling.
    let size_physical = computed.size();
    let size = size_physical * computed.inverse_scale_factor();
    if size.x <= 1.0 || size.y <= 1.0 {
        return;
    }
    let cfg = &settings_state.editing_shake_config;

    // Ensure handle entities exist for each curve point id; remove extras.
    let mut existing_ids: std::collections::HashMap<u64, Entity> = std::collections::HashMap::new();
    for (e, h, _node, _bg, _border) in handles.iter_mut() {
        existing_ids.insert(h.id, e);
    }

    let mut desired_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for axis in [ShakeAxis::X, ShakeAxis::Y, ShakeAxis::Z] {
        for p in curve_points(cfg, axis) {
            desired_ids.insert(p.id);
            if !existing_ids.contains_key(&p.id) {
                // spawn missing
                commands.entity(graph_entity).with_children(|graph| {
                    graph.spawn((
                        Button,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Px(14.0),
                            height: Val::Px(14.0),
                            ..default()
                        },
                        BackgroundColor(theme.surface_container_high),
                        BorderRadius::all(Val::Px(7.0)),
                        BorderColor::from(theme.outline_variant),
                        Interaction::None,
                        ShakeCurvePointHandle { id: p.id },
                    ));
                });
            }
        }
    }

    for (e, h, _node, _bg, _border) in handles.iter_mut() {
        if !desired_ids.contains(&h.id) {
            commands.entity(e).despawn();
        }
    }

    // Update handle positions/colors.
    let selected = settings_state.selected_shake_curve_point_id;
    for (_e, h, mut node, mut bg, mut border) in handles.iter_mut() {
        let Some(axis) = find_curve_point_axis(cfg, h.id) else {
            continue;
        };

        // Axis chips control which points are selectable; when disabled, hide the handle hitbox.
        if !axis_enabled(&settings_state, axis) {
            node.display = Display::None;
            continue;
        }
        node.display = Display::Flex;

        let axis_color = match axis {
            ShakeAxis::X => theme.primary,
            ShakeAxis::Y => theme.secondary,
            ShakeAxis::Z => theme.tertiary,
        };
        *border = BorderColor::all(axis_color);

        let Some(p) = curve_points(cfg, axis).iter().find(|p| p.id == h.id) else {
            continue;
        };
        let Some(pos) = shake_curve_t_v_to_local_px(size, p.t, p.value) else {
            continue;
        };
        node.left = Val::Px((pos.x - 7.0).clamp(0.0, (size.x - 14.0).max(0.0)));
        node.top = Val::Px((pos.y - 7.0).clamp(0.0, (size.y - 14.0).max(0.0)));
        *bg = if selected == Some(h.id) {
            BackgroundColor(axis_color)
        } else {
            BackgroundColor(theme.surface_container_high)
        };
    }

    // Spawn/despawn and position Bezier handles for the selected point.
    {
        use std::collections::{HashMap, HashSet};

        // Index existing Bezier handles.
        let mut existing: HashMap<(u64, ShakeCurveBezierHandleKind), Entity> = HashMap::new();
        for (e, h, _node, _bg, _border) in bezier_handles.iter_mut() {
            existing.insert((h.point_id, h.kind), e);
        }

        let mut desired: HashSet<(u64, ShakeCurveBezierHandleKind)> = HashSet::new();
        if let Some(sel_id) = selected {
            if let Some(axis) = find_curve_point_axis(cfg, sel_id) {
                let pts = curve_points(cfg, axis);
                if let Some(i) = pts.iter().position(|p| p.id == sel_id) {
                    if i > 0 {
                        desired.insert((sel_id, ShakeCurveBezierHandleKind::In));
                    }
                    if i + 1 < pts.len() {
                        desired.insert((sel_id, ShakeCurveBezierHandleKind::Out));
                    }

                    // Spawn any missing handle entities.
                    let axis_color = match axis {
                        ShakeAxis::X => theme.primary,
                        ShakeAxis::Y => theme.secondary,
                        ShakeAxis::Z => theme.tertiary,
                    };

                    for (pid, kind) in desired.iter().copied() {
                        if existing.contains_key(&(pid, kind)) {
                            continue;
                        }

                        commands.entity(graph_entity).with_children(|graph| {
                            graph.spawn((
                                Button,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    top: Val::Px(0.0),
                                    width: Val::Px(10.0),
                                    height: Val::Px(10.0),
                                    ..default()
                                },
                                BackgroundColor(theme.surface_container_high),
                                BorderRadius::all(Val::Px(5.0)),
                                BorderColor::all(axis_color),
                                Interaction::None,
                                ShakeCurveBezierHandle {
                                    point_id: pid,
                                    kind,
                                },
                            ));
                        });
                    }
                }
            }
        }

        // Despawn any stale Bezier handles.
        for (e, h, _node, _bg, _border) in bezier_handles.iter_mut() {
            if !desired.contains(&(h.point_id, h.kind)) {
                commands.entity(e).despawn();
            }
        }

        // Position Bezier handles that remain.
        if let Some(sel_id) = selected {
            if let Some(axis) = find_curve_point_axis(cfg, sel_id) {
                let pts = curve_points(cfg, axis);
                if let Some(i) = pts.iter().position(|p| p.id == sel_id) {
                    let p = pts[i];
                    let prev = if i > 0 { Some(pts[i - 1]) } else { None };
                    let next = if i + 1 < pts.len() {
                        Some(pts[i + 1])
                    } else {
                        None
                    };

                    let axis_color = match axis {
                        ShakeAxis::X => theme.primary,
                        ShakeAxis::Y => theme.secondary,
                        ShakeAxis::Z => theme.tertiary,
                    };

                    let default_in = prev.map(|a| {
                        let dt = (p.t - a.t).abs() * 0.25;
                        Vec2::new((p.t - dt).clamp(a.t.min(p.t), a.t.max(p.t)), p.value)
                    });
                    let default_out = next.map(|b| {
                        let dt = (b.t - p.t).abs() * 0.25;
                        Vec2::new((p.t + dt).clamp(p.t.min(b.t), p.t.max(b.t)), p.value)
                    });

                    for (_e, h, mut node, mut bg, mut border) in bezier_handles.iter_mut() {
                        if h.point_id != sel_id {
                            continue;
                        }

                        let handle_pos = match h.kind {
                            ShakeCurveBezierHandleKind::In => p
                                .in_handle
                                .or(default_in)
                                .unwrap_or(Vec2::new(p.t, p.value)),
                            ShakeCurveBezierHandleKind::Out => p
                                .out_handle
                                .or(default_out)
                                .unwrap_or(Vec2::new(p.t, p.value)),
                        };

                        let Some(pos) =
                            shake_curve_t_v_to_local_px(size, handle_pos.x, handle_pos.y)
                        else {
                            continue;
                        };
                        node.left = Val::Px((pos.x - 5.0).clamp(0.0, (size.x - 10.0).max(0.0)));
                        node.top = Val::Px((pos.y - 5.0).clamp(0.0, (size.y - 10.0).max(0.0)));

                        *border = BorderColor::all(axis_color);
                        *bg = BackgroundColor(theme.surface_container_high);
                    }
                }
            }
        }
    }

    for (dot, mut node) in dots.iter_mut() {
        // DOTS count is fixed at spawn time.
        let n = 80usize;
        let t = (dot.index as f32) / (n.saturating_sub(1) as f32).max(1.0);
        let v = sample_curve(curve_points(cfg, dot.axis), t).clamp(-1.0, 1.0);
        let Some(pos) = shake_curve_t_v_to_local_px(size, t, v) else {
            continue;
        };
        node.left = Val::Px((pos.x - 1.5).clamp(0.0, (size.x - 3.0).max(0.0)));
        node.top = Val::Px((pos.y - 1.5).clamp(0.0, (size.y - 3.0).max(0.0)));
    }

    // Ensure dots/handles are still children (in case of reparent issues).
    let _ = graph_children.get(graph_entity);
}

// ============================================================================
// Dice FX curve editor (Mask/Noise/Ramp) - mirrors shake curve UX
// ============================================================================

fn dice_fx_curve_t_v_to_local_px(graph_size: Vec2, t: f32, v: f32) -> Option<Vec2> {
    if graph_size.x <= 1.0 || graph_size.y <= 1.0 {
        return None;
    }

    const EDGE_PAD_PX: f32 = 7.0;
    let w = (graph_size.x - 2.0 * EDGE_PAD_PX).max(1.0);
    let h = (graph_size.y - 2.0 * EDGE_PAD_PX).max(1.0);

    let t = t.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let x = EDGE_PAD_PX + t * w;
    let y = EDGE_PAD_PX + (1.0 - v) * h;
    Some(Vec2::new(x, y))
}

fn dice_fx_graph_local_px_to_t_v(graph_size: Vec2, local: Vec2) -> Option<(f32, f32)> {
    if graph_size.x <= 1.0 || graph_size.y <= 1.0 {
        return None;
    }

    const EDGE_PAD_PX: f32 = 7.0;
    let w = (graph_size.x - 2.0 * EDGE_PAD_PX).max(1.0);
    let h = (graph_size.y - 2.0 * EDGE_PAD_PX).max(1.0);

    let lx = (local.x - EDGE_PAD_PX).clamp(0.0, w);
    let ly = (local.y - EDGE_PAD_PX).clamp(0.0, h);

    let t: f32 = (lx / w).clamp(0.0, 1.0);
    let v: f32 = (1.0 - (ly / h)).clamp(0.0, 1.0);
    Some((t, v))
}

fn dice_fx_channel_enabled(settings_state: &SettingsState, channel: DiceFxCurveChannel) -> bool {
    match channel {
        DiceFxCurveChannel::Mask => settings_state.dice_fx_curve_add_mask,
        DiceFxCurveChannel::Noise => settings_state.dice_fx_curve_add_noise,
        DiceFxCurveChannel::Ramp => settings_state.dice_fx_curve_add_ramp,
        DiceFxCurveChannel::Opacity => settings_state.dice_fx_curve_add_opacity,
        DiceFxCurveChannel::PlumeHeight => settings_state.dice_fx_curve_add_plume_height,
        DiceFxCurveChannel::PlumeRadius => settings_state.dice_fx_curve_add_plume_radius,
    }
}

fn dice_fx_curve_points(
    cfg: &CustomDiceFxSetting,
    channel: DiceFxCurveChannel,
) -> &[FxCurvePointSetting] {
    match channel {
        DiceFxCurveChannel::Mask => cfg.curve_points_mask.as_slice(),
        DiceFxCurveChannel::Noise => cfg.curve_points_noise.as_slice(),
        DiceFxCurveChannel::Ramp => cfg.curve_points_ramp.as_slice(),
        DiceFxCurveChannel::Opacity => cfg.curve_points_opacity.as_slice(),
        DiceFxCurveChannel::PlumeHeight => cfg.curve_points_plume_height.as_slice(),
        DiceFxCurveChannel::PlumeRadius => cfg.curve_points_plume_radius.as_slice(),
    }
}

fn dice_fx_curve_points_mut(
    cfg: &mut CustomDiceFxSetting,
    channel: DiceFxCurveChannel,
) -> &mut Vec<FxCurvePointSetting> {
    match channel {
        DiceFxCurveChannel::Mask => &mut cfg.curve_points_mask,
        DiceFxCurveChannel::Noise => &mut cfg.curve_points_noise,
        DiceFxCurveChannel::Ramp => &mut cfg.curve_points_ramp,
        DiceFxCurveChannel::Opacity => &mut cfg.curve_points_opacity,
        DiceFxCurveChannel::PlumeHeight => &mut cfg.curve_points_plume_height,
        DiceFxCurveChannel::PlumeRadius => &mut cfg.curve_points_plume_radius,
    }
}

fn sort_fx_curve_points(points: &mut [FxCurvePointSetting]) {
    points.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(Ordering::Equal));
}

fn find_fx_curve_point_channel(cfg: &CustomDiceFxSetting, id: u64) -> Option<DiceFxCurveChannel> {
    [
        DiceFxCurveChannel::Mask,
        DiceFxCurveChannel::Noise,
        DiceFxCurveChannel::Ramp,
        DiceFxCurveChannel::Opacity,
        DiceFxCurveChannel::PlumeHeight,
        DiceFxCurveChannel::PlumeRadius,
    ]
    .iter()
    .copied()
    .find(|ch| dice_fx_curve_points(cfg, *ch).iter().any(|p| p.id == id))
}

fn find_fx_curve_point_index(points: &[FxCurvePointSetting], id: u64) -> Option<usize> {
    points.iter().position(|p| p.id == id)
}

fn remove_fx_curve_point_by_id(cfg: &mut CustomDiceFxSetting, id: u64) -> bool {
    let mut removed = false;
    for ch in [
        DiceFxCurveChannel::Mask,
        DiceFxCurveChannel::Noise,
        DiceFxCurveChannel::Ramp,
        DiceFxCurveChannel::Opacity,
        DiceFxCurveChannel::PlumeHeight,
        DiceFxCurveChannel::PlumeRadius,
    ] {
        let pts = dice_fx_curve_points_mut(cfg, ch);
        let before = pts.len();
        pts.retain(|p| p.id != id);
        removed |= pts.len() != before;
    }
    removed
}

fn add_fx_curve_point(
    cfg: &mut CustomDiceFxSetting,
    channel: DiceFxCurveChannel,
    t: f32,
    value: f32,
) -> u64 {
    let new_id = cfg.next_curve_point_id;
    cfg.next_curve_point_id += 1;
    dice_fx_curve_points_mut(cfg, channel).push(FxCurvePointSetting {
        id: new_id,
        t: t.clamp(0.0, 1.0),
        value: value.clamp(0.0, 1.0),
        in_handle: None,
        out_handle: None,
    });
    sort_fx_curve_points(dice_fx_curve_points_mut(cfg, channel));
    new_id
}

fn find_nearest_fx_curve_point_id(
    cfg: &CustomDiceFxSetting,
    graph_size: Vec2,
    cursor_local: Vec2,
    settings_state: &SettingsState,
    threshold_px: f32,
) -> Option<u64> {
    let mut best: Option<(u64, f32, u8)> = None;

    let channel_rank = |ch: DiceFxCurveChannel| match ch {
        DiceFxCurveChannel::Mask => 0u8,
        DiceFxCurveChannel::Noise => 1u8,
        DiceFxCurveChannel::Ramp => 2u8,
        DiceFxCurveChannel::Opacity => 3u8,
        DiceFxCurveChannel::PlumeHeight => 4u8,
        DiceFxCurveChannel::PlumeRadius => 5u8,
    };

    for ch in [
        DiceFxCurveChannel::Mask,
        DiceFxCurveChannel::Noise,
        DiceFxCurveChannel::Ramp,
        DiceFxCurveChannel::Opacity,
        DiceFxCurveChannel::PlumeHeight,
        DiceFxCurveChannel::PlumeRadius,
    ] {
        if !dice_fx_channel_enabled(settings_state, ch) {
            continue;
        }
        for p in dice_fx_curve_points(cfg, ch) {
            let Some(pos) = dice_fx_curve_t_v_to_local_px(graph_size, p.t, p.value) else {
                continue;
            };
            let d = cursor_local.distance(pos);
            if d > threshold_px {
                continue;
            }

            let rank = channel_rank(ch);
            match best {
                None => best = Some((p.id, d, rank)),
                Some((_, best_d, best_rank)) => {
                    if d < best_d || (d == best_d && rank < best_rank) {
                        best = Some((p.id, d, rank));
                    }
                }
            }
        }
    }

    best.map(|(id, _, _)| id)
}

fn sample_fx_curve(points: &[FxCurvePointSetting], t: f32) -> f32 {
    if points.is_empty() {
        return 0.0;
    }
    if points.len() == 1 {
        return points[0].value;
    }

    let t = t.clamp(0.0, 1.0);

    let mut points_sorted: std::borrow::Cow<'_, [FxCurvePointSetting]> =
        std::borrow::Cow::Borrowed(points);
    if !points.windows(2).all(|w| w[0].t <= w[1].t) {
        let mut tmp = points.to_vec();
        tmp.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(Ordering::Equal));
        points_sorted = std::borrow::Cow::Owned(tmp);
    }
    let points = points_sorted.as_ref();

    if t <= points[0].t {
        return points[0].value;
    }
    if t >= points[points.len() - 1].t {
        return points[points.len() - 1].value;
    }

    for w in points.windows(2) {
        let a = &w[0];
        let b = &w[1];
        if t >= a.t && t <= b.t {
            let dt = (b.t - a.t).max(0.0001);
            let initial_u = ((t - a.t) / dt).clamp(0.0, 1.0);

            // Convert handle arrays -> Vec2.
            let mut p1 = a
                .out_handle
                .map(|h| Vec2::new(h[0], h[1]))
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 1.0 / 3.0), a.value));
            let mut p2 = b
                .in_handle
                .map(|h| Vec2::new(h[0], h[1]))
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 2.0 / 3.0), b.value));

            p1.x = p1.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p2.x = p2.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p1.y = p1.y.clamp(0.0, 1.0);
            p2.y = p2.y.clamp(0.0, 1.0);

            let mut u = initial_u;
            for _ in 0..8 {
                let x = cubic_bezier(a.t, p1.x, p2.x, b.t, u);
                let dx = cubic_bezier_derivative(a.t, p1.x, p2.x, b.t, u);
                if dx.abs() < 1e-5 {
                    break;
                }
                u = (u - (x - t) / dx).clamp(0.0, 1.0);
            }

            return cubic_bezier(a.value, p1.y, p2.y, b.value, u).clamp(0.0, 1.0);
        }
    }

    points[0].value
}

pub fn handle_dice_fx_curve_chip_clicks(
    mut click_events: MessageReader<ChipClickEvent>,
    edit_mode_chips: Query<&DiceFxCurveEditModeChip>,
    channel_chips: Query<&DiceFxCurveChannelChip>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !settings_state.show_modal {
        return;
    }

    for ev in click_events.read() {
        if let Ok(chip) = edit_mode_chips.get(ev.entity) {
            settings_state.dice_fx_curve_edit_mode =
                if settings_state.dice_fx_curve_edit_mode == chip.mode {
                    ShakeCurveEditMode::None
                } else {
                    chip.mode
                };

            if settings_state.dice_fx_curve_edit_mode == ShakeCurveEditMode::Delete {
                settings_state.dragging_dice_fx_curve_point_id = None;
                settings_state.dragging_dice_fx_curve_bezier = None;
            }
            continue;
        }

        if let Ok(chip) = channel_chips.get(ev.entity) {
            let channel = chip.channel;
            match channel {
                DiceFxCurveChannel::Mask => {
                    settings_state.dice_fx_curve_add_mask = !settings_state.dice_fx_curve_add_mask
                }
                DiceFxCurveChannel::Noise => {
                    settings_state.dice_fx_curve_add_noise = !settings_state.dice_fx_curve_add_noise
                }
                DiceFxCurveChannel::Ramp => {
                    settings_state.dice_fx_curve_add_ramp = !settings_state.dice_fx_curve_add_ramp
                }
                DiceFxCurveChannel::Opacity => {
                    settings_state.dice_fx_curve_add_opacity =
                        !settings_state.dice_fx_curve_add_opacity
                }
                DiceFxCurveChannel::PlumeHeight => {
                    settings_state.dice_fx_curve_add_plume_height =
                        !settings_state.dice_fx_curve_add_plume_height
                }
                DiceFxCurveChannel::PlumeRadius => {
                    settings_state.dice_fx_curve_add_plume_radius =
                        !settings_state.dice_fx_curve_add_plume_radius
                }
            }

            // Never allow all channels disabled.
            if !settings_state.dice_fx_curve_add_mask
                && !settings_state.dice_fx_curve_add_noise
                && !settings_state.dice_fx_curve_add_ramp
                && !settings_state.dice_fx_curve_add_opacity
                && !settings_state.dice_fx_curve_add_plume_height
                && !settings_state.dice_fx_curve_add_plume_radius
            {
                match channel {
                    DiceFxCurveChannel::Mask => settings_state.dice_fx_curve_add_mask = true,
                    DiceFxCurveChannel::Noise => settings_state.dice_fx_curve_add_noise = true,
                    DiceFxCurveChannel::Ramp => settings_state.dice_fx_curve_add_ramp = true,
                    DiceFxCurveChannel::Opacity => settings_state.dice_fx_curve_add_opacity = true,
                    DiceFxCurveChannel::PlumeHeight => {
                        settings_state.dice_fx_curve_add_plume_height = true
                    }
                    DiceFxCurveChannel::PlumeRadius => {
                        settings_state.dice_fx_curve_add_plume_radius = true
                    }
                }
            }

            // If the selected point is on a now-disabled channel, deselect it.
            if let Some(selected_id) = settings_state.selected_dice_fx_curve_point_id {
                if let Some(ch) =
                    find_fx_curve_point_channel(&settings_state.editing_custom_dice_fx, selected_id)
                {
                    if !dice_fx_channel_enabled(&settings_state, ch) {
                        settings_state.selected_dice_fx_curve_point_id = None;
                        if settings_state.dragging_dice_fx_curve_point_id == Some(selected_id) {
                            settings_state.dragging_dice_fx_curve_point_id = None;
                        }
                        settings_state.dragging_dice_fx_curve_bezier = None;
                    }
                }
            }
        }
    }
}

pub fn sync_dice_fx_curve_chip_ui(
    settings_state: Res<SettingsState>,
    mut edit_mode_chips: Query<
        (&DiceFxCurveEditModeChip, &mut MaterialChip),
        Without<DiceFxCurveChannelChip>,
    >,
    mut channel_chips: Query<
        (&DiceFxCurveChannelChip, &mut MaterialChip),
        Without<DiceFxCurveEditModeChip>,
    >,
) {
    if !settings_state.show_modal {
        return;
    }
    if !settings_state.is_changed() {
        return;
    }

    for (chip, mut material) in edit_mode_chips.iter_mut() {
        material.selected = settings_state.dice_fx_curve_edit_mode == chip.mode;
    }
    for (chip, mut material) in channel_chips.iter_mut() {
        material.selected = match chip.channel {
            DiceFxCurveChannel::Mask => settings_state.dice_fx_curve_add_mask,
            DiceFxCurveChannel::Noise => settings_state.dice_fx_curve_add_noise,
            DiceFxCurveChannel::Ramp => settings_state.dice_fx_curve_add_ramp,
            DiceFxCurveChannel::Opacity => settings_state.dice_fx_curve_add_opacity,
            DiceFxCurveChannel::PlumeHeight => settings_state.dice_fx_curve_add_plume_height,
            DiceFxCurveChannel::PlumeRadius => settings_state.dice_fx_curve_add_plume_radius,
        };
    }
}

pub fn handle_dice_fx_curve_graph_click_to_add_point(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<DiceFxCurveGraphPlotRoot>,
    >,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let mode = settings_state.dice_fx_curve_edit_mode;
    if settings_state.dragging_dice_fx_curve_point_id.is_some()
        || settings_state.dragging_dice_fx_curve_bezier.is_some()
    {
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let window = windows.iter().next();
    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    let cursor_local = if let Some(window) = window {
        let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
        cursor_in_ui_target.and_then(|c| {
            window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed)
        })
    } else {
        None
    };
    let Some(cursor_local) = cursor_local else {
        return;
    };
    if cursor_local.x < 0.0
        || cursor_local.y < 0.0
        || cursor_local.x > size.x
        || cursor_local.y > size.y
    {
        return;
    }

    let Some((t, v)) = dice_fx_graph_local_px_to_t_v(size, cursor_local) else {
        return;
    };

    let add_mask = settings_state.dice_fx_curve_add_mask;
    let add_noise = settings_state.dice_fx_curve_add_noise;
    let add_ramp = settings_state.dice_fx_curve_add_ramp;
    let add_opacity = settings_state.dice_fx_curve_add_opacity;
    let add_plume_height = settings_state.dice_fx_curve_add_plume_height;
    let add_plume_radius = settings_state.dice_fx_curve_add_plume_radius;

    match mode {
        ShakeCurveEditMode::Add => {
            if let Some(id) = find_nearest_fx_curve_point_id(
                &settings_state.editing_custom_dice_fx,
                size,
                cursor_local,
                &settings_state,
                16.0,
            ) {
                settings_state.selected_dice_fx_curve_point_id = Some(id);
                settings_state.dragging_dice_fx_curve_point_id = Some(id);
                return;
            }

            let new_selected = {
                let cfg = &mut settings_state.editing_custom_dice_fx;
                let mut new_selected: Option<u64> = None;
                if add_mask {
                    new_selected = Some(add_fx_curve_point(cfg, DiceFxCurveChannel::Mask, t, v));
                }
                if add_noise {
                    new_selected = Some(add_fx_curve_point(cfg, DiceFxCurveChannel::Noise, t, v));
                }
                if add_ramp {
                    new_selected = Some(add_fx_curve_point(cfg, DiceFxCurveChannel::Ramp, t, v));
                }
                if add_opacity {
                    new_selected = Some(add_fx_curve_point(cfg, DiceFxCurveChannel::Opacity, t, v));
                }
                if add_plume_height {
                    new_selected = Some(add_fx_curve_point(
                        cfg,
                        DiceFxCurveChannel::PlumeHeight,
                        t,
                        v,
                    ));
                }
                if add_plume_radius {
                    new_selected = Some(add_fx_curve_point(
                        cfg,
                        DiceFxCurveChannel::PlumeRadius,
                        t,
                        v,
                    ));
                }
                new_selected
            };

            if let Some(id) = new_selected {
                settings_state.selected_dice_fx_curve_point_id = Some(id);
                settings_state.dragging_dice_fx_curve_point_id = Some(id);
            }
        }
        ShakeCurveEditMode::Delete => {
            let removed_id = {
                let cfg = &mut settings_state.editing_custom_dice_fx;
                let mut best: Option<(u64, f32)> = None;
                let consider = |best: &mut Option<(u64, f32)>, id: u64, dist: f32| match best {
                    None => *best = Some((id, dist)),
                    Some((_, best_dist)) if dist < *best_dist => *best = Some((id, dist)),
                    _ => {}
                };

                let threshold = 22.0_f32;
                for ch in [
                    DiceFxCurveChannel::Mask,
                    DiceFxCurveChannel::Noise,
                    DiceFxCurveChannel::Ramp,
                    DiceFxCurveChannel::Opacity,
                    DiceFxCurveChannel::PlumeHeight,
                    DiceFxCurveChannel::PlumeRadius,
                ] {
                    let on = match ch {
                        DiceFxCurveChannel::Mask => add_mask,
                        DiceFxCurveChannel::Noise => add_noise,
                        DiceFxCurveChannel::Ramp => add_ramp,
                        DiceFxCurveChannel::Opacity => add_opacity,
                        DiceFxCurveChannel::PlumeHeight => add_plume_height,
                        DiceFxCurveChannel::PlumeRadius => add_plume_radius,
                    };
                    if !on {
                        continue;
                    }
                    for p in dice_fx_curve_points(cfg, ch) {
                        let Some(pos) = dice_fx_curve_t_v_to_local_px(size, p.t, p.value) else {
                            continue;
                        };
                        let d = cursor_local.distance(pos);
                        if d <= threshold {
                            consider(&mut best, p.id, d);
                        }
                    }
                }

                if let Some((id, _)) = best {
                    if remove_fx_curve_point_by_id(cfg, id) {
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(id) = removed_id {
                if settings_state.selected_dice_fx_curve_point_id == Some(id) {
                    settings_state.selected_dice_fx_curve_point_id = None;
                }
                if settings_state.dragging_dice_fx_curve_point_id == Some(id) {
                    settings_state.dragging_dice_fx_curve_point_id = None;
                }
            }
        }
        ShakeCurveEditMode::None => {
            if let Some(id) = find_nearest_fx_curve_point_id(
                &settings_state.editing_custom_dice_fx,
                size,
                cursor_local,
                &settings_state,
                16.0,
            ) {
                settings_state.selected_dice_fx_curve_point_id = Some(id);
                settings_state.dragging_dice_fx_curve_point_id = Some(id);
            } else {
                settings_state.selected_dice_fx_curve_point_id = None;
                settings_state.dragging_dice_fx_curve_point_id = None;
                settings_state.dragging_dice_fx_curve_bezier = None;
            }
        }
    }
}

pub fn handle_dice_fx_curve_point_press(
    mut settings_state: ResMut<SettingsState>,
    mut interactions: Query<(&Interaction, &DiceFxCurvePointHandle), Changed<Interaction>>,
    graph: Query<&Node, With<DiceFxCurveGraphPlotRoot>>,
) {
    if !settings_state.show_modal {
        return;
    }

    let Some(node) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    for (interaction, handle) in interactions.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if settings_state.dice_fx_curve_edit_mode == ShakeCurveEditMode::Delete {
            let id = handle.id;
            let removed = {
                let cfg = &mut settings_state.editing_custom_dice_fx;
                remove_fx_curve_point_by_id(cfg, id)
            };
            if removed {
                if settings_state.selected_dice_fx_curve_point_id == Some(id) {
                    settings_state.selected_dice_fx_curve_point_id = None;
                }
                if settings_state.dragging_dice_fx_curve_point_id == Some(id) {
                    settings_state.dragging_dice_fx_curve_point_id = None;
                }
                if settings_state
                    .dragging_dice_fx_curve_bezier
                    .map(|(pid, _)| pid)
                    == Some(id)
                {
                    settings_state.dragging_dice_fx_curve_bezier = None;
                }
            }
        } else {
            settings_state.selected_dice_fx_curve_point_id = Some(handle.id);
            settings_state.dragging_dice_fx_curve_point_id = Some(handle.id);
            settings_state.dragging_dice_fx_curve_bezier = None;
        }
    }
}

pub fn drag_dice_fx_curve_point(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<DiceFxCurveGraphPlotRoot>,
    >,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }
    if settings_state.dice_fx_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };
    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    if mouse.just_released(MouseButton::Left) {
        settings_state.dragging_dice_fx_curve_point_id = None;
        return;
    }

    let Some(drag_id) = settings_state.dragging_dice_fx_curve_point_id else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        settings_state.dragging_dice_fx_curve_point_id = None;
        return;
    }

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
    let local = cursor_in_ui_target
        .and_then(|c| window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed));
    let Some(local) = local else {
        return;
    };
    let Some((t, v)) = dice_fx_graph_local_px_to_t_v(size, local) else {
        return;
    };

    let cfg = &mut settings_state.editing_custom_dice_fx;
    if let Some(ch) = find_fx_curve_point_channel(cfg, drag_id) {
        let pts = dice_fx_curve_points_mut(cfg, ch);
        if let Some(i) = find_fx_curve_point_index(pts, drag_id) {
            let old_t = pts[i].t;
            let old_v = pts[i].value;
            pts[i].t = t;
            pts[i].value = v;

            let dt = pts[i].t - old_t;
            let dv = pts[i].value - old_v;
            if let Some(h) = pts[i].in_handle {
                pts[i].in_handle = Some([(h[0] + dt).clamp(0.0, 1.0), (h[1] + dv).clamp(0.0, 1.0)]);
            }
            if let Some(h) = pts[i].out_handle {
                pts[i].out_handle =
                    Some([(h[0] + dt).clamp(0.0, 1.0), (h[1] + dv).clamp(0.0, 1.0)]);
            }
            sort_fx_curve_points(pts);
        }
    }
}

pub fn handle_dice_fx_curve_bezier_handle_press(
    mut settings_state: ResMut<SettingsState>,
    mut interactions: Query<(&Interaction, &DiceFxCurveBezierHandle), Changed<Interaction>>,
) {
    if !settings_state.show_modal {
        return;
    }
    if settings_state.dice_fx_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    for (interaction, handle) in interactions.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        settings_state.selected_dice_fx_curve_point_id = Some(handle.point_id);
        settings_state.dragging_dice_fx_curve_point_id = None;
        settings_state.dragging_dice_fx_curve_bezier = Some((handle.point_id, handle.kind));
    }
}

pub fn drag_dice_fx_curve_bezier_handle(
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings_state: ResMut<SettingsState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    graph: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedUiTargetCamera,
            &Node,
        ),
        With<DiceFxCurveGraphPlotRoot>,
    >,
    cameras: Query<&Camera>,
) {
    if !settings_state.show_modal {
        return;
    }
    if settings_state.dice_fx_curve_edit_mode == ShakeCurveEditMode::Delete {
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        settings_state.dragging_dice_fx_curve_bezier = None;
        return;
    }

    let Some((point_id, kind)) = settings_state.dragging_dice_fx_curve_bezier else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) {
        settings_state.dragging_dice_fx_curve_bezier = None;
        return;
    }

    let Some((computed, transform, target_camera, node)) = graph.iter().next() else {
        return;
    };
    if node.display == Display::None {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };
    let ui_camera = target_camera.get().and_then(|e| cameras.get(e).ok());

    let size_physical = computed.size();
    let inv_sf = computed.inverse_scale_factor();
    let size = size_physical * inv_sf;
    let cursor_in_ui_target = ui_target_cursor_physical_px(window, ui_camera);
    let local = cursor_in_ui_target
        .and_then(|c| window_physical_cursor_to_ui_node_local_logical_px(c, transform, computed));
    let Some(local) = local else {
        return;
    };
    let Some((t, v)) = dice_fx_graph_local_px_to_t_v(size, local) else {
        return;
    };

    let cfg = &mut settings_state.editing_custom_dice_fx;
    let Some(ch) = find_fx_curve_point_channel(cfg, point_id) else {
        return;
    };
    let pts = dice_fx_curve_points_mut(cfg, ch);
    sort_fx_curve_points(pts);
    let Some(i) = find_fx_curve_point_index(pts, point_id) else {
        return;
    };

    let pt_t = pts[i].t;
    let prev_t = if i > 0 { pts[i - 1].t } else { pt_t };
    let next_t = if i + 1 < pts.len() {
        pts[i + 1].t
    } else {
        pt_t
    };

    let handle_t = match kind {
        ShakeCurveBezierHandleKind::In => t.clamp(prev_t.min(pt_t), pt_t.max(prev_t)),
        ShakeCurveBezierHandleKind::Out => t.clamp(pt_t.min(next_t), next_t.max(pt_t)),
    };
    let handle_pos = [handle_t, v.clamp(0.0, 1.0)];
    match kind {
        ShakeCurveBezierHandleKind::In => pts[i].in_handle = Some(handle_pos),
        ShakeCurveBezierHandleKind::Out => pts[i].out_handle = Some(handle_pos),
    }
}

pub fn sync_dice_fx_curve_graph_ui(
    mut commands: Commands,
    theme: Res<MaterialTheme>,
    settings_state: Res<SettingsState>,
    graph: Query<(Entity, &ComputedNode), With<DiceFxCurveGraphPlotRoot>>,
    mut dots: Query<
        (&DiceFxCurveGraphDot, &mut Node),
        (
            Without<DiceFxCurvePointHandle>,
            Without<DiceFxCurveBezierHandle>,
        ),
    >,
    mut handles: Query<
        (
            Entity,
            &DiceFxCurvePointHandle,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            Without<DiceFxCurveGraphDot>,
            Without<DiceFxCurveBezierHandle>,
        ),
    >,
    mut bezier_handles: Query<
        (
            Entity,
            &DiceFxCurveBezierHandle,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            Without<DiceFxCurvePointHandle>,
            Without<DiceFxCurveGraphDot>,
        ),
    >,
    graph_children: Query<&Children>,
) {
    if !settings_state.show_modal {
        return;
    }

    let Some((graph_entity, computed)) = graph.iter().next() else {
        return;
    };

    let size_physical = computed.size();
    let size = size_physical * computed.inverse_scale_factor();
    if size.x <= 1.0 || size.y <= 1.0 {
        return;
    }

    let cfg = &settings_state.editing_custom_dice_fx;

    // Ensure handle entities exist for each point id; remove extras.
    let mut existing_ids: std::collections::HashMap<u64, Entity> = std::collections::HashMap::new();
    for (e, h, _node, _bg, _border) in handles.iter_mut() {
        existing_ids.insert(h.id, e);
    }

    let mut desired_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for ch in [
        DiceFxCurveChannel::Mask,
        DiceFxCurveChannel::Noise,
        DiceFxCurveChannel::Ramp,
        DiceFxCurveChannel::Opacity,
        DiceFxCurveChannel::PlumeHeight,
        DiceFxCurveChannel::PlumeRadius,
    ] {
        for p in dice_fx_curve_points(cfg, ch) {
            desired_ids.insert(p.id);
            if !existing_ids.contains_key(&p.id) {
                commands.entity(graph_entity).with_children(|graph| {
                    graph.spawn((
                        Button,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Px(14.0),
                            height: Val::Px(14.0),
                            ..default()
                        },
                        BackgroundColor(theme.surface_container_high),
                        BorderRadius::all(Val::Px(7.0)),
                        BorderColor::from(theme.outline_variant),
                        Interaction::None,
                        DiceFxCurvePointHandle { id: p.id },
                    ));
                });
            }
        }
    }

    for (e, h, _node, _bg, _border) in handles.iter_mut() {
        if !desired_ids.contains(&h.id) {
            commands.entity(e).despawn();
        }
    }

    // Update handle positions/colors.
    let selected = settings_state.selected_dice_fx_curve_point_id;
    for (_e, h, mut node, mut bg, mut border) in handles.iter_mut() {
        let Some(ch) = find_fx_curve_point_channel(cfg, h.id) else {
            continue;
        };
        if !dice_fx_channel_enabled(&settings_state, ch) {
            node.display = Display::None;
            continue;
        }

        node.display = Display::Flex;

        let ch_color = match ch {
            DiceFxCurveChannel::Mask => theme.primary,
            DiceFxCurveChannel::Noise => theme.secondary,
            DiceFxCurveChannel::Ramp => theme.tertiary,
            DiceFxCurveChannel::Opacity => theme.primary,
            DiceFxCurveChannel::PlumeHeight => theme.secondary,
            DiceFxCurveChannel::PlumeRadius => theme.tertiary,
        };
        *border = BorderColor::all(ch_color);

        let Some(p) = dice_fx_curve_points(cfg, ch).iter().find(|p| p.id == h.id) else {
            continue;
        };
        let Some(pos) = dice_fx_curve_t_v_to_local_px(size, p.t, p.value) else {
            continue;
        };
        node.left = Val::Px((pos.x - 7.0).clamp(0.0, (size.x - 14.0).max(0.0)));
        node.top = Val::Px((pos.y - 7.0).clamp(0.0, (size.y - 14.0).max(0.0)));
        *bg = if selected == Some(h.id) {
            BackgroundColor(ch_color)
        } else {
            BackgroundColor(theme.surface_container_high)
        };
    }

    // Spawn/despawn and position Bezier handles for the selected point.
    {
        use std::collections::{HashMap, HashSet};

        let mut existing: HashMap<(u64, ShakeCurveBezierHandleKind), Entity> = HashMap::new();
        for (e, h, _node, _bg, _border) in bezier_handles.iter_mut() {
            existing.insert((h.point_id, h.kind), e);
        }

        let mut desired: HashSet<(u64, ShakeCurveBezierHandleKind)> = HashSet::new();
        if let Some(sel_id) = selected {
            if let Some(ch) = find_fx_curve_point_channel(cfg, sel_id) {
                if dice_fx_channel_enabled(&settings_state, ch) {
                    let pts = dice_fx_curve_points(cfg, ch);
                    if let Some(i) = pts.iter().position(|p| p.id == sel_id) {
                        if i > 0 {
                            desired.insert((sel_id, ShakeCurveBezierHandleKind::In));
                        }
                        if i + 1 < pts.len() {
                            desired.insert((sel_id, ShakeCurveBezierHandleKind::Out));
                        }

                        let ch_color = match ch {
                            DiceFxCurveChannel::Mask => theme.primary,
                            DiceFxCurveChannel::Noise => theme.secondary,
                            DiceFxCurveChannel::Ramp => theme.tertiary,
                            DiceFxCurveChannel::Opacity => theme.primary,
                            DiceFxCurveChannel::PlumeHeight => theme.secondary,
                            DiceFxCurveChannel::PlumeRadius => theme.tertiary,
                        };

                        for (pid, kind) in desired.iter().copied() {
                            if existing.contains_key(&(pid, kind)) {
                                continue;
                            }
                            commands.entity(graph_entity).with_children(|graph| {
                                graph.spawn((
                                    Button,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        width: Val::Px(10.0),
                                        height: Val::Px(10.0),
                                        ..default()
                                    },
                                    BackgroundColor(theme.surface_container_high),
                                    BorderRadius::all(Val::Px(5.0)),
                                    BorderColor::all(ch_color),
                                    Interaction::None,
                                    DiceFxCurveBezierHandle {
                                        point_id: pid,
                                        kind,
                                    },
                                ));
                            });
                        }
                    }
                }
            }
        }

        for (e, h, _node, _bg, _border) in bezier_handles.iter_mut() {
            if !desired.contains(&(h.point_id, h.kind)) {
                commands.entity(e).despawn();
            }
        }

        if let Some(sel_id) = selected {
            if let Some(ch) = find_fx_curve_point_channel(cfg, sel_id) {
                if !dice_fx_channel_enabled(&settings_state, ch) {
                    // Selected point isn't currently being edited/shown.
                    return;
                }
                let pts = dice_fx_curve_points(cfg, ch);
                if let Some(i) = pts.iter().position(|p| p.id == sel_id) {
                    let p = &pts[i];
                    let prev = if i > 0 { Some(&pts[i - 1]) } else { None };
                    let next = if i + 1 < pts.len() {
                        Some(&pts[i + 1])
                    } else {
                        None
                    };

                    let ch_color = match ch {
                        DiceFxCurveChannel::Mask => theme.primary,
                        DiceFxCurveChannel::Noise => theme.secondary,
                        DiceFxCurveChannel::Ramp => theme.tertiary,
                        DiceFxCurveChannel::Opacity => theme.primary,
                        DiceFxCurveChannel::PlumeHeight => theme.secondary,
                        DiceFxCurveChannel::PlumeRadius => theme.tertiary,
                    };

                    let default_in = prev.map(|a| {
                        let dt = (p.t - a.t).abs() * 0.25;
                        Vec2::new((p.t - dt).clamp(a.t.min(p.t), a.t.max(p.t)), p.value)
                    });
                    let default_out = next.map(|b| {
                        let dt = (b.t - p.t).abs() * 0.25;
                        Vec2::new((p.t + dt).clamp(p.t.min(b.t), p.t.max(b.t)), p.value)
                    });

                    for (_e, h, mut node, mut bg, mut border) in bezier_handles.iter_mut() {
                        if h.point_id != sel_id {
                            continue;
                        }

                        let handle_pos = match h.kind {
                            ShakeCurveBezierHandleKind::In => p
                                .in_handle
                                .map(|a| Vec2::new(a[0], a[1]))
                                .or(default_in)
                                .unwrap_or(Vec2::new(p.t, p.value)),
                            ShakeCurveBezierHandleKind::Out => p
                                .out_handle
                                .map(|a| Vec2::new(a[0], a[1]))
                                .or(default_out)
                                .unwrap_or(Vec2::new(p.t, p.value)),
                        };

                        let Some(pos) =
                            dice_fx_curve_t_v_to_local_px(size, handle_pos.x, handle_pos.y)
                        else {
                            continue;
                        };
                        node.left = Val::Px((pos.x - 5.0).clamp(0.0, (size.x - 10.0).max(0.0)));
                        node.top = Val::Px((pos.y - 5.0).clamp(0.0, (size.y - 10.0).max(0.0)));
                        *border = BorderColor::all(ch_color);
                        *bg = BackgroundColor(theme.surface_container_high);
                    }
                }
            }
        }
    }

    // Update dot positions.
    for (dot, mut node) in dots.iter_mut() {
        if !dice_fx_channel_enabled(&settings_state, dot.channel) {
            node.display = Display::None;
            continue;
        }

        node.display = Display::Flex;
        node.width = Val::Px(4.0);
        node.height = Val::Px(4.0);

        let n = 80usize;
        let t = (dot.index as f32) / (n.saturating_sub(1) as f32).max(1.0);
        let v = sample_fx_curve(dice_fx_curve_points(cfg, dot.channel), t).clamp(0.0, 1.0);
        let Some(pos) = dice_fx_curve_t_v_to_local_px(size, t, v) else {
            continue;
        };
        node.left = Val::Px((pos.x - 2.0).clamp(0.0, (size.x - 4.0).max(0.0)));
        node.top = Val::Px((pos.y - 2.0).clamp(0.0, (size.y - 4.0).max(0.0)));
    }

    let _ = graph_children.get(graph_entity);
}

/// Handle Cancel button click
pub fn handle_settings_cancel_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    cancel_query: Query<(), With<SettingsCancelButton>>,
    mut settings_state: ResMut<SettingsState>,
    mut theme: ResMut<MaterialTheme>,
    mut dice_query: Query<(&Die, &mut Transform)>,
) {
    for event in click_events.read() {
        if cancel_query.get(event.entity).is_err() {
            continue;
        }

        // Revert any live theme preview changes.
        apply_theme_override(&settings_state.settings, &mut theme);

        // Revert any live dice scale preview changes.
        let scales = settings_state.settings.dice_scales.clone();
        for (die, mut transform) in dice_query.iter_mut() {
            transform.scale = Vec3::splat(
                scales.scale_for(die.die_type) * die.die_type.uniform_size_scale_factor(),
            );
        }

        // Discard changes and close modal
        settings_state.show_modal = false;
        settings_state.modal_kind = crate::dice3d::types::ActiveModalKind::None;
    }
}

/// Reset panel layout to a predictable side-by-side arrangement under the results panel.
pub fn handle_settings_reset_layout_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    reset_query: Query<(), With<SettingsResetLayoutButton>>,
    mut settings_state: ResMut<SettingsState>,
    mut panel_nodes: ParamSet<(
        Query<&mut Node, With<crate::dice3d::types::SliderGroupRoot>>,
        Query<&mut Node, With<crate::dice3d::types::QuickRollPanel>>,
        Query<&mut Node, With<crate::dice3d::types::CommandHistoryPanelRoot>>,
        Query<&mut Node, With<crate::dice3d::types::ResultsPanelRoot>>,
    )>,
) {
    if !(settings_state.show_modal
        && settings_state.modal_kind == crate::dice3d::types::ActiveModalKind::DiceRollerSettings)
    {
        return;
    }

    for event in click_events.read() {
        if reset_query.get(event.entity).is_err() {
            continue;
        }

        // Layout: results stays top-left; other panels go below it side-by-side.
        // Note: y=230 is chosen to be "after results" with typical font sizes.
        settings_state.settings.results_panel_position.x = 10.0;
        settings_state.settings.results_panel_position.y = 50.0;

        let y = 230.0;
        settings_state.settings.slider_group_position.x = 10.0;
        settings_state.settings.slider_group_position.y = y;

        settings_state.settings.command_history_panel_position.x = 132.0;
        settings_state.settings.command_history_panel_position.y = y;

        settings_state.settings.quick_roll_panel_position.x = 342.0;
        settings_state.settings.quick_roll_panel_position.y = y;

        // Immediately move any spawned panels so the user sees it.
        if let Some(mut node) = panel_nodes.p3().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.results_panel_position.x);
            node.top = Val::Px(settings_state.settings.results_panel_position.y);
        }
        if let Some(mut node) = panel_nodes.p0().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.slider_group_position.x);
            node.top = Val::Px(settings_state.settings.slider_group_position.y);
        }
        if let Some(mut node) = panel_nodes.p2().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.command_history_panel_position.x);
            node.top = Val::Px(settings_state.settings.command_history_panel_position.y);
        }
        if let Some(mut node) = panel_nodes.p1().iter_mut().next() {
            node.left = Val::Px(settings_state.settings.quick_roll_panel_position.x);
            node.top = Val::Px(settings_state.settings.quick_roll_panel_position.y);
        }

        settings_state.is_modified = true;
    }
}

/// Handle RGBA slider changes
pub fn handle_color_slider_changes(
    mut events: MessageReader<SliderChangeEvent>,
    slider_query: Query<&ColorSlider>,
    mut settings_state: ResMut<SettingsState>,
) {
    if !settings_state.show_modal {
        return;
    }

    for event in events.read() {
        let Ok(slider) = slider_query.get(event.entity) else {
            continue;
        };

        let value = event.value.clamp(0.0, 1.0);
        match slider.component {
            ColorComponent::Alpha => settings_state.editing_color.a = value,
            ColorComponent::Red => settings_state.editing_color.r = value,
            ColorComponent::Green => settings_state.editing_color.g = value,
            ColorComponent::Blue => settings_state.editing_color.b = value,
        }

        settings_state.color_input_text = settings_state.editing_color.to_hex();
    }
}

/// Update color preview and slider handles when editing color changes
pub fn update_color_ui(
    settings_state: Res<SettingsState>,
    mut preview_queries: ParamSet<(
        Query<&mut BackgroundColor, With<ColorPreview>>,
        Query<&mut BackgroundColor, With<HighlightColorPreview>>,
    )>,
    mut slider_query: Query<(&ColorSlider, &mut MaterialSlider)>,
    mut label_query: Query<(&ColorValueLabel, &mut Text)>,
    mut input_queries: ParamSet<(
        Query<&mut MaterialTextField, With<ColorTextInput>>,
        Query<&mut MaterialTextField, With<HighlightColorTextInput>>,
        Query<&mut MaterialTextField, With<ThemeSeedTextInput>>,
    )>,
) {
    if !settings_state.is_changed() {
        return;
    }

    if !settings_state.show_modal {
        return;
    }

    let color = &settings_state.editing_color;
    let highlight_color = &settings_state.editing_highlight_color;

    // Update preview
    for mut bg in preview_queries.p0().iter_mut() {
        bg.0 = color.to_color();
    }

    for mut bg in preview_queries.p1().iter_mut() {
        bg.0 = highlight_color.to_color();
    }

    // Sync slider values
    for (slider, mut material_slider) in slider_query.iter_mut() {
        material_slider.value = match slider.component {
            ColorComponent::Alpha => color.a,
            ColorComponent::Red => color.r,
            ColorComponent::Green => color.g,
            ColorComponent::Blue => color.b,
        };
    }

    // Update value labels
    for (label, mut text) in label_query.iter_mut() {
        let value = match label.component {
            ColorComponent::Alpha => color.a,
            ColorComponent::Red => color.r,
            ColorComponent::Green => color.g,
            ColorComponent::Blue => color.b,
        };
        **text = format!("{:.2}", value);
    }

    // Sync text field value (avoid stomping while the user is typing)
    for mut field in input_queries.p0().iter_mut() {
        if field.focused {
            continue;
        }

        field.value = settings_state.color_input_text.clone();
        field.has_content = !field.value.is_empty();
        if ColorSetting::parse(&field.value).is_some() {
            field.error = false;
            field.error_text = None;
        } else {
            field.error = true;
            field.error_text = Some("Invalid color (hex, labeled, csv, or name)".to_string());
        }
        field.supporting_text = Some(
            "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray"
                .to_string(),
        );
    }

    // Sync highlight text field value (avoid stomping while the user is typing)
    for mut field in input_queries.p1().iter_mut() {
        if field.focused {
            continue;
        }

        field.value = settings_state.highlight_input_text.clone();
        field.has_content = !field.value.is_empty();
        if ColorSetting::parse(&field.value).is_some() {
            field.error = false;
            field.error_text = None;
        } else {
            field.error = true;
            field.error_text = Some("Invalid color (hex, labeled, csv, or name)".to_string());
        }
        field.supporting_text = Some(
            "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray"
                .to_string(),
        );
    }

    // Sync theme seed text field
    for mut field in input_queries.p2().iter_mut() {
        if field.focused {
            continue;
        }

        field.value = settings_state.theme_seed_input_text.clone();
        field.has_content = !field.value.is_empty();

        let trimmed = field.value.trim();
        if trimmed.is_empty() || ColorSetting::parse(trimmed).is_some() {
            field.error = false;
            field.error_text = None;
        } else {
            field.error = true;
            field.error_text = Some("Invalid color (hex or name)".to_string());
        }

        field.supporting_text = Some(
            "#RRGGBB, #AARRGGBB (alpha ignored), or a name like red/green/blue (converts to hex on commit)"
                .to_string(),
        );
    }
}

/// Handle keyboard input for color text field
pub fn handle_color_text_input(
    mut settings_state: ResMut<SettingsState>,
    mut change_events: MessageReader<TextFieldChangeEvent>,
    mut submit_events: MessageReader<TextFieldSubmitEvent>,
    mut field_queries: ParamSet<(
        Query<&mut MaterialTextField, With<ColorTextInput>>,
        Query<&mut MaterialTextField, With<HighlightColorTextInput>>,
        Query<&mut MaterialTextField, With<ThemeSeedTextInput>>,
    )>,
    mut theme: ResMut<MaterialTheme>,
) {
    if !settings_state.show_modal {
        return;
    }

    // Live update from the text field
    for ev in change_events.read() {
        if let Ok(mut field) = field_queries.p0().get_mut(ev.entity) {
            settings_state.color_input_text = ev.value.clone();

            if let Some(parsed) = ColorSetting::parse(&settings_state.color_input_text) {
                settings_state.editing_color = parsed;
                field.error = false;
                field.error_text = None;
                field.supporting_text = Some(
                    "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray"
                        .to_string(),
                );
            } else {
                field.error = true;
                field.error_text = Some("Invalid color (hex, labeled, csv, or name)".to_string());
            }

            continue;
        }

        if let Ok(mut field) = field_queries.p1().get_mut(ev.entity) {
            settings_state.highlight_input_text = ev.value.clone();

            if let Some(parsed) = ColorSetting::parse(&settings_state.highlight_input_text) {
                settings_state.editing_highlight_color = parsed;
                field.error = false;
                field.error_text = None;
                field.supporting_text = Some(
                    "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray"
                        .to_string(),
                );
            } else {
                field.error = true;
                field.error_text = Some("Invalid color (hex, labeled, csv, or name)".to_string());
            }

            continue;
        }

        if let Ok(mut field) = field_queries.p2().get_mut(ev.entity) {
            settings_state.theme_seed_input_text = ev.value.clone();

            let trimmed = settings_state.theme_seed_input_text.trim();
            if trimmed.is_empty() {
                settings_state.editing_theme_seed_override = None;
                field.error = false;
                field.error_text = None;

                let mode = theme.mode;
                *theme = match mode {
                    ThemeMode::Dark => MaterialTheme::dark(),
                    ThemeMode::Light => MaterialTheme::light(),
                };
            } else if let Some(mut parsed) = ColorSetting::parse(trimmed) {
                parsed.a = 1.0;
                let seed = parsed.to_color();
                settings_state.editing_theme_seed_override = Some(parsed);
                field.error = false;
                field.error_text = None;

                *theme = MaterialTheme::from_seed(seed, theme.mode);
            } else {
                field.error = true;
                field.error_text = Some("Invalid color (hex or name)".to_string());
            }
        }
    }

    // On submit (Enter), validate and apply preview.
    // We intentionally keep the user's entered text (e.g. "rebeccapurple") in the field
    // and only use the parsed color for preview / persistence.
    for ev in submit_events.read() {
        if let Ok(mut field) = field_queries.p0().get_mut(ev.entity) {
            if let Some(parsed) = ColorSetting::parse(&ev.value) {
                settings_state.editing_color = parsed;
                settings_state.color_input_text = ev.value.clone();

                field.value = settings_state.color_input_text.clone();
                field.has_content = !field.value.is_empty();
                field.error = false;
                field.error_text = None;
                field.supporting_text = Some(
                    "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray"
                        .to_string(),
                );
            } else {
                field.error = true;
                field.error_text = Some("Invalid color (hex, labeled, csv, or name)".to_string());
            }

            continue;
        }

        if let Ok(mut field) = field_queries.p1().get_mut(ev.entity) {
            if let Some(parsed) = ColorSetting::parse(&ev.value) {
                settings_state.editing_highlight_color = parsed;
                settings_state.highlight_input_text = ev.value.clone();

                field.value = settings_state.highlight_input_text.clone();
                field.has_content = !field.value.is_empty();
                field.error = false;
                field.error_text = None;
                field.supporting_text = Some(
                    "#AARRGGBB, A:1 R:0.5 G:0.3 B:0.2, 1,0.5,0.3,0.2, or a name like rebeccapurple/light gray"
                        .to_string(),
                );
            } else {
                field.error = true;
                field.error_text = Some("Invalid color (hex, labeled, csv, or name)".to_string());
            }

            continue;
        }

        if let Ok(mut field) = field_queries.p2().get_mut(ev.entity) {
            let trimmed = ev.value.trim();
            if trimmed.is_empty() {
                settings_state.theme_seed_input_text.clear();
                settings_state.editing_theme_seed_override = None;

                field.value.clear();
                field.has_content = false;
                field.error = false;
                field.error_text = None;

                let mode = theme.mode;
                *theme = match mode {
                    ThemeMode::Dark => MaterialTheme::dark(),
                    ThemeMode::Light => MaterialTheme::light(),
                };
            } else if let Some(mut parsed) = ColorSetting::parse(trimmed) {
                parsed.a = 1.0;
                let seed = parsed.to_color();

                // Keep the user's entered text (e.g. "red") in the field.
                settings_state.theme_seed_input_text = trimmed.to_string();
                settings_state.editing_theme_seed_override = Some(parsed);

                field.value = settings_state.theme_seed_input_text.clone();
                field.has_content = true;
                field.error = false;
                field.error_text = None;

                *theme = MaterialTheme::from_seed(seed, theme.mode);
            } else {
                field.error = true;
                field.error_text = Some("Invalid color (hex or name)".to_string());
            }
        }
    }
}

/// Handle keyboard input for the shake duration text field.
pub fn handle_shake_duration_text_input(
    mut settings_state: ResMut<SettingsState>,
    mut change_events: MessageReader<TextFieldChangeEvent>,
    mut submit_events: MessageReader<TextFieldSubmitEvent>,
    mut field_query: Query<&mut MaterialTextField, With<ShakeDurationTextInput>>,
) {
    if !settings_state.show_modal {
        return;
    }

    for ev in change_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };

        settings_state.shake_duration_input_text = ev.value.clone();

        let parsed = ev.value.trim().parse::<f32>();
        if let Ok(mut seconds) = parsed {
            if seconds.is_finite() && seconds > 0.0 {
                seconds = seconds.max(0.01);
                settings_state.editing_shake_config.duration_seconds = seconds;
                field.error = false;
                field.error_text = None;
            } else {
                field.error = true;
                field.error_text = Some("Enter a positive number".to_string());
            }
        } else {
            field.error = true;
            field.error_text = Some("Enter a number".to_string());
        }
    }

    for ev in submit_events.read() {
        let Ok(mut field) = field_query.get_mut(ev.entity) else {
            continue;
        };

        let parsed = ev.value.trim().parse::<f32>();
        if let Ok(mut seconds) = parsed {
            if seconds.is_finite() && seconds > 0.0 {
                seconds = seconds.max(0.01);
                settings_state.editing_shake_config.duration_seconds = seconds;
                settings_state.shake_duration_input_text = format!("{:.3}", seconds);

                field.value = settings_state.shake_duration_input_text.clone();
                field.has_content = !field.value.is_empty();
                field.error = false;
                field.error_text = None;
            } else {
                field.error = true;
                field.error_text = Some("Enter a positive number".to_string());
            }
        } else {
            field.error = true;
            field.error_text = Some("Enter a number".to_string());
        }
    }
}

/// Apply settings on startup
pub fn apply_initial_settings(
    settings_state: Res<SettingsState>,
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = settings_state.settings.background_color.to_color();

    info!(
        "Applied startup background color: {}",
        settings_state.settings.background_color.to_hex()
    );
}
