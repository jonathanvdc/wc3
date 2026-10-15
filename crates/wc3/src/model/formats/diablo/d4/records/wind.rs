//! Native wind records.
use super::*;
/// Native `SpeedTreeWind9Params` (1640 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWind9Params {
    /// Native `fStrengthResponse` at byte 0.
    #[d4(value)]
    pub strength_response: f32,
    /// Native `fDirectionResponse` at byte 4.
    #[d4(value)]
    pub direction_response: f32,
    /// Native `fGustFrequency` at byte 8.
    #[d4(value)]
    pub gust_frequency: f32,
    /// Native `fGustStrengthMin` at byte 12.
    #[d4(value)]
    pub gust_strength_min: f32,
    /// Native `fGustStrengthMax` at byte 16.
    #[d4(value)]
    pub gust_strength_max: f32,
    /// Native `fGustDurationMin` at byte 20.
    #[d4(value)]
    pub gust_duration_min: f32,
    /// Native `fGustDurationMax` at byte 24.
    #[d4(value)]
    pub gust_duration_max: f32,
    /// Native `fGustRiseScalar` at byte 28.
    #[d4(value)]
    pub gust_rise_scalar: f32,
    /// Native `fGustFallScalar` at byte 32.
    #[d4(value)]
    pub gust_fall_scalar: f32,
    /// Native `vBoundingBoxMin` at byte 36.
    #[d4(value)]
    pub bounding_box_min: [f32; 3],
    /// Native `vBoundingBoxMax` at byte 48.
    #[d4(value)]
    pub bounding_box_max: [f32; 3],
    /// Native `unk_a966ba1` at byte 60.
    #[d4(value)]
    pub unk_a966ba1: f32,
    /// Native `unk_9f57522` at byte 64.
    #[d4(value)]
    pub unk_9f57522: f32,
    /// Native `sShared` at byte 68.
    pub s_shared: SpeedTree9BranchWindLevel,
    /// Native `sBranch1` at byte 472.
    pub s_branch1: SpeedTree9BranchWindLevel,
    /// Native `sBranch2` at byte 876.
    pub s_branch2: SpeedTree9BranchWindLevel,
    /// Native `sRipple` at byte 1280.
    pub s_ripple: SpeedTree9RippleGroup,
    /// Native `unk_96a49a4` at byte 1608.
    #[d4(value)]
    pub unk_96a49a4: f32,
    /// Native `bDoShared` at byte 1612.
    #[d4(value)]
    pub do_shared: i32,
    /// Native `bDoBranch1` at byte 1616.
    #[d4(value)]
    pub do_branch1: i32,
    /// Native `bDoBranch2` at byte 1620.
    #[d4(value)]
    pub do_branch2: i32,
    /// Native `bDoRipple` at byte 1624.
    #[d4(value)]
    pub do_ripple: i32,
    /// Native `bDoShimmer` at byte 1628.
    #[d4(value)]
    pub do_shimmer: i32,
    /// Native `unk_5a3c411` at byte 1632.
    #[d4(value)]
    pub unk_5a3c411: i32,
    /// Native `fWindIndependence` at byte 1636.
    #[d4(value)]
    pub wind_independence: f32,
}

/// Native `SpeedTree9RippleGroup` (328 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTree9RippleGroup {
    /// Native `afPlanar` at byte 0.
    #[d4(value)]
    pub af_planar: [f32; 20],
    /// Native `afDirectional` at byte 80.
    #[d4(value)]
    pub af_directional: [f32; 20],
    /// Native `afSpeed` at byte 160.
    #[d4(value)]
    pub af_speed: [f32; 20],
    /// Native `afFlexibility` at byte 240.
    #[d4(value)]
    pub af_flexibility: [f32; 20],
    /// Native `fIndependence` at byte 320.
    #[d4(value)]
    pub independence: f32,
    /// Native `fShimmer` at byte 324.
    #[d4(value)]
    pub shimmer: f32,
}

/// Native `SpeedTreeWind8` (1000 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWind8 {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tParams` at byte 16.
    pub params: SpeedTreeWind8Params,
}

/// Native `SpeedTreeWind9` (1656 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWind9 {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tParams` at byte 16.
    pub params: SpeedTreeWind9Params,
}

