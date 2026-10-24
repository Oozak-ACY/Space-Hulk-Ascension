# ARCHITECTURE & GAME DESIGN SPECIFICATION (v3.0)
**Project:** WH40K Space Hulk Roguelike (Working Title: *Space Hulk Ascension*)
**Target Stack:** Rust + Bevy Engine (0.15+) + WebAssembly (`wasm32-unknown-unknown` via `Trunk`)
**Visual Style:** 2D Top-Down Pixel Art (480x270 virtual resolution, Nearest-Neighbor filtering)
**Core Gameplay:** 360° Twin-Stick Shooter in fixed single-screen Rooms (*Binding of Isaac* / *Enter the Gungeon* feel) + Dynamic Psyche, Concurrent Quests & secret, risk-based Ascensions.

> **Supersedes v2.** Vocabulary is defined in [`CONTEXT.md`](CONTEXT.md); every term in **bold** below means exactly what the glossary says. Architectural decisions are recorded in [`docs/adr/`](docs/adr/). When this spec and the glossary disagree, the glossary wins.

---

## 1. PITCH & VISION

### 1.1 The Elevator Pitch
You wake up as a nameless, fragile human — a **Mortal** — trapped inside a colossal, drifting **Space Hulk** in the Warhammer 40,000 universe: ancient Imperial warships, Chaos-corrupted hulks and Xenos vessels fused together by the Warp.

Every **Run** starts the same way: a basic laspistol and a mortal body. As you descend **Floor** by Floor, every choice — who you help, what you equip, which whispers you answer, which surgical or spiritual **Trials** you endure — reshapes your **Psyche**, your body and the Hulk itself.

You may remain a servant of the Emperor and **Ascend** into a Space Marine, a Sister of Battle, a Grey Knight or even a Living Saint. You may strip away your humanity with Mechanicus **Augmentics**. Or you may embrace Chaos and become a Heretic Astartes, a Daemon Prince or a rampaging Chaos Spawn. **As you change, the game inverts:** fall to heresy and the cultists become your allies while the Imperium sends Kill Teams and Grey Knights to purge you.

Death is final. The next Run starts again from a plain Mortal, with nothing carried over. The only measure of progress is how deep you get.

### 1.2 Core Design Pillars
1. **Zero to Demigod (or Abomination):** Classes are never chosen in a menu; they are earned mid-Run through dangerous, possibly fatal Trials gated by Sex, Stats, encounters and Psyche.
2. **Secret Ascensions:** The game never reveals which Ascensions exist or which the player has found. They are discovered by accident — the right NPC, the right relic, the right moment. No codex, no unlocks (ADR 0004).
3. **The Psyche Axis (`-100` to `+100`):** A single spiritual scalar. High Psyche brings zealotry and miracles, low Psyche brings Warp madness and mutation. **Allegiance**, which follows Psyche but can be locked, decides who is hostile (ADR 0002).
4. **Concurrent & Conflicting Quests:** Several **Active Quests** at once (Living Weapons, NPC Penance, Bounties, Trials). Completing one objective may instantly fail another.
5. **Long-Term Build Planning via Bounties:** Rare optional bosses announced early on deep Floors let the player specialize over many Floors — or skip them to survive.
6. **Balance is data:** Every number a balance patch could touch lives in `.ron` data files, never in Rust code (ADR 0003).

---

## 2. THE THREE GAMEPLAY LOOPS

### 2.1 Micro-Loop: Room Combat (30–60 seconds)
* **Controls:** 360° twin-stick (`WASD/ZQSD` move + 360° mouse aim + **Dodge Roll**).
* **Dodge Roll:** short roll with invincibility frames and a cooldown. Distance, duration, i-frame window and cooldown are tunables.
* **Room Entry:** Every **Room** is exactly one fixed 480x270 screen (ADR 0001). On entry, doors seal. The **Psyche Director** spawns enemies according to the player's **Allegiance** (Chaos/Xenos when Imperium or Unaligned; Imperial Purge Squads/Inquisition when Chaos).
* **During combat:** the player manages any **Class Meters** and respects their Active Quests.
* **Resolution:** Clearing the Room unlocks doors, ticks cooldowns, evaluates quest progress and failures, and spawns rewards.

