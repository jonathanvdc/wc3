//! Discriminated inline payloads and referenced material bodies.
use super::*;
use crate::model::wem::{
    self, Decoder, Encoder, NativeAbi, ReadError, Ref, ReferenceVisitor, Validate, VisitReferences,
    WriteError,
};
use crate::model::wem::{Read as _, Write as _};
use std::fmt::Debug;
use std::marker::PhantomData;
/// Discriminator and typed payload of a MaterialBody value.
#[derive(Clone, Debug, PartialEq, wem::Validate, wem::VisitReferences)]
pub enum MaterialBody {
    /// Composite payload selected by its native discriminator.
    Composite(Ref<targets::CompositeBody>),
    /// Combiners payload selected by its native discriminator.
    Combiners(Ref<targets::CombinersBody>),
    /// LegacyDeferred payload selected by its native discriminator.
    LegacyDeferred(Ref<targets::LegacyDeferredBody>),
    /// PbrDeferred payload selected by its native discriminator.
    PbrDeferred(Ref<targets::PbrDeferredBody>),
    /// Count payload selected by its native discriminator.
    Count,
}
impl Default for MaterialBody {
    fn default() -> Self {
        Self::Composite(Ref::<targets::CompositeBody>::default())
    }
}
impl<'data> wem::Read<'data> for MaterialBody
where
    Ref<targets::CompositeBody>: wem::Read<'data>,
    Ref<targets::CombinersBody>: wem::Read<'data>,
    Ref<targets::LegacyDeferredBody>: wem::Read<'data>,
    Ref<targets::PbrDeferredBody>: wem::Read<'data>,
{
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        Ok(match kind {
            0 => Self::Composite(<Ref<targets::CompositeBody> as wem::Read<'data>>::read(
                input,
            )?),
            1 => Self::Combiners(<Ref<targets::CombinersBody> as wem::Read<'data>>::read(
                input,
            )?),
            2 => Self::LegacyDeferred(
                <Ref<targets::LegacyDeferredBody> as wem::Read<'data>>::read(input)?,
            ),
            3 => Self::PbrDeferred(<Ref<targets::PbrDeferredBody> as wem::Read<'data>>::read(
                input,
            )?),
            4 => Self::Count,
            _ => return Err(ReadError::unsupported(at, "MaterialBody discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        match kind {
            0 => <Ref<targets::CompositeBody> as wem::Read<'data>>::skip(input),
            1 => <Ref<targets::CombinersBody> as wem::Read<'data>>::skip(input),
            2 => <Ref<targets::LegacyDeferredBody> as wem::Read<'data>>::skip(input),
            3 => <Ref<targets::PbrDeferredBody> as wem::Read<'data>>::skip(input),
            4 => Ok(()),
            _ => Err(ReadError::unsupported(at, "MaterialBody discriminator")),
        }
    }
}
impl wem::Write for MaterialBody
where
    Ref<targets::CompositeBody>: wem::Write,
    Ref<targets::CombinersBody>: wem::Write,
    Ref<targets::LegacyDeferredBody>: wem::Write,
    Ref<targets::PbrDeferredBody>: wem::Write,
{
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        match self {
            Self::Composite(value) => {
                0_u8.write(output)?;
                value.write(output)?;
            }
            Self::Combiners(value) => {
                1_u8.write(output)?;
                value.write(output)?;
            }
            Self::LegacyDeferred(value) => {
                2_u8.write(output)?;
                value.write(output)?;
            }
            Self::PbrDeferred(value) => {
                3_u8.write(output)?;
                value.write(output)?;
            }
            Self::Count => {
                4_u8.write(output)?;
            }
        }
        Ok(())
    }
}
impl wem::RecordLayout for MaterialBody {
    fn minimum_size() -> usize {
        1
    }
}
/// Discriminator and typed payload of a FeaturePayload value.
#[derive(Clone, Debug, PartialEq, wem::Validate, wem::VisitReferences)]
pub enum FeaturePayload {
    /// Fresnel payload selected by its native discriminator.
    Fresnel(Ref<targets::FresnelFeature>),
    /// UvAnimation payload selected by its native discriminator.
    UvAnimation(Ref<targets::UvAnimationFeature>),
    /// LayerShading payload selected by its native discriminator.
    LayerShading(Ref<targets::LayerShadingFeature>),
    /// Count payload selected by its native discriminator.
    Count,
}
impl Default for FeaturePayload {
    fn default() -> Self {
        Self::Fresnel(Ref::<targets::FresnelFeature>::default())
    }
}
impl<'data> wem::Read<'data> for FeaturePayload
where
    Ref<targets::FresnelFeature>: wem::Read<'data>,
    Ref<targets::UvAnimationFeature>: wem::Read<'data>,
    Ref<targets::LayerShadingFeature>: wem::Read<'data>,
{
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        Ok(match kind {
            0 => Self::Fresnel(<Ref<targets::FresnelFeature> as wem::Read<'data>>::read(
                input,
            )?),
            1 => Self::UvAnimation(
                <Ref<targets::UvAnimationFeature> as wem::Read<'data>>::read(input)?,
            ),
            2 => Self::LayerShading(
                <Ref<targets::LayerShadingFeature> as wem::Read<'data>>::read(input)?,
            ),
            3 => Self::Count,
            _ => return Err(ReadError::unsupported(at, "FeaturePayload discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        match kind {
            0 => <Ref<targets::FresnelFeature> as wem::Read<'data>>::skip(input),
            1 => <Ref<targets::UvAnimationFeature> as wem::Read<'data>>::skip(input),
            2 => <Ref<targets::LayerShadingFeature> as wem::Read<'data>>::skip(input),
            3 => Ok(()),
            _ => Err(ReadError::unsupported(at, "FeaturePayload discriminator")),
        }
    }
}
impl wem::Write for FeaturePayload
where
    Ref<targets::FresnelFeature>: wem::Write,
    Ref<targets::UvAnimationFeature>: wem::Write,
    Ref<targets::LayerShadingFeature>: wem::Write,
{
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        match self {
            Self::Fresnel(value) => {
                0_u8.write(output)?;
                value.write(output)?;
            }
            Self::UvAnimation(value) => {
                1_u8.write(output)?;
                value.write(output)?;
            }
            Self::LayerShading(value) => {
                2_u8.write(output)?;
                value.write(output)?;
            }
            Self::Count => {
                3_u8.write(output)?;
            }
        }
        Ok(())
    }
}
impl wem::RecordLayout for FeaturePayload {
    fn minimum_size() -> usize {
        1
    }
}
/// Discriminator and typed payload of a NativeMaterial value.
#[derive(Clone, Debug, PartialEq, wem::Validate, wem::VisitReferences, Default)]
pub enum NativeMaterial {
    /// None payload selected by its native discriminator.
    #[default]
    None,
    /// Mdx payload selected by its native discriminator.
    Mdx(Ref<targets::MdxMaterial>),
    /// M2 payload selected by its native discriminator.
    M2(Ref<targets::M2Material>),
    /// M3 payload selected by its native discriminator.
    M3(Ref<targets::M3Material>),
    /// D3 payload selected by its native discriminator.
    D3(Ref<targets::D3Material>),
}

