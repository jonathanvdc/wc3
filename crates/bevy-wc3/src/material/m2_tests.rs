use super::*;
use crate::material::native_fixtures::m2;
use bevy::math::{Vec3, Vec4};
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use wc3::model::values::model::BatchFlags;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
fn surface(
    slot_values: &mut SlotValues,
    shader: u16,
    coordinates: &[i16],
) -> m2::Material<'static> {
    m2::Material {
        shader,
        color: slot_values.add(Vec3::from_array([0.25, 0.5, 0.75])),
        opacity: slot_values.add(0.8),
        textures: coordinates
            .iter()
            .map(|&source| {
                let mut unit = m2::TextureUnit::construct(slot_values);
                unit.coordinates.source = source;
                unit.contribution = slot_values.add(0.25);
                unit
            })
            .collect(),
        ..SlotConstruct::construct(slot_values)
    }
}

fn compile(surface: &m2::Material<'_>, slot_values: &SlotValues) -> SurfacePass {
    let model = {
        let mut draft = Model::<wc3::model::profiles::Universal>::builder(ModelMetadata::M2(
            Default::default(),
        ));
        draft.import_values(slot_values).unwrap();
        draft.finish().unwrap()
    };
    compile_m2(&model, &surface.clone().into(), &TextureImages::default()).unwrap()
}

#[test]
fn programs_require_exact_input_counts_and_do_not_wrap_unknown_tables() {
    for offset in (0..=29).chain([32, 33, 34, 35]) {
        let count = match offset {
            3 | 9 | 17 | 18 | 20 | 24 | 26 | 28 => 3,
            19 | 25 | 32 | 34 => 1,
            22 => 4,
            _ => 2,
        };
        assert!(program(0x8000 + offset, count).is_some());
        assert!(program(0x8000 + offset, count - 1).is_none());
        assert!(program(0x8000 + offset, count + 1).is_none());
    }
    assert!(program(0x8024, 2).is_none());
    assert_eq!(program(0x8000, 2).unwrap().environment, [false, true]);
    assert_eq!(program(0x88, 2).unwrap().environment, [true, true]);
}

#[test]
fn batch_flag_skips_global_weight_without_losing_shader_contribution() {
    let mut slot_values = SlotValues::default();
    let mut shader = surface(&mut slot_values, 0x8010, &[0, 0]);
    let weighted = compile(&shader, &slot_values);
    assert!((weighted.material.base_color.alpha() - 0.2).abs() < 1e-6);
    assert!(weighted.animation.weight.is_some());

    shader.batch_flags = BatchFlags::from_bits_retain(0x40);
    let independent = compile(&shader, &slot_values);
    assert!((independent.material.base_color.alpha() - 0.8).abs() < 1e-6);
    assert!(independent.animation.weight.is_none());
    assert_eq!(independent.state.combiner.weights.x, 0.25);
    assert_eq!(
        slot_values[independent.animation.input_weights[0].as_ref().unwrap()],
        0.25
    );
}

#[test]
fn armor_alpha_mask_preserves_authored_opacity_and_generated_sampling() {
    let mut slot_values = SlotValues::default();
    let shader = surface(&mut slot_values, 0x8000, &[0, 0]);
    let pass = compile(&shader, &slot_values);
    assert_eq!(pass.state.combiner.mode.x, 2);
    assert_eq!(pass.state.combiner.coordinates.x, 0);
    assert_eq!(pass.state.combiner.coordinates.y, 1);
    assert!((pass.material.base_color.alpha() - 0.2).abs() < 1e-6);
    assert!(pass.diagnostics.is_empty());
}

#[test]
fn generated_inputs_ignore_authored_transforms_but_uv_inputs_keep_them() {
    let mut slot_values = SlotValues::default();
    let mut shader = surface(&mut slot_values, 0x8000, &[0, 0]);
    for input in &mut shader.textures {
        input.coordinates.transform.translation = slot_values.add(Vec3::from_array([0.3, 0.7, 0.]));
    }
    let pass = compile(&shader, &slot_values);
    assert!(pass.animation.transform.is_some());
    assert_ne!(pass.material.uv_transform, Affine2::IDENTITY);
    assert!(pass.animation.secondary_transform.is_none());
    assert_eq!(pass.state.combiner.uv_transform, Mat3::IDENTITY);

    shader.shader = 0x80;
    shader.textures.truncate(1);
    let primary_generated = compile(&shader, &slot_values);
    assert!(primary_generated.animation.transform.is_none());
    assert_eq!(primary_generated.material.uv_transform, Affine2::IDENTITY);
    assert_eq!(primary_generated.state.combiner.coordinates.x, 1);
}

