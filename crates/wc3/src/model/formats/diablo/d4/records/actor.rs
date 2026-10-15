//! Native actor records.
use super::*;
/// Native `ActorFoleySoundOverride` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorFoleySoundOverride {
    /// Native `tFoleyMaterialTag` at byte 0.
    pub foley_material_tag: FoleyMaterialTag,
    /// Native `unk_f77a839` at byte 4.
    #[d4(value)]
    pub unk_f77a839: [i32; 4],
}

/// Native `ActorHitCollisionShape` (36 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorHitCollisionShape {
    /// Native `unk_20b45d7` at byte 0.
    #[d4(value)]
    pub unk_20b45d7: u32,
    /// Native `eType` at byte 4.
    #[d4(value)]
    pub type_: i32,
    /// Native `flRadius` at byte 8.
    #[d4(value)]
    pub radius: f32,
    /// Native `flLength` at byte 12.
    #[d4(value)]
    pub length: f32,
    /// Native `aPitch` at byte 16.
    #[d4(value)]
    pub a_pitch: f32,
    /// Native `aYaw` at byte 20.
    #[d4(value)]
    pub a_yaw: f32,
    /// Native `vBoneLocalOffset` at byte 24.
    #[d4(value)]
    pub bone_local_offset: [f32; 3],
}

/// Native `ActorProjectileData` (156 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorProjectileData {
    /// Native `fDestroyOnImpact` at byte 0.
    #[d4(value)]
    pub destroy_on_impact: i32,
    /// Native `bDeleteOnOwnerDeath` at byte 4.
    #[d4(value)]
    pub delete_on_owner_death: i32,
    /// Native `unk_dca784d` at byte 8.
    #[d4(value)]
    pub unk_dca784d: i32,
    /// Native `flMinimumDestroyOnImpactVelocity` at byte 12.
    #[d4(value)]
    pub minimum_destroy_on_impact_velocity: f32,
    /// Native `fProjectileMatchYawToVelocity` at byte 16.
    #[d4(value)]
    pub projectile_match_yaw_to_velocity: i32,
    /// Native `fProjectileMatchPitchToVelocity` at byte 20.
    #[d4(value)]
    pub projectile_match_pitch_to_velocity: i32,
    /// Native `fProjectileTurnsInstantly` at byte 24.
    #[d4(value)]
    pub projectile_turns_instantly: i32,
    /// Native `fProjectileCanBeReflected` at byte 28.
    #[d4(value)]
    pub projectile_can_be_reflected: i32,
    /// Native `fProjectileCanBeSlowed` at byte 32.
    #[d4(value)]
    pub projectile_can_be_slowed: i32,
    /// Native `fProjectileUsesWeaponAppearance` at byte 36.
    #[d4(value)]
    pub projectile_uses_weapon_appearance: i32,
    /// Native `unk_7c4ac65` at byte 40.
    #[d4(value)]
    pub unk_7c4ac65: i32,
    /// Native `tHardpointLink` at byte 44.
    pub hardpoint_link: HardpointLink,
    /// Native `tAttachHardpointLink` at byte 52.
    pub attach_hardpoint_link: HardpointLink,
    /// Native `flGravityMultiplier` at byte 60.
    #[d4(value)]
    pub gravity_multiplier: f32,
    /// Native `fHugsGround` at byte 64.
    #[d4(value)]
    pub hugs_ground: i32,
    /// Native `fIgnoreSceneCollision` at byte 68.
    #[d4(value)]
    pub ignore_scene_collision: i32,
    /// Native `bIgnoreActorCollision` at byte 72.
    #[d4(value)]
    pub ignore_actor_collision: i32,
    /// Native `unk_b79ecf8` at byte 76.
    #[d4(value)]
    pub unk_b79ecf8: i32,
    /// Native `unk_4673605` at byte 80.
    #[d4(value)]
    pub unk_4673605: f32,
    /// Native `unk_cdf7878` at byte 84.
    #[d4(value)]
    pub unk_cdf7878: f32,
    /// Native `unk_3168e93` at byte 88.
    #[d4(value)]
    pub unk_3168e93: i32,
    /// Native `unk_1458258` at byte 92.
    #[d4(value)]
    pub unk_1458258: i32,
    /// Native `eSeekingBehavior` at byte 96.
    #[d4(value)]
    pub seeking_behavior: i32,
    /// Native `unk_30866cd` at byte 100.
    #[d4(value)]
    pub unk_30866cd: i32,
    /// Native `dwSeekingUpdateRate` at byte 104.
    #[d4(value)]
    pub seeking_update_rate: u32,
    /// Native `unk_c220883` at byte 108.
    #[d4(value)]
    pub unk_c220883: f32,
    /// Native `bReSeek` at byte 112.
    #[d4(value)]
    pub re_seek: i32,
    /// Native `unk_721bb83` at byte 116.
    #[d4(value)]
    pub unk_721bb83: f32,
    /// Native `flDefaultSeekRange` at byte 120.
    #[d4(value)]
    pub default_seek_range: f32,
    /// Native `flFallbackSeekRange` at byte 124.
    #[d4(value)]
    pub fallback_seek_range: f32,
    /// Native `snoOnBounceEffectGroup` at byte 128.
    #[d4(value)]
    pub on_bounce_effect_group: i32,
    /// Native `bBouncesOffCollision` at byte 132.
    #[d4(value)]
    pub bounces_off_collision: i32,
    /// Native `bBouncesOffNavMesh` at byte 136.
    #[d4(value)]
    pub bounces_off_nav_mesh: i32,
    /// Native `fProjectileOverridesCollidableFlags` at byte 140.
    #[d4(value)]
    pub projectile_overrides_collidable_flags: i32,
    /// Native `unk_4400585` at byte 144.
    #[d4(value)]
    pub unk_4400585: i32,
    /// Native `dwCollidableFlags` at byte 148.
    #[d4(value)]
    pub collidable_flags: u32,
    /// Native `unk_6250c06` at byte 152.
    #[d4(value)]
    pub unk_6250c06: i32,
}

/// Native `ActorMovingData` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorMovingData {
    /// Native `fGenerateFootstepParticles` at byte 0.
    #[d4(value)]
    pub generate_footstep_particles: i32,
    /// Native `unk_56d76cc` at byte 4.
    #[d4(value)]
    pub unk_56d76cc: i32,
    /// Native `fFastMovementUpdate` at byte 8.
    #[d4(value)]
    pub fast_movement_update: i32,
}

