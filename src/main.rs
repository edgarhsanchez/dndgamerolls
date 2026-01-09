// Hide console window on Windows for release builds (GUI app).
// In debug builds, keep the console so panics/backtraces are visible.
// CLI mode will re-attach to parent console if available.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy_hanabi::prelude::HanabiPlugin;
use bevy_material_ui::prelude::*;
use bevy_rapier3d::prelude::*;
use clap::{Parser, Subcommand};
use colored::Colorize;
use rand::Rng;

use dndgamerolls::dice3d::{
    animate_container_shake,
    apply_crystal_material_to_container_models,
    apply_dice_scale_settings_to_existing_dice,
    apply_editing_dice_scales_to_existing_dice_while_open,
    apply_initial_settings,
    apply_initial_shake_config,
    apply_spawn_points_to_dice_when_ready,
    autosave_and_apply_shake_config,
    cache_dice_box_lid_animation_player,
    center_container_models_in_view,
    check_dice_settled,
    collect_dice_spawn_points_from_gltf,
    drag_shake_curve_bezier_handle,
    drag_shake_curve_point,
    DiceDesignerTextureLoader,
    ensure_buttons_have_interaction,
    ensure_dice_box_lid_animation_assets,
    // Legacy SQLite -> SurrealDB conversion (character screen)
    finalize_sqlite_conversion_if_done,
    fix_dice_scale_slider_thumb_hitbox,
    handle_character_list_clicks,
    handle_character_sheet_die_type_select_change,
    handle_character_sheet_settings_button_click,
    handle_character_sheet_settings_cancel_click,
    handle_character_sheet_settings_save_click,
    handle_color_slider_changes,
    handle_color_text_input,
    handle_command_history_item_clicks,
    handle_command_input,
    handle_default_roll_uses_shake_switch_change,
    handle_delete_click,
    handle_dice_box_rotate_click,
    handle_dice_box_shake_box_click,
    handle_dice_box_toggle_container_click,
    handle_dice_fx_param_slider_changes,
    handle_dice_roll_fx_mapping_select_change,
    handle_dice_scale_slider_changes,
    handle_dice_list_clicks,
    handle_face_selector_clicks,
    update_face_selector_for_dice,
    handle_master_volume_slider_change,
    handle_expertise_toggle,
    handle_proficiency_toggle,
    handle_group_add_click,
    handle_group_edit_toggle,
    handle_input,
    handle_label_click,
    handle_new_character_click,
    handle_new_entry_cancel,
    handle_new_entry_confirm,
    handle_new_entry_input,
    handle_quick_roll_clicks,
    handle_quick_roll_die_type_select_change,
    handle_roll_all_stats_click,
    handle_roll_attribute_click,
    handle_roll_skill_click,
    handle_gizmo_interactions,
    handle_preview_drag_rotation,
    handle_save_click,
    handle_scroll_input,
    handle_settings_button_click,
    handle_settings_cancel_click,
    handle_settings_ok_click,
    handle_settings_reset_layout_click,
    handle_shake_curve_bezier_handle_press,
    handle_shake_curve_chip_clicks,
    handle_shake_curve_graph_click_to_add_point,
    handle_shake_curve_point_press,
    handle_shake_duration_text_input,
    handle_shake_slider_changes,
    // Character sheet tab systems
    handle_sheet_tab_clicks,
    handle_slider_group_drag,
    handle_sqlite_conversion_no_click,
    handle_sqlite_conversion_ok_click,
    handle_sqlite_conversion_yes_click,
    handle_stat_field_click,
    handle_strength_slider_changes,
    handle_tab_clicks,
    handle_text_input,
    handle_texture_file_clicks,
    poll_file_picker_tasks,
    process_dice_designer_texture_loads,
    rebuild_texture_inputs_section,
    update_texture_previews,
    handle_theme_seed_select_change,
    handle_vsync_switch_change,
    handle_zoom_slider_changes,
    init_character_manager,
    init_collision_sounds,
    init_contributors,
    init_dice_designer_preview_render_target,
    init_dice_face_label_assets,
    init_dice_scale_preview_render_target,
    init_settings_ui_images,
    load_icons,
    load_settings_state_from_db,
    load_dice_designer_state_from_db,
    manage_character_sheet_settings_modal,
    manage_dice_scale_preview_scene,
    manage_settings_modal,
    open_lid_on_roll_completed,
    persist_settings_to_db,
    persist_dice_designer_state_to_db,
    play_dice_container_collision_sfx,
    process_avatar_loads,
    process_pending_roll_with_lid,
    rebuild_character_list_on_change,
    rebuild_character_panel_on_change,
    rebuild_command_history_panel,
    rebuild_quick_roll_panel,
    record_character_screen_roll_on_settle,
    refresh_character_display,
    refresh_scrollbar_colors_on_theme_change,
    request_avatars,
    rotate_camera,
    run_sqlite_conversion_step,
    setup,
    setup_character_screen,
    setup_contributors_screen,
    setup_dice_designer_screen,
    setup_dice_preview,
    setup_dnd_info_screen,
    setup_tab_bar,
    spawn_colliders_from_gltf_guides,
    start_sqlite_conversion_if_needed,
    sync_character_screen_roll_result_texts,
    sync_dice_container_mode_text,
    sync_dice_container_toggle_icon,
    sync_dice_scale_preview_dice,
    sync_shake_curve_chip_ui,
    sync_shake_curve_graph_ui,
    tint_recent_theme_dropdown_items,
    update_avatar_images,
    update_character_list_modified_indicator,
    update_color_ui,
    update_dice_box_highlight,
    update_dice_fx_param_ui,
    update_preview_die_type,
    update_preview_material_textures,
    update_preview_rotation,
    update_preview_visibility,
    update_rotation_ring_transforms,
    update_rotation_ring_visibility,
    update_dice_scale_ui,
    update_sound_video_ui,
    update_editing_display,
    update_new_entry_input_display,
    update_results_display,
    update_save_button_appearance,
    update_sheet_tab_styles,
    update_sheet_tab_visibility,
    update_sqlite_conversion_dialog_ui,
    update_tab_styles,
    update_tab_visibility,
    update_throw_arrow,
    update_throw_from_mouse,
    update_ui_pointer_capture,
    AddingEntryState,
    AvatarLoader,
    CharacterData,
    CharacterScreenRollBridge,
    CommandHistory,
    CommandInput,
    ContainerShakeAnimation,
    ContainerShakeConfig,
    Dice3dEmbeddedAssetsPlugin,
    DiceBoxHighlightMaterial,
    DiceBoxLidAnimationController,
    DiceConfig,
    DiceContainerStyle,
    DiceDesignerState,
    DiceFxPlugin,
    DiceResults,
    DiceSpawnPoints,
    DiceSpawnPointsApplied,
    DiceType,
    GroupEditState,
    RollState,
    SettingsState,
    ShakeState,
    ThrowControlState,
    UiState,
    ZoomState,
};