#[test]
fn explicit_environment_source_and_independent_weights_survive_compilation() {
    let mut slot_values = SlotValues::default();
    let mut shader = surface(&mut slot_values, 0x8003, &[0, -1, 0]);
    for (input, weight) in shader.textures.iter_mut().zip([0.2, 0.4, 0.6]) {
        input.contribution = slot_values.add(weight);
    }
    let pass = compile(&shader, &slot_values);
    assert_eq!(
        pass.state.combiner.weights.truncate(),
        Vec3::new(0.2, 0.4, 0.6)
    );
    for (binding, weight) in pass.animation.input_weights.iter().zip([0.2, 0.4, 0.6]) {
        assert_eq!(slot_values[binding.as_ref().unwrap()], weight);
    }
    assert_eq!(pass.state.combiner.coordinates.y, 1);
    assert!(pass.animation.third_transform.is_some());
}

#[test]
fn unknown_coordinates_or_programs_report_fallback() {
    let mut slot_values = SlotValues::default();
    for shader in [
        surface(&mut slot_values, 0x8000, &[0, -2]),
        surface(&mut slot_values, 0x8000, &[0]),
        surface(&mut slot_values, 0x8024, &[0, 0]),
    ] {
        let pass = compile(&shader, &slot_values);
        assert_eq!(pass.state.combiner.mode.x, 0);
        assert!(pass
            .diagnostics
            .iter()
            .any(|message| message.message.contains("approximated")));
    }
}