/// Native `ActorMonsterData` (648 fixed bytes).
#[derive(Clone, Debug, PartialEq, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorMonsterData<'data> {
    /// Native `eGender` at byte 0.
    #[d4(value)]
    pub gender: i32,
    /// Native `fIsNPC` at byte 4.
    #[d4(value)]
    pub is_npc: i32,
    /// Native `unk_7c902e4` at byte 8.
    #[d4(value)]
    pub unk_7c902e4: i32,
    /// Native `fIsMetaQuestTurnInNPC` at byte 12.
    #[d4(value)]
    pub is_meta_quest_turn_in_npc: i32,
    /// Native `fIsAltMetaQuestTurnInNPC` at byte 16.
    #[d4(value)]
    pub is_alt_meta_quest_turn_in_npc: i32,
    /// Native `snoNPCOperateSound` at byte 20.
    #[d4(value)]
    pub npc_operate_sound: i32,
    /// Native `snoNPCPowerOperate` at byte 24.
    #[d4(value)]
    pub npc_power_operate: i32,
    /// Native `flNPCInteractDistance` at byte 28.
    #[d4(value)]
    pub npc_interact_distance: f32,
    /// Native `flNPCInteractCancelDistance` at byte 32.
    #[d4(value)]
    pub npc_interact_cancel_distance: f32,
    /// Native `tNPCServiceData` at byte 36.
    pub npc_service_data: NPCServiceData,
    /// Native `fConvertsDamageToDamageStates` at byte 88.
    #[d4(value)]
    pub converts_damage_to_damage_states: i32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `unk_6bce993` at byte 96.
    #[d4(array)]
    pub unk_6bce993: Vec<MonsterDamageStateThresholds>,
    /// Native `unk_6b4a81d` at byte 112.
    #[d4(value)]
    pub unk_6b4a81d: f32,
    /// Native `snoMonsterFamily` at byte 116.
    #[d4(value)]
    pub monster_family: i32,
    /// Native `szArchetype` at byte 120.
    #[d4(value)]
    pub archetype: u32,
    /// Native `eBaseElement` at byte 124.
    #[d4(value)]
    pub base_element: i32,
    /// Native `snoMainHand` at byte 128.
    #[d4(value)]
    pub main_hand: i32,
    /// Native `snoOffHand` at byte 132.
    #[d4(value)]
    pub off_hand: i32,
    /// Native `unk_f8ee66e` at byte 136.
    #[d4(value)]
    pub unk_f8ee66e: i32,
    /// Native `nGoldGranted` at byte 140.
    #[d4(value)]
    pub gold_granted: i32,
    /// Native `unk_64df7b` at byte 144.
    #[d4(value)]
    pub unk_64df7b: i32,
    /// Native `bForceLootDropOnDeath` at byte 148.
    #[d4(value)]
    pub force_loot_drop_on_death: i32,
    /// Native `snoTreasureClass` at byte 152.
    #[d4(value)]
    pub treasure_class: i32,
    /// Native `snoTreasureClassElite` at byte 156.
    #[d4(value)]
    pub treasure_class_elite: i32,
    /// Native `snoTreasureClassSuperElite` at byte 160.
    #[d4(value)]
    pub treasure_class_super_elite: i32,
    /// Native `unk_c3861c3` at byte 164.
    #[d4(value)]
    pub unk_c3861c3: f32,
    /// Native `flSummonLifetime` at byte 168.
    #[d4(value)]
    pub summon_lifetime: f32,
    /// Native `nSummonMaxConcurrent` at byte 172.
    #[d4(value)]
    pub summon_max_concurrent: i32,
    /// Native `nSummonMaxTotal` at byte 176.
    #[d4(value)]
    pub summon_max_total: i32,
    /// Native `arAIBehaviors` at byte 180.
    #[d4(value)]
    pub ai_behaviors: [i32; 6],
    /// Native `snoTreasureClassSummoned` at byte 204.
    #[d4(value)]
    pub treasure_class_summoned: i32,
    /// Native `bKillSummonsOnDeath` at byte 208.
    #[d4(value)]
    pub kill_summons_on_death: i32,
    /// Native `snoSummonActor` at byte 212.
    #[d4(value)]
    pub summon_actor: [i32; 6],
    /// Unclassified native bytes at offset 236.
    #[d4(value)]
    pub reserved_236: [u8; 4],
    /// Native `arSummonActors` at byte 240.
    #[d4(array)]
    pub summon_actors: Vec<SummonActorEntry>,
    /// Native `unk_43a5b25` at byte 256.
    #[d4(array)]
    pub unk_43a5b25: ZeroVec<'data, i32>,
    /// Native `bDisableRename` at byte 272.
    #[d4(value)]
    pub disable_rename: i32,
    /// Native `tFixedAffixData` at byte 276.
    #[d4(inline_array)]
    pub fixed_affix_data: [MonsterFixedAffixData; 4],
    /// Unclassified native bytes at offset 308.
    #[d4(value)]
    pub reserved_308: [u8; 4],
    /// Native `snoDisallowedAffixes` at byte 312.
    #[d4(array)]
    pub disallowed_affixes: ZeroVec<'data, i32>,
    /// Native `arForcedMonsterTags` at byte 328.
    #[d4(array)]
    pub forced_monster_tags: ZeroVec<'data, i32>,
    /// Native `eAITargetStyleNormal` at byte 344.
    #[d4(value)]
    pub ai_target_style_normal: i32,
    /// Native `eAITargetStyleElite` at byte 348.
    #[d4(value)]
    pub ai_target_style_elite: i32,
    /// Native `gbidAITargetingStyleNormal` at byte 352.
    #[d4(value)]
    pub ai_targeting_style_normal: i32,
    /// Native `gbidAITargetingStyleElite` at byte 356.
    #[d4(value)]
    pub ai_targeting_style_elite: i32,
    /// Native `dwLookAtTargetingFlags` at byte 360.
    #[d4(value)]
    pub look_at_targeting_flags: u32,
    /// Native `szInternalName` at byte 364.
    #[d4(value)]
    pub internal_name: [u8; 128],
    /// Native `bDoesRetreat` at byte 492.
    #[d4(value)]
    pub does_retreat: i32,
    /// Native `unk_50569ba` at byte 496.
    #[d4(value)]
    pub unk_50569ba: i32,
    /// Native `dwCCImmunityFlags` at byte 500.
    #[d4(value)]
    pub cc_immunity_flags: u32,
    /// Native `snoMount` at byte 504.
    #[d4(value)]
    pub mount: i32,
    /// Native `flMountScale` at byte 508.
    #[d4(value)]
    pub mount_scale: f32,
    /// Native `vMountAttachOffset` at byte 512.
    #[d4(value)]
    pub mount_attach_offset: [f32; 3],
    /// Native `snoPowerPossessed` at byte 524.
    #[d4(value)]
    pub power_possessed: i32,
    /// Native `snoFootsteps` at byte 528.
    #[d4(value)]
    pub footsteps: i32,
    /// Native `eOverriddenMonsterRarity` at byte 532.
    #[d4(value)]
    pub overridden_monster_rarity: i32,
    /// Native `unk_7859c5b` at byte 536.
    #[d4(value)]
    pub unk_7859c5b: i32,
    /// Native `bIsWorldBoss` at byte 540.
    #[d4(value)]
    pub is_world_boss: i32,
    /// Native `unk_4b1672a` at byte 544.
    #[d4(value)]
    pub unk_4b1672a: i32,
    /// Native `unk_1e3ad71` at byte 548.
    #[d4(value)]
    pub unk_1e3ad71: i32,
    /// Native `unk_2fc83ba` at byte 552.
    #[d4(value)]
    pub unk_2fc83ba: f32,
    /// Native `unk_68f046c` at byte 556.
    #[d4(value)]
    pub unk_68f046c: i32,
    /// Native `unk_e4c90d3` at byte 560.
    #[d4(value)]
    pub unk_e4c90d3: i32,
    /// Native `unk_12ecd6a` at byte 564.
    #[d4(value)]
    pub unk_12ecd6a: i32,
    /// Native `unk_c0565bf` at byte 568.
    #[d4(value)]
    pub unk_c0565bf: i32,
    /// Native `flHPScalarOverride` at byte 572.
    #[d4(value)]
    pub hp_scalar_override: f32,
    /// Native `flDamageScalarOverride` at byte 576.
    #[d4(value)]
    pub damage_scalar_override: f32,
    /// Native `unk_c2cc767` at byte 580.
    #[d4(value)]
    pub unk_c2cc767: i32,
    /// Native `snoStaggerPower` at byte 584.
    #[d4(value)]
    pub stagger_power: i32,
    /// Native `snoStaggerSettings` at byte 588.
    #[d4(value)]
    pub stagger_settings: i32,
    /// Native `snoBossObserver` at byte 592.
    #[d4(value)]
    pub boss_observer: i32,
    /// Native `unk_1627b04` at byte 596.
    #[d4(value)]
    pub unk_1627b04: f32,
    /// Native `flBossCameraBiasFromPlayerMin` at byte 600.
    #[d4(value)]
    pub boss_camera_bias_from_player_min: f32,
    /// Native `flBossCameraBiasFromPlayerMax` at byte 604.
    #[d4(value)]
    pub boss_camera_bias_from_player_max: f32,
    /// Native `flBossCameraBiasDistanceMin` at byte 608.
    #[d4(value)]
    pub boss_camera_bias_distance_min: f32,
    /// Native `flBossCameraBiasDistanceMax` at byte 612.
    #[d4(value)]
    pub boss_camera_bias_distance_max: f32,
    /// Native `flBossCameraSmoothing` at byte 616.
    #[d4(value)]
    pub boss_camera_smoothing: f32,
    /// Native `flBossCameraMaxMoveRatePerSecond` at byte 620.
    #[d4(value)]
    pub boss_camera_max_move_rate_per_second: f32,
    /// Native `flBossCameraActivationRadius` at byte 624.
    #[d4(value)]
    pub boss_camera_activation_radius: f32,
    /// Native `flBossCameraDectivationRadius` at byte 628.
    #[d4(value)]
    pub boss_camera_dectivation_radius: f32,
    /// Native `flBossCameraZoomIntroTime` at byte 632.
    #[d4(value)]
    pub boss_camera_zoom_intro_time: f32,
    /// Native `flBossCameraZoomOutroTime` at byte 636.
    #[d4(value)]
    pub boss_camera_zoom_outro_time: f32,
    /// Native `unk_dec81bb` at byte 640.
    #[d4(value)]
    pub unk_dec81bb: f32,
    /// Unclassified native bytes at offset 644.
    #[d4(value)]
    pub reserved_644: [u8; 4],
}
impl Default for ActorMonsterData<'_> {
    fn default() -> Self {
        Self {
            reserved_92: [0; 4],
            reserved_236: [0; 4],
            reserved_308: [0; 4],
            reserved_644: [0; 4],
            gender: Default::default(),
            is_npc: Default::default(),
            unk_7c902e4: Default::default(),
            is_meta_quest_turn_in_npc: Default::default(),
            is_alt_meta_quest_turn_in_npc: Default::default(),
            npc_operate_sound: Default::default(),
            npc_power_operate: Default::default(),
            npc_interact_distance: Default::default(),
            npc_interact_cancel_distance: Default::default(),
            npc_service_data: Default::default(),
            converts_damage_to_damage_states: Default::default(),
            unk_6bce993: Default::default(),
            unk_6b4a81d: Default::default(),
            monster_family: Default::default(),
            archetype: Default::default(),
            base_element: Default::default(),
            main_hand: Default::default(),
            off_hand: Default::default(),
            unk_f8ee66e: Default::default(),
            gold_granted: Default::default(),
            unk_64df7b: Default::default(),
            force_loot_drop_on_death: Default::default(),
            treasure_class: Default::default(),
            treasure_class_elite: Default::default(),
            treasure_class_super_elite: Default::default(),
            unk_c3861c3: Default::default(),
            summon_lifetime: Default::default(),
            summon_max_concurrent: Default::default(),
            summon_max_total: Default::default(),
            ai_behaviors: core::array::from_fn(|_| Default::default()),
            treasure_class_summoned: Default::default(),
            kill_summons_on_death: Default::default(),
            summon_actor: core::array::from_fn(|_| Default::default()),
            summon_actors: Default::default(),
            unk_43a5b25: Default::default(),
            disable_rename: Default::default(),
            fixed_affix_data: core::array::from_fn(|_| Default::default()),
            disallowed_affixes: Default::default(),
            forced_monster_tags: Default::default(),
            ai_target_style_normal: Default::default(),
            ai_target_style_elite: Default::default(),
            ai_targeting_style_normal: Default::default(),
            ai_targeting_style_elite: Default::default(),
            look_at_targeting_flags: Default::default(),
            internal_name: core::array::from_fn(|_| Default::default()),
            does_retreat: Default::default(),
            unk_50569ba: Default::default(),
            cc_immunity_flags: Default::default(),
            mount: Default::default(),
            mount_scale: Default::default(),
            mount_attach_offset: core::array::from_fn(|_| Default::default()),
            power_possessed: Default::default(),
            footsteps: Default::default(),
            overridden_monster_rarity: Default::default(),
            unk_7859c5b: Default::default(),
            is_world_boss: Default::default(),
            unk_4b1672a: Default::default(),
            unk_1e3ad71: Default::default(),
            unk_2fc83ba: Default::default(),
            unk_68f046c: Default::default(),
            unk_e4c90d3: Default::default(),
            unk_12ecd6a: Default::default(),
            unk_c0565bf: Default::default(),
            hp_scalar_override: Default::default(),
            damage_scalar_override: Default::default(),
            unk_c2cc767: Default::default(),
            stagger_power: Default::default(),
            stagger_settings: Default::default(),
            boss_observer: Default::default(),
            unk_1627b04: Default::default(),
            boss_camera_bias_from_player_min: Default::default(),
            boss_camera_bias_from_player_max: Default::default(),
            boss_camera_bias_distance_min: Default::default(),
            boss_camera_bias_distance_max: Default::default(),
            boss_camera_smoothing: Default::default(),
            boss_camera_max_move_rate_per_second: Default::default(),
            boss_camera_activation_radius: Default::default(),
            boss_camera_dectivation_radius: Default::default(),
            boss_camera_zoom_intro_time: Default::default(),
            boss_camera_zoom_outro_time: Default::default(),
            unk_dec81bb: Default::default(),
        }
    }
}

