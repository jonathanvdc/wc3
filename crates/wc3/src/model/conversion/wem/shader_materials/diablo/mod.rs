//! Diablo shader inputs retain external dependencies and independent native state.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::diablo::d3::{Group, Material};
use crate::model::semantic::profiles::wem::materials::diablo::{DiabloMaterial, VariantFlags};
use crate::model::wem::records::D3MaterialDialect;
use crate::model::wem::records::{D3Group, D3Material, D3RenderState, D3TagValue, D3VariantFlags};

use crate::model::wem::Ref;
use crate::model::wem::{records, InlineList, RawDocument};
mod resources;
use resources::{
    read_asset_ref, read_tag_value, read_uber_material, write_asset_ref, write_tag_value,
    write_uber_material,
};
mod passes;
use passes::{read_render_state, write_render_state};
fn read_group(value: D3Group) -> Group {
    match value {
        D3Group::Unspecified => Group::Unspecified,
        D3Group::Actor => Group::Actor,
        D3Group::Adventure => Group::Adventure,
        D3Group::AIBehavior => Group::AIBehavior,
        D3Group::AIState => Group::AIState,
        D3Group::AmbientSound => Group::AmbientSound,
        D3Group::Anim => Group::Anim,
        D3Group::Anim2D => Group::Anim2D,
        D3Group::AnimSet => Group::AnimSet,
        D3Group::Appearance => Group::Appearance,
        D3Group::Hero => Group::Hero,
        D3Group::Cloth => Group::Cloth,
        D3Group::Conversation => Group::Conversation,
        D3Group::ConversationList => Group::ConversationList,
        D3Group::EffectGroup => Group::EffectGroup,
        D3Group::Encounter => Group::Encounter,
        D3Group::Explosion => Group::Explosion,
        D3Group::FlagSet => Group::FlagSet,
        D3Group::Font => Group::Font,
        D3Group::GameBalance => Group::GameBalance,
        D3Group::Globals => Group::Globals,
        D3Group::LevelArea => Group::LevelArea,
        D3Group::Light => Group::Light,
        D3Group::MarkerSet => Group::MarkerSet,
        D3Group::Monster => Group::Monster,
        D3Group::Observer => Group::Observer,
        D3Group::Particle => Group::Particle,
        D3Group::Physics => Group::Physics,
        D3Group::Power => Group::Power,
        D3Group::Quest => Group::Quest,
        D3Group::Rope => Group::Rope,
        D3Group::Scene => Group::Scene,
        D3Group::SceneGroup => Group::SceneGroup,
        D3Group::Script => Group::Script,
        D3Group::ShaderMap => Group::ShaderMap,
        D3Group::Shaders => Group::Shaders,
        D3Group::Shakes => Group::Shakes,
        D3Group::SkillKit => Group::SkillKit,
        D3Group::Sound => Group::Sound,
        D3Group::SoundBank => Group::SoundBank,
        D3Group::StringList => Group::StringList,
        D3Group::Surface => Group::Surface,
        D3Group::Textures => Group::Textures,
        D3Group::Trail => Group::Trail,
        D3Group::UI => Group::UI,
        D3Group::Weather => Group::Weather,
        D3Group::Worlds => Group::Worlds,
        D3Group::Recipe => Group::Recipe,
        D3Group::Condition => Group::Condition,
        D3Group::TreasureClass => Group::TreasureClass,
        D3Group::Account => Group::Account,
        D3Group::Conductor => Group::Conductor,
        D3Group::TimedEvent => Group::TimedEvent,
        D3Group::Act => Group::Act,
        D3Group::Material => Group::Material,
        D3Group::QuestRange => Group::QuestRange,
        D3Group::Lore => Group::Lore,
        D3Group::Reverb => Group::Reverb,
        D3Group::PhysMesh => Group::PhysMesh,
        D3Group::Music => Group::Music,
        D3Group::Tutorial => Group::Tutorial,
        D3Group::BossEncounter => Group::BossEncounter,
        D3Group::ControlScheme => Group::ControlScheme,
        D3Group::Accolade => Group::Accolade,
        D3Group::AnimTree => Group::AnimTree,
        D3Group::Vibrations => Group::Vibrations,
        D3Group::DungeonFinder => Group::DungeonFinder,
        D3Group::Unknown(code) => Group::Unknown(code),
    }
}
fn write_group(value: Group) -> D3Group {
    match value {
        Group::Unspecified => D3Group::Unspecified,
        Group::Actor => D3Group::Actor,
        Group::Adventure => D3Group::Adventure,
        Group::AIBehavior => D3Group::AIBehavior,
        Group::AIState => D3Group::AIState,
        Group::AmbientSound => D3Group::AmbientSound,
        Group::Anim => D3Group::Anim,
        Group::Anim2D => D3Group::Anim2D,
        Group::AnimSet => D3Group::AnimSet,
        Group::Appearance => D3Group::Appearance,
        Group::Hero => D3Group::Hero,
        Group::Cloth => D3Group::Cloth,
        Group::Conversation => D3Group::Conversation,
        Group::ConversationList => D3Group::ConversationList,
        Group::EffectGroup => D3Group::EffectGroup,
        Group::Encounter => D3Group::Encounter,
        Group::Explosion => D3Group::Explosion,
        Group::FlagSet => D3Group::FlagSet,
        Group::Font => D3Group::Font,
        Group::GameBalance => D3Group::GameBalance,
        Group::Globals => D3Group::Globals,
        Group::LevelArea => D3Group::LevelArea,
        Group::Light => D3Group::Light,
        Group::MarkerSet => D3Group::MarkerSet,
        Group::Monster => D3Group::Monster,
        Group::Observer => D3Group::Observer,
        Group::Particle => D3Group::Particle,
        Group::Physics => D3Group::Physics,
        Group::Power => D3Group::Power,
        Group::Quest => D3Group::Quest,
        Group::Rope => D3Group::Rope,
        Group::Scene => D3Group::Scene,
        Group::SceneGroup => D3Group::SceneGroup,
        Group::Script => D3Group::Script,
        Group::ShaderMap => D3Group::ShaderMap,
        Group::Shaders => D3Group::Shaders,
        Group::Shakes => D3Group::Shakes,
        Group::SkillKit => D3Group::SkillKit,
        Group::Sound => D3Group::Sound,
        Group::SoundBank => D3Group::SoundBank,
        Group::StringList => D3Group::StringList,
        Group::Surface => D3Group::Surface,
        Group::Textures => D3Group::Textures,
        Group::Trail => D3Group::Trail,
        Group::UI => D3Group::UI,
        Group::Weather => D3Group::Weather,
        Group::Worlds => D3Group::Worlds,
        Group::Recipe => D3Group::Recipe,
        Group::Condition => D3Group::Condition,
        Group::TreasureClass => D3Group::TreasureClass,
        Group::Account => D3Group::Account,
        Group::Conductor => D3Group::Conductor,
        Group::TimedEvent => D3Group::TimedEvent,
        Group::Act => D3Group::Act,
        Group::Material => D3Group::Material,
        Group::QuestRange => D3Group::QuestRange,
        Group::Lore => D3Group::Lore,
        Group::Reverb => D3Group::Reverb,
        Group::PhysMesh => D3Group::PhysMesh,
        Group::Music => D3Group::Music,
        Group::Tutorial => D3Group::Tutorial,
        Group::BossEncounter => D3Group::BossEncounter,
        Group::ControlScheme => D3Group::ControlScheme,
        Group::Accolade => D3Group::Accolade,
        Group::AnimTree => D3Group::AnimTree,
        Group::Vibrations => D3Group::Vibrations,
        Group::DungeonFinder => D3Group::DungeonFinder,
        Group::Unknown(code) => D3Group::Unknown(code),
    }
}
pub(in crate::model::conversion::wem) trait VariantField {
    fn options(&self) -> Option<VariantFlags>;
}
impl VariantField for () {
    fn options(&self) -> Option<VariantFlags> {
        None
    }
}
impl VariantField for D3VariantFlags {
    fn options(&self) -> Option<VariantFlags> {
        Some(VariantFlags::from_bits_retain(self.bits()))
    }
}
pub(in crate::model::conversion::wem) trait ListField<T> {
    fn rows(&self) -> &[T];
}
impl<T> ListField<T> for InlineList<T> {
    fn rows(&self) -> &[T] {
        &self.0
    }
}
impl<T> ListField<T> for () {
    fn rows(&self) -> &[T] {
        &[]
    }
}

