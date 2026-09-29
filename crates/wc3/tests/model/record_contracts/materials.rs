use super::*;

#[test]
fn named_shader_mapping_and_new_flags_preserve_raw_storage() {
    for (id, name) in [
        (0, "Shader_SD_Legacy"),
        (1, "Shader_HD_DefaultUnit"),
        (2, "Shader_SD_FixedFunction"),
        (24, "Shader_HD_Crystal"),
    ] {
        let shader = ShaderType::new(id);
        assert_eq!(shader.id(), id);
        assert_eq!(shader.name(), Some(name));
        assert_eq!(
            ShaderType::from_name(&name.to_ascii_lowercase()),
            Some(shader)
        );
        let mut layer = Layer::<V1100>::new();
        layer.set_shader_type(shader);
        assert_eq!(
            Layer::<V1100>::decode_mdx(&layer.encode_mdx().unwrap())
                .unwrap()
                .shader_type(),
            shader
        );
    }
    assert_eq!(ShaderType::default(), ShaderType::SD_LEGACY);
    for raw in [3u32, u32::MAX] {
        let bytes = raw.to_le_bytes();
        let shader = ShaderType::decode_mdx(&bytes).unwrap();
        assert_eq!(shader, ShaderType::new(raw));
        assert_eq!(shader.name(), None);
        assert_eq!(shader.encode_mdx().unwrap(), bytes);
        let mut layer = Layer::<V1100>::new();
        layer.set_shader_type(shader);
        let wire = layer.encode_mdx().unwrap();
        assert_eq!(
            Layer::<V1100>::decode_mdx(&wire).unwrap().shader_type(),
            shader
        );
    }
    assert!(ShaderType::new(3).name().is_none());
    assert!(ShaderType::from_name("typo").is_none());
    let mut flags = LayerShadingFlags(0);
    flags.set_wrap_width(true);
    flags.set_wrap_height(true);
    flags.set_back_faces_for_shadows(true);
    flags.set_ambient_occlusion(true);
    assert_eq!(flags.bits(), 0x60c);
    let mut layer = Layer::<V800>::new();
    layer.shading_flags = flags;
    assert_eq!(
        Layer::<V800>::decode_mdx(&layer.encode_mdx().unwrap())
            .unwrap()
            .shading_flags,
        flags
    );
    let mut material_flags = MaterialRenderFlags(0);
    material_flags.set_two_sided(true);
    material_flags.set_sort_primitives_near_z(true);
    assert_eq!(material_flags.bits(), 0xa);
}