### 2.2 Meso-Loop: Floor Exploration (~10 minutes)
* **Topology:** Rooms are placed on an Isaac-style grid with an organic, branching layout. Each Floor contains combat Rooms, hazards, 1–3 **Encounter Rooms**, and an Exit Elevator guarded by the Floor Boss.
* **Encounter Rooms:** non-combat Rooms with survivors, Apothecaries, Confessors, Inquisitors or bound Daemons. What they offer depends on Psyche (saving survivors vs. dark pacts and organ harvesting).
* **Redemption Path:** even deep in Chaos, a small chance of a captive Imperial Priest offering a **Penance Quest**, which purges **Corrupted Blessings** and can unlock a Chaos-locked Allegiance.
* **Post-Boss Sanctuary:** defeating a Floor Boss may open an Imperial Chapel or a Warp Breach depending on Floor performance and Psyche.
* **Suspend Save:** taken when the player enters a new Floor (see §8.8).

### 2.3 Macro-Loop: The Run (~20 Floors, several hours)
* **Pre-Run:** the player starts as a Mortal and picks their **Sex** (`Male` / `Female`), a permanent prerequisite for lore-bound Ascensions.
* **The Auspicium:** early in the Run, 1–3 **Bounties** are revealed on specific deep Floors with their damage profile and dual rewards. *(Floor placement is a balance question — see §10.)*
* **Trials:** meeting the right NPCs and requirements opens surgical shrines or spiritual Trials. Each resolves a **Survival Roll** (hard cap in `tuning.ron`, default 99%). Failure may mean death or a botched transformation; success permanently (for the Run) changes Class or grants a **Calling**.
* **Death-triggered Ascensions:** some Ascensions are attempted at the moment of death. Their chance is never certain, but it can be raised by actions during the Run.
* **Bounty Resolution:** winning a Bounty fight forces a choice — **claim the Corrupted Artifact** (power, heavy −Psyche) or **banish and claim the Imperial reward** (+Psyche, master-crafted gear).
* **Run end:** death ends the Run, deletes the Suspend Save and shows the deepest Floor reached. Nothing is recorded.

---

## 3. CHARACTER MODEL

### 3.1 Class, Calling, Augmentic
The player has:
* exactly one **Class** (body/identity),
* zero or more **Callings** (stackable gifts/roles),
* zero or more **Augmentics** and implants.

Combinations replace dedicated classes: a **Librarian** is an Astartes with the Psyker Calling; a **World Eater** is an Astartes or Heretic Astartes fitted with the Butcher's Nails implant.

### 3.2 Stats
Five primary **Stats**: `Toughness`, `Willpower`, `BallisticSkill`, `WeaponSkill`, `Agility`. There is no Luck stat. Randomness is randomness.

### 3.3 Ascension Catalog
*(Reference for design validation. All numeric values are placeholders living in data files.)*

