# Space Hulk Ascension

A 2D top-down twin-stick roguelike set inside a Warhammer 40,000 Space Hulk, where a fragile human survivor is reshaped room by room — mind, body and allegiance — through Psyche shifts, Ascensions and quests.

## Run structure

**Run**:
One attempt from a fresh Mortal to death or victory, roughly 20 Floors long. Death ends the Run permanently, and nothing carries over to the next Run.
_Avoid_: Meta-progression, unlocks

**Suspend Save**:
The single snapshot of the current Run, taken when the player enters a new Floor and deleted when the Run ends. Nothing else is ever stored.
_Avoid_: Save slot, checkpoint

**Floor**:
One level of the Space Hulk: a procedurally connected set of Rooms ending in an Exit Elevator guarded by the Floor Boss.
_Avoid_: Level, stage

**Room**:
A single fixed screen of play whose doors seal on entry and unlock when it is cleared.
_Avoid_: Chunk, area

**Encounter Room**:
A non-combat Room holding an NPC or shrine (Apothecary, Confessor, bound Daemon…).
_Avoid_: Safe room, shrine room

## Psyche & allegiance

**Psyche**:
The player's single spiritual/mental scalar, from -100 to +100, starting at 0.
_Avoid_: Alignment, sanity, corruption

**Psyche State**:
A named bracket of Psyche — Sanctified, Epiphany, Zealous, Lucid, Fractured, Abyssal, Possessed — that colours the world's look, dialogue and Blessings.
_Avoid_: Using class names (Living Saint, Daemonhood) for states

**Allegiance**:
Which side the player belongs to — Imperium, Unaligned or Chaos — and therefore which factions are hostile. Follows Psyche State by default but can be locked by events (e.g. becoming a Heretic Astartes) until undone by Penance.
_Avoid_: Loyal/Heretic as states, faction

**Blessing**:
A passive mutation or boon granted by the Emperor (raising Psyche) or the Chaos Gods (lowering it).
_Avoid_: Buff, perk

**Dormant Blessing**:
A Blessing that stops working because Psyche is on the opposite side of 0 from its source; it reawakens if Psyche returns.

**Corrupted Blessing**:
A Chaos Blessing that persists until purged by a Penance Quest.

**Augmentic**:
A Mechanicus cybernetic implant that pulls Psyche's limits toward 0.
_Avoid_: Implant, cyberware

## Character

**Sex**:
The player's biological sex (Male or Female), chosen once before the Run and used as a prerequisite for lore-bound Ascensions.
_Avoid_: Gender

**Stat**:
One of the player's primary attributes: Toughness, Willpower, Ballistic Skill, Weapon Skill, Agility.
_Avoid_: Endurance (use Toughness), Luck (not a stat)

**Class**:
The player's single body/identity: Mortal, Astartes, Heretic Astartes, Sister of Battle, Living Saint, Legion of the Damned, Grey Knight, Custodes, Skitarii, Chaos Spawn, Daemon Prince. Exactly one at a time.

**Calling**:
A stackable gift or role layered on top of a Class, gained through a Trial: Psyker, Inquisitor.
_Avoid_: Subclass

**Librarian**:
An Astartes who also holds the Psyker Calling.

**World Eater**:
An Astartes or Heretic Astartes fitted with the Butcher's Nails — a combination, not a Class.

**Mortal**:
The base, un-Ascended human every Run starts as.

**Ascension**:
A permanent (for the Run) change of the player's Class or gain of a Calling (e.g. Mortal → Astartes). Ascensions are secret: the game never shows which exist or which the player has found.
_Avoid_: Transcendence, class evolution, secret evolution

**Trial**:
A risky event that may grant an Ascension, resolved by a Survival Roll. Not every Ascension has one (some are automatic on crossing a threshold).
_Avoid_: Class trial, transcendence trial

**Survival Roll**:
The chance-based check that decides whether an Ascension attempt succeeds, scaled by stats and capped at 99%. Applies both to Trials and to death-triggered Ascensions.

**Death-triggered Ascension**:
An Ascension attempted at the moment the player would die (e.g. Living Saint, Legion of the Damned); success cancels the death. Its chance is never certain, but actions taken during the Run can raise it.

**Class Meter**:
A class-specific tension gauge filled during combat. Kinds include the Control Meter (World Eater, Chaos Spawn), the Warp Peril Meter (Psyker) and the Fervor Meter (Sister of Battle).
_Avoid_: Secondary meter

**Hijack**:
A Class Meter crossing its threshold and temporarily taking control away from the player (e.g. Frenzy).

## Equipment & quests

**Living Weapon**:
A weapon with a dormant spirit that issues its own quest and changes when that quest is completed.
_Avoid_: Evolving weapon

**Evolution**:
A Living Weapon advancing to its next stage. Reserved for weapons only.

**Active Quest**:
Any objective the player currently holds; several can be held at once and may conflict.

**Quest Failure**:
The end of an Active Quest when one of its fail triggers fires. What it costs is defined by the quest's source (a Living Weapon goes dormant and re-offers, a Penance is lost for the Run, a Bounty vault seals, a Trial path closes).

**Penance Quest**:
A quest offered by an Imperial Priest that purges Chaos Blessings and can unlock a Chaos-locked Allegiance.

**Bounty**:
A rare optional boss announced early in the Run on a specific deep Floor, letting the player plan a build around it. Listed on the Auspicium.
_Avoid_: HVT (acceptable in casual talk), optional boss
