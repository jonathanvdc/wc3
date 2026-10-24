//! Public source-scoped look selection through the ordinary Universal compiler.
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy_wc3::assembly::{
    compile_assembly, Assembly, Atlas, Models, Part, RgbaImage, Source, TextureReplacement,
};
use bevy_wc3::{compile_model_with_options, CompileOptions, JointMap, ModelError, PartBinding};
use wc3::model::materials::mdlx::{MaterialInputs, MaterialStep as MdlxStep, Shader};
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::profiles::Universal;
use wc3::model::{
    AddressMode, ColorSpace as ModelColorSpace, Draw, ExplicitBinding, GeometryData,
    GeometryInfluences, ImageResource, Influence, InstanceId, Look, LookBinding, LookId, Material,
    MaterialId, MaterialSelection, MaterialSurface, Mesh as ModelMesh, MeshInstance, Model,
    ModelMetadata, ModelMount, MountTarget, NodeId, ResourceLocator, Sampler as ModelSampler, Skin,
    Texture, UniversalBinding,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

struct Fixture {
    model: Model<'static>,
    node: NodeId,
    instance: InstanceId,
    looks: [LookId; 2],
    materials: [MaterialId; 2],
    texture: Option<Texture>,
}
fn resource(id: u32) -> ResourceLocator<'static> {
    ResourceLocator::FileDataId(id)
}
fn fixture(textured: bool) -> Fixture {
    fixture_with_sampling(
        textured,
        ModelColorSpace::Unspecified,
        ModelSampler::default(),
    )
}
fn fixture_with_sampling(
    textured: bool,
    color_space: ModelColorSpace,
    sampler: ModelSampler,
) -> Fixture {
    let mut values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("joint").unwrap();
    let texture = textured.then(|| {
        let mut texture = builder.add_file_texture("original-appearance.blp").unwrap();
        texture.color_space = color_space;
        texture.sampler = sampler;
        texture
    });
    let skin = builder
        .add_skin(Skin::new(
            vec![node],
            UniversalBinding::Explicit(ExplicitBinding::new([Mat4::IDENTITY]).unwrap()),
        ))
        .unwrap();
    let mut geometry =
        GeometryData::triangles([Vec3::ZERO, Vec3::X, Vec3::Y], [[0, 1, 2]]).unwrap();
    geometry.set_texture_coordinates(vec![[Vec2::ZERO, Vec2::X, Vec2::Y].into_iter().collect()]);
    let mut influences = [Influence {
        joint: 0,
        weight: 0.,
    }; 4];
    influences[0].weight = 1.;
    geometry.influences = GeometryInfluences::Four(vec![influences; 3].into_iter().collect());
    let mut mesh = ModelMesh::new(geometry);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let materials = [0.25, 0.75].map(|opacity| {
        let mut step = MdlxStep {
            shader: Shader::Classic(MaterialInputs {
                opacity: values.add(opacity),
                ..SlotConstruct::construct(&mut values)
            }),
            ..SlotConstruct::construct(&mut values)
        };
        step.shader.inputs_mut().binding.image = values.add(texture);
        builder
            .add_material(Material {
                name: format!("opacity-{opacity}").into(),
                surface: MaterialSurface::from(MdlxMaterial {
                    declaration: Default::default(),
                    steps: vec![step],
                }),
            })
            .unwrap()
    });
    let mut placement = MeshInstance::new(node, mesh);
    placement.skin = Some(skin);
    let draw = Draw::new(subset, Default::default());
    let draw_id = draw.id();
    placement.draws.push(draw);
    let instance = builder
        .add_instance_with_materials(placement, vec![materials[0]])
        .unwrap();
    let first = builder.model().default_look().unwrap();
    let second = builder
        .add_look(Look {
            name: "alternative".into(),
            bindings: vec![LookBinding {
                instance,
                draw: draw_id,
                material: materials[1],
            }],
        })
        .unwrap();
    builder.set_default_look(None).unwrap();
    builder.import_values(&values).unwrap();
    Fixture {
        model: builder.finish().unwrap(),
        node,
        instance,
        looks: [first, second],
        materials,
        texture,
    }
}
fn source(id: u32, look: LookId) -> Source {
    let mut source = Source::new(resource(id));
    source.material = MaterialSelection::Draw(look);
    source
}
fn pixels(_: &ImageResource<'static>) -> Result<RgbaImage, ModelError> {
    Ok(RgbaImage::new(1, 1, vec![255; 4]).unwrap())
}