use dndgamerolls::dice3d::types::database::CharacterDatabase;
use dndgamerolls::dice3d::types::ui::UiPointerCapture;

/// DnD Game Rolls - CLI and 3D Visualization
#[derive(Parser)]
#[command(name = "dndgamerolls")]
#[command(
    author,
    version,
    about = "DnD Game Rolls - D&D dice roller with CLI and 3D visualization"
)]
struct Cli {
    /// Run in CLI mode (no GUI)
    #[arg(long)]
    cli: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Select a character by name from the local database (SurrealDB)
    #[arg(long)]
    character: Option<String>,

    /// Select a character by id from the local database (SurrealDB)
    #[arg(long)]
    character_id: Option<i64>,

    /// Dice to roll (e.g., "2d6", "1d20", "d8"). Can specify multiple.
    #[arg(short, long, value_parser = parse_dice_arg)]
    dice: Option<Vec<(usize, DiceType)>>,

    /// Check to apply modifier for (skill, ability, or save name)
    #[arg(long)]
    checkon: Option<String>,

    /// Custom modifier to add to the roll
    #[arg(short, long, default_value = "0")]
    modifier: i32,

    /// Roll with advantage (roll twice, take higher)
    #[arg(short, long)]
    advantage: bool,

    /// Roll with disadvantage (roll twice, take lower)
    #[arg(short = 'D', long)]
    disadvantage: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Roll a strength check
    #[command(visible_alias = "str")]
    Strength,

    /// Roll a dexterity check
    #[command(visible_alias = "dex")]
    Dexterity,

    /// Roll a constitution check
    #[command(visible_alias = "con")]
    Constitution,

    /// Roll an intelligence check
    #[command(visible_alias = "int")]
    Intelligence,

    /// Roll a wisdom check
    #[command(visible_alias = "wis")]
    Wisdom,

    /// Roll a charisma check
    #[command(visible_alias = "cha")]
    Charisma,

    /// Roll an initiative check (Dexterity based)
    Initiative,

    /// Roll a skill check
    Skill {
        /// Skill name (e.g., stealth, perception, acrobatics)
        name: String,
    },

    /// Roll a saving throw
    Save {
        /// Ability name (str, dex, con, int, wis, cha)
        ability: String,
    },

    /// Roll an attack
    Attack {
        /// Weapon name
        weapon: String,
    },

    /// Display character stats
    Stats,
}

fn parse_dice_arg(s: &str) -> Result<(usize, DiceType), String> {
    let s = s.to_lowercase();

    let (count_str, die_str) = if s.starts_with('d') {
        ("1", s.as_str())
    } else if let Some(pos) = s.find('d') {
        (&s[..pos], &s[pos..])
    } else {
        return Err(format!(
            "Invalid dice format: {}. Use format like '2d6' or 'd20'",
            s
        ));
    };

    let count: usize = count_str
        .parse()
        .map_err(|_| format!("Invalid count: {}", count_str))?;
    let die_type = DiceType::parse(die_str).ok_or_else(|| {
        format!(
            "Unknown die type: {}. Valid: d4, d6, d8, d10, d12, d20",
            die_str
        )
    })?;

    Ok((count, die_type))
}

fn main() {
    let cli = Cli::parse();

    // Determine mode: CLI subcommands, --cli flag with dice, or 3D mode
    if cli.command.is_some() || (cli.cli && (cli.dice.is_some() || cli.checkon.is_some())) {
        // Attach to parent console for CLI output (Windows only)
        #[cfg(windows)]
        attach_parent_console();

        run_cli_mode(cli);
    } else if cli.cli {
        #[cfg(windows)]
        attach_parent_console();

        eprintln!("CLI mode requires either a subcommand or --dice/--checkon options");
        eprintln!("Examples:");
        eprintln!("  dndgamerolls --cli skill stealth");
        eprintln!("  dndgamerolls --cli --dice 2d6 --checkon perception");
        eprintln!("  dndgamerolls stats");
        std::process::exit(1);
    } else {
        // GUI mode - no console needed (windows_subsystem = "windows" handles this)
        run_3d_mode(cli);
    }
}

/// Attach to the parent process's console on Windows.
/// This is needed because we use windows_subsystem = "windows" to hide the console,
/// but CLI mode needs to output to the terminal.
#[cfg(windows)]
fn attach_parent_console() {
    use std::io::{self, Write};

    #[link(name = "kernel32")]
    extern "system" {
        fn AttachConsole(dwProcessId: u32) -> i32;
        fn AllocConsole() -> i32;
    }

    const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;

    unsafe {
        // Try to attach to parent console (e.g., when run from cmd/PowerShell)
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            // If no parent console, allocate a new one
            AllocConsole();
        }
    }

    // Reinitialize stdout/stderr to use the attached console
    // This is necessary on Windows after AttachConsole
    let _ = io::stdout().flush();
    let _ = io::stderr().flush();
}

// ============================================================================
// 3D Mode
// ============================================================================