| Target | Kind | Sex | Psyche Window | Unlock Conditions | Resolution | Core Mechanics |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Mortal** | Class | Any | `0` (start) | Starting character. | — | Base stats, standard hitbox. |
| **Astartes** | Class | `Male` | `>= +30` | Injured Apothecary NPC → Gene-Seed Surgery Room. | Trial: Survival Roll (Toughness + Psyche scaling). Fail = death or Servitor. | Larger sprite/hitbox, heavy armor/bolter proficiency, Hypno-Indoctrination (reduced negative Psyche shifts). |
| **Heretic Astartes** | Class | `Male` | `< -50` | Be Astartes and drive Psyche below the threshold. | Automatic on crossing. Locks Allegiance to Chaos. | Keeps Astartes physique, loses Imperial access, gains Chaos Blessings and marks. |
| **Legion of the Damned** | Class | `Male` | High `+` | Astartes + cursed/spectral Relic (or rare Damned NPC). | Death-triggered. | Spectral resurrection; projectiles phase through cover and ignite; unique health/time rules. |
| **Sister of Battle** | Class | `Female` | `>= +50` | Ecclesiarchy encounter or shrine. | Trial: Willpower. | Power armor, Acts of Faith, **Fervor Meter**, scaling from Emperor Blessings. |
| **Living Saint** | Class | `Female` | Very high `+` | Sister of Battle in the Psyche window. | Death-triggered (or Martyrdom Trial). | Golden wings (fly over hazards), burning aura, resurrection miracles. |
| **Grey Knight** | Class | `Male` | `>= +85` | Psyker Calling + exceptional Psyche + rare NPC. | Trial: the 666 Rituals. | Aegis Armor, immune to negative Psyche, anti-Daemon Nemesis weapons. |
| **Adeptus Custodes** | Class | `Male` | Max `+` | Max Psyche + near-perfect Stats + ultra-rare Emissary. | Trial: Alchemical Bio-Sculpting. | Pinnacle size/stats, Guardian Spear, near-impervious to corruption. |
| **Skitarii / Magos** | Class | Any | Near `0` | Mechanicus Explorator + several Augmentics. | Trial: Flesh Ablation Surgery. | Psyche clamped near 0, heavy weapon mounts, turret summons. |
| **Chaos Spawn** | Class | Any | Min `-` | Too many Chaos Blessings with low Willpower, or a failed Daemonhood Trial. | Automatic / failed Trial. | Mutated sprite, natural attacks replace weapon slots, permanent **Control Meter**. |
| **Daemon Prince** | Class | Any | Min `-` | Minimum Psyche + major Chaos Patron questline or Bounty slain for the Gods. | Trial: Apotheosis Ritual. Locks Allegiance to Chaos. | Warp entity, wings and mutations, cataclysmic abilities; hunted by Grey Knights. |
| **Psyker** | Calling | Any | Any | Astral Shrine or Psyker NPC. | Trial: Willpower (fail = Perils of the Warp explosion). | Warp Staffs and powers + **Warp Peril Meter**. |
| **Inquisitor** | Calling | Any | High `+` (or Radical) | Inquisitorial Rosette / NPC chain. | Trial: Interrogation / Authority. | Retinue/summons, Imperial dialogue overrides, bonus Bounty rewards. |
| **Butcher's Nails** | Implant | `Male` | `< -40` | Astartes or Heretic Astartes + Khorne source. | Augmentation. | Adds the Control Meter (Frenzy Hijack). |