/// Native `SpeedTreeWindParams` (1360 fixed bytes).
#[derive(Clone, Debug, PartialEq, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWindParams {
    /// Native `fStrengthResponse` at byte 0.
    #[d4(value)]
    pub strength_response: f32,
    /// Native `fDirectionResponse` at byte 4.
    #[d4(value)]
    pub direction_response: f32,
    /// Native `fAnchorOffset` at byte 8.
    #[d4(value)]
    pub anchor_offset: f32,
    /// Native `fAnchorDistanceScale` at byte 12.
    #[d4(value)]
    pub anchor_distance_scale: f32,
    /// Native `afFrequencies` at byte 16.
    #[d4(value)]
    pub af_frequencies: [f32; 100],
    /// Native `fGlobalHeight` at byte 416.
    #[d4(value)]
    pub global_height: f32,
    /// Native `fGlobalHeightExponent` at byte 420.
    #[d4(value)]
    pub global_height_exponent: f32,
    /// Native `afGlobalDistance` at byte 424.
    #[d4(value)]
    pub af_global_distance: [f32; 10],
    /// Native `afGlobalDirectionAdherence` at byte 464.
    #[d4(value)]
    pub af_global_direction_adherence: [f32; 10],
    /// Native `atBranch` at byte 504.
    #[d4(inline_array)]
    pub at_branch: [SpeedTreeBranchWindLevel; 2],
    /// Native `atLeaf` at byte 768.
    #[d4(inline_array)]
    pub at_leaf: [SpeedTreeWindGroup; 2],
    /// Native `afFrondRippleDistance` at byte 1216.
    #[d4(value)]
    pub af_frond_ripple_distance: [f32; 10],
    /// Native `fFrondRippleTile` at byte 1256.
    #[d4(value)]
    pub frond_ripple_tile: f32,
    /// Native `fFrondRippleLightingScalar` at byte 1260.
    #[d4(value)]
    pub frond_ripple_lighting_scalar: f32,
    /// Native `fRollingNoiseSize` at byte 1264.
    #[d4(value)]
    pub rolling_noise_size: f32,
    /// Native `fRollingNoiseTwist` at byte 1268.
    #[d4(value)]
    pub rolling_noise_twist: f32,
    /// Native `fRollingNoiseTurbulence` at byte 1272.
    #[d4(value)]
    pub rolling_noise_turbulence: f32,
    /// Native `fRollingNoisePeriod` at byte 1276.
    #[d4(value)]
    pub rolling_noise_period: f32,
    /// Native `fRollingNoiseSpeed` at byte 1280.
    #[d4(value)]
    pub rolling_noise_speed: f32,
    /// Native `fRollingBranchRipple` at byte 1284.
    #[d4(value)]
    pub rolling_branch_ripple: f32,
    /// Native `fRollingBranchFieldMin` at byte 1288.
    #[d4(value)]
    pub rolling_branch_field_min: f32,
    /// Native `fRollingBranchLightingAdjust` at byte 1292.
    #[d4(value)]
    pub rolling_branch_lighting_adjust: f32,
    /// Native `fRollingBranchVerticalOffset` at byte 1296.
    #[d4(value)]
    pub rolling_branch_vertical_offset: f32,
    /// Native `unk_b06b705` at byte 1300.
    #[d4(value)]
    pub unk_b06b705: f32,
    /// Native `unk_f7ea482` at byte 1304.
    #[d4(value)]
    pub unk_f7ea482: f32,
    /// Native `fGustFrequency` at byte 1308.
    #[d4(value)]
    pub gust_frequency: f32,
    /// Native `fGustStrengthMin` at byte 1312.
    #[d4(value)]
    pub gust_strength_min: f32,
    /// Native `fGustStrengthMax` at byte 1316.
    #[d4(value)]
    pub gust_strength_max: f32,
    /// Native `fGustDurationMin` at byte 1320.
    #[d4(value)]
    pub gust_duration_min: f32,
    /// Native `fGustDurationMax` at byte 1324.
    #[d4(value)]
    pub gust_duration_max: f32,
    /// Native `fGustRiseScalar` at byte 1328.
    #[d4(value)]
    pub gust_rise_scalar: f32,
    /// Native `fGustFallScalar` at byte 1332.
    #[d4(value)]
    pub gust_fall_scalar: f32,
    /// Native `unk_5c5413` at byte 1336.
    #[d4(value)]
    pub unk_5c5413: i32,
    /// Native `unk_5c5414` at byte 1340.
    #[d4(value)]
    pub unk_5c5414: i32,
    /// Native `vBranchWindAnchor` at byte 1344.
    #[d4(value)]
    pub branch_wind_anchor: [f32; 3],
    /// Native `unk_2e7a7c5` at byte 1356.
    #[d4(value)]
    pub unk_2e7a7c5: f32,
}
impl Default for SpeedTreeWindParams {
    fn default() -> Self {
        Self {
            strength_response: Default::default(),
            direction_response: Default::default(),
            anchor_offset: Default::default(),
            anchor_distance_scale: Default::default(),
            af_frequencies: core::array::from_fn(|_| Default::default()),
            global_height: Default::default(),
            global_height_exponent: Default::default(),
            af_global_distance: core::array::from_fn(|_| Default::default()),
            af_global_direction_adherence: core::array::from_fn(|_| Default::default()),
            at_branch: core::array::from_fn(|_| Default::default()),
            at_leaf: core::array::from_fn(|_| Default::default()),
            af_frond_ripple_distance: core::array::from_fn(|_| Default::default()),
            frond_ripple_tile: Default::default(),
            frond_ripple_lighting_scalar: Default::default(),
            rolling_noise_size: Default::default(),
            rolling_noise_twist: Default::default(),
            rolling_noise_turbulence: Default::default(),
            rolling_noise_period: Default::default(),
            rolling_noise_speed: Default::default(),
            rolling_branch_ripple: Default::default(),
            rolling_branch_field_min: Default::default(),
            rolling_branch_lighting_adjust: Default::default(),
            rolling_branch_vertical_offset: Default::default(),
            unk_b06b705: Default::default(),
            unk_f7ea482: Default::default(),
            gust_frequency: Default::default(),
            gust_strength_min: Default::default(),
            gust_strength_max: Default::default(),
            gust_duration_min: Default::default(),
            gust_duration_max: Default::default(),
            gust_rise_scalar: Default::default(),
            gust_fall_scalar: Default::default(),
            unk_5c5413: Default::default(),
            unk_5c5414: Default::default(),
            branch_wind_anchor: core::array::from_fn(|_| Default::default()),
            unk_2e7a7c5: Default::default(),
        }
    }
}