fn run_3d_mode(cli: Cli) {
    // Character persistence is database-backed; character selection/loading is handled
    // by the in-game character manager.
    let character_data = CharacterData::default();

    let mut dice_to_roll = Vec::new();
    let mut modifier = cli.modifier;
    let mut modifier_name = String::new();

    // Handle --checkon: apply modifier to custom dice
    if let Some(check) = &cli.checkon {
        let check_lower = check.to_lowercase();

        if let Some(skill_mod) = character_data.get_skill_modifier(&check_lower) {
            modifier += skill_mod;
            modifier_name = check.to_string();
        } else if let Some(ability_mod) = character_data.get_ability_modifier(&check_lower) {
            modifier += ability_mod;
            modifier_name = format!("{} check", check);
        } else if check_lower.ends_with(" save") || check_lower.ends_with(" saving") {
            let save_name = check_lower
                .trim_end_matches(" save")
                .trim_end_matches(" saving")
                .trim();
            if let Some(save_mod) = character_data.get_saving_throw_modifier(save_name) {
                modifier += save_mod;
                modifier_name = format!("{} save", save_name);
            } else {
                modifier_name = check.clone();
                eprintln!("Warning: '{}' not found in character sheet", check);
            }
        } else if let Some(save_mod) = character_data.get_saving_throw_modifier(&check_lower) {
            modifier += save_mod;
            modifier_name = format!("{} save", check);
        } else {
            modifier_name = check.clone();
            eprintln!("Warning: '{}' not found in character sheet", check);
        }

        if let Some(dice_args) = &cli.dice {
            for (count, die_type) in dice_args {
                for _ in 0..*count {
                    dice_to_roll.push(*die_type);
                }
            }
        }
    } else if let Some(dice_args) = &cli.dice {
        for (count, die_type) in dice_args {
            for _ in 0..*count {
                dice_to_roll.push(*die_type);
            }
        }
    }

    if dice_to_roll.is_empty() {
        dice_to_roll.push(DiceType::D20);
    }

    println!(
        "Rolling: {:?}",
        dice_to_roll.iter().map(|d| d.name()).collect::<Vec<_>>()
    );
    if modifier != 0 {
        let sign = if modifier >= 0 { "+" } else { "" };
        println!("Modifier: {}{} ({})", sign, modifier, modifier_name);
    }

    // System to set the window icon from embedded resources
    fn set_window_icon(
        windows: Option<NonSend<WinitWindows>>,
        primary_query: Query<Entity, With<bevy::window::PrimaryWindow>>,
    ) {
        let Some(windows) = windows else {
            return;
        };
        let Ok(primary_entity) = primary_query.single() else {
            return;
        };
        let Some(primary) = windows.get_window(primary_entity) else {
            return;
        };

        // Try multiple icon locations (in order of preference)
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut icon_paths = vec![
            // Development paths
            std::path::PathBuf::from("assets/icon.ico"),
            std::path::PathBuf::from("../assets/icon.ico"),
            // Installed path (icon in root of install folder)
            std::path::PathBuf::from("icon.ico"),
        ];

        // Add paths relative to executable location
        if let Some(ref exe_dir) = exe_dir {
            icon_paths.push(exe_dir.join("assets/icon.ico"));
            icon_paths.push(exe_dir.join("icon.ico"));
        }

        for path in &icon_paths {
            if path.exists() {
                if let Ok(icon_data) = std::fs::read(path) {
                    // Parse ICO file - ICO files have a header followed by image data
                    // We'll use the image crate to decode it
                    if let Ok(img) = image::load_from_memory(&icon_data) {
                        let rgba = img.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        let icon = winit::window::Icon::from_rgba(rgba.into_raw(), width, height);
                        if let Ok(icon) = icon {
                            primary.set_window_icon(Some(icon));
                            return;
                        }
                    }
                }
            }
        }
    }

    let dice_config = DiceConfig {
        dice_to_roll,
        modifier,
        modifier_name,
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DnD Game Rolls".to_string(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                // Keep app logs at info, but silence bevy_material_ui scroll spam.
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "info,wgpu=error,bevy_material_ui=warn,bevy_material_ui::scroll=off"
                        .to_string(),
                    ..default()
                }),
        )
        .add_plugins(HanabiPlugin)
        .add_plugins(bevy::pbr::MaterialPlugin::<DiceBoxHighlightMaterial>::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(MaterialUiPlugin)
        .add_plugins(Dice3dEmbeddedAssetsPlugin)
        .add_plugins(DiceFxPlugin)
        // Ensure UI Buttons spawned without ButtonBundle still receive click events
        .add_systems(PreUpdate, ensure_buttons_have_interaction)
        .insert_resource(dice_config)
        .insert_resource(character_data)
        .insert_resource(DiceResults::default())
        .insert_resource(RollState::default())
        .insert_resource(CommandInput::default())
        .insert_resource(CommandHistory::default())
        .insert_resource(ZoomState::default())
        .insert_resource(UiState::default())
        .insert_resource(DiceContainerStyle::default())
        .insert_resource(ShakeState::default())
        .insert_resource(ContainerShakeAnimation::default())
        .insert_resource(ContainerShakeConfig::default())
        .insert_resource(GroupEditState::default())
        .insert_resource(AddingEntryState::default())
        .insert_resource(SettingsState::default())
        .insert_resource(CharacterScreenRollBridge::default())
        .insert_resource(UiPointerCapture::default())
        .insert_resource(ThrowControlState::default())
        .insert_resource(DiceSpawnPoints::default())
        .insert_resource(DiceSpawnPointsApplied::default())
        .insert_resource(AvatarLoader::default())
        .insert_resource(DiceDesignerTextureLoader::default())
        .insert_resource(DiceBoxLidAnimationController::default())
        .insert_resource(dndgamerolls::dice3d::systems::color_picker::ColorPickerState::default())
        .insert_resource(DiceDesignerState::default())
        .add_systems(
            Startup,
            (
                set_window_icon,
                load_icons,
                init_dice_face_label_assets,
                init_character_manager,
                load_settings_state_from_db,
                load_dice_designer_state_from_db,
                init_dice_scale_preview_render_target,
                init_dice_designer_preview_render_target,
                init_settings_ui_images,
                init_contributors,
                apply_initial_shake_config,
                init_collision_sounds,
                dndgamerolls::dice3d::systems::color_picker::setup_color_wheel_texture,
                setup,
                setup_tab_bar,
                setup_character_screen,
                setup_dice_designer_screen,
                setup_dnd_info_screen,
                setup_contributors_screen,
                apply_initial_settings,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                // Dice roller systems
                check_dice_settled,
                update_results_display,
                handle_input,
                handle_command_input,
                rebuild_command_history_panel,
                handle_quick_roll_clicks,
                rebuild_quick_roll_panel,
                rotate_camera,
                handle_zoom_slider_changes,
                sync_dice_container_mode_text,
                sync_dice_container_toggle_icon,
                handle_dice_box_rotate_click,
                handle_dice_box_shake_box_click,
                animate_container_shake,
                handle_dice_box_toggle_container_click,
                // Mouse-controlled throw systems
                update_throw_from_mouse,
                update_dice_box_highlight,
                handle_strength_slider_changes,
                handle_shake_slider_changes,
                update_throw_arrow,
            ),
        )
        .add_systems(
            Update,
            update_ui_pointer_capture
                .before(handle_input)
                .before(update_throw_from_mouse)
                .before(update_dice_box_highlight),
        )
        .add_systems(Update, ensure_dice_box_lid_animation_assets)
        .add_systems(
            Update,
            cache_dice_box_lid_animation_player.after(ensure_dice_box_lid_animation_assets),
        )
        .add_systems(
            Update,
            process_pending_roll_with_lid
                .after(handle_input)
                .after(handle_quick_roll_clicks),
        )
        .add_systems(Update, open_lid_on_roll_completed.after(check_dice_settled))
        .add_systems(Update, play_dice_container_collision_sfx)
        .add_systems(
            Update,
            center_container_models_in_view
                .before(spawn_colliders_from_gltf_guides)
                .before(apply_crystal_material_to_container_models)
                .before(collect_dice_spawn_points_from_gltf)
                .before(apply_spawn_points_to_dice_when_ready)
                .before(update_dice_box_highlight),
        )
        // Separate to avoid Bevy's tuple-size limit, and ensure it runs before highlight tagging.
        .add_systems(
            Update,
            spawn_colliders_from_gltf_guides
                .before(handle_dice_box_toggle_container_click)
                .before(update_dice_box_highlight),
        )
        .add_systems(
            Update,
            apply_crystal_material_to_container_models
                .before(handle_dice_box_toggle_container_click)
                .before(update_dice_box_highlight),
        )
        .add_systems(
            Update,
            collect_dice_spawn_points_from_gltf
                .before(handle_dice_box_toggle_container_click)
                .before(update_dice_box_highlight),
        )
        .add_systems(
            Update,
            apply_spawn_points_to_dice_when_ready
                .before(handle_dice_box_toggle_container_click)
                .before(update_dice_box_highlight),
        )
        .add_systems(Update, handle_command_history_item_clicks)
        .add_systems(
            Update,
            (
                // Avatar loading systems
                request_avatars,
                process_avatar_loads,
                update_avatar_images,
                // Tab and character screen systems
                handle_tab_clicks,
                update_tab_visibility,
                // Legacy SQLite -> SurrealDB conversion (character screen)
                start_sqlite_conversion_if_needed,
                run_sqlite_conversion_step,
                update_sqlite_conversion_dialog_ui,
                handle_sqlite_conversion_ok_click,
                handle_sqlite_conversion_yes_click,
                handle_sqlite_conversion_no_click,
                finalize_sqlite_conversion_if_done,
                handle_character_list_clicks,
                handle_new_character_click,
                handle_save_click,
            ),
        )
        .add_systems(
            Update,
            record_character_screen_roll_on_settle.after(check_dice_settled),
        )
        .add_systems(Update, handle_slider_group_drag)
        .add_systems(
            Update,
            (
                // Tab styling (separate to avoid tuple size limit)
                update_tab_styles,
                // Character sheet tab systems
                handle_sheet_tab_clicks,
                update_sheet_tab_styles,
                update_sheet_tab_visibility,
                // Character editing systems - input handling
                handle_scroll_input,
                handle_stat_field_click,
                handle_label_click,
                handle_text_input,
                handle_proficiency_toggle,
                handle_expertise_toggle,
                handle_group_edit_toggle,
                handle_group_add_click,
                handle_delete_click,
                handle_new_entry_confirm,
                handle_new_entry_cancel,
                handle_new_entry_input,
                // Dice roll buttons for attributes
                handle_roll_all_stats_click,
                handle_roll_attribute_click,
                handle_roll_skill_click,
            ),
        )
        .add_systems(
            Update,
            (
                // Character editing systems - display updates (run after input handling)
                update_new_entry_input_display,
                update_editing_display,
                update_save_button_appearance,
                update_character_list_modified_indicator,
                refresh_character_display,
                rebuild_character_list_on_change,
                rebuild_character_panel_on_change,
                sync_character_screen_roll_result_texts,
            )
                .chain()
                .after(handle_new_entry_confirm)
                .after(handle_delete_click)
                .after(handle_roll_all_stats_click)
                .after(handle_roll_attribute_click)
                .after(handle_roll_skill_click)
                .after(handle_text_input),
        )
        .add_systems(
            Update,
            (
                (
                    // Settings systems
                    (
                        handle_settings_button_click,
                        manage_settings_modal,
                        manage_dice_scale_preview_scene,
                        fix_dice_scale_slider_thumb_hitbox.after(manage_settings_modal),
                        handle_settings_ok_click,
                        handle_settings_cancel_click,
                        handle_settings_reset_layout_click,
                    ),
                    (
                        (
                            handle_quick_roll_die_type_select_change,
                            handle_theme_seed_select_change,
                            handle_default_roll_uses_shake_switch_change,
                            handle_master_volume_slider_change,
                            handle_vsync_switch_change,
                            handle_color_slider_changes,
                            handle_dice_scale_slider_changes,
                            handle_dice_fx_param_slider_changes,
                            handle_dice_roll_fx_mapping_select_change,
                            handle_color_text_input,
                            handle_shake_duration_text_input,
                        ),
                        (
                            handle_shake_curve_chip_clicks,
                            (
                                handle_shake_curve_point_press,
                                handle_shake_curve_bezier_handle_press,
                                handle_shake_curve_graph_click_to_add_point,
                                drag_shake_curve_bezier_handle,
                                drag_shake_curve_point,
                                sync_shake_curve_graph_ui,
                            )
                                .chain(),
                            sync_shake_curve_chip_ui,
                        ),
                    ),
                    (
                        update_color_ui,
                        dndgamerolls::dice3d::systems::settings_tabs::colors_interactions::handle_color_preview_clicks,
                            update_sound_video_ui,
                        update_dice_scale_ui,
                        update_dice_fx_param_ui,
                        sync_dice_scale_preview_dice,
                        autosave_and_apply_shake_config.after(sync_shake_curve_graph_ui),
                    ),
                ),
                (
                    // Character sheet dice settings modal
                    handle_character_sheet_settings_button_click,
                    manage_character_sheet_settings_modal,
                    handle_character_sheet_die_type_select_change,
                    handle_character_sheet_settings_save_click,
                    handle_character_sheet_settings_cancel_click,
                ),
            ),
        )
        .add_systems(
            Update,
            apply_editing_dice_scales_to_existing_dice_while_open
                .after(handle_dice_scale_slider_changes),
        )
        .add_systems(
            Update,
            apply_dice_scale_settings_to_existing_dice.after(handle_settings_ok_click),
        )
        .add_systems(
            Update,
            refresh_scrollbar_colors_on_theme_change
                .after(handle_color_text_input)
                .after(handle_theme_seed_select_change)
                .after(handle_settings_ok_click)
                .after(handle_settings_cancel_click),
        )
        .add_systems(PostUpdate, tint_recent_theme_dropdown_items)
        .add_systems(PostUpdate, persist_settings_to_db)
        .add_systems(PostUpdate, persist_dice_designer_state_to_db)
        .add_systems(
            Update,
            (
                // Dice Designer systems
                setup_dice_preview,
                handle_dice_list_clicks,
                handle_face_selector_clicks,
                update_face_selector_for_dice,
                handle_texture_file_clicks,
                poll_file_picker_tasks,
                process_dice_designer_texture_loads,
                rebuild_texture_inputs_section,
                update_texture_previews,
                update_preview_visibility,
                update_preview_rotation,
                update_preview_die_type,
                update_preview_material_textures,
                update_rotation_ring_visibility,
                update_rotation_ring_transforms,
                handle_preview_drag_rotation,
                handle_gizmo_interactions,
            ),
        )
        .add_systems(
            Update,
            (
                dndgamerolls::dice3d::systems::color_picker::spawn_color_picker_ui,
                dndgamerolls::dice3d::systems::color_picker::handle_color_picker_interactions,
                dndgamerolls::dice3d::systems::color_picker::update_color_picker_preview,
            ),
        )
        .run();
}

