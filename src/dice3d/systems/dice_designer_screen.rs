//! Dice Designer Screen
//!
//! The main UI for customizing dice face textures. Layout:
//! - Left panel: List of dice types (D4, D6, D8, D10, D12, D20)
//! - Right panel: Settings for selected die
//!   - 3D preview with rotation gizmo (300x300px)
//!   - Divider
//!   - Per-face texture inputs (color, depth, normal)

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{IoTaskPool, Task, poll_once, block_on};
use bevy_material_ui::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::dice3d::systems::dice_preview::{spawn_dice_preview_ui, DiceDesignerPreviewRenderTarget};
use crate::dice3d::types::{
    DiceDesignerListItem, DiceDesignerListPanel, DiceDesignerScreenRoot,
    DiceDesignerSettingsPanel, DiceDesignerState, DiceDesignerTextureSection,
    DiceType, FaceSelectorDropdown, FaceSelectorItem, TextureFileInput,
    TexturePreviewImage, TextureType,
};

/// Height of the tab bar (matches other screens)
const TAB_BAR_HEIGHT: f32 = 48.0;

// ============================================================================
// File Picker Task Component
// ============================================================================

/// Component for tracking an async file picker task
#[derive(Component)]
pub struct FilePickerTask {
    pub task: Task<Option<PathBuf>>,
    pub texture_type: TextureType,
    pub face_value: Option<u32>,
    pub die_type: DiceType,
}

// ============================================================================
// Local Texture Loader (filesystem -> Bevy Image)
// ============================================================================

/// Background loader for images selected via the native file picker.
///
/// We cannot use `AssetServer::load(PathBuf)` here because Bevy rejects
/// unapproved/absolute paths. Instead, we read/decode bytes and register
/// the resulting `Image` directly into `Assets<Image>`.
#[derive(Resource, Default)]
pub struct DiceDesignerTextureLoader {
    loading: HashMap<PathBuf, ()>,
    completed: Arc<Mutex<Vec<CompletedTexture>>>,
    failed: Arc<Mutex<Vec<PathBuf>>>,
    pub cache: HashMap<PathBuf, Handle<Image>>,
    pub failed_paths: HashMap<PathBuf, ()>,
}

struct CompletedTexture {
    path: PathBuf,
    image_data: Vec<u8>,
    width: u32,
    height: u32,
    srgb: bool,
}

impl DiceDesignerTextureLoader {
    pub fn request(&mut self, path: &PathBuf, srgb: bool) {
        if self.loading.contains_key(path)
            || self.cache.contains_key(path)
            || self.failed_paths.contains_key(path)
        {
            return;
        }

        self.loading.insert(path.clone(), ());

        let path_clone = path.clone();
        let completed = Arc::clone(&self.completed);
        let failed = Arc::clone(&self.failed);

        thread::spawn(move || {
            let bytes = match std::fs::read(&path_clone) {
                Ok(b) => b,
                Err(_) => {
                    failed.lock().unwrap().push(path_clone);
                    return;
                }
            };

            let img = match image::load_from_memory(&bytes) {
                Ok(img) => img,
                Err(_) => {
                    failed.lock().unwrap().push(path_clone);
                    return;
                }
            };

            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();

            completed.lock().unwrap().push(CompletedTexture {
                path: path_clone,
                image_data: rgba.into_raw(),
                width,
                height,
                srgb,
            });
        });
    }

    pub fn get(&self, path: &PathBuf) -> Option<Handle<Image>> {
        self.cache.get(path).cloned()
    }

    pub fn is_loading(&self, path: &PathBuf) -> bool {
        self.loading.contains_key(path)
    }

    pub fn has_failed(&self, path: &PathBuf) -> bool {
        self.failed_paths.contains_key(path)
    }
}

