use wc3::model::emitters::{Particle2FilterMode, Particle2Frames};
use wc3::model::emitters::{ParticleEmitter2, PopcornEmitter};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::{Camera, CameraVariant};
use wc3::model::{
    mdl, DynamicModel, Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800,
    V900,
};

const SMOKE: &str = include_str!("../../../fixtures/mdl/smoke.mdl");
const POPCORN: &str = include_str!("../../../fixtures/mdl/popcorn_fire.mdl");
const CAMERA: &str = include_str!("../../../fixtures/mdl/portrait.mdl");

fn whole_model<V: ModelVersion>() {
    let popcorn = if V::NUMBER >= 900 { POPCORN } else { "" };
    let source = format!(
        "Version {{ FormatVersion {}, }} Model \"All\" {{}} {SMOKE} {CAMERA} {popcorn}",
        V::NUMBER
    );
    let record = Model::<V>::decode_mdl(&source).unwrap();
    let text = record.encode_mdl().unwrap();
    assert_eq!(
        DynamicModel::decode_mdl(&text)
            .unwrap()
            .encode_mdl()
            .unwrap(),
        text
    );
    let decoded = Model::<V>::decode_mdl(&text).unwrap();
    assert_eq!(decoded.cameras().len(), 1);
    assert_eq!(decoded.particle_emitters2().len(), 1);
    assert_eq!(
        Model::<V>::decode_mdx(&decoded.encode_mdx().unwrap())
            .unwrap()
            .encode_mdl()
            .unwrap(),
        text
    );
}

mod cameras;
mod fixtures;
mod model_io;
mod particle2;
mod popcorn;
mod tracks;