/// Native `SpeedTreeWindGroup` (224 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWindGroup {
    /// Native `afRippleDistance` at byte 0.
    #[d4(value)]
    pub af_ripple_distance: [f32; 10],
    /// Native `afTumbleFlip` at byte 40.
    #[d4(value)]
    pub af_tumble_flip: [f32; 10],
    /// Native `afTumbleTwist` at byte 80.
    #[d4(value)]
    pub af_tumble_twist: [f32; 10],
    /// Native `afTumbleDirectionAdherence` at byte 120.
    #[d4(value)]
    pub af_tumble_direction_adherence: [f32; 10],
    /// Native `afTwitchThrow` at byte 160.
    #[d4(value)]
    pub af_twitch_throw: [f32; 10],
    /// Native `fTwitchSharpness` at byte 200.
    #[d4(value)]
    pub twitch_sharpness: f32,
    /// Native `fRollMaxScale` at byte 204.
    #[d4(value)]
    pub roll_max_scale: f32,
    /// Native `fRollMinScale` at byte 208.
    #[d4(value)]
    pub roll_min_scale: f32,
    /// Native `fRollSpeed` at byte 212.
    #[d4(value)]
    pub roll_speed: f32,
    /// Native `fRollSeparation` at byte 216.
    #[d4(value)]
    pub roll_separation: f32,
    /// Native `fLeewardScalar` at byte 220.
    #[d4(value)]
    pub leeward_scalar: f32,
}

/// Native `SpeedTreeWind` (1376 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWind {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tParams` at byte 16.
    pub params: SpeedTreeWindParams,
}

/// Native `SpeedTreeWindBase` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWindBase {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
}