#[test]
fn shared_composition_shader_parses_and_validates() {
    let source = include_str!("../shaders/combiner.wgsl")
        .lines()
        .filter(|line| !line.starts_with("#define_import_path") && !line.starts_with("#import"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("#{MATERIAL_BIND_GROUP}", "2");
    let source = format!("{}\n{source}", super::super::shared_material_data_types());
    let module =
        parse_str(&source).unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error:?}"));
}

#[test]
fn third_coordinates_keep_their_own_source_and_transform() {
    let mut slot_values = SlotValues::default();
    let mut shader = surface(&mut slot_values, 0x8014, &[0, -1, 1]);
    shader.textures[2].coordinates.transform.translation =
        slot_values.add(Vec3::from_array([0.2, 0.4, 0.]));
    let pass = compile(&shader, &slot_values);
    assert_eq!(pass.third_coordinate, Some(1));
    assert_ne!(pass.state.combiner.third_transform, Mat3::IDENTITY);
    assert!(pass.animation.third_transform.is_some());
    shader.textures[2].coordinates.source = -1;
    let generated = compile(&shader, &slot_values);
    assert_eq!(generated.state.combiner.coordinates.w, 1);
    assert_eq!(generated.state.combiner.third_transform, Mat3::IDENTITY);
    assert!(generated.animation.third_transform.is_none());
}

#[test]
fn edge_programs_are_explicit_and_keep_native_input_counts() {
    let mut slot_values = SlotValues::default();
    for (offset, count) in [(19, 1), (21, 2), (25, 1), (33, 2), (34, 1), (35, 2)] {
        let pass = compile(
            &surface(&mut slot_values, 0x8000 + offset, &vec![0; count]),
            &slot_values,
        );
        assert_eq!(pass.state.combiner.clip.y, 1.);
        assert!(pass.diagnostics.is_empty());
    }
    assert_eq!(
        compile(&surface(&mut slot_values, 0x8019, &[0]), &slot_values)
            .state
            .combiner
            .coordinates
            .x,
        1
    );
    for offset in [29, 30, 31, 36] {
        assert!(program(0x8000 + offset, 3).is_none());
    }
}

#[test]
fn legacy_surplus_bindings_use_the_established_two_input_equation_with_diagnostics() {
    let mut slot_values = SlotValues::default();
    for selector in [0x4011, 0x4013] {
        let pass = compile(
            &surface(&mut slot_values, selector, &[0, 1, -2]),
            &slot_values,
        );
        assert_eq!(pass.state.combiner.coordinates.z, 2);
        assert!(pass.third_coordinate.is_none());
        assert!(pass
            .diagnostics
            .iter()
            .any(|message| message.message.contains("surplus")));
        assert_ne!(pass.state.combiner.mode.x, 0);
    }
}

#[test]
fn legacy_guild_programmes_use_external_palettes_and_reject_six_input_layouts() {
    let mut slot_values = SlotValues::default();
    let palette = m2::GuildPalette {
        background: Vec3::from_array([0.2, 0.3, 0.4]),
        emblem: Vec3::from_array([0.5, 0.6, 0.7]),
        border: Vec3::from_array([0.8, 0.9, 1.]),
    };
    for (index, mode, coordinates) in [
        (26, 24, vec![0, 1, 0]),
        (27, 25, vec![0, 1]),
        (28, 26, vec![0, 1, 0]),
    ] {
        let mut shader = surface(&mut slot_values, 0x8000 + index, &coordinates);
        shader.guild_palette = palette;
        let pass = compile(&shader, &slot_values);
        assert_eq!(pass.state.combiner.mode.x, mode);
        assert_eq!(
            pass.state.combiner.guild_background,
            Vec4::new(0.2, 0.3, 0.4, 1.)
        );
        assert_eq!(
            pass.state.combiner.guild_emblem,
            Vec4::new(0.5, 0.6, 0.7, 1.)
        );
        assert_eq!(
            pass.state.combiner.guild_border,
            Vec4::new(0.8, 0.9, 1., 1.)
        );
        assert!(pass.diagnostics.is_empty());
        assert!(!pass.animation.is_animated());
    }
    let six = compile(
        &surface(&mut slot_values, 0x801c, &[0, 1, 0, 0, 1, 0]),
        &slot_values,
    );
    assert_eq!(six.state.combiner.mode.x, 0);
    assert!(six
        .diagnostics
        .iter()
        .any(|message| message.message.contains("approximated")));
}

#[test]
fn unknown_legacy_selector_controls_do_not_silently_use_a_known_equation() {
    let mut slot_values = SlotValues::default();
    for bit in 8..14 {
        let pass = compile(&surface(&mut slot_values, 1 << bit, &[0]), &slot_values);
        assert_eq!(pass.state.combiner.mode.x, 0);
        assert!(pass
            .diagnostics
            .iter()
            .any(|message| message.message.contains("approximated")));
    }
    assert!(program(0x4011, 2).is_some());
}

fn program(shader: u16, count: usize) -> Option<ShaderProgram> {
    let mut slot_values = SlotValues::default();
    super::program(
        &m2::Material {
            shader,
            textures: (0..count)
                .map(|_| SlotConstruct::construct(&mut slot_values))
                .collect(),
            ..SlotConstruct::construct(&mut slot_values)
        }
        .into(),
    )
}

#[test]
fn illumination_keeps_diffuse_opacity_weights_and_uv_transforms_independent() {
    let mut values = SlotValues::default();
    let mut shader = surface(&mut values, 0x801d, &[0, 0]);
    shader.textures[0].coordinates.transform.translation = values.add(Vec3::new(0.125, 0., 0.));
    shader.textures[1].coordinates.transform.translation = values.add(Vec3::new(0.5, 0.25, 0.));
    let primary = shader.textures[0].contribution;
    let secondary = shader.textures[1].contribution;
    let mut opacity = None;
    for weight in [0., 0.5, 1., 2.] {
        values.set(secondary, weight).unwrap();
        let pass = compile(&shader, &values);
        assert_eq!(pass.state.combiner.mode, UVec4::new(27, 0, 0, 0));
        assert_eq!(pass.state.combiner.weights.y, weight);
        assert_eq!(pass.animation.weight, Some(primary));
        assert_eq!(pass.animation.input_weights[1], Some(secondary));
        assert_eq!(pass.coordinate, 0);
        assert_eq!(
            pass.secondary_coordinate, None,
            "equal raw UV sources do not require another stream"
        );
        assert_ne!(
            Mat3::from(pass.material.uv_transform),
            pass.state.combiner.uv_transform
        );
        assert!(
            !pass.material.unlit,
            "illumination does not force unlit shading"
        );
        let alpha = pass.material.base_color.alpha();
        assert_eq!(
            *opacity.get_or_insert(alpha),
            alpha,
            "secondary weight must not fade diffuse coverage"
        );
        assert!(pass
            .diagnostics
            .iter()
            .any(|issue| issue.kind == RenderDiagnosticKind::Unverified));
        assert!(!pass
            .diagnostics
            .iter()
            .any(|issue| issue.message.contains("first input")));
    }
    shader.batch_flags = BatchFlags::from_bits_retain(0x40);
    let pass = compile(&shader, &values);
    assert_eq!(pass.animation.weight, None);
    assert!((pass.material.base_color.alpha() - 0.8).abs() < 1e-6);
    assert_eq!(
        pass.state.combiner.weights.y, 2.,
        "emission weights are not saturated"
    );
}
