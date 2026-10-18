use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy_wc3::assembly::{
    compile_assembly, Assembly, Atlas, Models, Part, PixelRect, Resolver, RgbaImage, Source,
    TextureLayer, TextureReplacement,
};
use bevy_wc3::{BakeSpace, ModelError, PartBinding, PoseBaker, PoseOptions};
use wc3::model::materials::mdlx::{Layer, LayeredSurface};
use wc3::model::ModelBuilder;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{
    Draw, GeometryData, ImageResource, Material, MaterialSurface, Mesh as ModelMesh, MeshInstance,
    Model, ModelMount, MountTarget, NodeId, ResourceLocator, Texture, Transform as ModelTransform,
};

fn resource(name: &'static str) -> ResourceLocator<'static> {
    ResourceLocator::Named {
        namespace: "workshop".into(),
        name: name.into(),
    }
}
fn model(textured: bool) -> (Model<'static>, NodeId, Option<Texture>) {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("mounting frame").unwrap();
    let texture = textured.then(|| builder.add_file_texture("original.png").unwrap());
    let mut geometry =
        GeometryData::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]).unwrap();
    geometry.set_texture_coordinates(vec![[[0., 0.], [1., 0.], [0., 1.]].into_iter().collect()]);
    let mut mesh = ModelMesh::new(geometry);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let mut layer = Layer {
        opacity: slot_values.add(1.),
        ..SlotConstruct::construct(&mut slot_values)
    };
    layer.binding.image = slot_values.add(texture);
    let material = builder
        .add_material(Material {
            name: "paint".into(),
            surface: MaterialSurface::try_from(LayeredSurface {
                layers: vec![layer],
                ..SlotConstruct::construct(&mut slot_values)
            })
            .unwrap(),
        })
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.draws.push(Draw::new(subset, Default::default()));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    (
        {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap(),
        node,
        texture,
    )
}

// These choices belong to the application; the renderer imposes no inventory,
// body-category or customization vocabulary on them.
enum Chassis {
    Tripod,
}
enum Module {
    Sensor(Vec3),
    Counterweight(Vec3),
}
struct Configuration {
    chassis: Chassis,
    modules: Vec<Module>,
}
struct Workshop {
    mounting_frame: NodeId,
    paint: Texture,
}
impl Resolver<Configuration> for Workshop {
    type Error = ModelError;
    fn resolve(&self, selection: &Configuration, _: &Models<'_>) -> Result<Assembly, ModelError> {
        let root = match selection.chassis {
            Chassis::Tripod => Source::new(resource("chassis")),
        };
        let mut assembly = Assembly::new(root);
        for module in &selection.modules {
            let offset = match module {
                Module::Sensor(offset) | Module::Counterweight(offset) => *offset,
            };
            let mut source = Source::new(resource("module"));
            source.images.insert(
                self.paint.resource,
                TextureReplacement::Atlas {
                    atlas: Atlas {
                        width: 1,
                        height: 1,
                        clear: [0; 4],
                    },
                    layers: vec![TextureLayer {
                        resource: resource("paint").into(),
                        source: None,
                        destination: PixelRect {
                            x: 0,
                            y: 0,
                            width: 1,
                            height: 1,
                        },
                        opacity: 255,
                    }],
                },
            );
            assembly.parts.push(Part::new(
                source,
                PartBinding::Mounted(ModelMount {
                    transform: ModelTransform {
                        translation: offset.to_array(),
                        ..Default::default()
                    },
                    ..ModelMount::new(MountTarget::Node(self.mounting_frame))
                }),
            ));
        }
        Ok(assembly)
    }
}

#[test]
fn application_defined_machine_configuration_compiles_without_native_character_metadata() {
    let mut slot_values = SlotValues::default();
    let (chassis, mounting_frame, _) = model(false);
    let (module, _, paint) = model(true);
    assert!(chassis
        .nodes()
        .iter()
        .all(|(_, node)| node.controls.is_none()));
    assert!(module
        .nodes()
        .iter()
        .all(|(_, node)| node.controls.is_none()));
    let models: Models<'_> = [(resource("chassis"), chassis), (resource("module"), module)].into();
    let selection = Configuration {
        chassis: Chassis::Tripod,
        modules: vec![
            Module::Sensor(Vec3::X),
            Module::Sensor(Vec3::Y),
            Module::Counterweight(Vec3::Z),
        ],
    };
    let plan = Workshop {
        mounting_frame,
        paint: paint.unwrap(),
    }
    .resolve(&selection, &models)
    .unwrap();
    let mut images = Assets::default();
    let mut decodes = 0;
    let compiled = compile_assembly(&plan, &models, &mut images, |source| {
        assert_eq!(*source, ImageResource::from(resource("paint")));
        decodes += 1;
        Ok(RgbaImage::new(1, 1, vec![240, 40, 20, 255]).unwrap())
    })
    .unwrap();
    assert_eq!(decodes, 1);
    assert_eq!(images.len(), 1);
    let mut meshes = Assets::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mounts: Vec<_> = prepared.mounts().collect();
    assert_eq!(mounts.len(), 3);
    let shared_meshes: Vec<_> = mounts
        .iter()
        .map(|mount| mount.model.parts().next().unwrap().mesh.id())
        .collect();
    assert_eq!(shared_meshes, vec![shared_meshes[0]; 3]);
    let baked = PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                space: BakeSpace::Authored,
                ..SlotConstruct::construct(&mut slot_values)
            },
        )
        .unwrap();
    assert_eq!(baked.parts.len(), 4);
    for (part, expected) in baked.parts[1..].iter().zip([Vec3::X, Vec3::Y, Vec3::Z]) {
        assert!(part.visible);
        let positions = part
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        assert!(Vec3::from_array(positions[0]).abs_diff_eq(expected, 1e-5));
    }
}