### 3.4 Class Meters & Hijack
A player can hold several **Class Meters** at once, one per source (a Librarian with the Butcher's Nails would have both a Warp Peril Meter and a Control Meter). When a meter crosses its threshold it may trigger a **Hijack** — e.g. Frenzy (AI takes over movement, boosted melee) or Warp Overload (explosion, self-damage).

### 3.5 Blessings
* **Active**: the Blessing works.
* **Dormant**: Psyche is on the opposite side of 0 from the Blessing's source; it stops working and reawakens if Psyche returns.
* **Corrupted**: a Chaos Blessing that persists until purged by a Penance Quest.

### 3.6 Living Weapons & Quest Conflicts
* **Living Weapons** (Daemon Weapons, Chapter Relics, Archeotech) attach an Active Quest when equipped. Completing it triggers an **Evolution** to the next stage.
  * *Relic Bolter:* "Clear 3 Rooms taking 0 damage" → Stage 2 (+Psyche). Fails if Psyche drops below a threshold.
  * *Daemon Blade (post-M1, needs melee):* "Kill 25 enemies in melee within 3 Rooms without firing a ranged shot" → Stage 2 (−Psyche).
* **Concurrency:** every combat action broadcasts events (`EnemyKilled`, `WeaponFired`, `DamageTaken`, `PsycheShifted`) that can advance some quests and fail others in the same frame.
* **Quest Failure:** the quest ends; its *source* defines the cost:
  * Living Weapon → spirit goes dormant, re-offers its quest next Floor.
  * Penance → lost for the Run.
  * Bounty → vault seals.
  * Trial quest → that Ascension path closes.

---

## 4. ARCHITECTURAL RULES FOR CODING AGENTS

1. **Strict ECS decoupling:** no OOP inheritance. Every mechanic is data Components evaluated by isolated Systems communicating through Bevy events.
2. **Data-driven content:** items, Blessings, Classes, Callings, quests, Bounties and enemies are defined in `assets/*.ron` and deserialized with `serde`. Never hardcode them in Rust.
3. **No magic numbers (ADR 0003):** every value a balance patch could touch — damage, fire rates, Stat values and scalings, Psyche thresholds and shift amounts, Survival Roll bases and caps, Class Meter rates, quest targets, spawn weights, Room/Floor counts, movement and dodge timings — comes from data. Global values live in `tuning.ron`. A balance patch edits data files and rebuilds; it touches no gameplay code.
4. **WASM first:** no `std::fs` at runtime. `.ron` data is compiled in with `include_str!` (a patch = a new build, which is acceptable). Textures and audio go through `AssetServer`.
5. **Intent-based input:** never wire input directly to movement or projectile spawning. Go through `MoveIntent`, `AimIntent` and `ShotRequest`, so modifiers, Hijacks and curses can intercept them.
6. **Use the glossary's words** (`CONTEXT.md`) in type names, file names and UI text. Banned synonyms in the glossary's `_Avoid_` lines must not appear in code.

---

## 5. WORKSPACE & MODULE STRUCTURE

```text
space-hulk-ascension/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── CONTEXT.md
├── docs/adr/
├── assets/
│   ├── tuning.ron          # Global balance values (Psyche brackets, roll caps, speeds, dodge, floor gen…)
│   ├── stats.ron           # Base Stats per Class
│   ├── blessings.ron
│   ├── weapons.ron
│   ├── enemies.ron
│   ├── quests.ron
│   ├── ascensions.ron      # Classes, Callings, implants and their requirements/rolls
│   └── bounties.ron
└── src/
    ├── main.rs             # App entry, pixel-perfect config (480x270 Nearest), SystemSet ordering
    ├── state.rs            # AppState (Loading, InGame, Dialogue, Trial, GameOver)
    ├── data.rs             # include_str! + ron deserialization of every asset file into Resources
    ├── save.rs             # Suspend Save to/from localStorage
    ├── core/
    │   ├── psyche.rs       # Psyche + PsycheState (brackets from tuning.ron)
    │   ├── allegiance.rs   # Allegiance + lock/unlock rules
    │   ├── stats.rs        # Flat + multiplicative Stat aggregator + Resistances
    │   └── requirements.rs # Generic Requirement evaluator
    ├── player/
    │   ├── controller.rs   # Intents, Dodge Roll, Hijack
    │   ├── paper_doll.rs   # Layered Z-ordered sprites
    │   └── ascension.rs    # Class, Callings, Survival Roll, death-triggered Ascensions
    ├── combat/
    │   ├── collision.rs    # Custom AABB + circle on a spatial hash
    │   ├── pipeline.rs     # ShotRequest → modifier systems → projectile spawner
    │   ├── weapons.rs      # Weapon slots, reload/overheat, Living Weapon Evolution hooks
    │   └── damage.rs       # Damage types, Resistances, FatalDamageEvent interception
    ├── progression/
    │   ├── blessings.rs    # Active / Dormant / Corrupted evaluation
    │   ├── quests.rs       # Concurrent Active Quests, fail triggers, failure outcomes
    │   └── bounties.rs     # Auspicium + reward resolution
    └── world/
        ├── generator.rs    # Grid-based Floor layout of single-screen Rooms
        ├── director.rs     # Allegiance/Psyche-weighted enemy & NPC spawning
        └── encounters.rs   # Encounter Rooms, shrines, Penance
```

---

## 6. SYSTEM EXECUTION ORDER

All gameplay systems MUST respect this `SystemSet` order:

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSystemSet {
    InputAndIntent,      // 1. Keyboard/mouse OR Hijack → MoveIntent / AimIntent / ShotRequest / Dodge
    ShotModification,    // 2. Blessings & weapons mutate ShotRequest
    PhysicsAndCombat,    // 3. Movement, projectile spawn, custom collision
    DamageAndDeathHooks, // 4. Apply damage, FatalDamageEvent → death-triggered Ascension rolls
    EventEvaluation,     // 5. Psyche, Allegiance, Active Quests (progress & failure), Bounties
    WorldReaction,       // 6. Doors, Room clear, Blessing Active/Dormant updates
    RenderSync,          // 7. Paper doll, camera shake, Psyche post-process shaders
}
```

---

## 7. GAMEPLAY MODELS

* **Health:** continuous HP plus per-damage-type percentage Resistances (needed for Bounty build planning). The HUD may present HP in readable chunks.
* **Weapons:** infinite ammo; each weapon has a reload or overheat profile (lasguns overheat, bolters reload).
* **Collision:** custom lightweight AABB + circle collision on a spatial hash. No third-party physics engine.
* **Camera:** fixed per Room. Room transitions slide or cut between screens.

---

## 8. CORE DATA STRUCTURES & CONTRACTS (RUST)

### 8.1 Psyche (`src/core/psyche.rs`)
```rust
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Psyche {
    pub value: i32,             // Clamped between min_limit and max_limit
    pub min_limit: i32,         // Raised toward 0 by Augmentics
    pub max_limit: i32,         // Lowered toward 0 by Augmentics
    pub loss_multiplier: f32,   // Derived from Class/Callings each frame, never set by hand
    pub state: PsycheState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PsycheState {
    Sanctified,
    Epiphany,
    Zealous,
    Lucid,      // Starting state
    Fractured,
    Abyssal,
    Possessed,
}
// Bracket boundaries live in tuning.ron, e.g. psyche_brackets: [(Sanctified, 85), (Epiphany, 50), ...]
```

### 8.2 Allegiance (`src/core/allegiance.rs`)
```rust
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Allegiance {
    pub side: Side,             // Imperium, Unaligned, Chaos
    pub locked_by: Option<String>, // e.g. "HereticAstartes"; None = follows PsycheState
}
```
The mapping PsycheState → Side lives in `tuning.ron`. Only a Penance Quest completion clears `locked_by`.

### 8.3 Requirements & Tags (`src/core/requirements.rs`)
```rust
#[derive(Component, Default, Debug, Clone, Serialize, Deserialize)]
pub struct EntityTags(pub HashSet<String>); // "Male", "Astartes", "Psyker", "Met_Apothecary"...

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Requirement {
    Sex(Sex),
    HasTag(String),
    MissingTag(String),
    PsycheBetween { min: i32, max: i32 },
    AllegianceIs(Side),
    MinStat { stat: StatType, value: f32 },
    HoldingWeapon(String),
    HoldingRelic(String),
    QuestCompleted(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatType { Toughness, Willpower, BallisticSkill, WeaponSkill, Agility }
```

### 8.4 Input & Class Meters (`src/player/controller.rs`)
```rust
#[derive(Component, Default)]
pub struct MoveIntent(pub Vec2);

#[derive(Component, Default)]
pub struct AimIntent { pub direction: Vec2, pub wants_to_shoot: bool }

#[derive(Component, Default)]
pub struct DodgeIntent(pub bool);

#[derive(Component, Default, Debug, Clone)]
pub struct ClassMeters(pub Vec<ClassMeter>); // Several meters can coexist

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMeter {
    pub kind: MeterKind,        // Control, WarpPeril, Fervor
    pub source: String,         // Class, Calling or implant id that granted it
    pub current: f32,
    pub max: f32,
    pub combat_delta_per_sec: f32,
    pub on_threshold: Option<HijackBehavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HijackBehavior {
    Frenzy { duration: f32, speed_mult: f32, dmg_mult: f32 },
    WarpOverload { radius: f32, self_damage: f32 },
}
```

### 8.5 Ascensions & Survival Roll (`src/player/ascension.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AscensionDef {
    pub id: String,
    pub kind: AscensionKind,                 // Class, Calling, Implant
    pub trigger: AscensionTrigger,           // Trial, Automatic, OnDeath
    pub requirements: Vec<Requirement>,
    pub base_chance: f32,
    pub chance_modifiers: Vec<ChanceModifier>, // Stat scalings or run actions that raise the chance
    pub on_fail: TrialFailureOutcome,        // Death, BecomeServitor, BecomeChaosSpawn, Nothing
    pub granted_tags: Vec<String>,
    pub locks_allegiance: Option<Side>,
    pub hitbox_scale: f32,
    pub psyche_loss_multiplier: f32,
}
```
The final chance is clamped to `tuning.ron`'s `survival_roll_cap` (default 0.99). Death-triggered Ascensions use their own lower cap so they are never certain.

### 8.6 Quests (`src/progression/quests.rs`)
```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuest {
    pub id: String,
    pub title: String,
    pub source: QuestSource,            // LivingWeapon, Penance, Bounty, Trial
    pub objective: QuestObjective,
    pub current_progress: u32,
    pub target_progress: u32,
    pub fail_triggers: Vec<QuestFailTrigger>,
    pub on_failure: QuestFailureOutcome, // Dormant/reoffer, LostForRun, SealVault, ClosePath
    pub reward: QuestReward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjective {
    KillWithWeaponTag(String),
    ClearRoomsUntouched,
    SurviveHijacks(u32),
    SlayBounty(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestFailTrigger {
    UsedForbiddenWeaponTag(String),
    FiredRangedWeapon,
    TookDamage,
    PsycheCrossedBelow(i32),
    PsycheCrossedAbove(i32),
    LeftFloorWithoutCompleting,
}
```

### 8.7 Death Interception (`src/combat/damage.rs`)
Never go to Game Over directly on `HP <= 0`.
1. Emit `FatalDamageEvent { entity }`.
2. `death_interception_system` collects every `AscensionDef` with `trigger: OnDeath` whose requirements pass (e.g. Living Saint: `Sex(Female)` + `HasTag("SisterOfBattle")` + `PsycheBetween`), computes its chance and rolls.
3. On success: cancel death and apply the Ascension. Otherwise → `AppState::GameOver`, delete the Suspend Save.

### 8.8 Suspend Save (`src/save.rs`)
* One slot in browser `localStorage`, nothing on any server.
* Written when the player enters a new Floor; contains the whole Run state (Psyche, Allegiance, Class, Callings, Stats, inventory, Active Quests, Bounties, RNG seed, Floor number).
* Loaded on startup if present; deleted on death or Run completion.
* Nothing else is ever persisted: no records, no unlocks, no discovery log.

---

## 9. MILESTONE 1 — PLAYABLE ROOM-LOOP SLICE

Goal: prove the twin-stick feel and the event bus that quests depend on.

* One Floor of ~8–12 single-screen Rooms on a grid, plus a Floor Boss and Exit Elevator (ends the slice).
* Mortal with a laspistol; ranged combat only; Dodge Roll.
* Two enemy types: melee rusher cultist, ranged cultist.
* One Encounter Room with a Psyche choice: help a survivor (+Psyche) or harvest them (−Psyche).
* One Living Weapon: the Relic Bolter and its quest (including a Psyche fail trigger).
* Psyche, PsycheState and Allegiance working end-to-end.
* HUD: HP, Psyche value and State, Active Quest.
* All values in `tuning.ron` / content files from day one.

**Out of scope for M1:** Ascensions, Callings, Class Meters, Blessings, Bounties, melee, Suspend Save, multi-Floor generation.

### 9.1 Starting combat tuning (validated by prototype)
These values were playtested in the combat-feel prototype (branch `prototype/combat-feel`) and felt right. They are the initial contents of `tuning.ron` / `weapons.ron` / `enemies.ron`.

| Area | Values |
| :--- | :--- |
| Player (Mortal) | move speed 110 px/s, movement snappiness 25, max HP 100, 0.6 s invulnerability after a hit, radius 5 |
| Dodge Roll | speed 260 px/s, duration 0.25 s, invincibility 0.30 s, cooldown 0.8 s, cannot shoot while rolling |
| Laspistol (overheat) | damage 10, 0.20 s between shots, bolt speed 360, spread 4°, **heat 15 per shot** (of 100), cooling 40/s, 1.0 s overheat lockout |
| Relic Bolter (reload) | damage 18, 0.25 s between shots, speed 300, spread 6°, magazine 12, reload 1.6 s |
| Cultist rusher | HP 30, speed 70, contact damage 10 |
| Cultist gunner | HP 25, speed 50, keeps 120 px away, fires every 1.2 s after a 0.35 s aim warning, bullet speed 160, damage 6 |
| Room spawns | spawn ≥ 100 px from the player, 0.6 s spawn warning, enemy bullets cleared when the Room is cleared |
| Juice | shake 1 / 3 / 6 (hit / kill / hurt), freeze frame 40 ms on kill and 80 ms when hurt, knockback 60, muzzle flash on |

Room combat was played in endless waves in the prototype; real Rooms use the director's per-Room counts instead.

---

## 10. PARKED OPEN ITEMS (not blocking M1)

1. Psyche shift multipliers per Class/Calling (Astartes reduction, Grey Knight immunity) — exact values and stacking rules.
2. How Augmentics clamp Psyche limits (per-implant amounts, stacking).
3. Bounty Floor placement.
4. Which Run actions raise death-triggered Ascension chances.
5. Melee combat (needed for the Daemon Blade and World Eater).
6. Larger Room shapes (2x1, L, 2x2) — possible later extension of ADR 0001.
7. Movement profile per Class (e.g. Astartes heavier and slower to turn than a Mortal soldier) — movement and Dodge Roll values become per-Class data rather than global.
8. Class-specific evasion: some Classes or Callings could replace the Dodge Roll with another move on the same button (e.g. a Psyker teleport).
