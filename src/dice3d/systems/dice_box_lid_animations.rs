use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::animation::prelude::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex};
use bevy::prelude::MessageReader;
use bevy_rapier3d::prelude::Velocity;
use rand::Rng;

use crate::dice3d::embedded_assets::BOX_MODEL_GLTF_PATH;
use crate::dice3d::systems::dice_box_controls::start_container_shake;
use crate::dice3d::systems::{calculate_dice_position, spawn_die};
use crate::dice3d::throw_control::ThrowControlState;
use crate::dice3d::types::*;

#[derive(bevy::ecs::system::SystemParam)]
pub struct PendingRollExecutionParams<'w, 's> {
    pub roll_state: ResMut<'w, RollState>,
    pub dice_results: ResMut<'w, DiceResults>,
    pub dice_query: Query<'w, 's, (&'static mut Transform, &'static mut Velocity), (With<Die>, Without<DiceBox>)>,
    pub dice_config: ResMut<'w, DiceConfig>,
    pub db: Res<'w, CharacterDatabase>,
    pub command_history: ResMut<'w, CommandHistory>,
    pub throw_state: Res<'w, ThrowControlState>,
    pub settings_state: Res<'w, SettingsState>,

    pub shake_state: Res<'w, ShakeState>,
    pub shake_config: Res<'w, ContainerShakeConfig>,
    pub shake_anim: ResMut<'w, ContainerShakeAnimation>,
    pub container_query: Query<'w, 's, (Entity, &'static Transform), With<DiceBox>>,

    pub dice_entities: Query<'w, 's, Entity, With<Die>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
}

fn find_first_animation_player_under(
    root: Entity,
    children: &Query<&Children>,
    players: &Query<(), With<AnimationPlayer>>,
) -> Option<Entity> {
    let mut stack: Vec<Entity> = vec![root];
    while let Some(e) = stack.pop() {
        if players.get(e).is_ok() {
            return Some(e);
        }
        if let Ok(ch) = children.get(e) {
            for child in ch.iter() {
                stack.push(child);
            }
        }
    }
    None
}

fn find_all_animation_players_under(
    root: Entity,
    children: &Query<&Children>,
    players: &Query<(), With<AnimationPlayer>>,
) -> Vec<Entity> {
    let mut found: Vec<Entity> = Vec::new();
    let mut stack: Vec<Entity> = vec![root];
    while let Some(e) = stack.pop() {
        if players.get(e).is_ok() {
            found.push(e);
        }
        if let Ok(ch) = children.get(e) {
            for child in ch.iter() {
                stack.push(child);
            }
        }
    }
    found
}

#[cfg(debug_assertions)]
fn debug_trace_lid_controller(ctrl: &mut DiceBoxLidAnimationController) {
    let lid_state = ctrl.lid_state;
    let pending_roll_some = ctrl.pending_roll.is_some();
    let pending_open = ctrl.pending_open_after_roll;

    let state_changed = ctrl.debug_last_lid_state != Some(lid_state);
    let roll_changed = ctrl.debug_last_pending_roll_some != Some(pending_roll_some);
    let open_changed = ctrl.debug_last_pending_open_after_roll != Some(pending_open);

    if state_changed || roll_changed || open_changed {
        info!(
            "Lid ctrl: state={:?} timer={:.3} pending_roll={} pending_open_after_roll={} animator={:?}",
            lid_state,
            ctrl.state_timer,
            pending_roll_some,
            pending_open,
            ctrl.animator_entity
        );

        ctrl.debug_last_lid_state = Some(lid_state);
        ctrl.debug_last_pending_roll_some = Some(pending_roll_some);
        ctrl.debug_last_pending_open_after_roll = Some(pending_open);
    }
}

fn clip_duration_seconds(clips: &Assets<AnimationClip>, handle: &Handle<AnimationClip>) -> f32 {
    clips
        .get(handle)
        .map(|c| c.duration())
        .unwrap_or(0.6)
        .max(0.01)
}

fn play_once(
    ctrl: &mut DiceBoxLidAnimationController,
    players: &mut Query<&mut AnimationPlayer>,
    node: AnimationNodeIndex,
    duration_seconds: f32,
    new_state: DiceBoxLidState,
) {
    let Some(e) = ctrl.animator_entity else {
        return;
    };
    let Ok(mut player) = players.get_mut(e) else {
        ctrl.animator_entity = None;
        return;
    };

    // Ensure only one lid animation is active at once.
    // (If the AnimationPlayer has any lingering tracks, stopping only known nodes can miss them.)
    player.stop_all();

    // We are about to play a transient one-shot, so no idle should be considered active.
    ctrl.active_idle_node = None;

    // Play from the start, once.
    player.start(node);

    ctrl.lid_state = new_state;
    ctrl.state_timer = duration_seconds.max(0.01);
}

fn ensure_idle_looping(ctrl: &mut DiceBoxLidAnimationController, players: &mut Query<&mut AnimationPlayer>) {
    let Some(e) = ctrl.animator_entity else {
        return;
    };
    let Ok(mut player) = players.get_mut(e) else {
        ctrl.animator_entity = None;
        return;
    };

    // Only keep idle running when we're not actively opening/closing.
    if ctrl.lid_state == DiceBoxLidState::Opening || ctrl.lid_state == DiceBoxLidState::Closing {
        if let Some(idle_opened) = ctrl.idle_opened_node {
            player.stop(idle_opened);
        }
        if let Some(idle_closed) = ctrl.idle_closed_node {
            player.stop(idle_closed);
        }

        #[cfg(debug_assertions)]
        {
            if ctrl.debug_last_idle_node.take().is_some() {
                info!(
                    "Lid idle stopped (lid_state={:?})",
                    ctrl.lid_state
                );
            }
        }

        ctrl.active_idle_node = None;
        return;
    }

    let desired_idle = match ctrl.lid_state {
        DiceBoxLidState::Open => ctrl.idle_opened_node,
        DiceBoxLidState::Closed => ctrl.idle_closed_node,
        DiceBoxLidState::Opening | DiceBoxLidState::Closing => None,
    };

    let Some(desired_idle) = desired_idle else {
        return;
    };

    // Only start/restart the idle loop when switching which idle we want.
    // This avoids restarting at loop boundaries, which can look like flicker.
    if ctrl.active_idle_node != Some(desired_idle) {
        player.stop_all();
        player.play(desired_idle).repeat();
        ctrl.active_idle_node = Some(desired_idle);
    }

    #[cfg(debug_assertions)]
    {
        let last = ctrl.debug_last_idle_node;
        if last != Some(desired_idle) {
            let which = if ctrl.idle_opened_node == Some(desired_idle) {
                "LidIdleOpened"
            } else if ctrl.idle_closed_node == Some(desired_idle) {
                "LidIdleClosed"
            } else {
                "(unknown idle)"
            };
            info!(
                "Lid idle -> {} (lid_state={:?})",
                which,
                ctrl.lid_state
            );
            ctrl.debug_last_idle_node = Some(desired_idle);
        }
    }
}

pub fn ensure_dice_box_lid_animation_assets(
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    clips: Res<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut ctrl: ResMut<DiceBoxLidAnimationController>,
) {
    if ctrl.gltf_handle.is_none() {
        ctrl.gltf_handle = Some(asset_server.load(BOX_MODEL_GLTF_PATH));
    }

    let Some(h) = ctrl.gltf_handle.as_ref() else {
        return;
    };
    let Some(gltf) = gltfs.get(h) else {
        return;
    };

    // Prefer name-based lookup (Blender Action names).
    if ctrl.opening_clip.is_none() {
        if let Some(h) = gltf.named_animations.get("LidOpening") {
            ctrl.opening_clip = Some(h.clone());
        }
    }
    if ctrl.closing_clip.is_none() {
        if let Some(h) = gltf.named_animations.get("LidClosing") {
            ctrl.closing_clip = Some(h.clone());
        }
    }
    if ctrl.idle_opened_clip.is_none() {
        if let Some(h) = gltf.named_animations.get("LidIdleOpened") {
            ctrl.idle_opened_clip = Some(h.clone());
        }
    }
    if ctrl.idle_closed_clip.is_none() {
        if let Some(h) = gltf.named_animations.get("LidIdleClosed") {
            ctrl.idle_closed_clip = Some(h.clone());
        }
    }

    // Keep durations up-to-date once clips actually load.
    // (We can discover the clip handles before `Assets<AnimationClip>` contains them.
    // If we cached a fallback duration earlier, we'd incorrectly advance state early.)
    if let Some(h) = ctrl.opening_clip.as_ref() {
        let d = clip_duration_seconds(&clips, h);
        if (ctrl.open_duration - d).abs() > 0.001 {
            ctrl.open_duration = d;
        }
    }
    if let Some(h) = ctrl.closing_clip.as_ref() {
        let d = clip_duration_seconds(&clips, h);
        if (ctrl.close_duration - d).abs() > 0.001 {
            ctrl.close_duration = d;
        }
    }
    if let Some(h) = ctrl.idle_opened_clip.as_ref() {
        let d = clip_duration_seconds(&clips, h);
        if (ctrl.idle_opened_duration - d).abs() > 0.001 {
            ctrl.idle_opened_duration = d;
        }
    }
    if let Some(h) = ctrl.idle_closed_clip.as_ref() {
        let d = clip_duration_seconds(&clips, h);
        if (ctrl.idle_closed_duration - d).abs() > 0.001 {
            ctrl.idle_closed_duration = d;
        }
    }

    if ctrl.animation_graph.is_none() {
        let (Some(opening), Some(closing), Some(idle_opened), Some(idle_closed)) = (
            ctrl.opening_clip.clone(),
            ctrl.closing_clip.clone(),
            ctrl.idle_opened_clip.clone(),
            ctrl.idle_closed_clip.clone(),
        ) else {
            return;
        };

        let mut graph = AnimationGraph::new();
        let opening_node = graph.add_clip(opening, 1.0, graph.root);
        let closing_node = graph.add_clip(closing, 1.0, graph.root);
        let idle_opened_node = graph.add_clip(idle_opened, 1.0, graph.root);
        let idle_closed_node = graph.add_clip(idle_closed, 1.0, graph.root);
        let graph_handle = graphs.add(graph);

        ctrl.opening_node = Some(opening_node);
        ctrl.closing_node = Some(closing_node);
        ctrl.idle_opened_node = Some(idle_opened_node);
        ctrl.idle_closed_node = Some(idle_closed_node);
        ctrl.animation_graph = Some(graph_handle);
    }
}

pub fn cache_dice_box_lid_animation_player(
    mut commands: Commands,
    mut ctrl: ResMut<DiceBoxLidAnimationController>,
    container_style: Res<DiceContainerStyle>,
    visual_roots: Query<Entity, With<DiceBoxVisualSceneRoot>>,
    children: Query<&Children>,
    players: Query<(), With<AnimationPlayer>>,
    player_graphs: Query<Option<&AnimationGraphHandle>, With<AnimationPlayer>>,
) {
    if *container_style != DiceContainerStyle::Box {
        ctrl.animator_entity = None;
        return;
    }

    let Some(root) = visual_roots.iter().next() else {
        return;
    };

    // Prefer selecting the AnimationPlayer that is already wired to our graph, if present.
    // This avoids accidentally picking a different player in the same scene tree.
    if ctrl.animator_entity.is_none() {
        let all_players = find_all_animation_players_under(root, &children, &players);
        let preferred = ctrl
            .animation_graph
            .clone()
            .and_then(|graph| {
                all_players
                    .iter()
                    .copied()
                    .find(|e| player_graphs.get(*e).ok().flatten().is_some_and(|h| h.0 == graph))
            });

        ctrl.animator_entity = preferred.or_else(|| find_first_animation_player_under(root, &children, &players));

        #[cfg(debug_assertions)]
        {
            if !ctrl.debug_logged_player_scan {
                ctrl.debug_logged_player_scan = true;
                info!(
                    "DiceBox lid: found {} AnimationPlayer(s) under {:?}; using {:?}",
                    all_players.len(),
                    root,
                    ctrl.animator_entity
                );
                if all_players.len() > 1 {
                    info!("DiceBox lid: AnimationPlayers={:?}", all_players);
                }
            }
        }
    }

    // Important: the AnimationPlayer may be discovered before the AnimationGraph is built.
    // Always ensure the player entity has the current graph handle once both are available.
    if let (Some(animator), Some(graph)) = (ctrl.animator_entity, ctrl.animation_graph.clone()) {
        let needs_insert = match player_graphs.get(animator) {
            Ok(Some(existing)) => existing.0 != graph,
            Ok(None) => true,
            Err(_) => true,
        };

        if needs_insert {
            commands.entity(animator).insert(AnimationGraphHandle(graph));
        }
    }
}

pub fn process_pending_roll_with_lid(
    mut commands: Commands,
    time: Res<Time>,
    container_style: Res<DiceContainerStyle>,
    mut players: Query<&mut AnimationPlayer>,
    mut ctrl: ResMut<DiceBoxLidAnimationController>,

    mut exec: PendingRollExecutionParams,
) {
    if *container_style != DiceContainerStyle::Box {
        ctrl.pending_roll = None;
        return;
    }

    #[cfg(debug_assertions)]
    debug_trace_lid_controller(&mut ctrl);

    // Tick lid animation timer.
    if ctrl.lid_state == DiceBoxLidState::Closing || ctrl.lid_state == DiceBoxLidState::Opening {
        ctrl.state_timer = (ctrl.state_timer - time.delta_secs()).max(0.0);
        if ctrl.state_timer <= 0.0001 {
            ctrl.lid_state = match ctrl.lid_state {
                DiceBoxLidState::Closing => DiceBoxLidState::Closed,
                DiceBoxLidState::Opening => DiceBoxLidState::Open,
                other => other,
            };
        }
    }

    // If the last roll finished, we should open the lid and then remain in LidIdleOpened.
    // We run this here (instead of directly in the roll-completed handler) so the opening
    // animation still plays even if the event arrives before animation assets/nodes exist.
    if ctrl.pending_open_after_roll {
        match ctrl.lid_state {
            DiceBoxLidState::Open => {
                ctrl.pending_open_after_roll = false;
            }
            DiceBoxLidState::Opening => {
                // Already opening; don't retrigger.
                ctrl.pending_open_after_roll = false;
            }
            DiceBoxLidState::Closed => {
                if ctrl.pending_roll.is_none() {
                    if let Some(node) = ctrl.opening_node {
                        let duration = ctrl.open_duration;

                        #[cfg(debug_assertions)]
                        info!("Lid: play opening (duration={:.3})", duration);

                        play_once(&mut ctrl, &mut players, node, duration, DiceBoxLidState::Opening);
                        ctrl.pending_open_after_roll = false;
                    }
                    // If the node isn't ready yet, keep the request queued.
                }
            }
            DiceBoxLidState::Closing => {
                // Wait until fully closed.
            }
        }
    }

    // If a roll is queued, ensure the lid is CLOSED before executing it.
    if ctrl.pending_roll.is_some() {
        // Don't interrupt an in-progress opening animation.
        // Wait until it's fully open, then close.
        if ctrl.lid_state == DiceBoxLidState::Open {
            if ctrl.lid_state != DiceBoxLidState::Closing {
                if let Some(node) = ctrl.closing_node {
                    let duration = ctrl.close_duration;

                    #[cfg(debug_assertions)]
                    info!("Lid: play closing (duration={:.3})", duration);

                    play_once(&mut ctrl, &mut players, node, duration, DiceBoxLidState::Closing);
                } else {
                    // No animation clip available: treat as instantly closed.
                    ctrl.lid_state = DiceBoxLidState::Closed;
                    ctrl.state_timer = 0.0;
                }
            }
        }
    }

    // Once closed, execute the queued roll.
    if ctrl.lid_state == DiceBoxLidState::Closed {
        let Some(req) = ctrl.pending_roll.take() else {
            return;
        };

        match req {
            PendingRollRequest::RerollExisting => {
                if exec.roll_state.rolling {
                    return;
                }

                exec.roll_state.rolling = true;
                exec.dice_results.results.clear();

                let mut rng = rand::thread_rng();
                let num_dice = exec.dice_config.dice_to_roll.len();

                let use_shake = exec.settings_state.settings.default_roll_uses_shake;
                let base_velocity = exec.throw_state.calculate_throw_velocity();

                for (i, (mut transform, mut velocity)) in exec.dice_query.iter_mut().enumerate() {
                    let position = calculate_dice_position(i, num_dice);
                    transform.translation = position
                        + Vec3::new(
                            rng.gen_range(-0.3..0.3),
                            rng.gen_range(0.0..0.3),
                            rng.gen_range(-0.3..0.3),
                        );
                    transform.rotation = Quat::from_euler(
                        EulerRot::XYZ,
                        rng.gen_range(0.0..std::f32::consts::TAU),
                        rng.gen_range(0.0..std::f32::consts::TAU),
                        rng.gen_range(0.0..std::f32::consts::TAU),
                    );

                    if use_shake {
                        velocity.linvel = Vec3::ZERO;
                        velocity.angvel = Vec3::ZERO;
                    } else {
                        velocity.linvel = base_velocity
                            + Vec3::new(
                                rng.gen_range(-0.5..0.5),
                                rng.gen_range(-0.3..0.0),
                                rng.gen_range(-0.5..0.5),
                            );
                        velocity.angvel = exec.throw_state.calculate_angular_velocity(&mut rng);
                    }
                }

                if use_shake {
                    let _started = start_container_shake(
                        &exec.shake_state,
                        &exec.shake_config,
                        &mut exec.shake_anim,
                        &exec.container_query,
                    );
                }
            }
            PendingRollRequest::QuickRollSingleDie {
                die_type,
                modifier,
                modifier_name,
            } => {
                if exec.roll_state.rolling {
                    return;
                }

                // Remove old dice (Quick Rolls always uses exactly one die)
                for e in exec.dice_entities.iter() {
                    commands.entity(e).despawn();
                }

                // Update dice config
                exec.dice_config.dice_to_roll.clear();
                exec.dice_config.dice_to_roll.push(die_type);
                exec.dice_config.modifier = modifier;
                exec.dice_config.modifier_name = modifier_name.clone();

                // Add to command history (matches old behavior)
                let sign = if modifier >= 0 { "+" } else { "" };
                exec.command_history.add_command(format!(
                    "1d{} --checkon {} ({}{})",
                    die_type.max_value(),
                    modifier_name,
                    sign,
                    modifier
                ));
                let _ = exec.db.save_command_history(&exec.command_history.commands);

                // Trigger the roll
                exec.roll_state.rolling = true;
                exec.dice_results.results.clear();

                let die_scale = exec.settings_state.settings.dice_scales.scale_for(die_type);
                let die_entity = spawn_die(
                    &mut commands,
                    &mut exec.meshes,
                    &mut exec.materials,
                    die_type,
                    die_scale,
                    calculate_dice_position(0, 1),
                );

                let use_shake = exec.settings_state.settings.default_roll_uses_shake;
                let mut rng = rand::thread_rng();
                let base_velocity = exec.throw_state.calculate_throw_velocity();

                let transform = Transform::from_translation(Vec3::new(
                    rng.gen_range(-0.5..0.5),
                    1.0,
                    rng.gen_range(-0.5..0.5),
                ))
                .with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                ))
                .with_scale(Vec3::splat(die_scale));

                let velocity = if use_shake {
                    Velocity {
                        linvel: Vec3::ZERO,
                        angvel: Vec3::ZERO,
                    }
                } else {
                    Velocity {
                        linvel: base_velocity
                            + Vec3::new(
                                rng.gen_range(-0.5..0.5),
                                rng.gen_range(-0.3..0.0),
                                rng.gen_range(-0.5..0.5),
                            ),
                        angvel: exec.throw_state.calculate_angular_velocity(&mut rng),
                    }
                };

                commands.entity(die_entity).insert((transform, velocity));

                if use_shake {
                    let _started = start_container_shake(
                        &exec.shake_state,
                        &exec.shake_config,
                        &mut exec.shake_anim,
                        &exec.container_query,
                    );
                }
            }
            PendingRollRequest::StartNewRoll { config } => {
                if exec.roll_state.rolling {
                    return;
                }

                // Remove old dice
                for e in exec.dice_entities.iter() {
                    commands.entity(e).despawn();
                }

                // Apply config
                *exec.dice_config = config;
                exec.dice_results.results.clear();

                let use_shake = exec.settings_state.settings.default_roll_uses_shake;

                // Spawn new dice
                let mut spawned: Vec<Entity> = Vec::new();
                let total = exec.dice_config.dice_to_roll.len();
                for (i, die_type) in exec.dice_config.dice_to_roll.iter().copied().enumerate() {
                    let position = calculate_dice_position(i, total);
                    let die_scale = exec.settings_state.settings.dice_scales.scale_for(die_type);
                    let e = spawn_die(
                        &mut commands,
                        &mut exec.meshes,
                        &mut exec.materials,
                        die_type,
                        die_scale,
                        position,
                    );
                    spawned.push(e);
                }

                if use_shake {
                    for e in spawned {
                        commands.entity(e).insert(Velocity {
                            linvel: Vec3::ZERO,
                            angvel: Vec3::ZERO,
                        });
                    }

                    let _started = start_container_shake(
                        &exec.shake_state,
                        &exec.shake_config,
                        &mut exec.shake_anim,
                        &exec.container_query,
                    );
                }

                exec.roll_state.rolling = true;
            }
        }
    }

    // Whenever not opening/closing, keep the correct idle active.
    ensure_idle_looping(&mut ctrl, &mut players);
}

pub fn open_lid_on_roll_completed(
    mut events: MessageReader<DiceRollCompletedEvent>,
    container_style: Res<DiceContainerStyle>,
    mut players: Query<&mut AnimationPlayer>,
    mut ctrl: ResMut<DiceBoxLidAnimationController>,
) {
    if *container_style != DiceContainerStyle::Box {
        return;
    }

    if events.read().next().is_none() {
        return;
    }

    // Queue the opening so it reliably plays even if animation nodes are not ready yet.
    // The actual playback happens in `process_pending_roll_with_lid`.
    ctrl.pending_open_after_roll = true;

    // If we can play it immediately, do so (keeps behavior snappy).
    if ctrl.pending_roll.is_none() && ctrl.lid_state == DiceBoxLidState::Closed {
        if let Some(node) = ctrl.opening_node {
            let duration = ctrl.open_duration;
            play_once(&mut ctrl, &mut players, node, duration, DiceBoxLidState::Opening);
            ctrl.pending_open_after_roll = false;
        }
    }
}