pub(in crate::model::conversion::wem) fn read<'view, D: D3MaterialDialect>(
    document: &'view RawDocument<'_>,
    value: &D3Material<D>,
) -> Result<DiabloMaterial<'view>, Error>
where
    D::VariantFlags: VariantField,
    D::ShaderParams: ListField<D3TagValue>,
    D::OpaquePasses: ListField<D3RenderState>,
    D::TranslucentPasses: ListField<D3RenderState>,
{
    Ok(DiabloMaterial {
        variant_flags: value.variant_flags.options(),
        shader: Material {
            appearance_control: 0,
            uber: read_uber_material(document, &value.uber)?,
            base_material: read_asset_ref(document, &value.base_material)?,
            cloth: read_asset_ref(document, &value.cloth)?,
            shader_params: value
                .shader_params
                .rows()
                .iter()
                .map(|value| read_tag_value(document, value))
                .collect::<Result<Vec<_>, _>>()?,
            opaque_passes: value
                .opaque_passes
                .rows()
                .iter()
                .map(|value| read_render_state(document, value))
                .collect::<Result<Vec<_>, _>>()?,
            translucent_passes: value
                .translucent_passes
                .rows()
                .iter()
                .map(|value| read_render_state(document, value))
                .collect::<Result<Vec<_>, _>>()?,
        },
    })
}
/// An output schema selected by whether variant options are authored.
pub(in crate::model::conversion::wem) enum Encoded {
    Unflagged(D3Material<records::V1>),
    Flagged(D3Material<records::V2>),
}
impl Encoded {
    pub(in crate::model::conversion::wem) fn append(
        self,
        document: &mut RawDocument<'_>,
    ) -> Result<Ref<records::targets::D3Material>, Error> {
        let id = match self {
            Self::Unflagged(value) => document.append(&[value]),
            Self::Flagged(value) => document.append(&[value]),
        }
        .map_err(|error| Error::new("wem.material.diablo", error.to_string()))?;
        Ok(Ref::from_section(1, id.index()))
    }
}
pub(in crate::model::conversion::wem) fn write(
    document: &mut RawDocument<'_>,
    value: &DiabloMaterial<'_>,
    source_version: u32,
) -> Result<Encoded, Error> {
    let variant_flags = value.variant_flags;
    let value = &value.shader;
    if value.appearance_control != 0 {
        return Err(Error::new(
            "wem.material.diablo.appearance_control",
            "WEM ND3 does not encode the native appearance qualifier",
        ));
    }
    let uber = write_uber_material(document, &value.uber)?;
    let base_material = write_asset_ref(document, &value.base_material)?;
    let cloth = write_asset_ref(document, &value.cloth)?;
    let shader_params = InlineList(
        value
            .shader_params
            .iter()
            .map(|value| write_tag_value(document, value))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let opaque_passes = InlineList(
        value
            .opaque_passes
            .iter()
            .map(|value| write_render_state(document, value))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let translucent_passes = InlineList(
        value
            .translucent_passes
            .iter()
            .map(|value| write_render_state(document, value))
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(match variant_flags {
        None => Encoded::Unflagged(D3Material {
            source_version,
            variant_flags: (),
            uber,
            base_material,
            cloth,
            shader_params,
            opaque_passes,
            translucent_passes,
        }),
        Some(flags) => Encoded::Flagged(D3Material {
            source_version,
            variant_flags: flags.bits().into(),
            uber,
            base_material,
            cloth,
            shader_params,
            opaque_passes,
            translucent_passes,
        }),
    })
}

#[cfg(test)]
mod tests;
