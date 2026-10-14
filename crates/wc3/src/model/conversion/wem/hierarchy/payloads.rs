//! Node payload discriminators become actual attached behaviors or bone declarations.
use super::super::{actors, forces, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::common::{
    Attachment, Event, EventCategory, EventCode, Feature as CommonFeature,
};
use crate::model::semantic::profiles::wem::{Bone, Feature, SectionId};
use crate::model::semantic::{
    Bounds, FeatureId, MaterialSlotId, MeshId, ModelId, NodeId, TextureId,
};
use crate::model::wem::{records, RawDocument, Ref};
use actors::common::{self, Field};
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) struct ReadContext<'a, 'view, 'source> {
    pub document: &'view RawDocument<'source>,
    pub nodes: &'a [NodeId],
    pub meshes: &'a [MeshId],
    pub models: &'a [ModelId],
    pub features: &'a BTreeMap<u32, FeatureId>,
    pub slots: &'a [MaterialSlotId],
    pub textures: &'a [TextureId],
    pub sections: &'a [SectionId],
}
pub(in crate::model::conversion::wem) struct Decoded<'view> {
    pub bone: Option<Bone>,
    pub feature: Option<Feature<'view>>,
}
pub(in crate::model::conversion::wem) trait ReadPayload {
    fn read_payload<'view>(
        &self,
        context: &ReadContext<'_, 'view, '_>,
    ) -> Result<Decoded<'view>, Error>;
    fn has_feature(&self) -> bool;
}
impl Field<records::EventCategory> for records::EventCategory {
    fn read(&self) -> Option<records::EventCategory> {
        Some(*self)
    }
    fn write(value: Option<records::EventCategory>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.event.category",
                "selected vocabulary requires a category declaration",
            )
        })
    }
}
fn category(value: records::EventCategory) -> EventCategory {
    match value {
        records::EventCategory::None => EventCategory::Unspecified,
        records::EventCategory::Sound => EventCategory::Sound,
        records::EventCategory::Splat => EventCategory::Splat,
        records::EventCategory::Footprint => EventCategory::Footprint,
        records::EventCategory::UberSplat => EventCategory::LargeSplat,
        records::EventCategory::SpawnModel => EventCategory::SpawnModel,
        records::EventCategory::MorphShow => EventCategory::ShowMorph,
        records::EventCategory::MorphHide => EventCategory::HideMorph,
        records::EventCategory::Count => {
            EventCategory::Unknown(records::EventCategory::Count.raw())
        }
        records::EventCategory::Unknown(value) => EventCategory::Unknown(value),
    }
}
impl<D: records::NodePayloadDialect> ReadPayload for records::NodePayload<D>
where
    D::GateMesh: Field<u32>,
    D::ShadowCasting: Field<bool>,
    D::ShadowCastingStart: Field<f32>,
    D::ShadowCastingEnd: Field<f32>,
    D::QuadraticFalloff: Field<f32>,
    D::LinearFalloff: Field<f32>,
    D::Damping: Field<f32>,
    <D as records::CameraPayloadDialect>::Target: Field<[f32; 3]>,
    D::FocusDistance: Field<f32>,
    D::FocalLength: Field<f32>,
    D::FStop: Field<f32>,
    D::Category: Field<records::EventCategory>,
    D::Code: Field<Ref<records::targets::Character>>,
    D::Transform: forces::ForceTransform,
    D::Texture2Scale: Field<f32>,
    D::Texture3Scale: Field<f32>,
    D::Texture2ScrollMid: Field<[f32; 2]>,
    D::Texture3ScrollMid: Field<[f32; 2]>,
    D::Texture2ScrollRange: Field<[f32; 2]>,
    D::Texture3ScrollRange: Field<[f32; 2]>,
    D::InheritVelocityScale: Field<f32>,
    D::FollowSpeed1: Field<f32>,
    D::FollowScale1: Field<f32>,
    D::FollowSpeed2: Field<f32>,
    D::FollowScale2: Field<f32>,
    D::TumbleMin: Field<[f32; 3]>,
    D::TumbleMax: Field<[f32; 3]>,
    D::ColorMult: Field<f32>,
    D::AlphaMult: Field<f32>,
    D::AlphaCutoffTimes: Field<Ref<records::targets::Real>>,
    D::AlphaCutoffs: Field<Ref<records::targets::Real>>,
    D::ParticleModel: Field<records::AssetKey>,
    D::TrailModel: Field<records::AssetKey>,
{
    fn has_feature(&self) -> bool {
        !matches!(self, Self::Helper(_) | Self::Bone(_) | Self::Count)
    }
    fn read_payload<'view>(
        &self,
        context: &ReadContext<'_, 'view, '_>,
    ) -> Result<Decoded<'view>, Error> {
        let feature = match self {
            Self::Helper(_) => {
                return Ok(Decoded {
                    bone: None,
                    feature: None,
                });
            }
            Self::Bone(value) => {
                let visibility_mesh = value
                    .gate_mesh
                    .read()
                    .filter(|index| *index != u32::MAX)
                    .map(|index| {
                        context.meshes.get(index as usize).copied().ok_or_else(|| {
                            Error::new("wem.bone.mesh", "visibility mesh is outside the model")
                        })
                    })
                    .transpose()?;
                return Ok(Decoded {
                    bone: Some(Bone {
                        bounds: Bounds {
                            minimum: value.bounds.minimum,
                            maximum: value.bounds.maximum,
                            sphere_radius: value.bounds.sphere_radius,
                        },
                        sphere_center: value.sphere.center,
                        sphere_radius: value.sphere.radius,
                        visibility_mesh,
                    }),
                    feature: None,
                });
            }
            Self::Attachment(value) => Feature::Common(CommonFeature::Attachment(Attachment {
                external: actors::read_asset(context.document, &value.asset)?,
                internal: if value.model == u32::MAX {
                    None
                } else {
                    Some(*context.models.get(value.model as usize).ok_or_else(|| {
                        Error::new(
                            "wem.attachment.model",
                            "internal model is outside the document collection",
                        )
                    })?)
                },
            })),
            Self::Light(value) => Feature::Common(CommonFeature::Light(common::read_light(value)?)),
            Self::Camera(value) => {
                Feature::Common(CommonFeature::Camera(common::read_camera(value)?))
            }
            Self::Particle(value) => Feature::Common(CommonFeature::ParticleSystem(
                actors::read_asset(context.document, &value.system)?,
            )),
            Self::Ribbon(value) => Feature::Common(CommonFeature::RibbonSystem(
                actors::read_asset(context.document, &value.system)?,
            )),
            Self::Event(value) => {
                let code = match (value.category.read(), value.code.read()) {
                    (None, None) => None,
                    (Some(kind), Some(code)) => Some(EventCode {
                        category: category(kind),
                        code: text::read_bytes(context.document, code)?,
                    }),
                    _ => {
                        return Err(Error::new(
                            "wem.event.code",
                            "incomplete command classification",
                        ));
                    }
                };
                Feature::Common(CommonFeature::Event(Event {
                    identity: value.id,
                    code,
                }))
            }
            Self::Collision(value) => Feature::Collision(forces::read_collision(value)),
            Self::ForceField(value) => Feature::Force(forces::read_force(value)),
            Self::VertexWarp(value) => Feature::VertexWarp(forces::read_warp(value)),
            Self::M2ParticleEmitter(value) => {
                Feature::Wow((actors::wow::read(context.document, value, context.textures)?).into())
            }
            Self::Sc2ParticleEmitter(_) | Self::Sc2RibbonEmitter(_) => Feature::Starcraft(
                actors::starcraft::read(
                    context.document,
                    self,
                    context.nodes,
                    context.features,
                    context.slots,
                    context.sections,
                )?
                .ok_or_else(|| {
                    Error::new("wem.actor", "StarCraft discriminator was not decoded")
                })?,
            ),
            Self::Wc3ParticleEmitter1(_)
            | Self::Wc3ParticleEmitter2(_)
            | Self::Wc3RibbonEmitter(_)
            | Self::Wc3CornEmitter(_)
            | Self::Wc3FaceFx(_) => Feature::Warcraft(
                actors::read_warcraft(context.document, self, context.textures, context.slots)?
                    .ok_or_else(|| {
                        Error::new("wem.actor", "Warcraft discriminator was not decoded")
                    })?
                    .into(),
            ),
            Self::Count => {
                return Err(Error::new(
                    "wem.node.payload",
                    "sentinel does not declare an attached behavior",
                ));
            }
        };
        Ok(Decoded {
            bone: None,
            feature: Some(feature),
        })
    }
}
