//! Legacy SQLite conversion dialog for the Character Sheet.
//!
//! When a legacy SQLite database is detected, this module shows a modal dialog
//! with a progress bar while characters are converted into SurrealDB.

use bevy::prelude::*;
use bevy_material_ui::prelude::*;

use std::collections::HashSet;

use crate::dice3d::types::{
    sqlite_conversion, CharacterDatabase, CharacterListEntry, CharacterManager, CharacterSheet,
};

const IGNORE_LEGACY_SQLITE_SETTING_KEY: &str = "ignore_legacy_sqlite";

fn log_once(state: &mut HashSet<&'static str>, key: &'static str, message: impl FnOnce()) {
    if !state.insert(key) {
        return;
    }
    message();
}

fn find_legacy_sqlite_path(db: &CharacterDatabase) -> Option<std::path::PathBuf> {
    // Primary (historical) location: alongside the SurrealDB folder in app-data.
    if let Some(p) = sqlite_conversion::legacy_sqlite_path() {
        if p.exists() {
            return Some(p);
        }
    }

    // Secondary: inside the SurrealDB folder itself (common user expectation).
    let inside_surreal = db.db_path.join("characters.db");
    if inside_surreal.exists() {
        return Some(inside_surreal);
    }

    None
}

fn decode_legacy_character(data: &[u8]) -> Result<CharacterSheet, String> {
    match bincode::deserialize::<CharacterSheet>(data) {
        Ok(sheet) => Ok(sheet),
        Err(bincode_err) => {
            // Some legacy DBs stored TEXT payloads (RON or JSON) instead of bincode.
            let s = std::str::from_utf8(data).map_err(|utf8_err| {
                format!(
                    "bincode decode failed: {}; also not UTF-8: {}",
                    bincode_err, utf8_err
                )
            })?;

            if let Ok(sheet) = ron::de::from_str::<CharacterSheet>(s) {
                return Ok(sheet);
            }

            serde_json::from_str::<CharacterSheet>(s).map_err(|json_err| {
                format!(
                    "bincode decode failed: {}; RON/JSON decode failed: {}",
                    bincode_err, json_err
                )
            })
        }
    }
}

#[derive(Component)]
pub struct SqliteConversionOverlay;

#[derive(Component)]
pub struct SqliteConversionDialog;

#[derive(Component)]
pub struct SqliteConversionProgressBar;

#[derive(Component)]
pub struct SqliteConversionStatusText;

#[derive(Component)]
pub struct SqliteConversionOkButton;

#[derive(Component)]
pub struct SqliteConversionYesButton;

#[derive(Component)]
pub struct SqliteConversionNoButton;

#[derive(Component)]
pub struct SqliteConversionOkSlot;

#[derive(Component)]
pub struct SqliteConversionYesSlot;

#[derive(Component)]
pub struct SqliteConversionNoSlot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConversionPhase {
    Running,
    Finished,
    PromptIgnore,
    CloseRequested,
    CloseRequestedKeepLegacy,
    Failed,
}

#[derive(Resource)]
pub struct SqliteConversionState {
    phase: ConversionPhase,
    sqlite_path: std::path::PathBuf,
    rows: Vec<sqlite_conversion::LegacySqliteCharacterRow>,
    legacy_ids: Vec<i64>,
    total: usize,
    processed: usize,
    converted: usize,
    skipped: usize,
    failed: usize,
    last_error: Option<String>,
}

#[derive(Resource)]
pub struct LegacySqliteIgnorePromptDismissedThisRun;

impl SqliteConversionState {
    fn progress_percent(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            (self.processed as f32 / self.total as f32).clamp(0.0, 1.0)
        }
    }

    fn status_line(&self) -> String {
        match self.phase {
            ConversionPhase::Running => format!(
                "Converting characters… {}/{} (converted {}, skipped {}, failed {})",
                self.processed, self.total, self.converted, self.skipped, self.failed
            ),
            ConversionPhase::Finished => format!(
                "Conversion complete. {}/{} (converted {}, skipped {}, failed {})",
                self.processed, self.total, self.converted, self.skipped, self.failed
            ),
            ConversionPhase::PromptIgnore => {
                "Conversion verified. Ignore the old SQLite data in the future?".to_string()
            }
            ConversionPhase::CloseRequested => "Saving preference…".to_string(),
            ConversionPhase::CloseRequestedKeepLegacy => "Closing…".to_string(),
            ConversionPhase::Failed => {
                let err = self.last_error.as_deref().unwrap_or("Unknown error");
                format!(
                    "Conversion failed. {}/{} (converted {}, skipped {}, failed {}). {}",
                    self.processed, self.total, self.converted, self.skipped, self.failed, err
                )
            }
        }
    }
}