// ============================================================================
// CLI Mode Functions
// ============================================================================

fn run_cli_mode(cli: Cli) {
    // If using --dice with --checkon (new unified syntax)
    if cli.dice.is_some() || cli.checkon.is_some() {
        run_cli_dice_roll(&cli);
        return;
    }

    // Legacy subcommand mode
    let sheet = match load_character_sheet(cli.character.as_deref(), cli.character_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to load character: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Some(Commands::Strength) => {
            let modifier = sheet.modifiers.strength;
            roll_ability_check("Strength", modifier, cli.advantage, cli.disadvantage);
        }
        Some(Commands::Dexterity) => {
            let modifier = sheet.modifiers.dexterity;
            roll_ability_check("Dexterity", modifier, cli.advantage, cli.disadvantage);
        }
        Some(Commands::Constitution) => {
            let modifier = sheet.modifiers.constitution;
            roll_ability_check("Constitution", modifier, cli.advantage, cli.disadvantage);
        }
        Some(Commands::Intelligence) => {
            let modifier = sheet.modifiers.intelligence;
            roll_ability_check("Intelligence", modifier, cli.advantage, cli.disadvantage);
        }
        Some(Commands::Wisdom) => {
            let modifier = sheet.modifiers.wisdom;
            roll_ability_check("Wisdom", modifier, cli.advantage, cli.disadvantage);
        }
        Some(Commands::Charisma) => {
            let modifier = sheet.modifiers.charisma;
            roll_ability_check("Charisma", modifier, cli.advantage, cli.disadvantage);
        }
        Some(Commands::Initiative) => {
            let modifier = sheet.combat.initiative;
            roll_ability_check("Initiative", modifier, cli.advantage, cli.disadvantage);
        }
        Some(Commands::Skill { name }) => {
            if let Some((skill_name, skill)) = get_skill_by_name(&sheet.skills, &name) {
                let proficiency_str = if skill.expertise.unwrap_or(false) {
                    " (Expertise)"
                } else if skill.proficient {
                    " (Proficient)"
                } else {
                    ""
                };
                roll_ability_check(
                    &format!("{}{}", skill_name, proficiency_str),
                    skill.modifier,
                    cli.advantage,
                    cli.disadvantage,
                );
            } else {
                eprintln!("{} Unknown skill '{}'", "Error:".red().bold(), name);
                eprintln!("Available skills: acrobatics, animal handling, arcana, athletics,");
                eprintln!("  deception, history, insight, intimidation, investigation,");
                eprintln!("  medicine, nature, perception, performance, persuasion,");
                eprintln!("  religion, sleight of hand, stealth, survival");
                std::process::exit(1);
            }
        }
        Some(Commands::Save { ability }) => {
            if let Some((save_name, save)) = get_save_by_name(&sheet.saving_throws, &ability) {
                let proficiency_str = if save.proficient { " (Proficient)" } else { "" };
                roll_ability_check(
                    &format!("{} Save{}", save_name, proficiency_str),
                    save.modifier,
                    cli.advantage,
                    cli.disadvantage,
                );
            } else {
                eprintln!("{} Unknown saving throw '{}'", "Error:".red().bold(), ability);
                std::process::exit(1);
            }
        }
        Some(Commands::Attack { weapon }) => {
            let weapon_lower = weapon.to_lowercase();
            let Some(equipment) = sheet.equipment.as_ref() else {
                eprintln!(
                    "{} No equipment found on this character",
                    "Error:".red().bold()
                );
                std::process::exit(1);
            };

            if let Some(wpn) = equipment
                .weapons
                .iter()
                .find(|w| w.name.to_lowercase() == weapon_lower)
            {
                roll_attack(wpn, cli.advantage, cli.disadvantage);
            } else {
                eprintln!("{} Weapon '{}' not found", "Error:".red().bold(), weapon);
                eprintln!("Available weapons:");
                for wpn in &equipment.weapons {
                    eprintln!("  - {}", wpn.name);
                }
                std::process::exit(1);
            }
        }
        Some(Commands::Stats) => {
            display_stats(&sheet);
        }
        None => {
            eprintln!("{} No command specified", "Error:".red().bold());
            eprintln!("Use --help to see available commands");
            std::process::exit(1);
        }
    }
}

