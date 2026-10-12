use wc3::model::mdlx::animation::ValueKeyframe;
use wc3::model::mdlx::chunks::{GlidersChunk, ModelChunk, RawChunk};
use wc3::model::mdlx::emitters::{ParticleEmitter2, PopcornEmitter};
use wc3::model::mdlx::materials::{
    Layer, LayerShadingFlags, Material, MaterialRenderFlags, ShaderType,
};
use wc3::model::mdlx::mdl::{Read as _, Write as _};
use wc3::model::mdlx::mdx::{Read as _, Write as _};
use wc3::model::mdlx::scene::{Camera, EventObject, Glider, Light, LightFalloff, Node};
use wc3::model::mdlx::CommonModelAccess;
use wc3::model::mdlx::{ConversionOptions, DynamicModel, Model, V1100, V1800, V800};

mod cameras;
mod defaults;
mod gliders;
mod materials;
mod signed_fields;