#[test]
fn explicit_assembly_source_look_matches_direct_compilation_and_ambiguous_default() {
    let f = fixture(false);
    let models: Models<'_, '_> = [(resource(1), &f.model)].into();
    let model = models[&resource(1)];
    assert!(compile_model_with_options(model, CompileOptions::default(), |_| None).is_err());
    assert!(compile_assembly(
        &Assembly::new(Source::new(resource(1))),
        &models,
        &mut Assets::default(),
        pixels
    )
    .is_err());
    for (look, (material, alpha)) in f
        .looks
        .into_iter()
        .zip(f.materials.into_iter().zip([0.25, 0.75]))
    {
        let direct = compile_model_with_options(
            model,
            CompileOptions {
                material: MaterialSelection::Draw(look),
                ..Default::default()
            },
            |_| None,
        )
        .unwrap();
        let assembled = compile_assembly(
            &Assembly::new(source(1, look)),
            &models,
            &mut Assets::default(),
            pixels,
        )
        .unwrap();
        let mut meshes = Assets::default();
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        let direct = direct.prepare(&mut meshes, &mut binds);
        let assembled = assembled.prepare(&mut meshes, &mut binds);
        let a = direct.parts().next().unwrap();
        let b = assembled.parts().next().unwrap();
        assert_eq!(a.source.material, material);
        assert_eq!(a.source.material, b.source.material);
        assert_eq!(a.material.base.base_color, b.material.base.base_color);
        assert_eq!(a.material.base.base_color.alpha(), alpha);
        assert_eq!(
            meshes
                .get(a.mesh)
                .unwrap()
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3(),
            meshes
                .get(b.mesh)
                .unwrap()
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
        );
    }
}
#[test]
fn repeated_rigid_sources_keep_distinct_looks_and_share_identical_geometry() {
    let host = fixture(false);
    let child = fixture(false);
    let node = host.node;
    let host_look = host.looks[0];
    let looks = child.looks;
    let materials = child.materials;
    let models: Models<'_, '_> = [(resource(1), &host.model), (resource(2), &child.model)].into();
    let mut assembly = Assembly::new(source(1, host_look));
    for look in [looks[0], looks[1], looks[0]] {
        assembly.parts.push(Part::new(
            source(2, look),
            PartBinding::Mounted(ModelMount::new(MountTarget::Node(node))),
        ));
    }
    let compiled = compile_assembly(&assembly, &models, &mut Assets::default(), pixels).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mounts = prepared.mounts().collect::<Vec<_>>();
    assert_eq!(mounts.len(), 3);
    let parts = mounts
        .iter()
        .map(|mount| mount.model.parts().next().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        parts.iter().map(|p| p.source.material).collect::<Vec<_>>(),
        vec![materials[0], materials[1], materials[0]]
    );
    assert_eq!(
        parts
            .iter()
            .map(|p| p.material.base.base_color.alpha())
            .collect::<Vec<_>>(),
        vec![0.25, 0.75, 0.25]
    );
    assert!(parts.iter().all(|p| p.mesh.id() == parts[0].mesh.id()));
}
#[test]
fn shared_skeleton_repeats_coalesce_only_for_identical_material_choices() {
    let host = fixture(true);
    let child = fixture(false);
    let host_node = host.node;
    let child_node = child.node;
    let host_look = host.looks[0];
    let child_looks = child.looks;
    let models: Models<'_, '_> = [(resource(1), &host.model), (resource(2), &child.model)].into();
    let binding = PartBinding::Skinned {
        joints: JointMap::new([(child_node, host_node)]).unwrap(),
        transform: Mat4::IDENTITY,
    };
    let mut assembly = Assembly::new(source(1, host_look));
    assembly.parts = vec![
        Part::new(source(2, child_looks[0]), binding.clone()),
        Part::new(source(2, child_looks[0]), binding),
    ];
    let compiled = compile_assembly(&assembly, &models, &mut Assets::default(), pixels).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    assert_eq!(prepared.parts().len(), 2);
    assembly.parts[1].source.material = MaterialSelection::Draw(child_looks[1]);
    let mut decodes = 0;
    let result = compile_assembly(&assembly, &models, &mut Assets::default(), |image| {
        decodes += 1;
        pixels(image)
    });
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("incompatible material selections"));
    assert_eq!(decodes, 0, "conflicts must precede root texture decoding");
}
#[test]
fn foreign_look_is_rejected_even_when_its_source_geometry_is_hidden() {
    let host = fixture(true);
    let child = fixture(false);
    let foreign = host.looks[0];
    let node = host.node;
    let host_look = host.looks[1];
    let child_instance = child.instance;
    let models: Models<'_, '_> = [(resource(1), &host.model), (resource(2), &child.model)].into();
    let mut hidden = source(2, foreign);
    hidden.draws.hidden_instances.insert(child_instance);
    let mut assembly = Assembly::new(source(1, host_look));
    assembly.parts.push(Part::new(
        hidden,
        PartBinding::Mounted(ModelMount::new(MountTarget::Node(node))),
    ));
    let mut decodes = 0;
    let result = compile_assembly(&assembly, &models, &mut Assets::default(), |image| {
        decodes += 1;
        pixels(image)
    });
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("foreign material look"));
    assert_eq!(decodes, 0);
}
#[test]
fn default_and_explicit_default_look_coalesce_and_share_images() {
    let host = fixture(false);
    let mut child = fixture(true);
    let default = child.looks[0];
    let child_material = child.materials[0];
    child
        .model
        .edit(|draft| draft.set_default_look(Some(default)))
        .unwrap();
    let node = host.node;
    let child_node = child.node;
    let mut assembly = Assembly::new(source(1, host.looks[0]));
    let binding = PartBinding::Skinned {
        joints: JointMap::new([(child_node, node)]).unwrap(),
        transform: Mat4::IDENTITY,
    };
    assembly.parts = vec![
        Part::new(Source::new(resource(2)), binding.clone()),
        Part::new(source(2, default), binding),
        Part::new(
            Source::new(resource(2)),
            PartBinding::Mounted(ModelMount::new(MountTarget::Node(node))),
        ),
        Part::new(
            source(2, default),
            PartBinding::Mounted(ModelMount::new(MountTarget::Node(node))),
        ),
    ];
    let models: Models<'_, '_> = [(resource(1), &host.model), (resource(2), &child.model)].into();
    let mut images = Assets::default();
    let mut decodes = 0;
    let compiled = compile_assembly(&assembly, &models, &mut images, |image| {
        decodes += 1;
        pixels(image)
    })
    .unwrap();
    assert_eq!(decodes, 1);
    // Skinned draw union and whole-source rigid geometry compile separately,
    // but the same physical image interpretation is shared across both.
    assert_eq!(images.len(), 1);
    let prepared = compiled.prepare(&mut Assets::default(), &mut Assets::default());
    assert_eq!(
        prepared.parts().len(),
        2,
        "the two skinned requests coalesce"
    );
    let mounts = prepared.mounts().collect::<Vec<_>>();
    assert_eq!(mounts.len(), 2);
    let a = mounts[0].model.parts().next().unwrap();
    let b = mounts[1].model.parts().next().unwrap();
    assert_eq!(a.source.material, child_material);
    assert_eq!(b.source.material, child_material);
    assert_eq!(a.material.base.base_color.alpha(), 0.25);
    assert_eq!(b.material.base.base_color.alpha(), 0.25);
    assert!(a.material.base.base_color_texture.is_some());
    assert_eq!(
        a.material.base.base_color_texture,
        b.material.base.base_color_texture
    );
    assert_eq!(a.mesh.id(), b.mesh.id());
    assert_eq!(
        assembly.parts[0].source.material,
        MaterialSelection::Default
    );
    assert_eq!(
        assembly.parts[2].source.material,
        MaterialSelection::Default
    );
}

