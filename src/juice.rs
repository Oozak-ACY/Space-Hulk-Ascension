//! Screen shake and freeze frames: combat's punch, triggered by gameplay
//! events. Freezing holds gameplay time still; the client draws the shake.

use crate::{
    combat::{DamageTaken, EnemyDamaged, EnemyKilled},
    data::Tuning,
    run::StartRun,
    schedule::GameSystemSet,
};
use bevy::prelude::*;

/// How far the screen shakes, px; dies down over time.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct ScreenShake(pub f32);

/// Real seconds left during which gameplay time stands still.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct FreezeFrame(pub f32);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<ScreenShake>()
        .init_resource::<FreezeFrame>()
        .add_systems(
            Update,
            (
                (reset_on_new_run, thaw, settle_shake).in_set(GameSystemSet::InputAndIntent),
                react_to_combat.in_set(GameSystemSet::EventEvaluation),
            ),
        );
}

fn reset_on_new_run(
    mut requests: MessageReader<StartRun>,
    mut shake: ResMut<ScreenShake>,
    mut freeze: ResMut<FreezeFrame>,
    mut time: ResMut<Time<Virtual>>,
) {
    if requests.read().last().is_some() {
        *shake = ScreenShake::default();
        *freeze = FreezeFrame::default();
        time.set_relative_speed(1.0);
        time.unpause();
    }
}

/// Counts the freeze down in real time; gameplay time resumes once it is over.
fn thaw(real: Res<Time<Real>>, mut freeze: ResMut<FreezeFrame>, mut time: ResMut<Time<Virtual>>) {
    if freeze.0 <= 0.0 {
        return;
    }
    freeze.0 = (freeze.0 - real.delta_secs()).max(0.0);
    if freeze.0 <= 0.0 {
        time.set_relative_speed(1.0);
    }
}

/// The shake dies down in real time, so it settles on the Game over screen
/// and while paused.
fn settle_shake(real: Res<Time<Real>>, tuning: Res<Tuning>, mut shake: ResMut<ScreenShake>) {
    shake.0 = (shake.0 - tuning.juice.shake_decay * real.delta_secs()).max(0.0);
}

/// Hits shake a little, kills more, being hurt the most; kills and being
/// hurt also freeze gameplay. The strongest trigger wins.
fn react_to_combat(
    tuning: Res<Tuning>,
    mut hurt: MessageReader<DamageTaken>,
    mut hits: MessageReader<EnemyDamaged>,
    mut kills: MessageReader<EnemyKilled>,
    mut shake: ResMut<ScreenShake>,
    mut freeze: ResMut<FreezeFrame>,
    mut time: ResMut<Time<Virtual>>,
) {
    let juice = &tuning.juice;
    let mut freeze_for = 0.0_f32;
    if hits.read().count() > 0 {
        shake.0 = shake.0.max(juice.shake_on_hit);
    }
    if kills.read().count() > 0 {
        shake.0 = shake.0.max(juice.shake_on_kill);
        freeze_for = freeze_for.max(juice.freeze_on_kill);
    }
    if hurt.read().count() > 0 {
        shake.0 = shake.0.max(juice.shake_on_hurt);
        freeze_for = freeze_for.max(juice.freeze_on_hurt);
    }
    if freeze_for > freeze.0 {
        freeze.0 = freeze_for;
        time.set_relative_speed(0.0);
    }
}
