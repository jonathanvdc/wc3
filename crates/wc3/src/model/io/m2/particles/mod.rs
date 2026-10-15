//! Typed native particle fields and particle-age appearance conversion.
use super::*;
mod fields;
mod logical;
use fields::*;
pub(super) use fields::{gravity_vector, gravity_word};

/// Layout-specific particle fields shared by the native emitter adapter.
pub trait ParticleConversion: ModelLayout {
    /// Builds static logical fields in the selected native particle layout.
    fn write_logical_particle(
        value: &crate::model::profiles::m2::Particle<'_>,
        textures: &[crate::model::TextureId],
        context: &mut Context,
    ) -> Result<native::records::ParticleEmitter<'static, Self>, Error>;

    /// Imports independent logical emitter state while borrowing age and path arrays.
    fn logical_particle<'a, 'data: 'a>(
        value: &'a native::records::ParticleEmitter<'data, Self>,
        textures: &[crate::model::TextureId],
    ) -> Result<crate::model::profiles::m2::Particle<'a>, Error>;

    /// Returns physical core flags required by the selected scrolling representation.
    fn required_flags(value: &native::records::ParticleEmitter<'_, Self>) -> ModelFlags;

    /// Reads legacy particle-shape and head/tail control bytes when present.
    fn legacy_texture_scale(value: &native::records::ParticleEmitter<'_, Self>) -> Option<[u8; 2]>;
    /// Restores editable legacy controls, rejecting nondefault controls in modern layouts.
    fn restore_legacy_texture_scale(
        value: &mut native::records::ParticleEmitter<'_, Self>,
        parts: [u8; 2],
        context: &mut Context,
    ) -> Result<(), Error>;
    /// Reads the layout-selected ribbon ordering words.
    fn ribbon_tail(value: &<Self as native::records::RibbonLayout>::Tail) -> [i16; 3];
    /// Constructs the layout-selected ribbon ordering words.
    fn ribbon_tail_from(
        value: [i16; 3],
        context: &mut Context,
    ) -> Result<<Self as native::records::RibbonLayout>::Tail, Error>;
}

macro_rules! particle_layouts{($($version:ty),*)=>{$(impl ParticleConversion for $version{fn write_logical_particle(value:&crate::model::profiles::m2::Particle<'_>,textures:&[crate::model::TextureId],context:&mut Context)->Result<native::records::ParticleEmitter<'static,Self>,Error>{logical::write::<Self>(value,textures,context)}fn logical_particle<'a,'data:'a>(value:&'a native::records::ParticleEmitter<'data,Self>,textures:&[crate::model::TextureId])->Result<crate::model::profiles::m2::Particle<'a>,Error>{logical::read::<Self>(value,textures)}fn legacy_texture_scale(value:&native::records::ParticleEmitter<'_,Self>)->Option<[u8;2]>{value.texture_scale.legacy_parts()}fn restore_legacy_texture_scale(value:&mut native::records::ParticleEmitter<'_,Self>,parts:[u8;2],context:&mut Context)->Result<(),Error>{value.texture_scale.set_legacy_parts(parts,context)}fn required_flags(value:&native::records::ParticleEmitter<'_,Self>)->ModelFlags{value.multi_texture_scroll.required_flag()}fn ribbon_tail(value:&<Self as native::records::RibbonLayout>::Tail)->[i16;3]{value.parts()}fn ribbon_tail_from(value:[i16;3],context:&mut Context)->Result<<Self as native::records::RibbonLayout>::Tail,Error>{RibbonTailFields::make(value,context)}})*};}

particle_layouts!(
    V256,
    V257,
    V260,
    V263,
    V264,
    V265,
    V272,
    V272PreSplit,
    V273,
    V274
);
