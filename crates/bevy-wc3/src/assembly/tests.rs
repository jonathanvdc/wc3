use super::plan::plan;
use super::*;
use bevy::prelude::{Assets, Image};
use wc3::model::materials::layered::LayeredSurface;
use wc3::model::profiles::Universal;
use wc3::model::{
    Draw, DrawId, GeometryData, InstanceId, Material, MaterialSurface, Mesh, MeshInstance, Node,
};

fn model() -> (Model<'static>, Vec<(InstanceId, DrawId)>) {
    let mut builder = Model::builder("generic assembly geometry");
    let node = builder.add_node(Node::named("root")).unwrap();
    let material = builder
        .add_material(Material {
            name: "generic".into(),
            surface: MaterialSurface::Layered(Box::new(LayeredSurface::default())),
        })
        .unwrap();
    let mut ids = Vec::new();
    for _ in 0..2 {
        let geometry =
            GeometryData::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]])
                .unwrap();
        let mut mesh = Mesh::new(geometry);
        let subset = mesh.add_primary_subset(0, 1);
        let mesh = builder.add_mesh(mesh).unwrap();
        let mut instance = MeshInstance::new(node, mesh);
        let draw = Draw::new(subset, Default::default());
        let draw_id = draw.id();
        instance.draws.push(draw);
        let instance_id = builder
            .add_instance_with_materials(instance, vec![material])
            .unwrap();
        ids.push((instance_id, draw_id));
    }
    (builder.finish().unwrap(), ids)
}
#[test]
fn compatible_union_preserves_hidden_geometry_and_all_selection() {
    let (model, ids) = model();
    let models = [(Resource::FileDataId(1), model)].into_iter().collect();
    let source = Source::new(Resource::FileDataId(1));
    let mut a = PendingPart::new(source.clone(), true);
    a.source.draws.hidden_instances.insert(ids[0].0);
    let mut b = PendingPart::new(source.clone(), true);
    b.source.draws.draws = Some([ids[0].1].into_iter().collect());
    b.source.draws.hidden_instances.insert(ids[1].0);
    let assembly = PendingAssembly {
        root: source,
        parts: vec![a, b],
    };
    let result = plan(&assembly, &models, |b| *b).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].source.draws.draws.as_ref().unwrap().len(), 2);
    assert!(result[0].source.draws.hidden_instances.is_empty());
}
#[test]
fn incompatible_shared_textures_fail_before_image_provider() {
    let (model, _) = model();
    let texture = Model::<Universal>::builder("foreign")
        .add_texture(wc3::model::Texture::file("foreign"))
        .unwrap();
    let models = [(Resource::FileDataId(1), model)].into_iter().collect();
    let mut source = Source::new(Resource::FileDataId(1));
    source
        .textures
        .insert(texture, TextureReplacement::Image(Resource::FileDataId(99)));
    let assembly = Assembly::new(source);
    let mut calls = 0;
    let result = compile_assembly(&assembly, &models, &mut Assets::<Image>::default(), |_| {
        calls += 1;
        unreachable!()
    });
    assert!(result.is_err());
    assert_eq!(calls, 0);
}
#[test]
fn only_repeated_skinned_sources_coalesce() {
    let (a, _) = model();
    let (b, _) = model();
    let models = [(Resource::FileDataId(1), a), (Resource::FileDataId(2), b)]
        .into_iter()
        .collect();
    let one = Source::new(Resource::FileDataId(1));
    let two = Source::new(Resource::FileDataId(2));
    let assembly = PendingAssembly {
        root: one.clone(),
        parts: vec![
            PendingPart::new(one.clone(), false),
            PendingPart::new(one, false),
            PendingPart::new(two, true),
        ],
    };
    assert_eq!(plan(&assembly, &models, |b| *b).unwrap().len(), 3);
}