/// Converts completed decoded images into Bevy `Image` assets.
pub fn process_dice_designer_texture_loads(
    mut loader: ResMut<DiceDesignerTextureLoader>,
    mut images: ResMut<Assets<Image>>,
) {
    let completed: Vec<CompletedTexture> = {
        let mut lock = loader.completed.lock().unwrap();
        std::mem::take(&mut *lock)
    };

    let failed: Vec<PathBuf> = {
        let mut lock = loader.failed.lock().unwrap();
        std::mem::take(&mut *lock)
    };

    for tex in completed {
        loader.loading.remove(&tex.path);

        let format = if tex.srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        };

        let image = Image::new(
            Extent3d {
                width: tex.width,
                height: tex.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            tex.image_data,
            format,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        let handle = images.add(image);
        loader.cache.insert(tex.path, handle);
    }

    for path in failed {
        loader.loading.remove(&path);
        loader.failed_paths.insert(path, ());
    }
}

// ============================================================================
// Setup
// ============================================================================

/// Sets up the Dice Designer screen UI
pub fn setup_dice_designer_screen(
    mut commands: Commands,
    theme: Option<Res<MaterialTheme>>,
    render_target: Option<Res<DiceDesignerPreviewRenderTarget>>,
    designer_state: Option<Res<DiceDesignerState>>,
) {
    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let render_target_ref = render_target.as_deref();
    let selected_dice = designer_state
        .as_deref()
        .map(|s| s.selected_dice)
        .unwrap_or(DiceType::D4);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(TAB_BAR_HEIGHT),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(theme.surface),
            // Start hidden; tab visibility system shows it when selected
            Visibility::Hidden,
            DiceDesignerScreenRoot,
        ))
        .with_children(|root| {
            // Left panel - dice type list
            spawn_dice_list_panel(root, &theme, selected_dice);

            // Vertical divider
            root.spawn((
                Node {
                    width: Val::Px(1.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(theme.outline_variant),
            ));

            // Right panel - settings for selected die
            spawn_settings_panel(root, &theme, render_target_ref);
        });
}

/// Spawns the left panel with dice type list
fn spawn_dice_list_panel(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    selected_dice: DiceType,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme.surface_container),
            DiceDesignerListPanel,
        ))
        .with_children(|panel| {
            // Header
            panel.spawn((
                Text::new("Dice Types"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(theme.on_surface),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // List items for each dice type
            for die_type in DiceType::all() {
                spawn_dice_list_item(panel, theme, die_type, die_type == selected_dice);
            }
        });
}

/// Spawns a single dice type list item
fn spawn_dice_list_item(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    die_type: DiceType,
    is_selected: bool,
) {
    let bg_color = if is_selected {
        theme.secondary_container
    } else {
        Color::NONE
    };

    let text_color = if is_selected {
        theme.on_secondary_container
    } else {
        theme.on_surface
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(8.0)),
            DiceDesignerListItem { die_type },
        ))
        .with_child((
            Text::new(die_type.display_name()),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(text_color),
        ));
}

/// Spawns the right panel with settings
fn spawn_settings_panel(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    render_target: Option<&DiceDesignerPreviewRenderTarget>,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme.surface),
            DiceDesignerSettingsPanel,
        ))
        .with_children(|panel| {
            // Section: 3D Preview
            spawn_preview_section(panel, theme, render_target);

            // Divider
            panel.spawn((
                MaterialDivider::new(),
                Node {
                    width: Val::Percent(100.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Section: Face selector
            spawn_face_selector(panel, theme);

            // Section: Texture inputs
            spawn_texture_inputs_section(panel, theme);
        });
}

/// Spawns the 3D preview section
fn spawn_preview_section(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    render_target: Option<&DiceDesignerPreviewRenderTarget>,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|section| {
            // Label
            section.spawn((
                Text::new("3D Preview"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));

            // Preview container (300x300)
            spawn_dice_preview_ui(section, theme.surface_container, render_target);

            // Auto-rotate toggle
            section
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new("Auto-rotate"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.on_surface_variant),
                    ));
                    // Note: Would add a MaterialSwitch here for auto-rotate toggle
                });
        });
}

/// Spawns the face selector dropdown
fn spawn_face_selector(parent: &mut ChildSpawnerCommands, theme: &MaterialTheme) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|section| {
            section.spawn((
                Text::new("Select Face"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));

            // Dropdown container (simplified - would use MaterialSelect in full impl)
            section
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    FaceSelectorDropdown,
                ))
                .with_children(|dropdown| {
                    // "All faces" option
                    spawn_face_selector_chip(dropdown, theme, None, true);

                    // Individual face options (will be populated based on selected die)
                    for face in 1..=6 {
                        spawn_face_selector_chip(dropdown, theme, Some(face), false);
                    }
                });
        });
}

/// Spawns a face selector chip button
fn spawn_face_selector_chip(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    face_value: Option<u32>,
    is_selected: bool,
) {
    let label = face_value
        .map(|v| format!("{}", v))
        .unwrap_or_else(|| "All".to_string());

    let bg_color = if is_selected {
        theme.primary_container
    } else {
        theme.surface_container_high
    };

    let text_color = if is_selected {
        theme.on_primary_container
    } else {
        theme.on_surface
    };

    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(16.0)),
            FaceSelectorItem { face_value },
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(text_color),
        ));
}

/// Spawns the texture inputs section
fn spawn_texture_inputs_section(parent: &mut ChildSpawnerCommands, theme: &MaterialTheme) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            DiceDesignerTextureSection,
        ))
        .with_children(|section| {
            section.spawn((
                Text::new("Face Textures"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));

            // Color map input
            spawn_texture_input_row(section, theme, TextureType::Color, None);

            // Depth map input
            spawn_texture_input_row(section, theme, TextureType::Depth, None);

            // Normal map input
            spawn_texture_input_row(section, theme, TextureType::Normal, None);
        });
}