/// Native `SpeedTreeWind8Params` (984 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeWind8Params {
    /// Native `fStrengthResponse` at byte 0.
    #[d4(value)]
    pub strength_response: f32,
    /// Native `fDirectionResponse` at byte 4.
    #[d4(value)]
    pub direction_response: f32,
    /// Native `fGustFrequency` at byte 8.
    #[d4(value)]
    pub gust_frequency: f32,
    /// Native `fGustStrengthMin` at byte 12.
    #[d4(value)]
    pub gust_strength_min: f32,
    /// Native `fGustStrengthMax` at byte 16.
    #[d4(value)]
    pub gust_strength_max: f32,
    /// Native `fGustDurationMin` at byte 20.
    #[d4(value)]
    pub gust_duration_min: f32,
    /// Native `fGustDurationMax` at byte 24.
    #[d4(value)]
    pub gust_duration_max: f32,
    /// Native `fGustRiseScalar` at byte 28.
    #[d4(value)]
    pub gust_rise_scalar: f32,
    /// Native `fGustFallScalar` at byte 32.
    #[d4(value)]
    pub gust_fall_scalar: f32,
    /// Native `fGlobalHeight` at byte 36.
    #[d4(value)]
    pub global_height: f32,
    /// Native `fFullBranchMove` at byte 40.
    #[d4(value)]
    pub full_branch_move: f32,
    /// Native `fFullRipple` at byte 44.
    #[d4(value)]
    pub full_ripple: f32,
    /// Native `vBoundingBoxMin` at byte 48.
    #[d4(value)]
    pub bounding_box_min: [f32; 3],
    /// Native `vBoundingBoxMax` at byte 60.
    #[d4(value)]
    pub bounding_box_max: [f32; 3],
    /// Native `fRefreshSecondary` at byte 72.
    #[d4(value)]
    pub refresh_secondary: f32,
    /// Native `unk_fbebc6e` at byte 76.
    #[d4(value)]
    pub unk_fbebc6e: f32,
    /// Native `fBranchStretch` at byte 80.
    #[d4(value)]
    pub branch_stretch: f32,
    /// Native `fShimmer` at byte 84.
    #[d4(value)]
    pub shimmer: f32,
    /// Native `sShared` at byte 88.
    pub s_shared: SpeedTree8BranchWindLevel,
    /// Native `sBranch` at byte 412.
    pub s_branch: SpeedTree8BranchWindLevel,
    /// Native `sRipple` at byte 736.
    pub s_ripple: SpeedTree8RippleGroup,
    /// Native `fIsGrass` at byte 980.
    #[d4(value)]
    pub is_grass: f32,
}

/// Native `SpeedTreeBranchWindLevel` (132 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTreeBranchWindLevel {
    /// Native `afDistance` at byte 0.
    #[d4(value)]
    pub af_distance: [f32; 10],
    /// Native `afDirectionAdherence` at byte 40.
    #[d4(value)]
    pub af_direction_adherence: [f32; 10],
    /// Native `afWhip` at byte 80.
    #[d4(value)]
    pub af_whip: [f32; 10],
    /// Native `fTurbulence` at byte 120.
    #[d4(value)]
    pub turbulence: f32,
    /// Native `fTwitch` at byte 124.
    #[d4(value)]
    pub twitch: f32,
    /// Native `fTwitchFreqScale` at byte 128.
    #[d4(value)]
    pub twitch_freq_scale: f32,
}

/// Native `SpeedTree8BranchWindLevel` (324 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTree8BranchWindLevel {
    /// Native `afFrequency` at byte 0.
    #[d4(value)]
    pub af_frequency: [f32; 20],
    /// Native `afBend` at byte 80.
    #[d4(value)]
    pub af_bend: [f32; 20],
    /// Native `afTurbulence` at byte 160.
    #[d4(value)]
    pub af_turbulence: [f32; 20],
    /// Native `afFlexibility` at byte 240.
    #[d4(value)]
    pub af_flexibility: [f32; 20],
    /// Native `fTurbulenceIndependence` at byte 320.
    #[d4(value)]
    pub turbulence_independence: f32,
}

/// Native `SpeedTree9BranchWindLevel` (404 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTree9BranchWindLevel {
    /// Native `afBend` at byte 0.
    #[d4(value)]
    pub af_bend: [f32; 20],
    /// Native `afOscillation` at byte 80.
    #[d4(value)]
    pub af_oscillation: [f32; 20],
    /// Native `afSpeed` at byte 160.
    #[d4(value)]
    pub af_speed: [f32; 20],
    /// Native `afTurbulence` at byte 240.
    #[d4(value)]
    pub af_turbulence: [f32; 20],
    /// Native `afFlexibility` at byte 320.
    #[d4(value)]
    pub af_flexibility: [f32; 20],
    /// Native `fIndependence` at byte 400.
    #[d4(value)]
    pub independence: f32,
}

/// Native `SpeedTree8RippleGroup` (244 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SpeedTree8RippleGroup {
    /// Native `afBend` at byte 0.
    #[d4(value)]
    pub af_bend: [f32; 20],
    /// Native `afTurbulenceFrequency` at byte 80.
    #[d4(value)]
    pub af_turbulence_frequency: [f32; 20],
    /// Native `afTurbulence` at byte 160.
    #[d4(value)]
    pub af_turbulence: [f32; 20],
    /// Native `fTurbulenceIndependence` at byte 240.
    #[d4(value)]
    pub turbulence_independence: f32,
}
