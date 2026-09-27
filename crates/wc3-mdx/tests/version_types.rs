use wc3_mdx::chunks::{ModelChunk, RawChunk, UnknownChunk};
use wc3_mdx::geometry::Geoset;
use wc3_mdx::io::{DecodeError, Readable, ValueError, Writable};
use wc3_mdx::materials::{Layer, Material};
use wc3_mdx::scene::{Camera, CameraVariant, Light, Node};
use wc3_mdx::{AnyVersionModel, Model, V1000, V1800, V800, V900};

#[test]
fn version_is_shared_by_model_and_nested_records() {
    let mut material = Material::<V1800>::new();
    material.set_layers(&[Layer::<V1800>::new()]);

    let mut model = Model::<V1800>::new();
    model.set_materials(&[material]);
    model.set_geosets(&[Geoset::<V1800>::new(&[], &[], &[]).unwrap()]);
    model.set_lights(&[Light::<V1800>::new(Node::new("Lamp", 1).unwrap(), 0)]);
    model.set_cameras(&[Camera::<V1800>::new("View").unwrap()]);

    let bytes = model.encode().unwrap();
    let parsed = Model::<V1800>::decode(&bytes).unwrap();
    assert_eq!(parsed.materials().len(), 1);
    assert_eq!(parsed.geosets().len(), 1);
    assert_eq!(parsed.lights().len(), 1);
    assert_eq!(parsed.cameras()[0].variant(), CameraVariant::Variant3);
    assert!(matches!(
        AnyVersionModel::decode(&bytes, 800),
        Ok(AnyVersionModel::V1800(_))
    ));
}

#[test]
fn unknown_tags_are_explicit_and_known_tags_cannot_be_disguised() {
    assert!(UnknownChunk::<V800>::new(RawChunk::new(*b"VERS", vec![0; 4])).is_none());
    let unknown = UnknownChunk::<V800>::new(RawChunk::new(*b"FUTR", vec![1, 2])).unwrap();
    let mut model = Model::<V800>::new();
    model.push(ModelChunk::Unknown(unknown));
    let bytes = model.encode().unwrap();
    assert_eq!(
        Model::<V800>::decode(&bytes).unwrap().encode().unwrap(),
        bytes
    );
}

#[test]
fn version_markers_select_record_fields() {
    let classic = Material::<V800>::new();
    assert!(classic.try_shader().is_err());
    assert!(Layer::<V800>::new().try_emissive_gain().is_err());
    assert!(matches!(
        Material::<V1800>::new().try_shader(),
        Err(ValueError::UnsupportedField { actual: 1800, .. })
    ));

    let mut shader = Material::<V900>::new();
    shader.set_shader("Shaders\\Unit.shader").unwrap();
    assert_eq!(shader.shader().as_ref(), "Shaders\\Unit.shader");

    let mut layer = Layer::<V1000>::new();
    layer.set_emissive_gain(0.5);
    layer.set_fresnel_opacity(0.25);
    assert_eq!(layer.emissive_gain(), 0.5);
    assert_eq!(layer.fresnel_opacity(), 0.25);

    let classic_light = Light::<V800>::new(Node::new("Classic", 1).unwrap(), 0);
    let modern_light = Light::<V1800>::new(Node::new("Modern", 2).unwrap(), 0);
    assert!(classic_light.try_extended_words().is_err());
    assert_eq!(modern_light.extended_words(), [0; 7]);
    assert!(modern_light.try_extended_words().is_ok());

    let classic_geoset = Geoset::<V800>::new(&[], &[], &[]).unwrap();
    let mut modern_geoset = Geoset::<V1800>::new(&[], &[], &[]).unwrap();
    assert!(classic_geoset.try_tangents().is_err());
    assert_eq!(modern_geoset.try_tangents().unwrap(), None);
    assert_eq!(modern_geoset.try_skin_bone_indices().unwrap(), None);
    modern_geoset.set_skin_weights(Some(&[]));
    assert_eq!(modern_geoset.skin_weights(), Some([].as_slice()));
}

#[test]
fn runtime_dispatch_rejects_versions_without_a_layout() {
    let mut bytes = Model::<V800>::new().encode().unwrap();
    bytes[12..16].copy_from_slice(&777u32.to_le_bytes());
    assert!(matches!(
        AnyVersionModel::decode(&bytes, 800),
        Err(DecodeError::UnsupportedVersion { version: 777 })
    ));
}
