//! Native graph selectors and bit masks.
use crate::model::formats::m3;
use bitfield::bitfield;

bitfield! {
    /// Native ModelFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct ModelFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the e mdf tangents control.
    pub e_mdf_tangents, set_e_mdf_tangents: 0;
    /// Returns or changes the e mdf bones fixed control.
    pub e_mdf_bones_fixed, set_e_mdf_bones_fixed: 1;
    /// Returns or changes the e mdf uvdensities computed control.
    pub e_mdf_uvdensities_computed, set_e_mdf_uvdensities_computed: 2;
    /// Returns or changes the e mdf relative bounds control.
    pub e_mdf_relative_bounds, set_e_mdf_relative_bounds: 3;
    /// Returns or changes the e mdf section bounds fixed control.
    pub e_mdf_section_bounds_fixed, set_e_mdf_section_bounds_fixed: 4;
    /// Returns or changes the e mdf track sets computed control.
    pub e_mdf_track_sets_computed, set_e_mdf_track_sets_computed: 5;
    /// Returns or changes the e mdf track collection sorted control.
    pub e_mdf_track_collection_sorted, set_e_mdf_track_collection_sorted: 6;
    /// Returns or changes the e md model accepts splats control.
    pub e_md_model_accepts_splats, set_e_md_model_accepts_splats: 7;
    /// Returns or changes the e md track animated base flag valid control.
    pub e_md_track_animated_base_flag_valid, set_e_md_track_animated_base_flag_valid: 11;
    /// Returns or changes the e md file dirty control.
    pub e_md_file_dirty, set_e_md_file_dirty: 12;
    /// Returns or changes the e md fowdo not use tint control.
    pub e_md_fowdo_not_use_tint, set_e_md_fowdo_not_use_tint: 14;
    /// Returns or changes the e md instanced vb control.
    pub e_md_instanced_vb, set_e_md_instanced_vb: 15;
    /// Returns or changes the e md force sampled fow control.
    pub e_md_force_sampled_fow, set_e_md_force_sampled_fow: 16;
    /// Returns or changes the e md instanced model control.
    pub e_md_instanced_model, set_e_md_instanced_model: 17;
    /// Returns or changes the e md never use fow control.
    pub e_md_never_use_fow, set_e_md_never_use_fow: 18;
    /// Returns or changes the e md bone animated flag solved control.
    pub e_md_bone_animated_flag_solved, set_e_md_bone_animated_flag_solved: 19;
    /// Returns or changes the e md allow local light shadows control.
    pub e_md_allow_local_light_shadows, set_e_md_allow_local_light_shadows: 20;
    /// Returns or changes the e md avoid sampled fow control.
    pub e_md_avoid_sampled_fow, set_e_md_avoid_sampled_fow: 21;
}
impl From<u32> for ModelFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ModelFlags> for u32 {
    fn from(value: ModelFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native ReferenceFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct ReferenceFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
}
impl From<u32> for ReferenceFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ReferenceFlags> for u32 {
    fn from(value: ReferenceFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native FlagBits mask, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct FlagBits(u32);
    /// Complete native bit mask.
    pub bits, _: 31, 0;
}
impl From<u32> for FlagBits {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<FlagBits> for u32 {
    fn from(value: FlagBits) -> Self {
        value.bits()
    }
}