/// Native `ActorPlayerData` (272 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorPlayerData<'data> {
    /// Native `eGender` at byte 0.
    #[d4(value)]
    pub gender: i32,
    /// Native `ePlayerClass` at byte 4.
    #[d4(value)]
    pub player_class: i32,
    /// Native `arSkinColorChoices` at byte 8.
    #[d4(array)]
    pub skin_color_choices: Vec<PersonaSkinColor<'data>>,
    /// Native `snoPowerPassives` at byte 24.
    #[d4(array)]
    pub power_passives: ZeroVec<'data, i32>,
    /// Native `flIrisSize` at byte 40.
    #[d4(value)]
    pub iris_size: f32,
    /// Native `arWarningConversationLines` at byte 44.
    #[d4(inline_array)]
    pub warning_conversation_lines: [ConversationNodeReference; 26],
    /// Unclassified native bytes at offset 252.
    #[d4(value)]
    pub reserved_252: [u8; 4],
    /// Native `arConditionalAudioStates` at byte 256.
    #[d4(array)]
    pub conditional_audio_states: Vec<ConditionalAudioState<'data>>,
}

/// Native `ConditionalAudioState` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ConditionalAudioState<'data> {
    /// Native `tDefaultAudioState` at byte 0.
    pub default_audio_state: WwiseSwitchValue,
    /// Native `arStateSetters` at byte 8.
    #[d4(array)]
    pub state_setters: Vec<ConditionalAudioStateSetter<'data>>,
}

/// Native `ActorMountData` (288 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorMountData {
    /// Native `eMountRarity` at byte 0.
    #[d4(value)]
    pub mount_rarity: i32,
    /// Native `eMountType` at byte 4.
    #[d4(value)]
    pub mount_type: i32,
    /// Native `snoMountPowerOverride` at byte 8.
    #[d4(value)]
    pub mount_power_override: i32,
    /// Native `snoSkillOverrides` at byte 12.
    #[d4(value)]
    pub skill_overrides: [i32; 6],
    /// Native `snoEvadeOverride` at byte 36.
    #[d4(value)]
    pub evade_override: i32,
    /// Native `snoTownPortalOverride` at byte 40.
    #[d4(value)]
    pub town_portal_override: i32,
    /// Native `snoDismountOverride` at byte 44.
    #[d4(value)]
    pub dismount_override: i32,
    /// Native `snoPotionOverride` at byte 48.
    #[d4(value)]
    pub potion_override: i32,
    /// Native `snoObserverOverride` at byte 52.
    #[d4(value)]
    pub observer_override: i32,
    /// Native `flDamageScalar` at byte 56.
    #[d4(value)]
    pub damage_scalar: f32,
    /// Native `flDamageReduction` at byte 60.
    #[d4(value)]
    pub damage_reduction: f32,
    /// Native `flDodgeChanceBonus` at byte 64.
    #[d4(value)]
    pub dodge_chance_bonus: f32,
    /// Native `gbidDamageMitigation` at byte 68.
    #[d4(value)]
    pub damage_mitigation: i32,
    /// Native `gbidMountParams` at byte 72.
    #[d4(value)]
    pub mount_params: i32,
    /// Native `arWarningConversationLines` at byte 76.
    #[d4(inline_array)]
    pub warning_conversation_lines: [ConversationNodeReference; 26],
    /// Native `snoSpeaker` at byte 284.
    #[d4(value)]
    pub speaker: i32,
}

/// Native `ActorHitRotationJoint` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorHitRotationJoint {
    /// Native `szBoneName` at byte 0.
    #[d4(value)]
    pub bone_name: u32,
    /// Native `flBlendOutTime` at byte 4.
    #[d4(value)]
    pub blend_out_time: f32,
    /// Native `flDuration` at byte 8.
    #[d4(value)]
    pub duration: f32,
    /// Native `flRotationSpeed` at byte 12.
    #[d4(value)]
    pub rotation_speed: f32,
    /// Native `flRotationAngle` at byte 16.
    #[d4(value)]
    pub rotation_angle: f32,
    /// Native `unk_92ce5f2` at byte 20.
    #[d4(value)]
    pub unk_92ce5f2: i32,
    /// Native `pathDampeningCurve` at byte 24.
    pub path_dampening_curve: InterpolationPathFloat,
    /// Native `arChildRotationJoints` at byte 80.
    #[d4(array)]
    pub child_rotation_joints: Vec<ActorChildRotationJoint>,
}

/// Native `ActorAIDisperseData` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorAIDisperseData {
    /// Native `arTiers` at byte 0.
    #[d4(array)]
    pub tiers: Vec<ActorAIDisperseTierData>,
}

/// Native `ActorPropData` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorPropData {
    /// Native `fIsCorpse` at byte 0.
    #[d4(value)]
    pub is_corpse: i32,
    /// Native `fIsTraversalExit` at byte 4.
    #[d4(value)]
    pub is_traversal_exit: i32,
    /// Native `eCrafterType` at byte 8.
    #[d4(value)]
    pub crafter_type: i32,
    /// Native `fIsTraversalRope` at byte 12.
    #[d4(value)]
    pub is_traversal_rope: i32,
    /// Native `flTraversalRopeLength` at byte 16.
    #[d4(value)]
    pub traversal_rope_length: f32,
}

/// Native `ActorPhysicsData` (28 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorPhysicsData {
    /// Native `snoPhysics` at byte 0.
    #[d4(value)]
    pub physics: i32,
    /// Native `snoPartialRagdollPhysics` at byte 4.
    #[d4(value)]
    pub partial_ragdoll_physics: i32,
    /// Native `eMaterial` at byte 8.
    #[d4(value)]
    pub material: i32,
    /// Native `flExplosionFactor` at byte 12.
    #[d4(value)]
    pub explosion_factor: f32,
    /// Native `flWindFactor` at byte 16.
    #[d4(value)]
    pub wind_factor: f32,
    /// Native `flPartialRagdollResponsiveness` at byte 20.
    #[d4(value)]
    pub partial_ragdoll_responsiveness: f32,
    /// Native `dwExplosionChannel` at byte 24.
    #[d4(value)]
    pub explosion_channel: u32,
}

