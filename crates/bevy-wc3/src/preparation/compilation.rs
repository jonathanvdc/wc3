//! Selective ownership conversion of decoded runtime records.
use super::shared::{RuntimeNode, Shared};
use wc3::model::animation::{GeosetAnimation, TextureAnimation};
use wc3::model::chunks::ModelChunk;
use wc3::model::emitters::{ParticleEmitter, ParticleEmitter2, RibbonEmitter};
use wc3::model::materials::Material;
use wc3::model::scene::{Attachment, Camera, EventObject, Light};
use wc3::model::{Model, V1800};

#[derive(Default)]
pub(super) struct RuntimeRecords {
    pub nodes: Vec<RuntimeNode>,
    pub materials: Vec<Shared<Material<'static, V1800>>>,
    pub texture_animations: Vec<Shared<TextureAnimation<'static>>>,
    pub geoset_animations: Vec<Shared<GeosetAnimation<'static>>>,
    pub cameras: Vec<Shared<Camera<'static, V1800>>>,
    pub lights: Vec<Shared<Light<'static, V1800>>>,
    pub attachments: Vec<Shared<Attachment<'static>>>,
    pub particles: Vec<Shared<ParticleEmitter<'static>>>,
    pub particles2: Vec<Shared<ParticleEmitter2<'static>>>,
    pub ribbons: Vec<Shared<RibbonEmitter<'static>>>,
    pub events: Vec<Shared<EventObject<'static>>>,
}

impl RuntimeRecords {
    pub fn consume(model: Model<'_, V1800>) -> Self {
        let mut result = Self::default();
        for chunk in model.chunks {
            macro_rules! definitions {
                ($chunk:expr, $field:ident) => {
                    result.$field.extend(
                        $chunk
                            .records
                            .into_iter()
                            .map(|record| Shared::new(record.into_owned())),
                    )
                };
            }
            macro_rules! nodes {
                ($chunk:expr, $($field:ident)?) => {
                    for record in $chunk.records {
                        let node = Shared::new(record$(.$field)?.into_owned());
                        result.nodes.push(RuntimeNode::new(node));
                    }
                };
            }
            macro_rules! features {
                ($chunk:expr, $field:ident) => {
                    for record in $chunk.records {
                        let record = Shared::new(record.into_owned());
                        result
                            .nodes
                            .push(RuntimeNode::new(record.map(|record| &record.node)));
                        result.$field.push(record);
                    }
                };
            }
            match chunk {
                ModelChunk::Helpers(c) => nodes!(c,),
                ModelChunk::Bones(c) => nodes!(c, node),
                ModelChunk::CollisionShapes(c) => nodes!(c, node),
                ModelChunk::PopcornEmitters(c) => nodes!(c, node),
                ModelChunk::Attachments(c) => features!(c, attachments),
                ModelChunk::Lights(c) => features!(c, lights),
                ModelChunk::EventObjects(c) => features!(c, events),
                ModelChunk::ParticleEmitters(c) => features!(c, particles),
                ModelChunk::ParticleEmitters2(c) => features!(c, particles2),
                ModelChunk::RibbonEmitters(c) => features!(c, ribbons),
                ModelChunk::Cameras(c) => definitions!(c, cameras),
                ModelChunk::Materials(c) => definitions!(c, materials),
                ModelChunk::TextureAnimations(c) => definitions!(c, texture_animations),
                ModelChunk::GeosetAnimations(c) => definitions!(c, geoset_animations),
                _ => {}
            }
        }
        result
    }
}