fn spawn_conversion_dialog(commands: &mut Commands, theme: &MaterialTheme) -> Entity {
    let dialog = MaterialDialog::new()
        .title("Converting Database")
        .open(true)
        .modal(true);

    let dialog_surface = dialog.surface_color(theme);

    let dialog_entity = commands
        .spawn((
            dialog,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: Val::Px(640.0),
                height: Val::Px(220.0),
                min_width: Val::Px(640.0),
                max_width: Val::Px(640.0),
                min_height: Val::Px(220.0),
                max_height: Val::Px(220.0),
                padding: UiRect::all(Val::Px(Spacing::EXTRA_LARGE)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(dialog_surface),
            BorderRadius::all(Val::Px(CornerRadius::EXTRA_LARGE)),
            BoxShadow::default(),
            SqliteConversionDialog,
            ZIndex(10_000),
        ))
        .id();

    let scrim_entity = commands
        .spawn((
            create_dialog_scrim_for(theme, dialog_entity, true),
            SqliteConversionOverlay,
            ZIndex(9_999),
        ))
        .id();

    commands.entity(scrim_entity).add_child(dialog_entity);

    commands.entity(dialog_entity).with_children(|dialog| {
        dialog.spawn((
            Text::new("Converting legacy character database"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(theme.on_surface),
        ));

        // Scrollable content area so long status/error text doesn't push buttons off-screen.
        dialog
            .spawn((
                ScrollContainer::vertical(),
                ScrollPosition::default(),
                Node {
                    width: Val::Percent(100.0),
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(0.0),
                    flex_grow: 1.0,
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
                        // Leave a little gutter so the scrollbar doesn't overlap text.
                        padding: UiRect {
                            left: Val::Px(2.0),
                            right: Val::Px(18.0),
                            top: Val::Px(2.0),
                            bottom: Val::Px(2.0),
                        },
                        ..default()
                    })
                    .with_children(|content| {
                        content.spawn((
                            Text::new("Starting…"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                            SqliteConversionStatusText,
                        ));

                        content.spawn((
                            LinearProgressBuilder::new().progress(0.0).build(theme),
                            SqliteConversionProgressBar,
                        ));

                        content.spawn((
                            Text::new("Please keep the app open until this finishes."),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(theme.on_surface_variant),
                        ));
                    });
            });

        // Buttons row
        dialog
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                column_gap: Val::Px(10.0),
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|buttons| {
                // OK button (disabled while conversion runs)
                buttons
                    .spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(36.0),
                            ..default()
                        },
                        SqliteConversionOkSlot,
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            MaterialButtonBuilder::new("OK").filled().build(theme),
                            SqliteConversionOkButton,
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

                // Yes button (hidden until after verification)
                buttons
                    .spawn((
                        Node {
                            display: Display::None,
                            width: Val::Px(100.0),
                            height: Val::Px(36.0),
                            ..default()
                        },
                        SqliteConversionYesSlot,
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            MaterialButtonBuilder::new("Yes").filled().build(theme),
                            SqliteConversionYesButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Yes"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(theme.on_primary),
                                ButtonLabel,
                            ));
                        });
                    });

                // No button (hidden until after verification)
                buttons
                    .spawn((
                        Node {
                            display: Display::None,
                            width: Val::Px(100.0),
                            height: Val::Px(36.0),
                            ..default()
                        },
                        SqliteConversionNoSlot,
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            MaterialButtonBuilder::new("No").outlined().build(theme),
                            SqliteConversionNoButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("No"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(theme.primary),
                                ButtonLabel,
                            ));
                        });
                    });
            });
    });

    scrim_entity
}