/// Native `ActorItemData` (2416 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorItemData {
    /// Native `snoItemInventorySound` at byte 0.
    #[d4(value)]
    pub item_inventory_sound: i32,
    /// Native `snoSalvageSound` at byte 4.
    #[d4(value)]
    pub salvage_sound: i32,
    /// Native `snoRepairSound` at byte 8.
    #[d4(value)]
    pub repair_sound: i32,
    /// Native `snoCraftedSound` at byte 12.
    #[d4(value)]
    pub crafted_sound: i32,
    /// Native `snoPickupOverrideSound` at byte 16.
    #[d4(value)]
    pub pickup_override_sound: i32,
    /// Native `snoFlippySoundOverride` at byte 20.
    #[d4(value)]
    pub flippy_sound_override: i32,
    /// Native `snoFlippyLandingSound` at byte 24.
    #[d4(value)]
    pub flippy_landing_sound: i32,
    /// Native `snoFlippyMouseoverSound` at byte 28.
    #[d4(value)]
    pub flippy_mouseover_sound: i32,
    /// Native `arFoleyMaterialTags` at byte 32.
    #[d4(inline_array)]
    pub foley_material_tags: [FoleyMaterialTag; 8],
    /// Native `unk_472eb5b` at byte 64.
    #[d4(value)]
    pub unk_472eb5b: i32,
    /// Native `snoLootFlippyParticle` at byte 68.
    #[d4(value)]
    pub loot_flippy_particle: i32,
    /// Native `snoLootSpecialFlippyActor` at byte 72.
    #[d4(value)]
    pub loot_special_flippy_actor: i32,
    /// Native `fLootLaunchFlippyForwards` at byte 76.
    #[d4(value)]
    pub loot_launch_flippy_forwards: i32,
    /// Native `fLootFlippyDontOverrideParticleActor` at byte 80.
    #[d4(value)]
    pub loot_flippy_dont_override_particle_actor: i32,
    /// Native `unk_13d073c` at byte 84.
    #[d4(value)]
    pub unk_13d073c: i32,
    /// Native `fVisibleToAll` at byte 88.
    #[d4(value)]
    pub visible_to_all: i32,
    /// Native `unk_842ce3a` at byte 92.
    #[d4(value)]
    pub unk_842ce3a: i32,
    /// Native `flLootFlippyLandingHeight` at byte 96.
    #[d4(value)]
    pub loot_flippy_landing_height: f32,
    /// Native `unk_62f6313` at byte 100.
    #[d4(value)]
    pub unk_62f6313: i32,
    /// Native `snoSwitchLootItemBeams` at byte 104.
    #[d4(value)]
    pub switch_loot_item_beams: i32,
    /// Native `snoPickupEffectGroup` at byte 108.
    #[d4(value)]
    pub pickup_effect_group: i32,
    /// Native `tAttachmentHardpointLink` at byte 112.
    pub attachment_hardpoint_link: HardpointLink,
    /// Native `unk_204fa35` at byte 120.
    #[d4(value)]
    pub unk_204fa35: i32,
    /// Native `unk_28e7959` at byte 124.
    #[d4(value)]
    pub unk_28e7959: i32,
    /// Native `unk_1427368` at byte 128.
    #[d4(value)]
    pub unk_1427368: i32,
    /// Native `tHairStateDefault` at byte 132.
    pub hair_state_default: HairAndBeardState,
    /// Native `fUseClassHairStateOverrides` at byte 256.
    #[d4(value)]
    pub use_class_hair_state_overrides: i32,
    /// Native `fUseClassHairStateGenderOverrides` at byte 260.
    #[d4(value)]
    pub use_class_hair_state_gender_overrides: i32,
    /// Native `tClassHairStateOverrides` at byte 264.
    #[d4(inline_array)]
    pub class_hair_state_overrides: [HairAndBeardState; 8],
    /// Native `tClassHairStateGenderOverrides` at byte 1256.
    #[d4(inline_array)]
    pub class_hair_state_gender_overrides: [HairStateGenderOverrides; 8],
    /// Native `snoAttachedActor` at byte 1448.
    #[d4(value)]
    pub attached_actor: i32,
    /// Native `hDefaultImage` at byte 1452.
    #[d4(value)]
    pub h_default_image: u32,
    /// Native `hSocketedImage` at byte 1456.
    #[d4(value)]
    pub h_socketed_image: u32,
    /// Native `hSocketedActiveImage` at byte 1460.
    #[d4(value)]
    pub h_socketed_active_image: u32,
    /// Native `fMustKeepInInventory` at byte 1464.
    #[d4(value)]
    pub must_keep_in_inventory: i32,
    /// Native `fBottomless` at byte 1468.
    #[d4(value)]
    pub bottomless: i32,
    /// Native `eItemUseType` at byte 1472.
    #[d4(value)]
    pub item_use_type: i32,
    /// Native `snoPowerUpPower` at byte 1476.
    #[d4(value)]
    pub power_up_power: i32,
    /// Native `snoItemUseLevelAreaRequirement` at byte 1480.
    #[d4(value)]
    pub item_use_level_area_requirement: i32,
    /// Unclassified native bytes at offset 1484.
    #[d4(value)]
    pub reserved_1484: [u8; 4],
    /// Native `tHardpointOffsets` at byte 1488.
    #[d4(inline_array)]
    pub hardpoint_offsets: [HardpointOffsets; 8],
    /// Native `tHardpointMountedOffsets` at byte 1808.
    #[d4(inline_array)]
    pub hardpoint_mounted_offsets: [HardpointOffsets; 8],
    /// Native `tHardpointOffsetsForMounts` at byte 2128.
    #[d4(inline_array)]
    pub hardpoint_offsets_for_mounts: [MountHardpointOffsets; 4],
    /// Native `tMirroredState` at byte 2224.
    #[d4(inline_array)]
    pub mirrored_state: [MirrorItemState; 8],
    /// Native `tWeaponData` at byte 2256.
    pub weapon_data: ActorWeaponData,
    /// Native `tVisualInvData` at byte 2272.
    pub visual_inv_data: ActorVisualInventoryData,
    /// Native `unk_e133e62` at byte 2404.
    #[d4(value)]
    pub unk_e133e62: i32,
    /// Native `unk_6990c4a` at byte 2408.
    #[d4(value)]
    pub unk_6990c4a: i32,
    /// Unclassified native bytes at offset 2412.
    #[d4(value)]
    pub reserved_2412: [u8; 4],
}

/// Native `ConditionalAudioStateSetter` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ConditionalAudioStateSetter<'data> {
    /// Native `tCondition` at byte 0.
    pub condition: ConditionWrapper<'data>,
    /// Native `unk_dd101c8` at byte 32.
    #[d4(value)]
    pub unk_dd101c8: i32,
    /// Native `tAudioState` at byte 36.
    pub audio_state: WwiseSwitchValue,
    /// Native `flMinimumDuration` at byte 44.
    #[d4(value)]
    pub minimum_duration: f32,
}

/// Native `ActorCollisionData` (192 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorCollisionData {
    /// Native `tCollisionSettings` at byte 0.
    pub collision_settings: CollisionSettings,
    /// Native `eCollisionShape` at byte 24.
    #[d4(value)]
    pub collision_shape: i32,
    /// Native `wcylCollision` at byte 28.
    pub wcyl_collision: AxialCylinder,
    /// Native `aabbCollision` at byte 48.
    pub aabb_collision: Bounds,
    /// Native `unk_3d328de` at byte 72.
    #[d4(value)]
    pub unk_3d328de: i32,
    /// Native `unk_7a83a6d` at byte 76.
    #[d4(value)]
    pub unk_7a83a6d: i32,
    /// Native `bAffectsRagdoll` at byte 80.
    #[d4(value)]
    pub affects_ragdoll: i32,
    /// Native `flStepUpHeight` at byte 84.
    #[d4(value)]
    pub step_up_height: f32,
    /// Native `eNavBehavior` at byte 88.
    #[d4(value)]
    pub nav_behavior: i32,
    /// Native `flForceSimScaleFactor` at byte 92.
    #[d4(value)]
    pub force_sim_scale_factor: f32,
    /// Native `ptHitCollisionShapes` at byte 96.
    #[d4(array)]
    pub hit_collision_shapes: Vec<ActorHitCollisionShape>,
    /// Native `tProjectileSceneCollisionShape` at byte 112.
    pub projectile_scene_collision_shape: ActorHitCollisionShape,
    /// Native `tProjectileActorCollisionShape` at byte 148.
    pub projectile_actor_collision_shape: ActorHitCollisionShape,
    /// Native `unk_c627b16` at byte 184.
    #[d4(value)]
    pub unk_c627b16: i32,
    /// Native `unk_7e60e04` at byte 188.
    #[d4(value)]
    pub unk_7e60e04: i32,
}

