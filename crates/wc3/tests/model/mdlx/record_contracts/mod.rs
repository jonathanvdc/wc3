use wc3::model::formats::mdlx::animation::ValueKeyframe;
use wc3::model::formats::mdlx::chunks::{GlidersChunk, ModelChunk, RawChunk};
use wc3::model::formats::mdlx::emitters::{ParticleEmitter2, PopcornEmitter};
use wc3::model::formats::mdlx::materials::{
    Layer, LayerShadingFlags, Material, MaterialRenderFlags, ShaderType,
};
use wc3::model::formats::mdlx::mdl::{Read as _, Write as _};
use wc3::model::formats::mdlx::mdx::{Read as _, Write as _};
use wc3::model::formats::mdlx::scene::{Camera, EventObject, Glider, Light, LightFalloff, Node};
use wc3::model::formats::mdlx::CommonModelAccess;
use wc3::model::formats::mdlx::{ConversionOptions, Document, DynamicDocument, V1100, V1800, V800};

mod cameras;
mod defaults;
mod gliders;
mod materials;
mod signed_fields;