/// Detect legacy SQLite data and begin conversion (spawns the dialog).
pub fn start_sqlite_conversion_if_needed(
    mut commands: Commands,
    theme: Option<Res<MaterialTheme>>,
    db: Option<Res<CharacterDatabase>>,
    already_running: Option<Res<SqliteConversionState>>,
    dismissed: Option<Res<LegacySqliteIgnorePromptDismissedThisRun>>,
    mut log_state: Local<HashSet<&'static str>>,
) {
    if already_running.is_some() {
        return;
    }

    // If the user declined the ignore prompt in this run, don't keep prompting.
    if dismissed.is_some() {
        return;
    }

    let Some(theme) = theme else {
        return;
    };

    let Some(db) = db else {
        return;
    };

    let ignore_legacy = db
        .get_setting::<bool>(IGNORE_LEGACY_SQLITE_SETTING_KEY)
        .ok()
        .flatten()
        .unwrap_or(false);
    log_once(&mut log_state, "ignore_setting", || {
        info!(
            "Legacy SQLite conversion: ignore_legacy_sqlite = {}",
            ignore_legacy
        );
    });

    if ignore_legacy {
        return;
    }

    let Some(sqlite_path) = find_legacy_sqlite_path(&db) else {
        // Helpful breadcrumb in case the user put the file somewhere else.
        let primary = sqlite_conversion::legacy_sqlite_path();
        log_once(&mut log_state, "missing_sqlite", || {
            info!(
                "Legacy SQLite conversion: no characters.db found (checked {:?} and {:?})",
                primary,
                db.db_path.join("characters.db")
            );
        });
        return;
    };

    log_once(&mut log_state, "found_sqlite", || {
        info!(
            "Legacy SQLite conversion: found legacy DB at {:?}",
            sqlite_path
        );
    });

    let rows = match sqlite_conversion::load_legacy_character_rows(&sqlite_path) {
        Ok(rows) => rows,
        Err(e) => {
            log_once(&mut log_state, "sqlite_read_failed", || {
                warn!(
                    "Legacy SQLite conversion: failed to read {:?}: {}",
                    sqlite_path, e
                );
            });
            return;
        }
    };

    if rows.is_empty() {
        log_once(&mut log_state, "sqlite_empty", || {
            info!(
                "Legacy SQLite conversion: {:?} contains 0 rows in 'characters' (no migration needed)",
                sqlite_path
            );
        });

        log_once(&mut log_state, "sqlite_empty_hints", || {
            let wal = sqlite_path.with_extension("db-wal");
            let shm = sqlite_path.with_extension("db-shm");
            let wal_exists = wal.exists();
            let shm_exists = shm.exists();

            if !wal_exists && !shm_exists {
                info!(
                    "Legacy SQLite conversion: no WAL/SHM sidecar files found (looked for {:?} and {:?}). If the legacy DB used WAL mode, copy these files alongside characters.db.",
                    wal,
                    shm
                );
            } else {
                info!(
                    "Legacy SQLite conversion: WAL/SHM sidecar presence: {:?}={}, {:?}={}",
                    wal, wal_exists, shm, shm_exists
                );
            }

            match sqlite_conversion::probe_legacy_sqlite(&sqlite_path) {
                Ok(probe) => {
                    info!(
                        "Legacy SQLite conversion: file_size_bytes={:?} modified={:?}",
                        probe.file_size_bytes, probe.file_modified
                    );
                    info!(
                        "Legacy SQLite conversion: journal_mode={:?} user_version={:?} schema_version={:?}",
                        probe.journal_mode,
                        probe.user_version,
                        probe.schema_version
                    );
                    info!(
                        "Legacy SQLite conversion: characters_count={:?} characters_non_null_data_count={:?}",
                        probe.characters_row_count,
                        probe.characters_non_null_data_count
                    );
                    info!(
                        "Legacy SQLite conversion: detected tables: {:?}",
                        probe.tables
                    );
                    info!(
                        "Legacy SQLite conversion: 'characters' columns: {:?}",
                        probe.characters_columns
                    );
                }
                Err(e) => {
                    info!(
                        "Legacy SQLite conversion: schema probe failed for {:?}: {}",
                        sqlite_path, e
                    );
                }
            }
        });
        return;
    }

    log_once(&mut log_state, "sqlite_rows", || {
        info!(
            "Legacy SQLite conversion: loaded {} legacy rows from {:?}",
            rows.len(),
            sqlite_path
        );
    });

    // If everything has already been converted/skipped, just back up the SQLite file.
    let mut any_unconverted = false;
    for row in &rows {
        let exists_by_id = db.character_id_exists(row.legacy_id).unwrap_or(false);
        let exists_by_legacy = db.legacy_sqlite_id_exists(row.legacy_id).unwrap_or(false);
        if !exists_by_id && !exists_by_legacy {
            any_unconverted = true;
            break;
        }
    }

    if !any_unconverted {
        log_once(&mut log_state, "already_converted", || {
            info!(
                "Legacy SQLite conversion: all rows already exist in SurrealDB; prompting for ignore preference for {:?}",
                sqlite_path
            );
        });

        // Everything is already migrated; still ask the user whether to ignore the legacy
        // sqlite in the future (and only back it up if they answer Yes).
        spawn_conversion_dialog(&mut commands, &theme);

        let legacy_ids = rows.iter().map(|r| r.legacy_id).collect::<Vec<_>>();
        let total = rows.len();

        commands.insert_resource(SqliteConversionState {
            phase: ConversionPhase::PromptIgnore,
            sqlite_path,
            total,
            rows: Vec::new(),
            legacy_ids,
            processed: total,
            converted: 0,
            skipped: total,
            failed: 0,
            last_error: None,
        });

        return;
    }

    spawn_conversion_dialog(&mut commands, &theme);

    let legacy_ids = rows.iter().map(|r| r.legacy_id).collect::<Vec<_>>();

    commands.insert_resource(SqliteConversionState {
        phase: ConversionPhase::Running,
        sqlite_path,
        total: rows.len(),
        rows,
        legacy_ids,
        processed: 0,
        converted: 0,
        skipped: 0,
        failed: 0,
        last_error: None,
    });
}