fn run_cli_dice_roll(cli: &Cli) {
    // Character persistence is SQLite-backed; the CLI dice roll path can still run
    // without a loaded character sheet.
    let character_data = CharacterData::default();

    let mut total_modifier = cli.modifier;
    let mut modifier_name = String::new();
    let mut dice_to_roll: Vec<DiceType> = Vec::new();

    // Parse dice
    if let Some(dice_args) = &cli.dice {
        for (count, die_type) in dice_args {
            for _ in 0..*count {
                dice_to_roll.push(*die_type);
            }
        }
    }

    // Apply checkon modifier
    if let Some(check) = &cli.checkon {
        let check_lower = check.to_lowercase();

        if let Some(skill_mod) = character_data.get_skill_modifier(&check_lower) {
            total_modifier += skill_mod;
            modifier_name = check.clone();
        } else if let Some(ability_mod) = character_data.get_ability_modifier(&check_lower) {
            total_modifier += ability_mod;
            modifier_name = format!("{} check", check);
        } else if let Some(save_mod) = character_data.get_saving_throw_modifier(&check_lower) {
            total_modifier += save_mod;
            modifier_name = format!("{} save", check);
        } else {
            modifier_name = check.clone();
            eprintln!("Warning: '{}' not found in character sheet", check);
        }
    }

    // Default to 1d20 if no dice specified
    if dice_to_roll.is_empty() {
        dice_to_roll.push(DiceType::D20);
    }

    // Roll the dice
    let mut rng = rand::rng();
    let mut results: Vec<(DiceType, u32)> = Vec::new();
    let mut total: i32 = 0;

    for die in &dice_to_roll {
        let roll = rng.random_range(1..=die.max_value());
        results.push((*die, roll));
        total += roll as i32;
    }

    // Handle advantage/disadvantage for d20 rolls
    if dice_to_roll.len() == 1 && dice_to_roll[0] == DiceType::D20 {
        if cli.advantage && !cli.disadvantage {
            let roll2 = rng.random_range(1..=20);
            let roll1 = results[0].1;
            let used = roll1.max(roll2);
            let dropped = roll1.min(roll2);
            results[0].1 = used;
            total = used as i32;

            println!("\n{}", "═══════════════════════════════════════".cyan());
            println!(
                "{} {} {}",
                "Rolling:".bold().white(),
                modifier_name.yellow().bold(),
                "(Advantage)".green()
            );
            println!(
                "{} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", used).bright_green().bold(),
                format!("[{}]", dropped).dimmed()
            );
        } else if cli.disadvantage && !cli.advantage {
            let roll2 = rng.random_range(1..=20);
            let roll1 = results[0].1;
            let used = roll1.min(roll2);
            let dropped = roll1.max(roll2);
            results[0].1 = used;
            total = used as i32;

            println!("\n{}", "═══════════════════════════════════════".cyan());
            println!(
                "{} {} {}",
                "Rolling:".bold().white(),
                modifier_name.yellow().bold(),
                "(Disadvantage)".red()
            );
            println!(
                "{} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", used).bright_red().bold(),
                format!("[{}]", dropped).dimmed()
            );
        } else {
            print_normal_roll(&results, &modifier_name);
        }
    } else {
        print_normal_roll(&results, &modifier_name);
    }

    // Print modifier and total
    if total_modifier != 0 {
        let modifier_str = if total_modifier >= 0 {
            format!("+{}", total_modifier).cyan()
        } else {
            format!("{}", total_modifier).cyan()
        };
        println!("{} {}", "Modifier:".bold().white(), modifier_str);
    }

    let final_total = total + total_modifier;
    let d20_roll = if dice_to_roll.len() == 1 && dice_to_roll[0] == DiceType::D20 {
        Some(results[0].1)
    } else {
        None
    };

    let total_color = match d20_roll {
        Some(20) => format!("{}", final_total).bright_green().bold(),
        Some(1) => format!("{}", final_total).bright_red().bold(),
        _ if final_total >= 20 => format!("{}", final_total).green().bold(),
        _ if final_total >= 15 => format!("{}", final_total).white().bold(),
        _ if final_total >= 10 => format!("{}", final_total).yellow(),
        _ => format!("{}", final_total).red(),
    };

    println!("{} {}", "Total:".bold().white(), total_color);

    if let Some(20) = d20_roll {
        println!(
            "{}",
            "🎉 NATURAL 20! CRITICAL SUCCESS! 🎉".bright_green().bold()
        );
    } else if let Some(1) = d20_roll {
        println!(
            "{}",
            "💀 NATURAL 1! CRITICAL FAILURE! 💀".bright_red().bold()
        );
    }

    println!("{}", "═══════════════════════════════════════".cyan());
}