/// Spawns a texture input row with file picker and preview
fn spawn_texture_input_row(
    parent: &mut ChildSpawnerCommands,
    theme: &MaterialTheme,
    texture_type: TextureType,
    face_value: Option<u32>,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
        ))
        .with_children(|row| {
            // Left side: Label + file input
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    width: Val::Percent(50.0),
                    ..default()
                },
            ))
            .with_children(|left| {
                // Label
                left.spawn((
                    Text::new(texture_type.display_name()),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.on_surface),
                ));

                // Description
                left.spawn((
                    Text::new(texture_type.description()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(theme.on_surface_variant),
                ));

                // File input button (simplified - would use MaterialTextField)
                left.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(theme.surface_container_high),
                    BorderRadius::all(Val::Px(4.0)),
                    TextureFileInput {
                        texture_type,
                        face_value,
                    },
                ))
                .with_child((
                    Text::new("Select PNG..."),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(theme.primary),
                ));
            });

            // Right side: Texture preview
            row.spawn((
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(theme.surface_container),
                BorderColor::all(theme.outline_variant),
                BorderRadius::all(Val::Px(4.0)),
                ImageNode::default(), // For displaying selected texture
                TexturePreviewImage {
                    texture_type,
                    face_value,
                },
            ))
            .with_child((
                Text::new("No image"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(theme.on_surface_variant),
            ));
        });
}

// ============================================================================
// Event Handlers
// ============================================================================

/// Handles clicks on dice list items
pub fn handle_dice_list_clicks(
    mut designer_state: Option<ResMut<DiceDesignerState>>,
    list_item_query: Query<(&Interaction, &DiceDesignerListItem), Changed<Interaction>>,
    mut list_items: Query<(&DiceDesignerListItem, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut TextColor>,
    theme: Option<Res<MaterialTheme>>,
) {
    let Some(ref mut state) = designer_state else {
        return;
    };
    let theme = theme.map(|t| t.clone()).unwrap_or_default();

    for (interaction, clicked_item) in list_item_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let new_die_type = clicked_item.die_type;
        if state.selected_dice == new_die_type {
            continue;
        }

        state.selected_dice = new_die_type;
        state.selected_face = None; // Reset face selection when changing dice

        // Update visual selection state
        for (item, mut bg_color, children) in list_items.iter_mut() {
            let is_selected = item.die_type == new_die_type;
            *bg_color = if is_selected {
                BackgroundColor(theme.secondary_container)
            } else {
                BackgroundColor(Color::NONE)
            };

            // Update text color
            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = if is_selected {
                        theme.on_secondary_container
                    } else {
                        theme.on_surface
                    };
                }
            }
        }
    }
}

/// Handles clicks on face selector chips
pub fn handle_face_selector_clicks(
    mut designer_state: Option<ResMut<DiceDesignerState>>,
    selector_query: Query<(&Interaction, &FaceSelectorItem), Changed<Interaction>>,
    mut all_selectors: Query<(&FaceSelectorItem, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut TextColor>,
    theme: Option<Res<MaterialTheme>>,
) {
    let Some(ref mut state) = designer_state else {
        return;
    };
    let theme = theme.map(|t| t.clone()).unwrap_or_default();

    for (interaction, clicked_item) in selector_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        state.selected_face = clicked_item.face_value;

        // Update visual selection state
        for (item, mut bg_color, children) in all_selectors.iter_mut() {
            let is_selected = item.face_value == state.selected_face;
            *bg_color = if is_selected {
                BackgroundColor(theme.primary_container)
            } else {
                BackgroundColor(theme.surface_container_high)
            };

            for child in children.iter() {
                if let Ok(mut text_color) = text_query.get_mut(child) {
                    text_color.0 = if is_selected {
                        theme.on_primary_container
                    } else {
                        theme.on_surface
                    };
                }
            }
        }
    }
}

/// Updates the face selector chips when the selected dice type changes
pub fn update_face_selector_for_dice(
    mut commands: Commands,
    designer_state: Option<Res<DiceDesignerState>>,
    dropdown_query: Query<(Entity, &Children), With<FaceSelectorDropdown>>,
    theme: Option<Res<MaterialTheme>>,
) {
    let Some(state) = designer_state else {
        return;
    };

    if !state.is_changed() {
        return;
    }

    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let face_count = state.selected_dice.face_count();

    for (dropdown_entity, children) in dropdown_query.iter() {
        // Despawn existing children
        for child in children.iter() {
            commands.entity(child).despawn();
        }

        // Rebuild face selector chips
        commands.entity(dropdown_entity).with_children(|dropdown| {
            // "All faces" option
            spawn_face_selector_chip(dropdown, &theme, None, state.selected_face.is_none());

            // Individual face options
            for face in 1..=face_count {
                let is_selected = state.selected_face == Some(face);
                spawn_face_selector_chip(dropdown, &theme, Some(face), is_selected);
            }
        });
    }
}