/// Convert a small number of rows per frame to keep the UI responsive.
pub fn run_sqlite_conversion_step(
    state: Option<ResMut<SqliteConversionState>>,
    db: Option<Res<CharacterDatabase>>,
) {
    let (Some(mut state), Some(db)) = (state, db) else {
        return;
    };

    if state.phase != ConversionPhase::Running {
        return;
    }

    const ROWS_PER_FRAME: usize = 5;

    for _ in 0..ROWS_PER_FRAME {
        let Some(row) = state.rows.pop() else {
            state.phase = ConversionPhase::Finished;
            break;
        };

        // We process from the back; total/progress are tracked independently.
        let legacy_id = row.legacy_id;

        let exists_by_id = db.character_id_exists(legacy_id).unwrap_or(false);
        let exists_by_legacy = db.legacy_sqlite_id_exists(legacy_id).unwrap_or(false);
        if exists_by_id || exists_by_legacy {
            state.skipped += 1;
            state.processed += 1;
            continue;
        }

        match decode_legacy_character(&row.data) {
            Ok(sheet) => match db.upsert_legacy_character(legacy_id, &sheet) {
                Ok(()) => {
                    state.converted += 1;
                    state.processed += 1;
                }
                Err(e) => {
                    state.failed += 1;
                    state.processed += 1;
                    state.last_error = Some(e);
                    // Keep going; we still want to attempt subsequent rows.
                }
            },
            Err(e) => {
                state.failed += 1;
                state.processed += 1;
                state.last_error = Some(format!(
                    "Failed to decode legacy character {}: {}",
                    legacy_id, e
                ));
            }
        }
    }
}