fn print_normal_roll(results: &[(DiceType, u32)], modifier_name: &str) {
    println!("\n{}", "═══════════════════════════════════════".cyan());
    if !modifier_name.is_empty() {
        println!(
            "{} {}",
            "Rolling:".bold().white(),
            modifier_name.yellow().bold()
        );
    } else {
        let dice_str: Vec<String> = results.iter().map(|(d, _)| d.name().to_string()).collect();
        println!(
            "{} {}",
            "Rolling:".bold().white(),
            dice_str.join(", ").yellow().bold()
        );
    }

    let rolls_str: Vec<String> = results
        .iter()
        .map(|(d, r)| {
            let roll_color = if d == &DiceType::D20 {
                match r {
                    20 => format!("[{}]", r).bright_green().bold().to_string(),
                    1 => format!("[{}]", r).bright_red().bold().to_string(),
                    _ => format!("[{}]", r).bright_white().bold().to_string(),
                }
            } else {
                format!("[{}]", r).bright_white().bold().to_string()
            };
            format!("{}: {}", d.name(), roll_color)
        })
        .collect();
    println!("{} {}", "Dice:".bold().white(), rolls_str.join(", "));
}

fn roll_d20() -> i32 {
    rand::rng().random_range(1..=20)
}

fn roll_with_advantage_disadvantage(advantage: bool, disadvantage: bool) -> (i32, Option<i32>) {
    if advantage && disadvantage {
        (roll_d20(), None)
    } else if advantage {
        let roll1 = roll_d20();
        let roll2 = roll_d20();
        (roll1.max(roll2), Some(roll1.min(roll2)))
    } else if disadvantage {
        let roll1 = roll_d20();
        let roll2 = roll_d20();
        (roll1.min(roll2), Some(roll1.max(roll2)))
    } else {
        (roll_d20(), None)
    }
}

