//! Typed native particle fields and particle-age appearance conversion.
use super::*;
mod appearance;
mod export;
mod fields;
mod import;
use appearance::*;
use export::export_particle;
use fields::*;
pub(super) use fields::{gravity_vector, gravity_word};
use import::import_particle;

/// Layout-specific particle fields shared by the native emitter adapter.
pub trait ParticleConversion: ModelLayout {
    /// Returns physical core flags required by the selected scrolling representation.
    fn required_flags(value: &native::records::ParticleEmitter<'_, Self>) -> u32;

    /// Reads the layout-selected ribbon ordering words.
    fn ribbon_tail(value: &<Self as native::records::RibbonLayout>::Tail) -> [i16; 3];
    /// Constructs the layout-selected ribbon ordering words.
    fn ribbon_tail_from(
        value: [i16; 3],
        context: &mut Context,
    ) -> Result<<Self as native::records::RibbonLayout>::Tail, Error>;

    /// Translates static emitter fields and particle-age ramps into active WEM records.
    fn import_particle(
        value: &native::records::ParticleEmitter<'_, Self>,
        doc: &mut wem::RawDocument<'static>,
    ) -> Result<wr::M2ParticleEmitterPayload<wr::V19>, Error>;
    /// Builds the layout-selected native emitter fields from active WEM records.
    fn export_particle(
        value: &wr::M2ParticleEmitterPayload<wr::V19>,
        doc: &wem::RawDocument<'_>,
        context: &mut Context,
    ) -> Result<native::records::ParticleEmitter<'static, Self>, Error>;
}

macro_rules! particle_layouts{($($version:ty),*)=>{$(impl ParticleConversion for $version{fn required_flags(value:&native::records::ParticleEmitter<'_,Self>)->u32{value.multi_texture_scroll.required_flag()}fn ribbon_tail(value:&<Self as native::records::RibbonLayout>::Tail)->[i16;3]{value.parts()}fn ribbon_tail_from(value:[i16;3],context:&mut Context)->Result<<Self as native::records::RibbonLayout>::Tail,Error>{RibbonTailFields::make(value,context)}fn import_particle(value:&native::records::ParticleEmitter<'_,Self>,doc:&mut wem::RawDocument<'static>)->Result<wr::M2ParticleEmitterPayload<wr::V19>,Error>{import_particle::<Self>(value,doc)}fn export_particle(value:&wr::M2ParticleEmitterPayload<wr::V19>,doc:&wem::RawDocument<'_>,context:&mut Context)->Result<native::records::ParticleEmitter<'static,Self>,Error>{export_particle::<Self>(value,doc,context)}})*};}

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
