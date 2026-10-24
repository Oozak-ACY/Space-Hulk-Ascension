# ARCHITECTURE & GAME DESIGN SPECIFICATION (v2.0)
**Project:** WH40K Space Hulk Roguelike (Working Title: *Hulk Psyche*)  
**Target Stack:** Rust + Bevy Engine (0.15+) + WebAssembly (`wasm32-unknown-unknown` via `Trunk`)  
**Visual Style:** 2D Top-Down Pixel Art (480x270 virtual resolution, Nearest-Neighbor filtering)  
**Core Gameplay:** 360° Twin-Stick Shooter (*Alien Swarm* feel) + *Binding of Isaac* Synergy/Room Loop + Dynamic Psyche, Concurrent Quests & Risk-Based Class Ascensions.

---

## 1. HIGH-LEVEL PITCH & GAME VISION (THE "SPEECH")

### 1.1 The Elevator Pitch
You wake up as a nameless, fragile human survivor—mere cannon fodder—trapped inside a colossal, drifting **Space Hulk** in the Warhammer 40,000 universe. A Space Hulk is a tangled amalgamation of ancient Imperial warships, Chaos-corrupted hulks, and Xenos vessels fused together by the Warp. 

Every run starts at the bottom of the food chain with a basic laspistol and a mortal body. As you delve deeper floor by floor, every choice you make—who you help, what relics you equip, what dark whispers you answer, and which surgical or spiritual trials you endure—permanently reshapes your **Psyche**, your body, and the Space Hulk itself. 

You can remain a pure servant of the Emperor, ascending into a Space Marine, a Sister of Battle, an Inquisitor, a Grey Knight, or even a Living Saint. You can strip away your humanity with Mechanicus cybernetics. Or you can embrace the ruinous powers of Chaos for overwhelming destructive power, mutating into a Heretic Astartes, a Daemon Prince, or a rampaging Chaos Spawn. **As you change, the game inverts:** fall to heresy, and the Space Hulk's cultists become your allies while the Imperium dispatches Kill Teams, Sisters of Battle, and Grey Knights to purge you.

### 1.2 Core Design Pillars
1. **Zero to Demigod (or Abomination):** Classes are not chosen in a main menu; they are earned mid-run through dangerous, potentially fatal in-game trials gated by your gender, stats, encounters, and Psyche.
2. **The Psyche Axis (`-100` to `+100`):** A single scalar mental/spiritual gauge replacing traditional alignment. High Psyche leads to Imperial zealotry and miracles; Low Psyche leads to Warp madness and daemonic mutations. Both sides grant "Blessings," locking out the opposite spectrum's equipment and paths.
3. **Concurrent & Conflicting Quests:** The core room-to-room tension comes from juggling multiple active quests simultaneously (from living weapons, NPCs, penance rituals, and bounty boards). Completing one objective in a room may instantly fail another, forcing constant tactical and moral trade-offs.
4. **Long-Term Build Planning via Bounties:** Rare, optional High-Value Targets (HVTs)—such as a named Daemon Prince of Tzeentch on Floor 20—are announced early in the run, allowing the player to deliberately specialize their resistances, weapons, and class ascensions over 15+ floors to prepare for the fight (or skip it to survive).

---

## 2. THE THREE GAMEPLAY LOOPS

