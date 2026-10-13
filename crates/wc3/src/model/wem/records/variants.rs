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
        let kind = MaterialKind::read(input)?;
        Ok(match kind {
            MaterialKind::Composite => Self::Composite(
                <Ref<targets::CompositeBody> as wem::Read<'data>>::read(input)?,
            ),
            MaterialKind::Combiners => Self::Combiners(
                <Ref<targets::CombinersBody> as wem::Read<'data>>::read(input)?,
            ),
            MaterialKind::LegacyDeferred => Self::LegacyDeferred(
                <Ref<targets::LegacyDeferredBody> as wem::Read<'data>>::read(input)?,
            ),
            MaterialKind::PBRDeferred => Self::PbrDeferred(
                <Ref<targets::PbrDeferredBody> as wem::Read<'data>>::read(input)?,
            ),
            MaterialKind::Count => Self::Count,
            _ => return Err(ReadError::unsupported(at, "MaterialBody discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = MaterialKind::read(input)?;
        match kind {
            MaterialKind::Composite => {
                <Ref<targets::CompositeBody> as wem::Read<'data>>::skip(input)
            }
            MaterialKind::Combiners => {
                <Ref<targets::CombinersBody> as wem::Read<'data>>::skip(input)
            }
            MaterialKind::LegacyDeferred => {
                <Ref<targets::LegacyDeferredBody> as wem::Read<'data>>::skip(input)
            }
            MaterialKind::PBRDeferred => {
                <Ref<targets::PbrDeferredBody> as wem::Read<'data>>::skip(input)
            }
            MaterialKind::Count => Ok(()),
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
                MaterialKind::Composite.write(output)?;
                value.write(output)?;
            }
            Self::Combiners(value) => {
                MaterialKind::Combiners.write(output)?;
                value.write(output)?;
            }
            Self::LegacyDeferred(value) => {
                MaterialKind::LegacyDeferred.write(output)?;
                value.write(output)?;
            }
            Self::PbrDeferred(value) => {
                MaterialKind::PBRDeferred.write(output)?;
                value.write(output)?;
            }
            Self::Count => {
                MaterialKind::Count.write(output)?;
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
        let kind = FeatureKind::read(input)?;
        Ok(match kind {
            FeatureKind::Fresnel => Self::Fresnel(<Ref<targets::FresnelFeature> as wem::Read<
                'data,
            >>::read(input)?),
            FeatureKind::UvAnimation => Self::UvAnimation(
                <Ref<targets::UvAnimationFeature> as wem::Read<'data>>::read(input)?,
            ),
            FeatureKind::LayerShading => Self::LayerShading(
                <Ref<targets::LayerShadingFeature> as wem::Read<'data>>::read(input)?,
            ),
            FeatureKind::Count => Self::Count,
            _ => return Err(ReadError::unsupported(at, "FeaturePayload discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = FeatureKind::read(input)?;
        match kind {
            FeatureKind::Fresnel => <Ref<targets::FresnelFeature> as wem::Read<'data>>::skip(input),
            FeatureKind::UvAnimation => {
                <Ref<targets::UvAnimationFeature> as wem::Read<'data>>::skip(input)
            }
            FeatureKind::LayerShading => {
                <Ref<targets::LayerShadingFeature> as wem::Read<'data>>::skip(input)
            }
            FeatureKind::Count => Ok(()),
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
                FeatureKind::Fresnel.write(output)?;
                value.write(output)?;
            }
            Self::UvAnimation(value) => {
                FeatureKind::UvAnimation.write(output)?;
                value.write(output)?;
            }
            Self::LayerShading(value) => {
                FeatureKind::LayerShading.write(output)?;
                value.write(output)?;
            }
            Self::Count => {
                FeatureKind::Count.write(output)?;
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
        let kind = NativeKind::read(input)?;
        Ok(match kind {
            NativeKind::None => Self::None,
            NativeKind::Mdx => Self::Mdx(<Ref<targets::MdxMaterial> as wem::Read<'data>>::read(
                input,
            )?),
            NativeKind::M2 => {
                Self::M2(<Ref<targets::M2Material> as wem::Read<'data>>::read(input)?)
            }
            NativeKind::M3 => {
                Self::M3(<Ref<targets::M3Material> as wem::Read<'data>>::read(input)?)
            }
            NativeKind::D3 => {
                Self::D3(<Ref<targets::D3Material> as wem::Read<'data>>::read(input)?)
            }
            _ => return Err(ReadError::unsupported(at, "NativeMaterial discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = NativeKind::read(input)?;
        match kind {
            NativeKind::None => Ok(()),
            NativeKind::Mdx => <Ref<targets::MdxMaterial> as wem::Read<'data>>::skip(input),
            NativeKind::M2 => <Ref<targets::M2Material> as wem::Read<'data>>::skip(input),
            NativeKind::M3 => <Ref<targets::M3Material> as wem::Read<'data>>::skip(input),
            NativeKind::D3 => <Ref<targets::D3Material> as wem::Read<'data>>::skip(input),
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
                NativeKind::None.write(output)?;
            }
            Self::Mdx(value) => {
                NativeKind::Mdx.write(output)?;
                value.write(output)?;
            }
            Self::M2(value) => {
                NativeKind::M2.write(output)?;
                value.write(output)?;
            }
            Self::M3(value) => {
                NativeKind::M3.write(output)?;
                value.write(output)?;
            }
            Self::D3(value) => {
                NativeKind::D3.write(output)?;
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
        let kind = TextureKeyKind::read(input)?;
        Ok(match kind {
            TextureKeyKind::None => Self::None,
            TextureKeyKind::Path => Self::Path(<TexturePath as wem::Read<'data>>::read(input)?),
            TextureKeyKind::FileDataId => {
                Self::FileDataId(<TextureFileDataId as wem::Read<'data>>::read(input)?)
            }
            TextureKeyKind::SnoId => Self::SnoId(<TextureSnoId as wem::Read<'data>>::read(input)?),
            _ => return Err(ReadError::unsupported(at, "TextureKey discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = TextureKeyKind::read(input)?;
        match kind {
            TextureKeyKind::None => Ok(()),
            TextureKeyKind::Path => <TexturePath as wem::Read<'data>>::skip(input),
            TextureKeyKind::FileDataId => <TextureFileDataId as wem::Read<'data>>::skip(input),
            TextureKeyKind::SnoId => <TextureSnoId as wem::Read<'data>>::skip(input),
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
                TextureKeyKind::None.write(output)?;
            }
            Self::Path(value) => {
                TextureKeyKind::Path.write(output)?;
                value.write(output)?;
            }
            Self::FileDataId(value) => {
                TextureKeyKind::FileDataId.write(output)?;
                value.write(output)?;
            }
            Self::SnoId(value) => {
                TextureKeyKind::SnoId.write(output)?;
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
        let kind = NodeKind::read(input)?;
        Ok(match kind {
            NodeKind::Helper => {
                Self::Helper(Box::new(<HelperPayload as wem::Read<'data>>::read(input)?))
            }
            NodeKind::Bone => {
                Self::Bone(Box::new(<BonePayload<D> as wem::Read<'data>>::read(input)?))
            }
            NodeKind::Attachment => Self::Attachment(Box::new(<AttachmentPayload as wem::Read<
                'data,
            >>::read(input)?)),
            NodeKind::Light => Self::Light(Box::new(<LightPayload<D> as wem::Read<'data>>::read(
                input,
            )?)),
            NodeKind::Camera => Self::Camera(Box::new(
                <CameraPayload<D> as wem::Read<'data>>::read(input)?,
            )),
            NodeKind::ParticleEmitter => Self::Particle(Box::new(
                <ParticlePayload as wem::Read<'data>>::read(input)?,
            )),
            NodeKind::RibbonEmitter => {
                Self::Ribbon(Box::new(<RibbonPayload as wem::Read<'data>>::read(input)?))
            }
            NodeKind::Event => Self::Event(Box::new(<EventPayload<D> as wem::Read<'data>>::read(
                input,
            )?)),
            NodeKind::CollisionShape => Self::Collision(Box::new(
                <CollisionPayload as wem::Read<'data>>::read(input)?,
            )),
            NodeKind::Wc3ParticleEmitter1 => {
                Self::Wc3ParticleEmitter1(Box::new(<Wc3ParticleEmitter1Payload as wem::Read<
                    'data,
                >>::read(input)?))
            }
            NodeKind::Wc3ParticleEmitter2 => {
                Self::Wc3ParticleEmitter2(Box::new(<Wc3ParticleEmitter2Payload as wem::Read<
                    'data,
                >>::read(input)?))
            }
            NodeKind::Wc3RibbonEmitter => Self::Wc3RibbonEmitter(Box::new(
                <Wc3RibbonEmitterPayload as wem::Read<'data>>::read(input)?,
            )),
            NodeKind::Sc2ParticleEmitter => {
                Self::Sc2ParticleEmitter(Box::new(<Sc2ParticleEmitterPayload as wem::Read<
                    'data,
                >>::read(input)?))
            }
            NodeKind::Sc2RibbonEmitter => Self::Sc2RibbonEmitter(Box::new(
                <Sc2RibbonEmitterPayload as wem::Read<'data>>::read(input)?,
            )),
            NodeKind::Wc3CornEmitter => Self::Wc3CornEmitter(Box::new(
                <Wc3CornEmitterPayload as wem::Read<'data>>::read(input)?,
            )),
            NodeKind::M2ParticleEmitter => {
                Self::M2ParticleEmitter(Box::new(<M2ParticleEmitterPayload<D> as wem::Read<
                    'data,
                >>::read(input)?))
            }
            NodeKind::ForceField => Self::ForceField(Box::new(
                <ForceFieldPayload<D> as wem::Read<'data>>::read(input)?,
            )),
            NodeKind::VertexWarp => Self::VertexWarp(Box::new(<VertexWarpPayload as wem::Read<
                'data,
            >>::read(input)?)),
            NodeKind::Wc3FaceFx => Self::Wc3FaceFx(Box::new(<Wc3FaceFxPayload as wem::Read<
                'data,
            >>::read(input)?)),
            NodeKind::Count => Self::Count,
            _ => return Err(ReadError::unsupported(at, "NodePayload discriminator")),
        })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let at = input.absolute_position();
        let kind = NodeKind::read(input)?;
        match kind {
            NodeKind::Helper => <HelperPayload as wem::Read<'data>>::skip(input),
            NodeKind::Bone => <BonePayload<D> as wem::Read<'data>>::skip(input),
            NodeKind::Attachment => <AttachmentPayload as wem::Read<'data>>::skip(input),
            NodeKind::Light => <LightPayload<D> as wem::Read<'data>>::skip(input),
            NodeKind::Camera => <CameraPayload<D> as wem::Read<'data>>::skip(input),
            NodeKind::ParticleEmitter => <ParticlePayload as wem::Read<'data>>::skip(input),
            NodeKind::RibbonEmitter => <RibbonPayload as wem::Read<'data>>::skip(input),
            NodeKind::Event => <EventPayload<D> as wem::Read<'data>>::skip(input),
            NodeKind::CollisionShape => <CollisionPayload as wem::Read<'data>>::skip(input),
            NodeKind::Wc3ParticleEmitter1 => {
                <Wc3ParticleEmitter1Payload as wem::Read<'data>>::skip(input)
            }
            NodeKind::Wc3ParticleEmitter2 => {
                <Wc3ParticleEmitter2Payload as wem::Read<'data>>::skip(input)
            }
            NodeKind::Wc3RibbonEmitter => {
                <Wc3RibbonEmitterPayload as wem::Read<'data>>::skip(input)
            }
            NodeKind::Sc2ParticleEmitter => {
                <Sc2ParticleEmitterPayload as wem::Read<'data>>::skip(input)
            }
            NodeKind::Sc2RibbonEmitter => {
                <Sc2RibbonEmitterPayload as wem::Read<'data>>::skip(input)
            }
            NodeKind::Wc3CornEmitter => <Wc3CornEmitterPayload as wem::Read<'data>>::skip(input),
            NodeKind::M2ParticleEmitter => {
                <M2ParticleEmitterPayload<D> as wem::Read<'data>>::skip(input)
            }
            NodeKind::ForceField => <ForceFieldPayload<D> as wem::Read<'data>>::skip(input),
            NodeKind::VertexWarp => <VertexWarpPayload as wem::Read<'data>>::skip(input),
            NodeKind::Wc3FaceFx => <Wc3FaceFxPayload as wem::Read<'data>>::skip(input),
            NodeKind::Count => Ok(()),
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
                NodeKind::Helper.write(output)?;
                value.write(output)?;
            }
            Self::Bone(value) => {
                NodeKind::Bone.write(output)?;
                value.write(output)?;
            }
            Self::Attachment(value) => {
                NodeKind::Attachment.write(output)?;
                value.write(output)?;
            }
            Self::Light(value) => {
                NodeKind::Light.write(output)?;
                value.write(output)?;
            }
            Self::Camera(value) => {
                NodeKind::Camera.write(output)?;
                value.write(output)?;
            }
            Self::Particle(value) => {
                NodeKind::ParticleEmitter.write(output)?;
                value.write(output)?;
            }
            Self::Ribbon(value) => {
                NodeKind::RibbonEmitter.write(output)?;
                value.write(output)?;
            }
            Self::Event(value) => {
                NodeKind::Event.write(output)?;
                value.write(output)?;
            }
            Self::Collision(value) => {
                NodeKind::CollisionShape.write(output)?;
                value.write(output)?;
            }
            Self::Wc3ParticleEmitter1(value) => {
                NodeKind::Wc3ParticleEmitter1.write(output)?;
                value.write(output)?;
            }
            Self::Wc3ParticleEmitter2(value) => {
                NodeKind::Wc3ParticleEmitter2.write(output)?;
                value.write(output)?;
            }
            Self::Wc3RibbonEmitter(value) => {
                NodeKind::Wc3RibbonEmitter.write(output)?;
                value.write(output)?;
            }
            Self::Sc2ParticleEmitter(value) => {
                NodeKind::Sc2ParticleEmitter.write(output)?;
                value.write(output)?;
            }
            Self::Sc2RibbonEmitter(value) => {
                NodeKind::Sc2RibbonEmitter.write(output)?;
                value.write(output)?;
            }
            Self::Wc3CornEmitter(value) => {
                NodeKind::Wc3CornEmitter.write(output)?;
                value.write(output)?;
            }
            Self::M2ParticleEmitter(value) => {
                NodeKind::M2ParticleEmitter.write(output)?;
                value.write(output)?;
            }
            Self::ForceField(value) => {
                NodeKind::ForceField.write(output)?;
                value.write(output)?;
            }
            Self::VertexWarp(value) => {
                NodeKind::VertexWarp.write(output)?;
                value.write(output)?;
            }
            Self::Wc3FaceFx(value) => {
                NodeKind::Wc3FaceFx.write(output)?;
                value.write(output)?;
            }
            Self::Count => {
                NodeKind::Count.write(output)?;
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
