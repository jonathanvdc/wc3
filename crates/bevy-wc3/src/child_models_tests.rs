use super::*;
use wc3::model::common::ZeroVec;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::m2::records::{
    Bone, ModelFlags, ParticleEmitter as NativeParticle, Texture, TextureSource,
};
use wc3::model::formats::m2::{Bundle, Document, Flat, V256};
use wc3::model::io::m2 as semantic_m2;
use wc3::model::profiles::Universal;
use wc3::model::ModelMetadata;

fn dual_resources() -> Bundle<'static, Flat<V256>> {
    let mut document = Document::<Flat<V256>>::new();
    let core = document.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"dual child roles\0");
    core.bones = vec![Bone::new()];
    core.textures = vec![Texture {
        kind: TextureSource::File,
        filename: ZeroVec::alloc_from_slice(b"white.tga\0"),
        ..Default::default()
    }];
    let particle = NativeParticle::new(ModelFlags::default());

    core.particles.push(particle);
    Bundle::from_model(document)
}
fn declare_dual_resources(model: &mut Model<'_>) {
    use wc3::model::ResourceLocator;
    let (feature, mut payload) = model
        .features()
        .iter()
        .find_map(|(id, feature)| {
            matches!(feature.payload, NodeFeature::Particle(_))
                .then(|| (id, feature.payload.clone()))
        })
        .unwrap();
    model
        .edit(|builder| {
            let geometry =
                builder.add_model_resource(ResourceLocator::Path(b"geometry.m2".as_slice().into()));
            let emitters =
                builder.add_model_resource(ResourceLocator::Path(b"emitters.m2".as_slice().into()));
            let NodeFeature::Particle(ParticleEmitter::M2(value)) = &mut payload else {
                unreachable!()
            };
            value.details.rendering.model = Some(geometry);
            value.details.rendering.child_model = Some(emitters);
            builder.set_feature(feature, payload)
        })
        .unwrap();
}

#[test]
fn one_universal_emitter_keeps_both_authored_roles_and_coordinate_policies() {
    let source = dual_resources();
    let mut model = semantic_m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    declare_dual_resources(&mut model);
    let requests = requests(&model);
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].0.feature, requests[1].0.feature);
    assert_ne!(requests[0].1, requests[1].1);
    assert_eq!(requests[0].0.role, ChildModelRole::ParticleGeometry);
    assert_eq!(requests[1].0.role, ChildModelRole::ParticleEmitters);
    let bindings = HashMap::from([
        (requests[0].0, ChildModelBinding::new(Handle::default())),
        (
            requests[1].0,
            ChildModelBinding::new(Handle::default()).with_coordinates(CoordinatePolicy::Child),
        ),
    ]);
    validate(&model, &bindings).unwrap();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[&requests[0].0].coordinates, CoordinatePolicy::Host);
    assert_eq!(
        bindings[&requests[1].0].coordinates,
        CoordinatePolicy::Child
    );
}
#[test]
fn binding_keys_reject_foreign_ownership_and_mismatched_roles() {
    let source = dual_resources();
    let mut model = semantic_m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    declare_dual_resources(&mut model);
    let request = requests(&model)[0].0;
    let wrong = HashMap::from([(
        ChildModelKey::attachment(request.feature),
        ChildModelBinding::new(Handle::default()),
    )]);
    assert!(validate(&model, &wrong)
        .unwrap_err()
        .to_string()
        .contains("does not match"));
    let mut other = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = other.reserve_node();
    let feature = other.reserve_feature(node);
    let foreign = HashMap::from([(
        ChildModelKey::particle_geometry(feature),
        ChildModelBinding::new(Handle::default()),
    )]);
    assert!(validate(&model, &foreign)
        .unwrap_err()
        .to_string()
        .contains("foreign feature"));
}
