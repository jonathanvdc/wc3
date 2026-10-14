//! Attached actor output preserves actual resource and hierarchy identities.
use super::{index, Context};
use crate::model::conversion::wem::actors::common::{self, Field};
use crate::model::conversion::wem::forces::ForceTransform;
use crate::model::conversion::wem::{actors, forces, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::common::{
    EventCategory, Feature as CommonFeature,
};
use crate::model::semantic::profiles::wem::{Bone, Feature};
use crate::model::wem::{records, RawDocument, Ref};

pub(super) trait WritePayload: Sized {
    fn supports(bone: Option<&Bone>, feature: Option<&Feature<'_>>) -> bool;
    fn write_payload<'view, 'data: 'view>(
        document: &mut RawDocument<'view>,
        bone: Option<&Bone>,
        feature: Option<&Feature<'data>>,
        context: &Context<'_>,
    ) -> Result<Self, Error>;
}
fn category(value: EventCategory) -> records::EventCategory {
    match value {
        EventCategory::Unspecified => records::EventCategory::None,
        EventCategory::Sound => records::EventCategory::Sound,
        EventCategory::Splat => records::EventCategory::Splat,
        EventCategory::Footprint => records::EventCategory::Footprint,
        EventCategory::LargeSplat => records::EventCategory::UberSplat,
        EventCategory::SpawnModel => records::EventCategory::SpawnModel,
        EventCategory::ShowMorph => records::EventCategory::MorphShow,
        EventCategory::HideMorph => records::EventCategory::MorphHide,
        EventCategory::Unknown(value) => records::EventCategory::Unknown(value),
    }
}
impl<D: records::NodePayloadDialect> WritePayload for records::NodePayload<D>
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
    fn supports(bone: Option<&Bone>, feature: Option<&Feature<'_>>) -> bool {
        if let Some(bone) = bone {
            return feature.is_none()
                && (bone.visibility_mesh.is_none() || D::GateMesh::write(Some(0)).is_ok());
        }
        match feature {
            Some(Feature::Common(CommonFeature::Light(value))) => {
                common::write_light::<D>(value).is_ok()
            }
            Some(Feature::Common(CommonFeature::Camera(value))) => {
                common::write_camera::<D>(value).is_ok()
            }
            Some(Feature::Common(CommonFeature::Event(value))) => {
                D::Category::write(value.code.as_ref().map(|value| category(value.category)))
                    .is_ok()
                    && D::Code::write(value.code.as_ref().map(|_| Ref::default())).is_ok()
            }
            Some(Feature::Force(value)) => D::Transform::write_matrix(value.transform).is_ok(),
            Some(Feature::Wow(value)) => {
                D::Texture2Scale::write(value.texture2_scale).is_ok()
                    && D::Texture3Scale::write(value.texture3_scale).is_ok()
                    && D::Texture2ScrollMid::write(value.texture2_scroll_mid).is_ok()
                    && D::Texture3ScrollMid::write(value.texture3_scroll_mid).is_ok()
                    && D::Texture2ScrollRange::write(value.texture2_scroll_range).is_ok()
                    && D::Texture3ScrollRange::write(value.texture3_scroll_range).is_ok()
                    && D::InheritVelocityScale::write(value.inherit_velocity_scale).is_ok()
                    && D::FollowSpeed1::write(value.follow_speed1).is_ok()
                    && D::FollowScale1::write(value.follow_scale1).is_ok()
                    && D::FollowSpeed2::write(value.follow_speed2).is_ok()
                    && D::FollowScale2::write(value.follow_scale2).is_ok()
                    && D::TumbleMin::write(value.tumble_min).is_ok()
                    && D::TumbleMax::write(value.tumble_max).is_ok()
                    && D::ColorMult::write(value.color_mult).is_ok()
                    && D::AlphaMult::write(value.alpha_mult).is_ok()
                    && D::AlphaCutoffTimes::write(
                        value.alpha_cutoff_times.as_ref().map(|_| Ref::default()),
                    )
                    .is_ok()
                    && D::AlphaCutoffs::write(value.alpha_cutoffs.as_ref().map(|_| Ref::default()))
                        .is_ok()
                    && D::ParticleModel::write(
                        value
                            .particle_model
                            .as_ref()
                            .map(|_| records::AssetKey::default()),
                    )
                    .is_ok()
                    && D::TrailModel::write(
                        value
                            .trail_model
                            .as_ref()
                            .map(|_| records::AssetKey::default()),
                    )
                    .is_ok()
            }
            _ => true,
        }
    }
    fn write_payload<'view, 'data: 'view>(
        document: &mut RawDocument<'view>,
        bone: Option<&Bone>,
        feature: Option<&Feature<'data>>,
        context: &Context<'_>,
    ) -> Result<Self, Error> {
        if let Some(bone) = bone {
            if feature.is_some() {
                return Err(Error::new(
                    "wem.node.payload",
                    "bone and attached feature compete for one payload",
                ));
            }
            let gate_mesh = bone
                .visibility_mesh
                .map(|id| index(context.meshes, &id))
                .transpose()?;
            return Ok(Self::Bone(Box::new(records::BonePayload {
                bounds: records::Extent {
                    minimum: bone.bounds.minimum,
                    maximum: bone.bounds.maximum,
                    sphere_radius: bone.bounds.sphere_radius,
                },
                sphere: records::Sphere {
                    center: bone.sphere_center,
                    radius: bone.sphere_radius,
                },
                gate_mesh: D::GateMesh::write(Some(gate_mesh.unwrap_or(u32::MAX))).or_else(
                    |error| {
                        if gate_mesh.is_none() {
                            D::GateMesh::write(None)
                        } else {
                            Err(error)
                        }
                    },
                )?,
            })));
        }
        Ok(match feature {
            None => Self::Helper(Box::new(records::HelperPayload {})),
            Some(Feature::Common(value)) => match value {
                CommonFeature::Light(value) => {
                    Self::Light(Box::new(common::write_light::<D>(value)?))
                }
                CommonFeature::Camera(value) => {
                    Self::Camera(Box::new(common::write_camera::<D>(value)?))
                }
                CommonFeature::Attachment(value) => {
                    Self::Attachment(Box::new(records::AttachmentPayload {
                        asset: actors::write_asset(document, &value.external)?,
                        model: value
                            .internal
                            .map(|id| index(context.models, &id))
                            .transpose()?
                            .unwrap_or(u32::MAX),
                    }))
                }
                CommonFeature::ParticleSystem(value) => {
                    Self::Particle(Box::new(records::ParticlePayload {
                        system: actors::write_asset(document, value)?,
                    }))
                }
                CommonFeature::RibbonSystem(value) => {
                    Self::Ribbon(Box::new(records::RibbonPayload {
                        system: actors::write_asset(document, value)?,
                    }))
                }
                CommonFeature::Event(value) => Self::Event(Box::new(records::EventPayload {
                    id: value.identity,
                    category: D::Category::write(
                        value.code.as_ref().map(|value| category(value.category)),
                    )?,
                    code: D::Code::write(
                        value
                            .code
                            .as_ref()
                            .map(|value| text::write_bytes(document, &value.code))
                            .transpose()?,
                    )?,
                })),
            },
            Some(Feature::Warcraft(value)) => {
                actors::write_warcraft::<D>(document, value, context.textures, context.slots)?
            }
            Some(Feature::Wow(value)) => Self::M2ParticleEmitter(Box::new(
                actors::wow::write::<D>(document, value, context.textures)?,
            )),
            Some(Feature::Starcraft(value)) => actors::starcraft::write::<D>(
                document,
                value,
                context.nodes,
                context.features,
                context.slots,
                context.sections,
            )?,
            Some(Feature::Collision(value)) => {
                Self::Collision(Box::new(forces::write_collision(value)))
            }
            Some(Feature::VertexWarp(value)) => {
                Self::VertexWarp(Box::new(forces::write_warp(value)))
            }
            Some(Feature::Force(value)) => {
                let value = forces::write_force(value);
                Self::ForceField(Box::new(records::ForceFieldPayload::<D> {
                    kind: value.kind,
                    volume: value.volume,
                    falloff: value.falloff,
                    height_gradient: value.height_gradient,
                    unbounded: value.unbounded,
                    affects_particles: value.affects_particles,
                    affects_bodies: value.affects_bodies,
                    channels: value.channels,
                    scope: value.scope,
                    strength: value.strength,
                    width: value.width,
                    height: value.height,
                    length: value.length,
                    transform: D::Transform::write_matrix(value.transform)?,
                }))
            }
        })
    }
}