/// Handles texture file input button clicks (opens file picker)
pub fn handle_texture_file_clicks(
    mut commands: Commands,
    texture_input_query: Query<(&Interaction, &TextureFileInput), Changed<Interaction>>,
    designer_state: Option<Res<DiceDesignerState>>,
) {
    let Some(state) = designer_state else {
        return;
    };

    for (interaction, input) in texture_input_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let texture_type = input.texture_type;
        let face_value = input.face_value.or(state.selected_face);
        let die_type = state.selected_dice;

        // Spawn an async task to open the file dialog
        let task_pool = IoTaskPool::get();
        let task = task_pool.spawn(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("PNG Image", &["png"])
                .add_filter("All Images", &["png", "jpg", "jpeg", "bmp", "tga"])
                .set_title(format!("Select {} Texture", texture_type.display_name()))
                .pick_file()
                .await;

            file.map(|f| f.path().to_path_buf())
        });

        // Spawn entity to track this task
        commands.spawn(FilePickerTask {
            task,
            texture_type,
            face_value,
            die_type,
        });

        info!(
            "Opening file picker for {:?} - face {:?}",
            texture_type, face_value
        );
    }
}

/// Polls file picker tasks and handles completed selections
pub fn poll_file_picker_tasks(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut FilePickerTask)>,
    mut designer_state: Option<ResMut<DiceDesignerState>>,
    mut texture_loader: ResMut<DiceDesignerTextureLoader>,
) {
    let Some(ref mut state) = designer_state else {
        return;
    };

    for (entity, mut task) in task_query.iter_mut() {
        if let Some(result) = block_on(poll_once(&mut task.task)) {
            // Task completed - despawn the task entity
            commands.entity(entity).despawn();

            if let Some(path) = result {
                info!(
                    "File selected: {:?} for {:?} face {:?}",
                    path, task.texture_type, task.face_value
                );

                // Start async load for UI previews (filesystem -> decoded image).
                let srgb = matches!(task.texture_type, TextureType::Color);
                texture_loader.request(&path, srgb);

                // Update the state with the selected file path
                if let Some(config) = state.dice_configs.get_mut(&task.die_type) {
                    let texture_set = if let Some(face) = task.face_value {
                        config.face_textures.entry(face).or_default()
                    } else {
                        &mut config.default_textures
                    };

                    match task.texture_type {
                        TextureType::Color => texture_set.color_path = Some(path),
                        TextureType::Depth => texture_set.depth_path = Some(path),
                        TextureType::Normal => texture_set.normal_path = Some(path),
                    }
                }
            }
        }
    }
}

/// Updates texture preview images when paths change
pub fn update_texture_previews(
    designer_state: Option<Res<DiceDesignerState>>,
    mut texture_loader: ResMut<DiceDesignerTextureLoader>,
    mut preview_query: Query<(
        &TexturePreviewImage,
        &mut BackgroundColor,
        &mut ImageNode,
        &Children,
    )>,
    mut text_query: Query<&mut Text>,
) {
    let Some(state) = designer_state else {
        return;
    };

    let config = match state.dice_configs.get(&state.selected_dice) {
        Some(c) => c,
        None => return,
    };

    for (preview, mut bg_color, mut image_node, children) in preview_query.iter_mut() {
        // Get the texture set for this preview's face
        let texture_set = if let Some(face) = preview.face_value {
            config.face_textures.get(&face)
        } else {
            Some(&config.default_textures)
        };

        let path = texture_set.and_then(|ts| match preview.texture_type {
            TextureType::Color => ts.color_path.as_ref(),
            TextureType::Depth => ts.depth_path.as_ref(),
            TextureType::Normal => ts.normal_path.as_ref(),
        });

        if let Some(path) = path {
            let srgb = matches!(preview.texture_type, TextureType::Color);

            // If not already loading/cached, request load.
            if texture_loader.get(path).is_none()
                && !texture_loader.is_loading(path)
                && !texture_loader.has_failed(path)
            {
                texture_loader.request(path, srgb);
            }

            if let Some(handle) = texture_loader.get(path) {
                image_node.image = handle;
                *bg_color = BackgroundColor(Color::WHITE);

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Image");
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = filename.to_string();
                    }
                }
            } else {
                image_node.image = Handle::default();
                let label = if texture_loader.has_failed(path) {
                    "Failed to load"
                } else {
                    "Loading..."
                };
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = label.to_string();
                    }
                }
            }
        } else {
            // No image - show placeholder
            image_node.image = Handle::default();
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = "No image".to_string();
                }
            }
        }
    }
}
