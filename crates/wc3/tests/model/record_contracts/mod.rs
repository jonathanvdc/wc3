use wc3::model::animation::{AnimationTrack, CameraVisibility, ValueKeyframe};
use wc3::model::chunks::{GlidersChunk, ModelChunk, RawChunk};
use wc3::model::emitters::{ParticleEmitter2, PopcornEmitter};
use wc3::model::materials::{Layer, LayerShadingFlags, Material, MaterialRenderFlags, ShaderType};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{Camera, CameraTrack, EventObject, Glider, Light, LightFalloff, Node};
use wc3::model::CommonModelAccess;
use wc3::model::{ConversionOptions, DynamicModel, Model, V1100, V1800, V800};

mod cameras;
mod defaults;
mod gliders;
mod materials;
mod signed_fields;