### 2.1 Micro-Loop: Room-to-Room Tactical Combat (30–60 seconds)
* **Controls:** 360° Twin-Stick shooter (`WASD/ZQSD` movement + 360° mouse aiming + Dodge/Active skill).
* **Room Entry:** Upon entering an unexplored procedural room, bulkhead doors seal. The **Psyche Director** reads the player's current `Psyche` and spawns appropriate enemies (Chaos/Xenos if Loyal/Lucid; Imperial Purge Squads/Inquisition if Heretic).
* **Tactical Execution:** While fighting, the player must manage their **Secondary Class Meter** (e.g., Frenzy/Control gauge for World Eaters or Chaos Spawns, Warp Peril gauge for Psykers) and respect their **Active Quests** (e.g., killing enemies with a specific Daemon Blade to awaken it, OR avoiding Chaos weapons entirely to pass a Priest's purification ritual).
* **Resolution:** Clearing the room unlocks doors, ticks cooldowns, evaluates quest progress/failures, and spawns room rewards.

### 2.2 Meso-Loop: Floor Exploration, Encounters & Quests (10–15 minutes)
* **Procedural Topology:** Each Space Hulk floor is generated organically (no fixed grid/cross). It contains combat corridors, environmental hazards, **1 to 3 Non-Combat Encounter Rooms**, and an Exit Elevator guarded by the Floor Boss.
* **Non-Combat NPC & Shrine Rooms:** Safe zones where the player interacts with survivors, Apothecaries, Confessors, Inquisitors, or bound Daemons.
  * NPCs change depending on `Psyche` (saving Imperial survivors when positive vs. striking dark pacts or harvesting organs when negative).
  * **The Redemption Path:** Even at deep negative Psyche (`Heretic`), there is a small (~5%) chance to encounter a captive Imperial Priest offering a grueling **Penance Quest** to purge Chaos Blessings and claw back toward the Light.
* **Post-Boss Alignment Pact:** Defeating a floor boss can open a special Sanctuary (Imperial Chapel vs. Warp Breach) depending on floor performance and Psyche.

### 2.3 Macro-Loop: Run-Wide Transcendence & Bounty Hunting (45–90 minutes)
* **Pre-Run Setup:** The player starts as a `Mortal` and chooses their starting biological sex (`Tag::Male` or `Tag::Female`), which acts as a permanent biological prerequisite for lore-accurate ascensions later in the run.
* **The Auspicium (Bounty Board):** Early in the run, 1 to 3 rare Optional Bosses (HVTs) are revealed at specific deep floors (e.g., Floor 20), showing their damage profile (e.g., Heavy Warp/Psychic damage) and dual rewards.
* **Class Transcendence Trials:** By meeting specific NPCs and fulfilling requirements, the player unlocks surgical shrines or spiritual trials. Entering a trial rolls a **Survival Check** (capped at 99% via stats). Failure can result in instant Permadeath or botched transformation; success permanently transforms the player's class, sprite size, passives, and mechanics.
* **Optional Boss Resolution:** Reaching an HVT's floor presents a sealed vault. If the player chooses to fight and wins, they face a major dilemma: **Claim the Boss's Corrupted Artifact/Blessing** (massive power, heavy `-Psyche` shift) OR **Banish & Claim the Imperial Bounty** (`+Psyche` shift and master-crafted Imperial gear).

---

## 3. COMPREHENSIVE CATALOG OF CLASSES, CONDITIONS & MECHANICS
*(Reference table for AI Grilling & Architecture Validation — All systems must support these exact cases without hardcoding)*

### 3.1 Class Ascensions & Secret Evolutions

| Target Class / State | Required Sex | Psyche Window | Unlock & Trial Conditions | Survival Roll / Trigger | Core Class Mechanics & Passives |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Mortal (Base)** | Any | `0` (Start) | Starting character. | 100% | Base human stats, standard hitbox, 1.0x Psyche shift multiplier. |
| **Astartes (Space Marine)** | `Male` | `>= +30` | Meet injured Apothecary NPC -> Unlock Gene-Seed Surgery room. | Base % + `Endurance` + `Psyche` (Max **99%**). *Fail = Death (or Servitor).* | Larger sprite/hitbox, heavy armor/bolter proficiency, **Hypno-Indoctrination** (-70% to all negative Psyche shifts). |
| **Heretic Astartes (Chaos Marine)** | `Male` | `< -50` | Must already be `Astartes` and force Psyche below `-50` via dark pacts/weapons. | Automatic transition upon crossing threshold. | Retains Astartes physique, loses Imperial access, unlocks Chaos Marine blessings & marks. |
| **World Eater / Berserker** | `Male` | `< -40` | `Astartes` / `Heretic Astartes` + Butcher's Nails implant / Khorne blessings. | Trial / Augmentation. | Adds **Control Meter**. When depleted in combat, triggers **Frenzy Hijack** (AI takes over movement, +200% melee damage/speed). |
| **Legion of the Damned** | `Male` | High `+` | Be `Astartes` + Hold specific cursed/spectral Relic (or spoke to rare Damned NPC) + **Die in combat**. | **Death Hook Interception** (`FatalDamageEvent` canceled). | Resurrects in spectral fire. Projectiles phase through cover/ignite enemies; unique health/time rules. |
| **Sister of Battle** | `Female` | `>= +50` | High faith + Ecclessiarchy encounter/shrine. | Faith/Willpower trial. | Power armor proficiency, Act of Faith active skills, bonus scaling from Emperor Blessings. |
| **Living Saint** | `Female` | `+100` (Max) | Be `Sister of Battle` (or max faith female) at `+100` Psyche + **Die in combat** (or Martyrdom Trial). | **Death Hook Interception** / Ascension. | Golden wings (flight over floor hazards), radiant burning aura, divine resurrection miracles. |
| **Inquisitor** | Any | High `+` (or Radical) | High Psyche + Specific Inquisitorial Rosette / NPC encounter chain. | Interrogation / Authority Trial. | Unlocks retinue/summons, special dialogue overrides with Imperial NPCs, bonus Bounty rewards. |
| **Sanctioned Psyker** | Any | Any | Awaken latent psychic gene via Astral Shrine or Psyker NPC. | Willpower roll (Fail = Perils of the Warp explosion). | Unlocks Warp Staffs/Psychic powers + **Warp Peril Meter** (overheating causes self-damage or daemon spawns). |
| **Grey Knight** | `Male` | `>= +85` | Extreme Psyche + Exceptional Psychic power (`Sanctioned Psyker`) + Rare Titan NPC encounter. | The 666 Rituals (High stat requirements, up to 99%). | Aegis Armor, **Immune to negative Psyche corruption**, devastating anti-Daemon Nemesis weapons. |
| **Adeptus Custodes** | `Male` | `+100` (Max) | Max Psyche + **God-tier stats across the board** + Ultra-rare Emissary NPC. | Alchemical Bio-Sculpting (Requires near-perfect stats to reach 99%). | Pinnacle physical size/stats, Guardian Spear mastery, near-impervious to mental corruption. |
| **Skitarii / Magos** | Any | Near `0` | Mechanicus Explorator NPC + Multiple `Augmentic` cybernetics equipped. | Flesh Ablation Surgery. | Clamps `Psyche` tightly around `0` (cannot become Saint or Daemon), heavy weapon mounts, turret summons. |
| **Chaos Spawn** | Any | `-100` (Min) | Accumulate too many Chaos Blessings with insufficient Willpower, or fail a Daemonhood trial. | Automatic / Failed Dark Trial. | Monstrous mutated sprite, loses normal weapon slots for grotesque natural attacks, permanent **Control Meter** (frequent Frenzy AI Hijacks in combat). |
| **Daemon Prince** | Any | `-100` (Min) | Reach `-100` Psyche + Complete a Major Chaos Patron Questline / Slay an HVT for the Gods. | Apotheosis Ritual. | Immortal Warp entity, wings/mutations, cataclysmic abilities; hunted relentlessly by Grey Knights. |

### 3.2 Living Weapons & Concurrent Quest Conflicts
* **Weapon Evolution:** Certain weapons (Daemon Weapons, Chapter Relics, Archeotech) contain a dormant spirit and attach an `ActiveQuest` when equipped.
  * *Example (Daemon Blade):* "Kill 25 enemies in melee within 3 rooms without firing a ranged shot." -> Evolves to Stage 2 (`-15 Psyche`, unlocks blood waves) and issues a Stage 3 quest.
  * *Example (Relic Bolter):* "Clear 3 rooms taking 0 damage." -> Evolves to Stage 2 (`+10 Psyche`). Fails instantly if `Psyche` drops below `-20`.
* **Quest Conflict Matrix:** The player can hold multiple quests at once (Weapon Evolution + Priest Penance + Apothecary Stat Goal + Floor Bounty). Every combat action broadcasts events (`EnemyKilled`, `WeaponFired`, `DamageTaken`, `PsycheShifted`) that simultaneously advance some quests and trigger `QuestFailTrigger` on conflicting quests.

---

## 4. ARCHITECTURAL VISION & AI AGENT RULES

### 4.1 Golden Rules for Coding Agents
1. **Strict ECS Decoupling:** Never use OOP inheritance. Every mechanic (weapons, blessings, quests, class ascensions, frenzy meters) is represented by pure Data Components and evaluated by isolated Systems via Bevy `EventReader` / `EventWriter`.
2. **Data-Driven Definitions (`.ron`):** Never hardcode items, blessings, classes, or quests in Rust code. Define them in `assets/*.ron` (Rusty Object Notation) deserialized via `serde`.
3. **WASM Compatibility First:** Do not use `std::fs` at runtime. Use `include_str!` for `.ron` data files during prototyping and Bevy's `AssetServer` for textures/audio.
4. **Intent-Based Input & Combat:** Never tie keyboard/mouse inputs directly to transform movement or projectile spawning. Always go through `MoveIntent`, `AimIntent`, and `ShotRequest` components so systems (modifiers, frenzy AI hijack, curses) can intercept and mutate them.

---

## 5. CORE GLOSSARY & DOMAIN MODEL

| Domain Term | Rust Struct / Enum | Description |
| :--- | :--- | :--- |
| **Psyche** | `Resource: Psyche` | Single scalar axis from `-100` (Daemonhood) to `+100` (Living Saint). Starts at `0` (Lucid Mortal). |
| **Psyche State** | `Enum: PsycheState` | Threshold brackets that alter world spawns, UI shaders, NPC dialogues, and active blessings. |
| **Blessing** | `Component: Blessing` | Unified passive mutation/buff granted by the Emperor (`+Psyche`) or Chaos Gods (`-Psyche`). |
| **Augmentic** | `Component: Augmentic` | Mechanicus cybernetics. Clamps `Psyche` min/max limits toward `0` (emotional lobotomy/immunity). |
| **Transcendence** | `Component: CharacterClass` | Risk-based mid-run class evolutions (Astartes, Sister of Battle, Psyker, Custodes, Chaos Spawn...). |
| **Control Meter** | `Component: SecondaryMeter` | Class-specific tension bar (Frenzy, Warp Peril, Act of Faith) that can trigger `HijackBehavior`. |
| **Living Weapon** | `Component: EvolvingWeapon` | Weapons with embedded `ActiveQuest` conditions that mutate into higher tiers upon completion. |
| **Bounty (HVT)** | `Struct: BountyTarget` | Rare, optional high-difficulty bosses announced early in the run to guide player build planning. |

---

## 6. WORKSPACE & MODULE STRUCTURE

```text
wh40k-roguelike/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── assets/
│   ├── blessings.ron
│   ├── weapons.ron
│   ├── quests.ron
│   ├── classes.ron
│   └── bounties.ron
└── src/
    ├── main.rs                 # App entry, Pixel-Perfect config (480x270 Nearest), SystemSet ordering
    ├── state.rs                # AppState (Loading, InGame, Dialogue, TranscendenceTrial, GameOver)
    ├── core/
    │   ├── psyche.rs           # Psyche resource (-100..100) + threshold events
    │   ├── stats.rs            # Flat + Multiplicative stat aggregator + Resistances
    │   └── requirements.rs     # Generic Requirement evaluator (Tags, Stats, Psyche, Gender)
    ├── player/
    │   ├── controller.rs       # MoveIntent, AimIntent + Frenzy AI Hijack (SecondaryMeter)
    │   ├── paper_doll.rs       # Layered Z-ordered 2D sprites (Legs, Torso, Head, Blessings, Weapon)
    │   └── classes.rs          # CharacterClass, survival roll calculator, Death Hooks
    ├── combat/
    │   ├── pipeline.rs         # ShotRequest -> Modifier Systems -> Projectile Spawner
    │   ├── weapons.rs          # Equipped weapon slots + Living Weapon evolution hooks
    │   └── damage.rs           # Damage types, Resistance mitigation, FatalDamageEvent interception
    ├── progression/
    │   ├── blessings.rs        # Active/Dormant/Corrupted blessing evaluation
    │   ├── quests.rs           # Concurrent ActiveQuest tracker + FailTrigger evaluator
    │   └── bounties.rs         # Auspicium HVT generator & reward resolution (Artifact vs Bounty)
    └── world/
        ├── generator.rs        # Procedural Space Hulk graph + chunk/WFC room layouts
        ├── director.rs         # Psyche-weighted enemy & NPC spawning per room
        └── interactions.rs     # Non-combat NPC rooms, shrines, penance/redemption trials
```

---

## 7. SYSTEM EXECUTION ORDER (FRAME PIPELINE)

To prevent race conditions in Bevy, all gameplay systems MUST adhere to this strict `SystemSet` order:

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSystemSet {
    InputAndIntent,      // 1. Read keyboard/mouse OR Frenzy AI Hijack -> write MoveIntent / AimIntent / ShotRequest
    ShotModification,    // 2. Active Blessings & Weapons mutate ShotRequest (split, homing, flags)
    PhysicsAndCombat,    // 3. Move entities, spawn projectiles from ShotRequest, resolve collisions
    DamageAndDeathHooks, // 4. Apply damage, check FatalDamageEvent (Legion of the Damned / Living Saint hooks)
    EventEvaluation,     // 5. Update Psyche, evaluate ActiveQuests (progress & fail triggers), check Bounties
    WorldReaction,       // 6. Lock/unlock doors, trigger room clears, update active/dormant Blessings
    RenderSync,          // 7. Update Paper Doll sprite layers, camera shake, Psyche post-process shaders
}
```

---

## 8. CORE DATA STRUCTURES & CONTRACTS (RUST)

### 8.1 The Psyche Resource (`src/core/psyche.rs`)
```rust
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Psyche {
    pub value: i32,              // Clamped between min_limit and max_limit (default -100..=100)
    pub min_limit: i32,          // Raised toward 0 by Mechanicus Augmentics
    pub max_limit: i32,          // Lowered toward 0 by Mechanicus Augmentics
    pub loss_multiplier: f32,    // e.g., 0.3 for Astartes (-70% loss), 0.0 for Grey Knight
    pub state: PsycheState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PsycheState {
    LivingSaint,    // +85 to +100
    Epiphany,       // +50 to +84
    Zealous,        // +20 to +49
    Lucid,          // -19 to +19 (Starting Human State)
    Fractured,      // -49 to -20
    AbyssalMadness, // -84 to -50
    Daemonhood,     // -100 to -85
}
```

### 8.2 Generic Requirements & Tags (`src/core/requirements.rs`)
Used to gate NPC options, Class Ascensions, Items, and Secret Evolutions without hardcoding.
```rust
#[derive(Component, Default, Debug, Clone, Serialize, Deserialize)]
pub struct EntityTags(pub HashSet<String>); // e.g., "Male", "Female", "Astartes", "Psyker", "Met_Apothecary"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Requirement {
    HasTag(String),
    MissingTag(String),
    PsycheBetween { min: i32, max: i32 },
    MinStat { stat: StatType, value: f32 },
    HoldingWeapon(String),
    HoldingRelic(String),
    QuestCompleted(String),
}
```

### 8.3 Input Decoupling & Frenzy Hijack (`src/player/controller.rs`)
Allows classes like *Chaos Spawn*, *World Eater*, or *Overloaded Psyker* to temporarily lose player control.
```rust
#[derive(Component, Default)]
pub struct MoveIntent(pub Vec2);

#[derive(Component, Default)]
pub struct AimIntent {
    pub direction: Vec2,
    pub wants_to_shoot: bool,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryMeter {
    pub meter_type: MeterType, // ControlFrenzy, WarpPeril, FaithFervor
    pub current: f32,
    pub max: f32,
    pub combat_delta_per_sec: f32,
    pub on_threshold_hijack: Option<HijackBehavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HijackBehavior {
    BerserkMeleeCharge { duration: f32, speed_mult: f32, dmg_mult: f32 },
    WarpOverloadExplosion { radius: f32, self_damage: f32 },
}
```

### 8.4 Class Transcendence & Survival Roll (`src/player/classes.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassTranscendenceDef {
    pub target_class_id: String,
    pub unlock_requirements: Vec<Requirement>,
    pub base_survival_chance: f32, // e.g., 0.25 (25%)
    pub stat_scalings: Vec<(StatType, f32)>, // Increases chance up to 0.99 (99% hard cap)
    pub on_fail_outcome: TrialFailureOutcome, // Permadeath, BecomeServitor, BecomeChaosSpawn
    pub granted_tags: Vec<String>,
    pub hitbox_scale: f32,
    pub psyche_loss_multiplier: f32,
}
```

### 8.5 Concurrent Quest & Weapon Evolution Engine (`src/progression/quests.rs`)
```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuest {
    pub id: String,
    pub title: String,
    pub source: QuestSource, // WeaponEvolution, NpcPenance, BountyBoard, ClassTrial
    pub objective: QuestObjective,
    pub current_progress: u32,
    pub target_progress: u32,
    pub fail_triggers: Vec<QuestFailTrigger>,
    pub reward: QuestReward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjective {
    KillWithWeaponTag(String),
    ClearRoomsUntouched,
    SurviveFrenzyEpisodes(u32),
    SlayBountyTarget(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestFailTrigger {
    UsedForbiddenWeaponTag(String), // e.g., Using a Chaos weapon fails a Priest's Penance quest
    FiredRangedWeapon,              // e.g., Fails a Khorne melee-only awakening quest
    TookDamage,
    PsycheCrossedBelow(i32),
    PsycheCrossedAbove(i32),
    LeftFloorWithoutCompleting,
}
```

### 8.6 Death Interception Hooks (`src/combat/damage.rs`)
Never trigger Game Over directly on `HP <= 0`.
1. Emit `FatalDamageEvent { entity: Player }`.
2. Run `death_interception_system`: checks if any equipped Relic, Blessing, or Class condition intercepts death:
   * `HasTag("Astartes") + HoldingRelic("Cursed_Censer")` -> Cancels death, mutates class to `LegionOfTheDamned`.
   * `HasTag("Female") + PsycheBetween(95, 100) + HasTag("SisterOfBattle")` -> Cancels death, ascends to `LivingSaint`.
3. If unintercepted -> Transition to `AppState::GameOver`.

---

## 9. OPEN QUESTIONS FOR PRE-DEVELOPMENT GRILLING
*(To be resolved with the Architect before writing Milestone 1 code)*

1. **Camera & Room Scale:** Fixed single-screen rooms (480x270 *Isaac* style) OR larger scrolling Space Hulk rooms/corridors with camera follow and line-of-sight fog of war (*Alien Swarm* style)?
2. **Weapon Economy:** Infinite ammo with reload/overheat mechanics (*Enter the Gungeon* style) OR finite ammo drops per weapon type (*Nuclear Throne* style)?
3. **Health & Mitigation Model:** Continuous numerical HP/Armor/Resistances (`f32` values for `%` elemental/Warp mitigation against HVTs) OR discrete heart/shield containers (*Isaac* style)?
4. **Core Primary Stats:** What is the exact list of 4–5 primary attributes used for Class Survival Rolls and equipment scaling (e.g., `Endurance`, `Willpower`, `BallisticSkill`, `MeleeSkill`, `Toughness`)?
5. **Save State & Meta-Progression:** Should runs auto-save to browser `localStorage` per floor? Does dying/completing achievements unlock new classes/weapons/HVTs into the global pool for future runs (Roguelite unlocks)?
6. **Collision & Physics Strategy:** Custom lightweight 2D Grid/Spatial-Hash AABB + Circle collision (recommended for WASM determinism and agent simplicity) OR third-party physics engine (`bevy_rapier2d`)?