/// Native `ActorGizmoData` (208 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorGizmoData {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `fDisableNonFriendlyOperation` at byte 16.
    #[d4(value)]
    pub disable_non_friendly_operation: i32,
    /// Native `fWalkToCenter` at byte 20.
    #[d4(value)]
    pub walk_to_center: i32,
    /// Native `fGizmoStartsDisabled` at byte 24.
    #[d4(value)]
    pub gizmo_starts_disabled: i32,
    /// Native `unk_3245c42` at byte 28.
    #[d4(value)]
    pub unk_3245c42: i32,
    /// Native `fUseStringListName` at byte 32.
    #[d4(value)]
    pub use_string_list_name: i32,
    /// Native `fForceMinimapIcon` at byte 36.
    #[d4(value)]
    pub force_minimap_icon: i32,
    /// Native `flOperationRadius` at byte 40.
    #[d4(value)]
    pub operation_radius: f32,
    /// Native `unk_d0b1df5` at byte 44.
    #[d4(value)]
    pub unk_d0b1df5: f32,
    /// Native `flGizmoUseTime` at byte 48.
    #[d4(value)]
    pub gizmo_use_time: f32,
    /// Native `unk_c61db5a` at byte 52.
    #[d4(value)]
    pub unk_c61db5a: i32,
    /// Native `snoChannelingEffect` at byte 56.
    #[d4(value)]
    pub channeling_effect: i32,
    /// Native `unk_2767229` at byte 60.
    #[d4(value)]
    pub unk_2767229: i32,
    /// Native `fGizmoShowProgressBar` at byte 64.
    #[d4(value)]
    pub gizmo_show_progress_bar: i32,
    /// Native `unk_6d9ebe1` at byte 68.
    #[d4(value)]
    pub unk_6d9ebe1: i32,
    /// Native `unk_5456245` at byte 72.
    #[d4(value)]
    pub unk_5456245: i32,
    /// Native `unk_af5815a` at byte 76.
    #[d4(value)]
    pub unk_af5815a: i32,
    /// Native `snoOnClickEffect` at byte 80.
    #[d4(value)]
    pub on_click_effect: i32,
    /// Native `snoPickedUpEffect` at byte 84.
    #[d4(value)]
    pub picked_up_effect: i32,
    /// Native `snoDropOnGroundEffect` at byte 88.
    #[d4(value)]
    pub drop_on_ground_effect: i32,
    /// Native `tConditionData` at byte 92.
    pub condition_data: GizmoConditionData,
    /// Native `snoPowerOperator` at byte 96.
    #[d4(value)]
    pub power_operator: i32,
    /// Native `snoPowerOperate` at byte 100.
    #[d4(value)]
    pub power_operate: i32,
    /// Native `snoPowerOperatorActive` at byte 104.
    #[d4(value)]
    pub power_operator_active: i32,
    /// Native `snoPowerOperateActive` at byte 108.
    #[d4(value)]
    pub power_operate_active: i32,
    /// Native `unk_359810e` at byte 112.
    #[d4(value)]
    pub unk_359810e: i32,
    /// Native `nCooldown` at byte 116.
    #[d4(value)]
    pub cooldown: u32,
    /// Native `unk_9c8478e` at byte 120.
    #[d4(value)]
    pub unk_9c8478e: u32,
    /// Native `bImmediatelyReusable` at byte 124.
    #[d4(value)]
    pub immediately_reusable: i32,
    /// Native `unk_d45ed2d` at byte 128.
    #[d4(value)]
    pub unk_d45ed2d: i32,
    /// Native `unk_f6b20b7` at byte 132.
    #[d4(value)]
    pub unk_f6b20b7: i32,
    /// Native `unk_9c038ce` at byte 136.
    #[d4(value)]
    pub unk_9c038ce: i32,
    /// Native `fCanRespawn` at byte 140.
    #[d4(value)]
    pub can_respawn: i32,
    /// Native `hGizmoNotOperableReasonMsgLabel` at byte 144.
    #[d4(value)]
    pub h_gizmo_not_operable_reason_msg_label: u32,
    /// Native `hGizmoOperateFlavorTextMsgLabel` at byte 148.
    #[d4(value)]
    pub h_gizmo_operate_flavor_text_msg_label: u32,
    /// Native `hGizmoOperateDescriptionMsgLabel` at byte 152.
    #[d4(value)]
    pub h_gizmo_operate_description_msg_label: u32,
    /// Native `flGizmoOperateOffset` at byte 156.
    #[d4(value)]
    pub gizmo_operate_offset: f32,
    /// Native `unk_ab58cd3` at byte 160.
    #[d4(value)]
    pub unk_ab58cd3: i32,
    /// Native `unk_bc2c5f5` at byte 164.
    #[d4(value)]
    pub unk_bc2c5f5: i32,
    /// Unclassified native bytes at offset 168.
    #[d4(value)]
    pub reserved_168: [u8; 4],
    /// Native `unk_695b5e1` at byte 172.
    #[d4(value)]
    pub unk_695b5e1: i32,
    /// Native `unk_91a3966` at byte 176.
    #[d4(value)]
    pub unk_91a3966: u32,
    /// Native `hGizmoChannelingLabel` at byte 180.
    #[d4(value)]
    pub h_gizmo_channeling_label: u32,
    /// Native `unk_4be4371` at byte 184.
    #[d4(value)]
    pub unk_4be4371: i32,
    /// Native `unk_920c13e` at byte 188.
    #[d4(value)]
    pub unk_920c13e: u32,
    /// Native `unk_697e40a` at byte 192.
    #[d4(value)]
    pub unk_697e40a: u32,
    /// Native `unk_8b8eac2` at byte 196.
    #[d4(value)]
    pub unk_8b8eac2: i32,
    /// Native `unk_361e33e` at byte 200.
    #[d4(value)]
    pub unk_361e33e: i32,
    /// Native `unk_88251d3` at byte 204.
    #[d4(value)]
    pub unk_88251d3: u32,
}

/// Native `ActorCombatData` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorCombatData {
    /// Native `snoHitBySoundTable` at byte 0.
    #[d4(value)]
    pub hit_by_sound_table: i32,
    /// Native `eWeaponHitsound` at byte 4.
    #[d4(value)]
    pub weapon_hitsound: i32,
    /// Native `arHitReactionOverrides` at byte 8.
    #[d4(array)]
    pub hit_reaction_overrides: Vec<HitReactionOverride>,
    /// Native `unk_5909501` at byte 24.
    #[d4(value)]
    pub unk_5909501: f32,
    /// Unclassified native bytes at offset 28.
    #[d4(value)]
    pub reserved_28: [u8; 4],
    /// Native `arJitterJoints` at byte 32.
    #[d4(array)]
    pub jitter_joints: Vec<ActorJitterJoint>,
    /// Native `arRotationJoints` at byte 48.
    #[d4(array)]
    pub rotation_joints: Vec<ActorHitRotationJoint>,
}

/// Native `ActorWeaponData` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorWeaponData {
    /// Native `eWeaponHitsound` at byte 0.
    #[d4(value)]
    pub weapon_hitsound: i32,
    /// Native `snoExplosionSno` at byte 4.
    #[d4(value)]
    pub explosion_sno: i32,
    /// Native `snoProjectileSno` at byte 8.
    #[d4(value)]
    pub projectile_sno: i32,
    /// Native `snoWeaponEffectSno` at byte 12.
    #[d4(value)]
    pub weapon_effect_sno: i32,
}

/// Native `ActorAIData` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorAIData {
    /// Native `ptAIBroadcasts` at byte 0.
    #[d4(array)]
    pub ai_broadcasts: Vec<AIBroadcast>,
    /// Native `gbidAIPropType` at byte 16.
    #[d4(value)]
    pub ai_prop_type: i32,
    /// Native `unk_af0baa5` at byte 20.
    #[d4(value)]
    pub unk_af0baa5: i32,
    /// Native `arAIPropPowerPairings` at byte 24.
    #[d4(array)]
    pub ai_prop_power_pairings: Vec<AIPropPowerPairing>,
}