fn roll_ability_check(name: &str, modifier: i32, advantage: bool, disadvantage: bool) {
    let (dice_roll, dropped_roll) = roll_with_advantage_disadvantage(advantage, disadvantage);
    let total = dice_roll + modifier;
    display_roll_result(
        name,
        dice_roll,
        modifier,
        total,
        dropped_roll,
        advantage,
        disadvantage,
    );
}

fn roll_attack(weapon: &dndgamerolls::dice3d::types::Weapon, advantage: bool, disadvantage: bool) {
    let (dice_roll, dropped_roll) = roll_with_advantage_disadvantage(advantage, disadvantage);
    let total = dice_roll + weapon.attack_bonus;

    println!("\n{}", "═══════════════════════════════════════".cyan());
    println!("{} {} Attack", "⚔️".bold(), weapon.name.bold().yellow());

    if let Some(dropped) = dropped_roll {
        if advantage {
            println!(
                "{} {} {} (dropped {})",
                "Attack Roll:".bold().white(),
                format!("[{}]", dice_roll).bright_green().bold(),
                "(advantage)".green(),
                format!("[{}]", dropped).dimmed()
            );
        } else if disadvantage {
            println!(
                "{} {} {} (dropped {})",
                "Attack Roll:".bold().white(),
                format!("[{}]", dice_roll).bright_red().bold(),
                "(disadvantage)".red(),
                format!("[{}]", dropped).dimmed()
            );
        }
    } else {
        let dice_color = match dice_roll {
            20 => format!("[{}]", dice_roll).bright_green().bold(),
            1 => format!("[{}]", dice_roll).bright_red().bold(),
            _ => format!("[{}]", dice_roll).bright_white().bold(),
        };
        println!("{} {}", "Attack Roll:".bold().white(), dice_color);
    }

    let modifier_str = if weapon.attack_bonus >= 0 {
        format!("+{}", weapon.attack_bonus).cyan()
    } else {
        format!("{}", weapon.attack_bonus).cyan()
    };
    println!("{} {}", "Attack Bonus:".bold().white(), modifier_str);

    let total_color = if dice_roll == 20 {
        format!("{}", total).bright_green().bold()
    } else if dice_roll == 1 {
        format!("{}", total).bright_red().bold()
    } else {
        format!("{}", total).white().bold()
    };
    println!("{} {}", "Total:".bold().white(), total_color);

    if dice_roll == 20 {
        println!("{}", "🎯 CRITICAL HIT! 🎯".bright_green().bold());
    } else if dice_roll == 1 {
        println!("{}", "💨 CRITICAL MISS! 💨".bright_red().bold());
    }

    println!(
        "{} {} ({})",
        "Damage:".bold().white(),
        weapon.damage.yellow(),
        weapon.damage_type.dimmed()
    );
    println!("{}", "═══════════════════════════════════════".cyan());
}

fn display_roll_result(
    roll_type: &str,
    dice_roll: i32,
    modifier: i32,
    total: i32,
    dropped_roll: Option<i32>,
    advantage: bool,
    disadvantage: bool,
) {
    println!("\n{}", "═══════════════════════════════════════".cyan());
    println!(
        "{} {}",
        "Rolling:".bold().white(),
        roll_type.bold().yellow()
    );

    if let Some(dropped) = dropped_roll {
        if advantage {
            println!(
                "{} {} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", dice_roll).bright_green().bold(),
                "(advantage)".green(),
                format!("[{}]", dropped).dimmed()
            );
        } else if disadvantage {
            println!(
                "{} {} {} (dropped {})",
                "Dice:".bold().white(),
                format!("[{}]", dice_roll).bright_red().bold(),
                "(disadvantage)".red(),
                format!("[{}]", dropped).dimmed()
            );
        }
    } else {
        let dice_color = match dice_roll {
            20 => format!("[{}]", dice_roll).bright_green().bold(),
            1 => format!("[{}]", dice_roll).bright_red().bold(),
            _ => format!("[{}]", dice_roll).bright_white().bold(),
        };
        println!("{} {}", "Dice:".bold().white(), dice_color);
    }

    let modifier_str = if modifier >= 0 {
        format!("+{}", modifier).cyan()
    } else {
        format!("{}", modifier).cyan()
    };
    println!("{} {}", "Modifier:".bold().white(), modifier_str);

    let total_color = if dice_roll == 20 {
        format!("{}", total).bright_green().bold()
    } else if dice_roll == 1 {
        format!("{}", total).bright_red().bold()
    } else if total >= 20 {
        format!("{}", total).green().bold()
    } else if total >= 15 {
        format!("{}", total).white().bold()
    } else if total >= 10 {
        format!("{}", total).yellow()
    } else {
        format!("{}", total).red()
    };

    println!("{} {}", "Total:".bold().white(), total_color);

    if dice_roll == 20 {
        println!(
            "{}",
            "🎉 NATURAL 20! CRITICAL SUCCESS! 🎉".bright_green().bold()
        );
    } else if dice_roll == 1 {
        println!(
            "{}",
            "💀 NATURAL 1! CRITICAL FAILURE! 💀".bright_red().bold()
        );
    }

    println!("{}", "═══════════════════════════════════════".cyan());
}

fn load_character_sheet(
    character_name: Option<&str>,
    character_id: Option<i64>,
) -> Result<dndgamerolls::dice3d::types::CharacterSheet, Box<dyn std::error::Error>> {
    let db = CharacterDatabase::open().map_err(|e| format!("Failed to open database: {}", e))?;

    if let Some(id) = character_id {
        return Ok(db.load_character(id)?);
    }

    let list = db.list_characters()?;
    if list.is_empty() {
        return Err("No characters found in local database".into());
    }

    if let Some(name) = character_name {
        if let Some(entry) = list.iter().find(|c| c.name.eq_ignore_ascii_case(name)) {
            return Ok(db.load_character(entry.id)?);
        }

        return Err(format!("Character '{}' not found", name).into());
    }

    Ok(db.load_character(list[0].id)?)
}