#[test]
fn assembly_image_sharing_preserves_sampling_color_and_replacement_identity() {
    let host = fixture(false);
    let node = host.node;
    let mut assembly = Assembly::new(source(1, host.looks[0]));
    let mut owners = vec![(resource(1), host.model)];
    let repeat = ModelSampler::default();
    let clamp = ModelSampler {
        u: AddressMode::Clamp,
        ..repeat
    };
    let atlas = TextureReplacement::Atlas {
        atlas: Atlas {
            width: 1,
            height: 1,
            clear: [20, 40, 60, 255],
        },
        layers: Vec::new(),
    };
    let replacement = ImageResource::from(resource(99));
    let cases = [
        (ModelColorSpace::Unspecified, repeat, None),
        (ModelColorSpace::Srgb, repeat, None),
        (ModelColorSpace::Srgb, clamp, None),
        (ModelColorSpace::Linear, repeat, None),
        (
            ModelColorSpace::Srgb,
            repeat,
            Some(TextureReplacement::Image(replacement.clone())),
        ),
        (ModelColorSpace::Srgb, repeat, Some(atlas.clone())),
        (ModelColorSpace::Srgb, repeat, Some(atlas)),
    ];
    for (index, (color, sampler, replacement)) in cases.into_iter().enumerate() {
        let child = fixture_with_sampling(true, color, sampler);
        let id = index as u32 + 2;
        let mut selection = source(id, child.looks[0]);
        if let Some(replacement) = replacement {
            selection
                .images
                .insert(child.texture.unwrap().resource, replacement);
        }
        owners.push((resource(id), child.model));
        assembly.parts.push(Part::new(
            selection,
            PartBinding::Mounted(ModelMount::new(MountTarget::Node(node))),
        ));
    }
    let models: Models<'_, '_> = owners
        .iter()
        .map(|(id, model)| (id.clone(), model))
        .collect();
    let mut images = Assets::default();
    let mut decoded = Vec::new();
    let compiled = compile_assembly(&assembly, &models, &mut images, |image| {
        decoded.push(image.clone());
        let rgba = if *image == replacement {
            vec![80, 100, 120, 255]
        } else {
            vec![255; 4]
        };
        Ok(RgbaImage::new(1, 1, rgba).unwrap())
    })
    .unwrap();
    // Seven independent source graphs need five GPU interpretations, while
    // decoded original pixels are shared across sampler and transfer variants.
    assert_eq!(decoded.len(), 2);
    assert_eq!(
        decoded
            .iter()
            .filter(|image| **image == replacement)
            .count(),
        1
    );
    assert_eq!(images.len(), 5);
    let prepared = compiled.prepare(&mut Assets::default(), &mut Assets::default());
    let handles = prepared
        .mounts()
        .map(|mount| {
            mount
                .model
                .parts()
                .next()
                .unwrap()
                .material
                .base
                .base_color_texture
                .clone()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        handles[0], handles[1],
        "effective color space, not declaration spelling, selects reuse"
    );
    assert_eq!(
        handles[5], handles[6],
        "equal atlases across sources share composition"
    );
    assert_eq!(
        images.get(&handles[0]).unwrap().texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(
        images.get(&handles[3]).unwrap().texture_descriptor.format,
        TextureFormat::Rgba8Unorm
    );
    for (position, &index) in [0, 2, 3, 4, 5].iter().enumerate() {
        for &other in &[0, 2, 3, 4, 5][position + 1..] {
            assert_ne!(handles[index], handles[other]);
        }
    }
    assert_eq!(
        images.get(&handles[5]).unwrap().data.as_deref(),
        Some([20, 40, 60, 255].as_slice())
    );
}