/// Native `ActorAIDisperseTierData` (52 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorAIDisperseTierData {
    /// Native `flDist` at byte 0.
    #[d4(value)]
    pub dist: f32,
    /// Native `nSlots` at byte 4.
    #[d4(value)]
    pub slots: i32,
    /// Native `unk_13fdbf` at byte 8.
    #[d4(value)]
    pub unk_13fdbf: i32,
    /// Native `sLayer` at byte 12.
    #[d4(value)]
    pub s_layer: u32,
    /// Native `unk_8680393` at byte 16.
    #[d4(value)]
    pub unk_8680393: i32,
    /// Native `unk_8337d1e` at byte 20.
    #[d4(value)]
    pub unk_8337d1e: i32,
    /// Native `unk_395f1b4` at byte 24.
    #[d4(value)]
    pub unk_395f1b4: i32,
    /// Native `tValidWindow` at byte 28.
    #[d4(value)]
    pub valid_window: [f32; 2],
    /// Native `unk_6ec4d4` at byte 36.
    #[d4(value)]
    pub unk_6ec4d4: i32,
    /// Native `flPositioningTolerance` at byte 40.
    #[d4(value)]
    pub positioning_tolerance: f32,
    /// Native `unk_6fb4c8e` at byte 44.
    #[d4(value)]
    pub unk_6fb4c8e: f32,
    /// Native `unk_2392c67` at byte 48.
    #[d4(value)]
    pub unk_2392c67: f32,
}

/// Native `ActorAppearanceAddOnEntry` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorAppearanceAddOnEntry {
    /// Native `gbidAddOnType` at byte 0.
    #[d4(value)]
    pub add_on_type: i32,
    /// Native `snoAppearance` at byte 4.
    #[d4(value)]
    pub appearance: i32,
    /// Native `dwShaderMapOverride` at byte 8.
    #[d4(value)]
    pub shader_map_override: u32,
}

/// Native `GizmoConditionData` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GizmoConditionData {
    /// Native `snoGizmoCondition` at byte 0.
    #[d4(value)]
    pub gizmo_condition: i32,
}

/// Native `ActorSoundTableData` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorSoundTableData {
    /// Native `szMonsterCategory` at byte 0.
    #[d4(value)]
    pub monster_category: u32,
    /// Native `bOverrideImportanceScore` at byte 4.
    #[d4(value)]
    pub override_importance_score: i32,
    /// Native `nImportanceScoreOverride` at byte 8.
    #[d4(value)]
    pub importance_score_override: i32,
    /// Native `snoRunSoundTable` at byte 12.
    #[d4(value)]
    pub run_sound_table: i32,
    /// Native `snoWalkSoundTable` at byte 16.
    #[d4(value)]
    pub walk_sound_table: i32,
    /// Native `snoSlideSoundTable` at byte 20.
    #[d4(value)]
    pub slide_sound_table: i32,
    /// Native `snoLandSoundTable` at byte 24.
    #[d4(value)]
    pub land_sound_table: i32,
    /// Native `snoNPCBarefootRunSoundTable` at byte 28.
    #[d4(value)]
    pub npc_barefoot_run_sound_table: i32,
    /// Native `snoNPCBarefootWalkSoundTable` at byte 32.
    #[d4(value)]
    pub npc_barefoot_walk_sound_table: i32,
    /// Native `snoNPCBarefootSlideSoundTable` at byte 36.
    #[d4(value)]
    pub npc_barefoot_slide_sound_table: i32,
    /// Native `snoNPCBarefootLandSoundTable` at byte 40.
    #[d4(value)]
    pub npc_barefoot_land_sound_table: i32,
    /// Native `snoNPCClothRunSoundTable` at byte 44.
    #[d4(value)]
    pub npc_cloth_run_sound_table: i32,
    /// Native `snoNPCClothWalkSoundTable` at byte 48.
    #[d4(value)]
    pub npc_cloth_walk_sound_table: i32,
    /// Native `snoNPCClothSlideSoundTable` at byte 52.
    #[d4(value)]
    pub npc_cloth_slide_sound_table: i32,
    /// Native `snoNPCClothLandSoundTable` at byte 56.
    #[d4(value)]
    pub npc_cloth_land_sound_table: i32,
    /// Native `snoNPCLeatherRunSoundTable` at byte 60.
    #[d4(value)]
    pub npc_leather_run_sound_table: i32,
    /// Native `snoNPCLeatherWalkSoundTable` at byte 64.
    #[d4(value)]
    pub npc_leather_walk_sound_table: i32,
    /// Native `snoNPCLeatherSlideSoundTable` at byte 68.
    #[d4(value)]
    pub npc_leather_slide_sound_table: i32,
    /// Native `snoNPCLeatherLandSoundTable` at byte 72.
    #[d4(value)]
    pub npc_leather_land_sound_table: i32,
    /// Native `snoVoiceSoundTable` at byte 76.
    #[d4(value)]
    pub voice_sound_table: i32,
    /// Native `arFoleyOverrides` at byte 80.
    #[d4(array)]
    pub foley_overrides: Vec<ActorFoleySoundOverride>,
}

/// Native `ActorCollisionFlags` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorCollisionFlags {
    /// Native `uDynamicFlagsNoOverlap` at byte 0.
    #[d4(value)]
    pub u_dynamic_flags_no_overlap: u32,
    /// Native `uDynamicMyFlags` at byte 4.
    #[d4(value)]
    pub u_dynamic_my_flags: u32,
}

/// Native `ActorCritterData` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorCritterData {
    /// Native `fScaredOfPlayers` at byte 0.
    #[d4(value)]
    pub scared_of_players: i32,
    /// Native `flSpeedWander` at byte 4.
    #[d4(value)]
    pub speed_wander: f32,
    /// Native `flSpeedRun` at byte 8.
    #[d4(value)]
    pub speed_run: f32,
    /// Native `tWanderDelayRange` at byte 12.
    #[d4(value)]
    pub wander_delay_range: [f32; 2],
    /// Native `flWanderTooLongTime` at byte 20.
    #[d4(value)]
    pub wander_too_long_time: f32,
    /// Native `tWanderRadiusRange` at byte 24.
    #[d4(value)]
    pub wander_radius_range: [f32; 2],
    /// Native `flScaredRadius` at byte 32.
    #[d4(value)]
    pub scared_radius: f32,
    /// Native `tScaredTimeRange` at byte 36.
    #[d4(value)]
    pub scared_time_range: [f32; 2],
    /// Native `fCanBeSquished` at byte 44.
    #[d4(value)]
    pub can_be_squished: i32,
    /// Native `tScaredRunAwayDistanceRange` at byte 48.
    #[d4(value)]
    pub scared_run_away_distance_range: [f32; 2],
    /// Native `fRunAnimWhenScared` at byte 56.
    #[d4(value)]
    pub run_anim_when_scared: i32,
    /// Native `flLethalExplosionMagnitude` at byte 60.
    #[d4(value)]
    pub lethal_explosion_magnitude: f32,
}