/// Update the dialog UI based on current progress.
pub fn update_sqlite_conversion_dialog_ui(
    state: Option<Res<SqliteConversionState>>,
    mut progress: Query<&mut MaterialLinearProgress, With<SqliteConversionProgressBar>>,
    mut status: Query<&mut Text, With<SqliteConversionStatusText>>,
    mut buttons: ParamSet<(
        Query<&mut MaterialButton, With<SqliteConversionOkButton>>,
        Query<&mut MaterialButton, With<SqliteConversionYesButton>>,
        Query<&mut MaterialButton, With<SqliteConversionNoButton>>,
    )>,
    mut slots: ParamSet<(
        Query<&mut Node, With<SqliteConversionOkSlot>>,
        Query<&mut Node, With<SqliteConversionYesSlot>>,
        Query<&mut Node, With<SqliteConversionNoSlot>>,
    )>,
) {
    let Some(state) = state else {
        return;
    };

    for mut p in &mut progress {
        p.progress = state.progress_percent();
    }

    for mut text in &mut status {
        *text = Text::new(state.status_line());
    }

    let is_finished = state.phase == ConversionPhase::Finished;
    let is_prompt = state.phase == ConversionPhase::PromptIgnore;
    let is_closing = state.phase == ConversionPhase::CloseRequested;
    let is_closing_keep = state.phase == ConversionPhase::CloseRequestedKeepLegacy;

    for mut btn in buttons.p0().iter_mut() {
        btn.disabled = !is_finished;
    }

    for mut btn in buttons.p1().iter_mut() {
        btn.disabled = !is_prompt;
    }

    for mut btn in buttons.p2().iter_mut() {
        btn.disabled = !is_prompt;
    }

    for mut node in slots.p0().iter_mut() {
        node.display = if is_prompt || is_closing || is_closing_keep {
            Display::None
        } else {
            Display::Flex
        };
    }

    for mut node in slots.p1().iter_mut() {
        node.display = if is_prompt {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in slots.p2().iter_mut() {
        node.display = if is_prompt {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// After conversion completes, OK verifies conversion and asks about ignoring legacy data.
pub fn handle_sqlite_conversion_ok_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    ok_query: Query<(), With<SqliteConversionOkButton>>,
    state: Option<ResMut<SqliteConversionState>>,
    db: Option<Res<CharacterDatabase>>,
) {
    let (Some(mut state), Some(db)) = (state, db) else {
        return;
    };

    if state.phase != ConversionPhase::Finished {
        return;
    }

    let mut clicked = false;
    for event in click_events.read() {
        if ok_query.get(event.entity).is_ok() {
            clicked = true;
            break;
        }
    }
    if !clicked {
        return;
    }

    // Verify that each legacy id exists in the new system.
    let mut missing: Vec<i64> = Vec::new();
    for legacy_id in &state.legacy_ids {
        let exists_by_id = db.character_id_exists(*legacy_id).unwrap_or(false);
        let exists_by_legacy = db.legacy_sqlite_id_exists(*legacy_id).unwrap_or(false);
        if !exists_by_id && !exists_by_legacy {
            missing.push(*legacy_id);
            if missing.len() >= 5 {
                break;
            }
        }
    }

    if missing.is_empty() {
        state.phase = ConversionPhase::PromptIgnore;
        state.last_error = None;
    } else {
        state.phase = ConversionPhase::Failed;
        state.last_error = Some(format!(
            "Verification failed: {} legacy characters still missing (e.g. {:?})",
            missing.len(),
            missing
        ));
    }
}

/// Yes saves the ignore option and closes the dialog.
pub fn handle_sqlite_conversion_yes_click(
    mut click_events: MessageReader<ButtonClickEvent>,
    yes_query: Query<(), With<SqliteConversionYesButton>>,
    state: Option<ResMut<SqliteConversionState>>,
    db: Option<Res<CharacterDatabase>>,
) {
    let (Some(mut state), Some(db)) = (state, db) else {
        return;
    };

    if state.phase != ConversionPhase::PromptIgnore {
        return;
    }

    let mut clicked = false;
    for event in click_events.read() {
        if yes_query.get(event.entity).is_ok() {
            clicked = true;
            break;
        }
    }
    if !clicked {
        return;
    }

    if let Err(e) = db.set_setting(IGNORE_LEGACY_SQLITE_SETTING_KEY, true) {
        state.phase = ConversionPhase::Failed;
        state.last_error = Some(format!("Failed to save preference: {}", e));
        return;
    }

    // Close the dialog and ignore legacy sqlite going forward.
    state.phase = ConversionPhase::CloseRequested;
    state.last_error = None;
}

/// No closes the dialog without saving the ignore preference.
pub fn handle_sqlite_conversion_no_click(
    mut commands: Commands,
    mut click_events: MessageReader<ButtonClickEvent>,
    no_query: Query<(), With<SqliteConversionNoButton>>,
    state: Option<ResMut<SqliteConversionState>>,
) {
    let Some(mut state) = state else {
        return;
    };

    if state.phase != ConversionPhase::PromptIgnore {
        return;
    }

    let mut clicked = false;
    for event in click_events.read() {
        if no_query.get(event.entity).is_ok() {
            clicked = true;
            break;
        }
    }
    if !clicked {
        return;
    }

    // Treat "No" as "not now": don't reprompt repeatedly in the same run.
    commands.insert_resource(LegacySqliteIgnorePromptDismissedThisRun);

    state.phase = ConversionPhase::CloseRequestedKeepLegacy;
    state.last_error = None;
}

/// Finalize conversion: refresh character list and back up legacy sqlite file.
pub fn finalize_sqlite_conversion_if_done(
    mut commands: Commands,
    state: Option<Res<SqliteConversionState>>,
    db: Option<Res<CharacterDatabase>>,
    manager: Option<ResMut<CharacterManager>>,
    overlay: Query<Entity, With<SqliteConversionOverlay>>,
) {
    let (Some(state), Some(db), Some(mut manager)) = (state, db, manager) else {
        return;
    };

    let should_backup_legacy = match state.phase {
        ConversionPhase::CloseRequested => true,
        ConversionPhase::CloseRequestedKeepLegacy => false,
        _ => return,
    };

    // Back up legacy sqlite only if the user chose to ignore it.
    if should_backup_legacy {
        let _ = sqlite_conversion::backup_legacy_sqlite(&state.sqlite_path);
    }

    // Refresh character list resource.
    let characters: Vec<CharacterListEntry> = db.list_characters().unwrap_or_default();
    manager.characters = characters;
    manager.current_character_id = None;
    manager.list_version = manager.list_version.wrapping_add(1);

    for e in &overlay {
        commands.entity(e).despawn();
    }

    commands.remove_resource::<SqliteConversionState>();
}