impl<'data> wem::Read<'data> for NativeMaterial
where
    Ref<targets::MdxMaterial>: wem::Read<'data>,
    Ref<targets::M2Material>: wem::Read<'data>,
    Ref<targets::M3Material>: wem::Read<'data>,
    Ref<targets::D3Material>: wem::Read<'data>,
{
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        Ok(match kind {
            0 => Self::None,
            1 => Self::Mdx(<Ref<targets::MdxMaterial> as wem::Read<'data>>::read(
                input,
            )?),
            2 => Self::M2(<Ref<targets::M2Material> as wem::Read<'data>>::read(input)?),
            3 => Self::M3(<Ref<targets::M3Material> as wem::Read<'data>>::read(input)?),
            4 => Self::D3(<Ref<targets::D3Material> as wem::Read<'data>>::read(input)?),
            _ => return Err(ReadError::unsupported(at, "NativeMaterial discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        match kind {
            0 => Ok(()),
            1 => <Ref<targets::MdxMaterial> as wem::Read<'data>>::skip(input),
            2 => <Ref<targets::M2Material> as wem::Read<'data>>::skip(input),
            3 => <Ref<targets::M3Material> as wem::Read<'data>>::skip(input),
            4 => <Ref<targets::D3Material> as wem::Read<'data>>::skip(input),
            _ => Err(ReadError::unsupported(at, "NativeMaterial discriminator")),
        }
    }
}
impl wem::Write for NativeMaterial
where
    Ref<targets::MdxMaterial>: wem::Write,
    Ref<targets::M2Material>: wem::Write,
    Ref<targets::M3Material>: wem::Write,
    Ref<targets::D3Material>: wem::Write,
{
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        match self {
            Self::None => {
                0_u8.write(output)?;
            }
            Self::Mdx(value) => {
                1_u8.write(output)?;
                value.write(output)?;
            }
            Self::M2(value) => {
                2_u8.write(output)?;
                value.write(output)?;
            }
            Self::M3(value) => {
                3_u8.write(output)?;
                value.write(output)?;
            }
            Self::D3(value) => {
                4_u8.write(output)?;
                value.write(output)?;
            }
        }
        Ok(())
    }
}
impl wem::RecordLayout for NativeMaterial {
    fn minimum_size() -> usize {
        1
    }
}
/// Discriminator and typed payload of a TextureKey value.
#[derive(Clone, Debug, PartialEq, wem::Validate, wem::VisitReferences, Default)]
pub enum TextureKey {
    /// None payload selected by its native discriminator.
    #[default]
    None,
    /// Path payload selected by its native discriminator.
    Path(TexturePath),
    /// FileDataId payload selected by its native discriminator.
    FileDataId(TextureFileDataId),
    /// SnoId payload selected by its native discriminator.
    SnoId(TextureSnoId),
}

impl<'data> wem::Read<'data> for TextureKey
where
    TexturePath: wem::Read<'data>,
    TextureFileDataId: wem::Read<'data>,
    TextureSnoId: wem::Read<'data>,
{
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        Ok(match kind {
            0 => Self::None,
            1 => Self::Path(<TexturePath as wem::Read<'data>>::read(input)?),
            2 => Self::FileDataId(<TextureFileDataId as wem::Read<'data>>::read(input)?),
            3 => Self::SnoId(<TextureSnoId as wem::Read<'data>>::read(input)?),
            _ => return Err(ReadError::unsupported(at, "TextureKey discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        match kind {
            0 => Ok(()),
            1 => <TexturePath as wem::Read<'data>>::skip(input),
            2 => <TextureFileDataId as wem::Read<'data>>::skip(input),
            3 => <TextureSnoId as wem::Read<'data>>::skip(input),
            _ => Err(ReadError::unsupported(at, "TextureKey discriminator")),
        }
    }
}
impl wem::Write for TextureKey
where
    TexturePath: wem::Write,
    TextureFileDataId: wem::Write,
    TextureSnoId: wem::Write,
{
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        match self {
            Self::None => {
                0_u8.write(output)?;
            }
            Self::Path(value) => {
                1_u8.write(output)?;
                value.write(output)?;
            }
            Self::FileDataId(value) => {
                2_u8.write(output)?;
                value.write(output)?;
            }
            Self::SnoId(value) => {
                3_u8.write(output)?;
                value.write(output)?;
            }
        }
        Ok(())
    }
}
impl wem::RecordLayout for TextureKey {
    fn minimum_size() -> usize {
        1
    }
}
/// Required static layouts for the payload alternatives of a node.
pub trait NodePayloadDialect:
    Revision
    + BonePayloadDialect
    + CameraPayloadDialect
    + EventPayloadDialect
    + ForceFieldPayloadDialect
    + LightPayloadDialect
    + M2ParticleEmitterPayloadDialect
{
}
impl NodePayloadDialect for V0 {}
impl NodePayloadDialect for V1 {}
impl NodePayloadDialect for V2 {}
impl NodePayloadDialect for V3 {}
impl NodePayloadDialect for V4 {}
impl NodePayloadDialect for V5 {}
impl NodePayloadDialect for V6 {}
impl NodePayloadDialect for V7 {}
impl NodePayloadDialect for V8 {}
impl NodePayloadDialect for V9 {}
impl NodePayloadDialect for V10 {}
impl NodePayloadDialect for V11 {}
impl NodePayloadDialect for V12 {}
impl NodePayloadDialect for V13 {}
impl NodePayloadDialect for V14 {}
impl NodePayloadDialect for V15 {}
impl NodePayloadDialect for V16 {}
impl NodePayloadDialect for V17 {}
impl NodePayloadDialect for V18 {}
impl NodePayloadDialect for V19 {}
/// Discriminator and typed payload of a NodePayload value.
#[derive(Clone, Debug, PartialEq, wem::Validate, wem::VisitReferences)]
pub enum NodePayload<D: NodePayloadDialect> {
    /// Helper payload selected by its native discriminator.
    Helper(Box<HelperPayload>),
    /// Bone payload selected by its native discriminator.
    Bone(Box<BonePayload<D>>),
    /// Attachment payload selected by its native discriminator.
    Attachment(Box<AttachmentPayload>),
    /// Light payload selected by its native discriminator.
    Light(Box<LightPayload<D>>),
    /// Camera payload selected by its native discriminator.
    Camera(Box<CameraPayload<D>>),
    /// Particle payload selected by its native discriminator.
    Particle(Box<ParticlePayload>),
    /// Ribbon payload selected by its native discriminator.
    Ribbon(Box<RibbonPayload>),
    /// Event payload selected by its native discriminator.
    Event(Box<EventPayload<D>>),
    /// Collision payload selected by its native discriminator.
    Collision(Box<CollisionPayload>),
    /// Wc3ParticleEmitter1 payload selected by its native discriminator.
    Wc3ParticleEmitter1(Box<Wc3ParticleEmitter1Payload>),
    /// Wc3ParticleEmitter2 payload selected by its native discriminator.
    Wc3ParticleEmitter2(Box<Wc3ParticleEmitter2Payload>),
    /// Wc3RibbonEmitter payload selected by its native discriminator.
    Wc3RibbonEmitter(Box<Wc3RibbonEmitterPayload>),
    /// Sc2ParticleEmitter payload selected by its native discriminator.
    Sc2ParticleEmitter(Box<Sc2ParticleEmitterPayload>),
    /// Sc2RibbonEmitter payload selected by its native discriminator.
    Sc2RibbonEmitter(Box<Sc2RibbonEmitterPayload>),
    /// Wc3CornEmitter payload selected by its native discriminator.
    Wc3CornEmitter(Box<Wc3CornEmitterPayload>),
    /// M2ParticleEmitter payload selected by its native discriminator.
    M2ParticleEmitter(Box<M2ParticleEmitterPayload<D>>),
    /// ForceField payload selected by its native discriminator.
    ForceField(Box<ForceFieldPayload<D>>),
    /// VertexWarp payload selected by its native discriminator.
    VertexWarp(Box<VertexWarpPayload>),
    /// Wc3FaceFx payload selected by its native discriminator.
    Wc3FaceFx(Box<Wc3FaceFxPayload>),
    /// Count payload selected by its native discriminator.
    Count,
}
impl<D: NodePayloadDialect> Default for NodePayload<D> {
    fn default() -> Self {
        Self::Helper(Default::default())
    }
}
impl<'data, D: NodePayloadDialect> wem::Read<'data> for NodePayload<D>
where
    HelperPayload: wem::Read<'data>,
    BonePayload<D>: wem::Read<'data>,
    AttachmentPayload: wem::Read<'data>,
    LightPayload<D>: wem::Read<'data>,
    CameraPayload<D>: wem::Read<'data>,
    ParticlePayload: wem::Read<'data>,
    RibbonPayload: wem::Read<'data>,
    EventPayload<D>: wem::Read<'data>,
    CollisionPayload: wem::Read<'data>,
    Wc3ParticleEmitter1Payload: wem::Read<'data>,
    Wc3ParticleEmitter2Payload: wem::Read<'data>,
    Wc3RibbonEmitterPayload: wem::Read<'data>,
    Sc2ParticleEmitterPayload: wem::Read<'data>,
    Sc2RibbonEmitterPayload: wem::Read<'data>,
    Wc3CornEmitterPayload: wem::Read<'data>,
    M2ParticleEmitterPayload<D>: wem::Read<'data>,
    ForceFieldPayload<D>: wem::Read<'data>,
    VertexWarpPayload: wem::Read<'data>,
    Wc3FaceFxPayload: wem::Read<'data>,
{
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        Ok(match kind {
            0 => Self::Helper(Box::new(<HelperPayload as wem::Read<'data>>::read(input)?)),
            1 => Self::Bone(Box::new(<BonePayload<D> as wem::Read<'data>>::read(input)?)),
            2 => Self::Attachment(Box::new(<AttachmentPayload as wem::Read<'data>>::read(
                input,
            )?)),
            3 => Self::Light(Box::new(<LightPayload<D> as wem::Read<'data>>::read(
                input,
            )?)),
            4 => Self::Camera(Box::new(<CameraPayload<D> as wem::Read<'data>>::read(
                input,
            )?)),
            5 => Self::Particle(Box::new(<ParticlePayload as wem::Read<'data>>::read(
                input,
            )?)),
            6 => Self::Ribbon(Box::new(<RibbonPayload as wem::Read<'data>>::read(input)?)),
            7 => Self::Event(Box::new(<EventPayload<D> as wem::Read<'data>>::read(
                input,
            )?)),
            8 => Self::Collision(Box::new(<CollisionPayload as wem::Read<'data>>::read(
                input,
            )?)),
            9 => Self::Wc3ParticleEmitter1(Box::new(<Wc3ParticleEmitter1Payload as wem::Read<
                'data,
            >>::read(input)?)),
            10 => Self::Wc3ParticleEmitter2(Box::new(<Wc3ParticleEmitter2Payload as wem::Read<
                'data,
            >>::read(input)?)),
            11 => Self::Wc3RibbonEmitter(Box::new(<Wc3RibbonEmitterPayload as wem::Read<
                'data,
            >>::read(input)?)),
            12 => Self::Sc2ParticleEmitter(Box::new(<Sc2ParticleEmitterPayload as wem::Read<
                'data,
            >>::read(input)?)),
            13 => Self::Sc2RibbonEmitter(Box::new(<Sc2RibbonEmitterPayload as wem::Read<
                'data,
            >>::read(input)?)),
            14 => Self::Wc3CornEmitter(Box::new(
                <Wc3CornEmitterPayload as wem::Read<'data>>::read(input)?,
            )),
            15 => Self::M2ParticleEmitter(Box::new(<M2ParticleEmitterPayload<D> as wem::Read<
                'data,
            >>::read(input)?)),
            16 => Self::ForceField(Box::new(<ForceFieldPayload<D> as wem::Read<'data>>::read(
                input,
            )?)),
            17 => Self::VertexWarp(Box::new(<VertexWarpPayload as wem::Read<'data>>::read(
                input,
            )?)),
            18 => Self::Wc3FaceFx(Box::new(<Wc3FaceFxPayload as wem::Read<'data>>::read(
                input,
            )?)),
            19 => Self::Count,
            _ => return Err(ReadError::unsupported(at, "NodePayload discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = <u8 as wem::Read>::read(input)?;
        match kind {
            0 => <HelperPayload as wem::Read<'data>>::skip(input),
            1 => <BonePayload<D> as wem::Read<'data>>::skip(input),
            2 => <AttachmentPayload as wem::Read<'data>>::skip(input),
            3 => <LightPayload<D> as wem::Read<'data>>::skip(input),
            4 => <CameraPayload<D> as wem::Read<'data>>::skip(input),
            5 => <ParticlePayload as wem::Read<'data>>::skip(input),
            6 => <RibbonPayload as wem::Read<'data>>::skip(input),
            7 => <EventPayload<D> as wem::Read<'data>>::skip(input),
            8 => <CollisionPayload as wem::Read<'data>>::skip(input),
            9 => <Wc3ParticleEmitter1Payload as wem::Read<'data>>::skip(input),
            10 => <Wc3ParticleEmitter2Payload as wem::Read<'data>>::skip(input),
            11 => <Wc3RibbonEmitterPayload as wem::Read<'data>>::skip(input),
            12 => <Sc2ParticleEmitterPayload as wem::Read<'data>>::skip(input),
            13 => <Sc2RibbonEmitterPayload as wem::Read<'data>>::skip(input),
            14 => <Wc3CornEmitterPayload as wem::Read<'data>>::skip(input),
            15 => <M2ParticleEmitterPayload<D> as wem::Read<'data>>::skip(input),
            16 => <ForceFieldPayload<D> as wem::Read<'data>>::skip(input),
            17 => <VertexWarpPayload as wem::Read<'data>>::skip(input),
            18 => <Wc3FaceFxPayload as wem::Read<'data>>::skip(input),
            19 => Ok(()),
            _ => Err(ReadError::unsupported(at, "NodePayload discriminator")),
        }
    }
}
impl<D: NodePayloadDialect> wem::Write for NodePayload<D>
where
    HelperPayload: wem::Write,
    BonePayload<D>: wem::Write,
    AttachmentPayload: wem::Write,
    LightPayload<D>: wem::Write,
    CameraPayload<D>: wem::Write,
    ParticlePayload: wem::Write,
    RibbonPayload: wem::Write,
    EventPayload<D>: wem::Write,
    CollisionPayload: wem::Write,
    Wc3ParticleEmitter1Payload: wem::Write,
    Wc3ParticleEmitter2Payload: wem::Write,
    Wc3RibbonEmitterPayload: wem::Write,
    Sc2ParticleEmitterPayload: wem::Write,
    Sc2RibbonEmitterPayload: wem::Write,
    Wc3CornEmitterPayload: wem::Write,
    M2ParticleEmitterPayload<D>: wem::Write,
    ForceFieldPayload<D>: wem::Write,
    VertexWarpPayload: wem::Write,
    Wc3FaceFxPayload: wem::Write,
{
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        match self {
            Self::Helper(value) => {
                0_u8.write(output)?;
                value.write(output)?;
            }
            Self::Bone(value) => {
                1_u8.write(output)?;
                value.write(output)?;
            }
            Self::Attachment(value) => {
                2_u8.write(output)?;
                value.write(output)?;
            }
            Self::Light(value) => {
                3_u8.write(output)?;
                value.write(output)?;
            }
            Self::Camera(value) => {
                4_u8.write(output)?;
                value.write(output)?;
            }
            Self::Particle(value) => {
                5_u8.write(output)?;
                value.write(output)?;
            }
            Self::Ribbon(value) => {
                6_u8.write(output)?;
                value.write(output)?;
            }
            Self::Event(value) => {
                7_u8.write(output)?;
                value.write(output)?;
            }
            Self::Collision(value) => {
                8_u8.write(output)?;
                value.write(output)?;
            }
            Self::Wc3ParticleEmitter1(value) => {
                9_u8.write(output)?;
                value.write(output)?;
            }
            Self::Wc3ParticleEmitter2(value) => {
                10_u8.write(output)?;
                value.write(output)?;
            }
            Self::Wc3RibbonEmitter(value) => {
                11_u8.write(output)?;
                value.write(output)?;
            }
            Self::Sc2ParticleEmitter(value) => {
                12_u8.write(output)?;
                value.write(output)?;
            }
            Self::Sc2RibbonEmitter(value) => {
                13_u8.write(output)?;
                value.write(output)?;
            }
            Self::Wc3CornEmitter(value) => {
                14_u8.write(output)?;
                value.write(output)?;
            }
            Self::M2ParticleEmitter(value) => {
                15_u8.write(output)?;
                value.write(output)?;
            }
            Self::ForceField(value) => {
                16_u8.write(output)?;
                value.write(output)?;
            }
            Self::VertexWarp(value) => {
                17_u8.write(output)?;
                value.write(output)?;
            }
            Self::Wc3FaceFx(value) => {
                18_u8.write(output)?;
                value.write(output)?;
            }
            Self::Count => {
                19_u8.write(output)?;
            }
        }
        Ok(())
    }
}
impl<D: NodePayloadDialect> wem::RecordLayout for NodePayload<D> {
    fn minimum_size() -> usize {
        1
    }
}

mod abi_sealed {
    pub trait Sealed {}
}
/// Word width used by the producer for NM3_'s inline material discriminator.
pub trait NativeLayout: abi_sealed::Sealed + Clone + Copy + Debug + Default + PartialEq {
    /// Read the producer-sized discriminator without depending on the host ABI.
    fn read_index(input: &mut Decoder<'_>) -> Result<u64, ReadError>;
    /// Write the discriminator in the selected producer width.
    fn write_index(index: u64, output: &mut Encoder) -> Result<(), WriteError>;
    /// Native discriminator width in bytes.
    const WIDTH: usize;
    /// Producer ABI selected by this static layout.
    const ABI: NativeAbi;
}
/// Four-byte native size_t representation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Abi32;
impl abi_sealed::Sealed for Abi32 {}
impl NativeLayout for Abi32 {
    const WIDTH: usize = 4;
    const ABI: NativeAbi = NativeAbi::Abi32;
    fn read_index(input: &mut Decoder<'_>) -> Result<u64, ReadError> {
        check_read_abi::<Self>(input)?;
        Ok(u32::read(input)? as u64)
    }
    fn write_index(index: u64, output: &mut Encoder) -> Result<(), WriteError> {
        check_write_abi::<Self>(output)?;
        u32::try_from(index)
            .map_err(|_| WriteError::invalid("native material discriminator"))?
            .write(output)
    }
}
/// Eight-byte native size_t representation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Abi64;
impl abi_sealed::Sealed for Abi64 {}
impl NativeLayout for Abi64 {
    const WIDTH: usize = 8;
    const ABI: NativeAbi = NativeAbi::Abi64;
    fn read_index(input: &mut Decoder<'_>) -> Result<u64, ReadError> {
        check_read_abi::<Self>(input)?;
        u64::read(input)
    }
    fn write_index(index: u64, output: &mut Encoder) -> Result<(), WriteError> {
        check_write_abi::<Self>(output)?;
        index.write(output)
    }
}
/// Inline native M3 material alternatives, independent of the preserved material kind word.
#[derive(Clone, Debug, PartialEq, wem::Validate, wem::VisitReferences)]
pub enum M3Body {
    /// Standard material parameters.
    Standard(Box<M3Standard>),
    /// Displacement material parameters.
    Displacement(Box<M3Displacement>),
    /// Composite material parameters.
    Composite(Box<M3Composite>),
    /// Terrain material parameters.
    Terrain(Box<M3Terrain>),
    /// Volume material parameters.
    Volume(Box<M3Volume>),
    /// Creep material parameters.
    Creep(Box<M3Creep>),
    /// VolumeNoise material parameters.
    VolumeNoise(Box<M3VolumeNoise>),
    /// Stb material parameters.
    Stb(Box<M3Stb>),
    /// Reflection material parameters.
    Reflection(Box<M3Reflection>),
    /// LensFlare material parameters.
    LensFlare(Box<M3LensFlare>),
    /// DataDriven material parameters.
    DataDriven(Box<M3DataDriven>),
}
impl Default for M3Body {
    fn default() -> Self {
        Self::Standard(Box::default())
    }
}
/// An inline M3 material body with an explicit producer discriminator width.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeM3Body<A: NativeLayout> {
    /// Actual inline material parameters selected by the serialized body index.
    pub body: M3Body,
    /// Zero-width type witness for the producer ABI.
    pub abi: PhantomData<A>,
}
impl<A: NativeLayout> Default for NativeM3Body<A> {
    fn default() -> Self {
        Self {
            body: M3Body::default(),
            abi: PhantomData,
        }
    }
}
impl<A: NativeLayout> Validate for NativeM3Body<A> {
    fn validate(&self) -> Result<(), ReadError> {
        self.body.validate()
    }
}
impl<A: NativeLayout> VisitReferences for NativeM3Body<A> {
    fn visit_references<V: ReferenceVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        self.body.visit_references(visitor)
    }
}
impl<A: NativeLayout> wem::RecordLayout for NativeM3Body<A> {
    fn minimum_size() -> usize {
        A::WIDTH
    }
}
impl<'data, A: NativeLayout> wem::Read<'data> for NativeM3Body<A> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let at = input.absolute_position();
        let index = A::read_index(input)?;
        let body = match index {
            0 => M3Body::Standard(Box::new(M3Standard::read(input)?)),
            1 => M3Body::Displacement(Box::new(M3Displacement::read(input)?)),
            2 => M3Body::Composite(Box::new(M3Composite::read(input)?)),
            3 => M3Body::Terrain(Box::new(M3Terrain::read(input)?)),
            4 => M3Body::Volume(Box::new(M3Volume::read(input)?)),
            5 => M3Body::Creep(Box::new(M3Creep::read(input)?)),
            6 => M3Body::VolumeNoise(Box::new(M3VolumeNoise::read(input)?)),
            7 => M3Body::Stb(Box::new(M3Stb::read(input)?)),
            8 => M3Body::Reflection(Box::new(M3Reflection::read(input)?)),
            9 => M3Body::LensFlare(Box::new(M3LensFlare::read(input)?)),
            10 => M3Body::DataDriven(Box::new(M3DataDriven::read(input)?)),
            _ => return Err(ReadError::unsupported(at, "native M3 body index")),
        };
        Ok(Self {
            body,
            abi: PhantomData,
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let index = A::read_index(input)?;
        match index {
            0 => M3Standard::skip(input),
            1 => M3Displacement::skip(input),
            2 => M3Composite::skip(input),
            3 => M3Terrain::skip(input),
            4 => M3Volume::skip(input),
            5 => M3Creep::skip(input),
            6 => M3VolumeNoise::skip(input),
            7 => M3Stb::skip(input),
            8 => M3Reflection::skip(input),
            9 => M3LensFlare::skip(input),
            10 => M3DataDriven::skip(input),
            _ => Err(ReadError::unsupported(at, "native M3 body index")),
        }
    }
}
impl<A: NativeLayout> wem::Write for NativeM3Body<A> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        match &self.body {
            M3Body::Standard(value) => {
                A::write_index(0, output)?;
                value.write(output)?;
            }
            M3Body::Displacement(value) => {
                A::write_index(1, output)?;
                value.write(output)?;
            }
            M3Body::Composite(value) => {
                A::write_index(2, output)?;
                value.write(output)?;
            }
            M3Body::Terrain(value) => {
                A::write_index(3, output)?;
                value.write(output)?;
            }
            M3Body::Volume(value) => {
                A::write_index(4, output)?;
                value.write(output)?;
            }
            M3Body::Creep(value) => {
                A::write_index(5, output)?;
                value.write(output)?;
            }
            M3Body::VolumeNoise(value) => {
                A::write_index(6, output)?;
                value.write(output)?;
            }
            M3Body::Stb(value) => {
                A::write_index(7, output)?;
                value.write(output)?;
            }
            M3Body::Reflection(value) => {
                A::write_index(8, output)?;
                value.write(output)?;
            }
            M3Body::LensFlare(value) => {
                A::write_index(9, output)?;
                value.write(output)?;
            }
            M3Body::DataDriven(value) => {
                A::write_index(10, output)?;
                value.write(output)?;
            }
        }
        Ok(())
    }
}

fn check_read_abi<A: NativeLayout>(input: &Decoder<'_>) -> Result<(), ReadError> {
    if input.native_abi.is_some_and(|abi| abi != A::ABI) {
        return Err(ReadError::invalid(
            input.absolute_position(),
            "native material ABI mismatch",
        ));
    }
    Ok(())
}
fn check_write_abi<A: NativeLayout>(output: &Encoder) -> Result<(), WriteError> {
    if output.native_abi.is_some_and(|abi| abi != A::ABI) {
        return Err(WriteError::invalid("native material ABI mismatch"));
    }
    Ok(())
}