/// Native `ActorDefinition` (768 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorRecord<'data> {
    /// Resource SNO identity from the root prefix.
    #[d4(value)]
    pub sno_id: i32,
    /// Root record flags.
    #[d4(value)]
    pub record_flags: u32,
    /// Native `eType` at byte 8.
    #[d4(value)]
    pub type_: i32,
    /// Native `dwFlags` at byte 12.
    #[d4(value)]
    pub flags: u32,
    /// Native `dwFlagsEx` at byte 16.
    #[d4(value)]
    pub flags_ex: u32,
    /// Native `snoAppearance` at byte 20.
    #[d4(value)]
    pub appearance: i32,
    /// Native `arCustomizationAppearances` at byte 24.
    #[d4(array)]
    pub customization_appearances: ZeroVec<'data, i32>,
    /// Native `arAppearanceSets` at byte 40.
    #[d4(array)]
    pub appearance_sets: ZeroVec<'data, i32>,
    /// Native `arAnimSets` at byte 56.
    #[d4(array)]
    pub anim_sets: ZeroVec<'data, i32>,
    /// Native `arStoreAnimSets` at byte 72.
    #[d4(array)]
    pub store_anim_sets: ZeroVec<'data, i32>,
    /// Native `snoAnimTree` at byte 88.
    #[d4(value)]
    pub anim_tree: i32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `arActorAppearanceAddOnEntries` at byte 96.
    #[d4(array)]
    pub actor_appearance_add_on_entries: Vec<ActorAppearanceAddOnEntry>,
    /// Native `snoVisualRemap` at byte 112.
    #[d4(value)]
    pub visual_remap: i32,
    /// Native `gbidPartyFinderTag` at byte 116.
    #[d4(value)]
    pub party_finder_tag: i32,
    /// Native `arUniqueSubObjectNameInfoPerAppearance` at byte 120.
    #[d4(array)]
    pub unique_sub_object_name_info_per_appearance: Vec<AppearanceUniqueSubObjectNameInfo>,
    /// Native `tAnimTreeOverride` at byte 136.
    pub anim_tree_override: AnimTreeOverride<'data>,
    /// Native `arSortedEventIndices` at byte 152.
    #[d4(array)]
    pub sorted_event_indices: ZeroVec<'data, i32>,
    /// Native `eVariantType` at byte 168.
    #[d4(value)]
    pub variant_type: i32,
    /// Native `tDefaultLook` at byte 172.
    pub default_look: LookLink,
    /// Native `dwDefaultShaderMapOverride` at byte 176.
    #[d4(value)]
    pub default_shader_map_override: u32,
    /// Native `gbidDefaultAddOnType` at byte 180.
    #[d4(value)]
    pub default_add_on_type: i32,
    /// Native `tScaleRange` at byte 184.
    #[d4(value)]
    pub scale_range: [f32; 2],
    /// Native `tScaleLimit` at byte 192.
    #[d4(value)]
    pub scale_limit: [f32; 2],
    /// Native `wcylBounds` at byte 200.
    pub wcyl_bounds: AxialCylinder,
    /// Native `aabbBounds` at byte 220.
    pub aabb_bounds: Bounds,
    /// Native `wsBounds` at byte 244.
    pub bounds: Sphere,
    /// Native `aabbAppearanceBoundingBox` at byte 260.
    pub aabb_appearance_bounding_box: Bounds,
    /// Native `wpLocationPowerSrc` at byte 284.
    #[d4(value)]
    pub location_power_src: [f32; 3],
    /// Native `ePickStyle` at byte 296.
    #[d4(value)]
    pub pick_style: i32,
    /// Native `flAttachmentScaleFactor` at byte 300.
    #[d4(value)]
    pub attachment_scale_factor: f32,
    /// Native `eDefaultTeam` at byte 304.
    #[d4(value)]
    pub default_team: i32,
    /// Native `eCullingLevel` at byte 308.
    #[d4(value)]
    pub culling_level: i32,
    /// Native `unk_b7a6ddf` at byte 312.
    #[d4(value)]
    pub unk_b7a6ddf: i32,
    /// Native `flFadeCorpseTimeUntilFade` at byte 316.
    #[d4(value)]
    pub fade_corpse_time_until_fade: f32,
    /// Native `flFadeCorpseTime` at byte 320.
    #[d4(value)]
    pub fade_corpse_time: f32,
    /// Native `aAbsoluteOrientation` at byte 324.
    #[d4(value)]
    pub a_absolute_orientation: f32,
    /// Native `flHitFlashPower` at byte 328.
    #[d4(value)]
    pub hit_flash_power: f32,
    /// Native `flHitFlashMultiplier` at byte 332.
    #[d4(value)]
    pub hit_flash_multiplier: f32,
    /// Native `flOverlayPower` at byte 336.
    #[d4(value)]
    pub overlay_power: f32,
    /// Native `eTopology` at byte 340.
    #[d4(value)]
    pub topology: i32,
    /// Native `ptMsgTriggeredEvents` at byte 344.
    #[d4(array)]
    pub msg_triggered_events: Vec<MsgTriggeredEvent<'data>>,
    /// Native `ptPhysData` at byte 360.
    #[d4(array)]
    pub phys_data: Vec<ActorPhysicsData>,
    /// Native `ptCollData` at byte 376.
    #[d4(array)]
    pub coll_data: Vec<ActorCollisionData>,
    /// Native `eActorGizmoType` at byte 392.
    #[d4(value)]
    pub actor_gizmo_type: i32,
    /// Native `unk_8fbba9b` at byte 396.
    #[d4(value)]
    pub unk_8fbba9b: i32,
    /// Native `ptGizmoData` at byte 400.
    #[d4(polymorphic = 1984123993)]
    pub gizmo_data: Vec<Polymorphic<'data>>,
    /// Native `ptMonsterData` at byte 424.
    #[d4(array)]
    pub monster_data: Vec<ActorMonsterData<'data>>,
    /// Native `ptCritterData` at byte 440.
    #[d4(array)]
    pub critter_data: Vec<ActorCritterData>,
    /// Native `ptPlayerData` at byte 456.
    #[d4(array)]
    pub player_data: Vec<ActorPlayerData<'data>>,
    /// Native `ptItemData` at byte 472.
    #[d4(array)]
    pub item_data: Vec<ActorItemData>,
    /// Native `ptCombatData` at byte 488.
    #[d4(array)]
    pub combat_data: Vec<ActorCombatData>,
    /// Native `ptDeathData` at byte 504.
    #[d4(array)]
    pub death_data: Vec<ActorDeathData>,
    /// Native `ptEffectData` at byte 520.
    #[d4(array)]
    pub effect_data: Vec<ActorEffectData>,
    /// Native `ptProjData` at byte 536.
    #[d4(array)]
    pub proj_data: Vec<ActorProjectileData>,
    /// Native `ptPropData` at byte 552.
    #[d4(array)]
    pub prop_data: Vec<ActorPropData>,
    /// Native `ptMovingData` at byte 568.
    #[d4(array)]
    pub moving_data: Vec<ActorMovingData>,
    /// Native `ptBrainData` at byte 584.
    #[d4(array)]
    pub brain_data: Vec<ActorBrainData>,
    /// Native `ptServerData` at byte 600.
    #[d4(array)]
    pub server_data: Vec<ActorServerData>,
    /// Native `ptUIData` at byte 616.
    #[d4(array)]
    pub ui_data: Vec<ActorUIData<'data>>,
    /// Native `ptAIData` at byte 632.
    #[d4(array)]
    pub ai_data: Vec<ActorAIData>,
    /// Native `ptVOData` at byte 648.
    #[d4(array)]
    pub vo_data: Vec<ActorVOData>,
    /// Native `ptNPCData` at byte 664.
    #[d4(array)]
    pub npc_data: Vec<ActorNPCData>,
    /// Native `ptSoundTableData` at byte 680.
    #[d4(array)]
    pub sound_table_data: Vec<ActorSoundTableData>,
    /// Native `ptMountData` at byte 696.
    #[d4(array)]
    pub mount_data: Vec<ActorMountData>,
    /// Native `ptPostprocessed` at byte 712.
    #[d4(value)]
    pub postprocessed: u64,
    /// Native `snoPrefabAttachment` at byte 720.
    #[d4(value)]
    pub prefab_attachment: i32,
    /// Native `unk_d2b5e35` at byte 724.
    #[d4(value)]
    pub unk_d2b5e35: f32,
    /// Native `snoActorService` at byte 728.
    #[d4(value)]
    pub actor_service: i32,
    /// Native `szPrefabLookOverride` at byte 732.
    #[d4(value)]
    pub prefab_look_override: u32,
    /// Native `dwPrefabShaderMapOverride` at byte 736.
    #[d4(value)]
    pub prefab_shader_map_override: u32,
    /// Unclassified native bytes at offset 740.
    #[d4(value)]
    pub reserved_740: [u8; 4],
    /// Native `arRTPCOverrides` at byte 744.
    #[d4(array)]
    pub rtpc_overrides: Vec<Type4174a154>,
    /// Native `gbidForcedPreloadList` at byte 760.
    #[d4(value)]
    pub forced_preload_list: i32,
    /// Unclassified native bytes at offset 764.
    #[d4(value)]
    pub reserved_764: [u8; 4],
}

/// Native `ActorUIData` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorUIData<'data> {
    /// Native `hPortraitImage` at byte 0.
    #[d4(value)]
    pub h_portrait_image: u32,
    /// Native `unk_e3dc66d` at byte 4.
    #[d4(value)]
    pub unk_e3dc66d: i32,
    /// Native `unk_a94748a` at byte 8.
    #[d4(value)]
    pub unk_a94748a: u32,
    /// Native `unk_a0b181c` at byte 12.
    #[d4(value)]
    pub unk_a0b181c: u32,
    /// Native `unk_c122f9c` at byte 16.
    #[d4(value)]
    pub unk_c122f9c: i32,
    /// Native `snoReputation` at byte 20.
    #[d4(value)]
    pub reputation: i32,
    /// Native `unk_124af06` at byte 24.
    #[d4(value)]
    pub unk_124af06: i32,
    /// Native `unk_8527f49` at byte 28.
    #[d4(value)]
    pub unk_8527f49: i32,
    /// Native `hMinimapIcon` at byte 32.
    #[d4(value)]
    pub h_minimap_icon: u32,
    /// Native `hMinimapIconPromptInteract` at byte 36.
    #[d4(value)]
    pub h_minimap_icon_prompt_interact: u32,
    /// Native `hMinimapIconGizmoUsed` at byte 40.
    #[d4(value)]
    pub h_minimap_icon_gizmo_used: u32,
    /// Native `fMinimapActiveByDefault` at byte 44.
    #[d4(value)]
    pub minimap_active_by_default: i32,
    /// Native `fMinimapRespectsFoW` at byte 48.
    #[d4(value)]
    pub minimap_respects_fo_w: i32,
    /// Native `fMinimapRespectsRevealRadius` at byte 52.
    #[d4(value)]
    pub minimap_respects_reveal_radius: i32,
    /// Native `unk_c26b08b` at byte 56.
    #[d4(value)]
    pub unk_c26b08b: i32,
    /// Native `fMinimapTooltipUsesShortenedName` at byte 60.
    #[d4(value)]
    pub minimap_tooltip_uses_shortened_name: i32,
    /// Native `unk_fab46e5` at byte 64.
    #[d4(value)]
    pub unk_fab46e5: f32,
    /// Native `flMinimapMaxVisRange` at byte 68.
    #[d4(value)]
    pub minimap_max_vis_range: f32,
    /// Native `flMinimapDiscoveryRadius` at byte 72.
    #[d4(value)]
    pub minimap_discovery_radius: f32,
    /// Native `tMinimapMaxVisRangeAttribute` at byte 76.
    pub minimap_max_vis_range_attribute: AttributeSelection,
    /// Unclassified native bytes at offset 84.
    #[d4(value)]
    pub reserved_84: [u8; 4],
    /// Native `unk_c6ec070` at byte 88.
    pub unk_c6ec070: ConditionWrapper<'data>,
    /// Native `unk_f190641` at byte 120.
    #[d4(value)]
    pub unk_f190641: i32,
    /// Native `unk_557e468` at byte 124.
    #[d4(value)]
    pub unk_557e468: i32,
    /// Native `unk_e1eb9fc` at byte 128.
    #[d4(value)]
    pub unk_e1eb9fc: i32,
    /// Native `unk_d2a4d05` at byte 132.
    #[d4(value)]
    pub unk_d2a4d05: i32,
    /// Native `unk_88f4495` at byte 136.
    #[d4(value)]
    pub unk_88f4495: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
}

