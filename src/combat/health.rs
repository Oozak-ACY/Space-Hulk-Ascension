//! HP, damage requests and what follows them: mitigation, post-hit
//! invulnerability, kills and fatal damage.

use crate::{
    combat::{DamageType, Resistances},
    data::{DodgeRollTuning, Tuning},
    enemies::Enemy,
    player::DodgeRoll,
    run::AppState,
    schedule::GameSystemSet,
};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    current: f32,
    max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    /// From 0 (dead) to 1 (unhurt).
    pub fn fraction(&self) -> f32 {
        self.current / self.max
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Seconds left during which damage is ignored, after being hurt. Only the
/// player has it.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct HurtInvulnerability(pub f32);

impl HurtInvulnerability {
    pub fn is_active(&self) -> bool {
        self.0 > 0.0
    }
}

/// Seconds left showing white after being hit.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct HitFlash(pub f32);

impl HitFlash {
    pub fn is_active(&self) -> bool {
        self.0 > 0.0
    }
}

/// Asks to hurt `target`; Resistances and invulnerability are applied after.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct DamageRequest {
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
    /// The weapon id it came from, if any.
    pub weapon: Option<String>,
}

/// The player took damage, after mitigation.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct DamageTaken {
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}

/// An enemy took damage, after mitigation.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct EnemyDamaged {
    pub enemy: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}

/// An enemy reached 0 HP and is gone.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct EnemyKilled {
    pub enemy: Entity,
    /// Its enemy id.
    pub id: String,
    /// The weapon id that dealt the killing blow, if any.
    pub weapon: Option<String>,
}

/// The player reached 0 HP. Death interception decides whether it is death.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FatalDamage {
    pub entity: Entity,
}

/// Cannot be hurt right now: Dodge Rolling through the invincibility
/// window, or just hurt.
pub fn is_untouchable(
    roll: Option<&DodgeRoll>,
    hurt: Option<&HurtInvulnerability>,
    roll_tuning: &DodgeRollTuning,
) -> bool {
    roll.is_some_and(|roll| roll.is_invincible(roll_tuning))
        || hurt.is_some_and(HurtInvulnerability::is_active)
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<DamageRequest>()
        .add_message::<DamageTaken>()
        .add_message::<EnemyDamaged>()
        .add_message::<EnemyKilled>()
        .add_message::<FatalDamage>()
        .add_systems(
            Update,
            (
                tick_hurt_timers.in_set(GameSystemSet::PhysicsAndCombat),
                (apply_damage, intercept_death)
                    .chain()
                    .in_set(GameSystemSet::DamageAndDeathHooks),
            ),
        );
}

fn tick_hurt_timers(
    time: Res<Time>,
    mut invulnerabilities: Query<&mut HurtInvulnerability>,
    mut flashes: Query<&mut HitFlash>,
) {
    let dt = time.delta_secs();
    for mut invulnerability in &mut invulnerabilities {
        invulnerability.0 = (invulnerability.0 - dt).max(0.0);
    }
    for mut flash in &mut flashes {
        flash.0 = (flash.0 - dt).max(0.0);
    }
}

type Target<'a> = (
    &'a mut Health,
    Option<&'a Resistances>,
    Option<&'a DodgeRoll>,
    Option<&'a mut HurtInvulnerability>,
    Option<&'a mut HitFlash>,
    Option<&'a Enemy>,
);

#[allow(clippy::too_many_arguments)]
fn apply_damage(
    tuning: Res<Tuning>,
    mut requests: MessageReader<DamageRequest>,
    mut targets: Query<Target>,
    mut taken: MessageWriter<DamageTaken>,
    mut damaged: MessageWriter<EnemyDamaged>,
    mut kills: MessageWriter<EnemyKilled>,
    mut fatal: MessageWriter<FatalDamage>,
    mut commands: Commands,
) {
    for request in requests.read() {
        let Ok((mut health, resistances, roll, hurt, flash, enemy)) =
            targets.get_mut(request.target)
        else {
            continue;
        };
        if health.is_dead() || is_untouchable(roll, hurt.as_deref(), &tuning.player.dodge_roll) {
            continue;
        }
        let amount = resistances.map_or(request.amount, |resistances| {
            resistances.mitigate(
                request.amount,
                request.damage_type,
                tuning.combat.max_resistance,
            )
        });
        health.current = (health.current - amount).max(0.0);
        if enemy.is_some() {
            damaged.write(EnemyDamaged {
                enemy: request.target,
                amount,
                damage_type: request.damage_type,
            });
        }
        // Only the player has post-hit invulnerability.
        let is_player = hurt.is_some();
        if let Some(mut hurt) = hurt {
            hurt.0 = tuning.player.hurt_invulnerability;
            taken.write(DamageTaken {
                target: request.target,
                amount,
                damage_type: request.damage_type,
            });
        }
        if let Some(mut flash) = flash {
            flash.0 = if is_player {
                tuning.juice.hurt_flash
            } else {
                tuning.juice.hit_flash
            };
        }
        if !health.is_dead() {
            continue;
        }
        if let Some(enemy) = enemy {
            kills.write(EnemyKilled {
                enemy: request.target,
                id: enemy.id().to_owned(),
                weapon: request.weapon.clone(),
            });
            commands.entity(request.target).despawn();
        } else {
            fatal.write(FatalDamage {
                entity: request.target,
            });
        }
    }
}

/// Every fatal blow passes through here before it is death, so a
/// Death-triggered Ascension can cancel it. None exist yet: it is Game over.
fn intercept_death(mut fatal: MessageReader<FatalDamage>, mut state: ResMut<NextState<AppState>>) {
    if fatal.read().last().is_some() {
        state.set(AppState::GameOver);
    }
}