fn get_skill_by_name<'a>(
    skills: &'a std::collections::HashMap<String, dndgamerolls::dice3d::types::Skill>,
    name: &str,
) -> Option<(String, &'a dndgamerolls::dice3d::types::Skill)> {
    let normalized = normalize_query(name);

    skills
        .values()
        .find(|s| normalize_query(&s.slug) == normalized || normalize_query(&s.name) == normalized)
        .map(|s| (s.name.clone(), s))
}

fn get_save_by_name<'a>(
    saves: &'a std::collections::HashMap<String, dndgamerolls::dice3d::types::SavingThrow>,
    ability: &str,
) -> Option<(String, &'a dndgamerolls::dice3d::types::SavingThrow)> {
    let normalized = match normalize_query(ability).as_str() {
        "str" => "strength".to_string(),
        "dex" => "dexterity".to_string(),
        "con" => "constitution".to_string(),
        "int" => "intelligence".to_string(),
        "wis" => "wisdom".to_string(),
        "cha" => "charisma".to_string(),
        other => other.to_string(),
    };

    saves
        .values()
        .find(|s| normalize_query(&s.slug) == normalized || normalize_query(&s.name) == normalized)
        .map(|s| (s.name.clone(), s))
}

fn normalize_query(value: &str) -> String {
    value
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "")
}

fn display_stats(character: &dndgamerolls::dice3d::types::CharacterSheet) {
    let info = &character.character;
    println!("\n{}", "═══════════════════════════════════════".cyan());
    println!("{}", "CHARACTER STATS".bold().yellow());
    println!("{}", "═══════════════════════════════════════".cyan());

    let name_display = if let Some(alter_ego) = &info.alter_ego {
        format!("{} ({})", info.name, alter_ego)
    } else {
        info.name.clone()
    };
    println!("{} {}", "Name:".bold().white(), name_display.green());

    let class_display = if let Some(subclass) = &info.subclass {
        format!("{} ({})", info.class, subclass)
    } else {
        info.class.clone()
    };
    println!(
        "{} Level {} {} {}",
        "Class:".bold().white(),
        info.level,
        class_display.cyan(),
        info.race.yellow()
    );

    println!("\n{}", "ATTRIBUTES".bold().yellow());
    println!(
        "  {} {} ({:+})",
        "STR:".bold(),
        character.attributes.strength,
        character.modifiers.strength
    );
    println!(
        "  {} {} ({:+})",
        "DEX:".bold(),
        character.attributes.dexterity,
        character.modifiers.dexterity
    );
    println!(
        "  {} {} ({:+})",
        "CON:".bold(),
        character.attributes.constitution,
        character.modifiers.constitution
    );
    println!(
        "  {} {} ({:+})",
        "INT:".bold(),
        character.attributes.intelligence,
        character.modifiers.intelligence
    );
    println!(
        "  {} {} ({:+})",
        "WIS:".bold(),
        character.attributes.wisdom,
        character.modifiers.wisdom
    );
    println!(
        "  {} {} ({:+})",
        "CHA:".bold(),
        character.attributes.charisma,
        character.modifiers.charisma
    );

    println!("\n{}", "COMBAT".bold().yellow());
    println!("  {} {}", "AC:".bold(), character.combat.armor_class);
    println!(
        "  {} {:+}",
        "Initiative:".bold(),
        character.combat.initiative
    );
    if let Some(hp) = character.combat.hit_points.as_ref() {
        println!("  {} {}/{}", "HP:".bold(), hp.current, hp.maximum);
    }
    println!(
        "  {} {:+}",
        "Proficiency Bonus:".bold(),
        character.proficiency_bonus
    );

    println!("\n{}", "WEAPONS".bold().yellow());
    if let Some(equipment) = character.equipment.as_ref() {
        for weapon in &equipment.weapons {
            println!(
                "  {} {:+} to hit, {} {}",
                weapon.name.bold(),
                weapon.attack_bonus,
                weapon.damage,
                weapon.damage_type.dimmed()
            );
        }
    }

    println!("{}", "═══════════════════════════════════════".cyan());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_d20_in_range() {
        for _ in 0..100 {
            let roll = roll_d20();
            assert!((1..=20).contains(&roll), "Roll {} out of range", roll);
        }
    }

    #[test]
    fn test_advantage_disadvantage_cancel() {
        for _ in 0..10 {
            let (_, dropped) = roll_with_advantage_disadvantage(true, true);
            assert!(dropped.is_none(), "Should cancel out with no dropped roll");
        }
    }

    #[test]
    fn test_normal_roll_no_dropped() {
        for _ in 0..10 {
            let (_, dropped) = roll_with_advantage_disadvantage(false, false);
            assert!(dropped.is_none(), "Normal roll should have no dropped roll");
        }
    }

    #[test]
    fn test_advantage_has_dropped_roll() {
        for _ in 0..10 {
            let (used, dropped) = roll_with_advantage_disadvantage(true, false);
            assert!(dropped.is_some(), "Advantage should have dropped roll");
            let dropped_val = dropped.unwrap();
            assert!(used >= dropped_val, "Advantage should use higher roll");
        }
    }

    #[test]
    fn test_disadvantage_has_dropped_roll() {
        for _ in 0..10 {
            let (used, dropped) = roll_with_advantage_disadvantage(false, true);
            assert!(dropped.is_some(), "Disadvantage should have dropped roll");
            let dropped_val = dropped.unwrap();
            assert!(used <= dropped_val, "Disadvantage should use lower roll");
        }
    }

    #[test]
    fn test_parse_dice_arg() {
        assert_eq!(parse_dice_arg("d20").unwrap(), (1, DiceType::D20));
        assert_eq!(parse_dice_arg("2d6").unwrap(), (2, DiceType::D6));
        assert_eq!(parse_dice_arg("1d8").unwrap(), (1, DiceType::D8));
        assert_eq!(parse_dice_arg("D20").unwrap(), (1, DiceType::D20));
        assert!(parse_dice_arg("invalid").is_err());
        assert!(parse_dice_arg("2d100").is_err());
    }
}