/// Native `ActorVOData` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorVOData {
    /// Native `fVOPlaysOnPlayer` at byte 0.
    #[d4(value)]
    pub vo_plays_on_player: i32,
    /// Native `snoSpeaker` at byte 4.
    #[d4(value)]
    pub speaker: i32,
    /// Native `snoConversationList` at byte 8.
    #[d4(value)]
    pub conversation_list: i32,
    /// Native `unk_94efee3` at byte 12.
    #[d4(value)]
    pub unk_94efee3: [i32; 2],
}

/// Native `ActorVisualInventoryData` (132 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorVisualInventoryData {
    /// Native `fEnableOverride` at byte 0.
    #[d4(value)]
    pub enable_override: i32,
    /// Native `snoActor` at byte 4.
    #[d4(value)]
    pub actor: [i32; 16],
    /// Native `snoActor2` at byte 68.
    #[d4(value)]
    pub actor2: [i32; 16],
}

/// Native `ActorNPCData` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorNPCData {
    /// Native `unk_370e8c2` at byte 0.
    #[d4(value)]
    pub unk_370e8c2: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `tGeneratedLooks` at byte 8.
    #[d4(array)]
    pub generated_looks: Vec<NPCComponentGroup>,
    /// Native `snoNPCComponentSet` at byte 24.
    #[d4(value)]
    pub npc_component_set: i32,
    /// Unclassified native bytes at offset 28.
    #[d4(value)]
    pub reserved_28: [u8; 4],
    /// Native `ptLoadOuts` at byte 32.
    #[d4(array)]
    pub load_outs: Vec<NPCItems>,
    /// Native `bOverrideObjectNpcLook` at byte 48.
    #[d4(value)]
    pub override_object_npc_look: i32,
    /// Native `eSheathPreference` at byte 52.
    #[d4(value)]
    pub sheath_preference: i32,
}

/// Native `ActorJitterJoint` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorJitterJoint {
    /// Native `szBoneName` at byte 0.
    #[d4(value)]
    pub bone_name: u32,
    /// Native `flJitterStrength` at byte 4.
    #[d4(value)]
    pub jitter_strength: f32,
    /// Native `flJitterSpeed` at byte 8.
    #[d4(value)]
    pub jitter_speed: f32,
    /// Native `flBlendOutTime` at byte 12.
    #[d4(value)]
    pub blend_out_time: f32,
    /// Native `flDuration` at byte 16.
    #[d4(value)]
    pub duration: f32,
}

/// Native `ActorServerData` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorServerData {
    /// Native `snoScript` at byte 0.
    #[d4(value)]
    pub script: i32,
    /// Native `fCannotBeTargetedByAI` at byte 4.
    #[d4(value)]
    pub cannot_be_targeted_by_ai: i32,
    /// Native `unk_195410b` at byte 8.
    #[d4(value)]
    pub unk_195410b: i32,
    /// Native `fNotConsideredInMarkersetBounds` at byte 12.
    #[d4(value)]
    pub not_considered_in_markerset_bounds: i32,
    /// Native `fInvulnerable` at byte 16.
    #[d4(value)]
    pub invulnerable: i32,
    /// Native `fDisableDamage` at byte 20.
    #[d4(value)]
    pub disable_damage: i32,
    /// Native `fCannotDie` at byte 24.
    #[d4(value)]
    pub cannot_die: i32,
    /// Native `unk_9ad1b04` at byte 28.
    #[d4(value)]
    pub unk_9ad1b04: i32,
    /// Native `flNoSpawnRadius` at byte 32.
    #[d4(value)]
    pub no_spawn_radius: f32,
    /// Native `snoPhasingVisibilityCondition` at byte 36.
    #[d4(value)]
    pub phasing_visibility_condition: i32,
}

/// Native `ConditionWrapper` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ConditionWrapper<'data> {
    /// Native `snoCondition` at byte 0.
    #[d4(value)]
    pub condition: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `ptInlineCondition` at byte 8.
    #[d4(polymorphic = 3748928721)]
    pub inline_condition: Vec<Polymorphic<'data>>,
}

/// Native `Subcondition` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Subcondition {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `bNegate` at byte 16.
    #[d4(value)]
    pub negate: i32,
    /// Native `nMaxDepth` at byte 20.
    #[d4(value)]
    pub max_depth: u32,
}

/// Native `ActorChildRotationJoint` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorChildRotationJoint {
    /// Native `szBoneName` at byte 0.
    #[d4(value)]
    pub bone_name: u32,
    /// Native `flAdditionalRotation` at byte 4.
    #[d4(value)]
    pub additional_rotation: f32,
}

/// Native `ActorEffectData` (132 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorEffectData {
    /// Native `nHardpointIndexForRope` at byte 0.
    #[d4(value)]
    pub hardpoint_index_for_rope: i32,
    /// Native `fDoesntCauseCriticalBloodEffect` at byte 4.
    #[d4(value)]
    pub doesnt_cause_critical_blood_effect: i32,
    /// Native `fNeverPlayGethitJitter` at byte 8.
    #[d4(value)]
    pub never_play_gethit_jitter: i32,
    /// Native `arNormalHitEffectGroups` at byte 12.
    #[d4(value)]
    pub normal_hit_effect_groups: [i32; 7],
    /// Native `arDirectionalHitEffectGroups` at byte 40.
    #[d4(value)]
    pub directional_hit_effect_groups: [i32; 7],
    /// Native `arDOTEffectGroups` at byte 68.
    #[d4(value)]
    pub dot_effect_groups: [i32; 7],
    /// Native `arDOTApplicationEffectGroups` at byte 96.
    #[d4(value)]
    pub dot_application_effect_groups: [i32; 7],
    /// Native `snoMonsterRarityEffectGroup` at byte 124.
    #[d4(value)]
    pub monster_rarity_effect_group: i32,
    /// Native `snoShieldBlockEffectGroup` at byte 128.
    #[d4(value)]
    pub shield_block_effect_group: i32,
}

/// Native `ActorBrainData` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorBrainData {
    /// Native `eDefaultBrain` at byte 0.
    #[d4(value)]
    pub default_brain: i32,
    /// Native `eBehaviorType` at byte 4.
    #[d4(value)]
    pub behavior_type: i32,
    /// Native `flRadiusTriggerRadius` at byte 8.
    #[d4(value)]
    pub radius_trigger_radius: f32,
    /// Native `tRadiusTriggerIntervalRange` at byte 12.
    #[d4(value)]
    pub radius_trigger_interval_range: [f32; 2],
    /// Native `fRadiusTriggerOneShot` at byte 20.
    #[d4(value)]
    pub radius_trigger_one_shot: i32,
    /// Native `tAIDisperseData` at byte 24.
    pub ai_disperse_data: ActorAIDisperseData,
    /// Native `snoRetreatOverride` at byte 40.
    #[d4(value)]
    pub retreat_override: i32,
    /// Unclassified native bytes at offset 44.
    #[d4(value)]
    pub reserved_44: [u8; 4],
}

/// Native `ActorDeathData` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ActorDeathData {
    /// Native `arDeathEffectGroups` at byte 0.
    #[d4(value)]
    pub death_effect_groups: [i32; 7],
    /// Native `gbidActorResurrectDataOverride` at byte 28.
    #[d4(value)]
    pub actor_resurrect_data_override: i32,
}

/// Native `LockedRotationBoneData` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LockedRotationBoneData {
    /// Native `arBones` at byte 0.
    #[d4(array)]
    pub bones: Vec<AppBone>,
}
