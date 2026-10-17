use std::borrow::Cow;
use wc3::model::materials::m3::{DataGroup, DataProperty, NamedShaderKey};
use wc3::model::materials::parameters::{OpaqueBytes, ParameterEntry};

#[test]
fn named_parameter_groups_preserve_hash_spelling_bytes_and_duplicate_order() {
    let key = NamedShaderKey {
        hash: 17,
        name: Cow::Borrowed("roughness"),
    };
    let first: DataProperty<'_> = ParameterEntry {
        key: key.clone(),
        value: OpaqueBytes(vec![0xff, 0]),
        controls: (),
    };
    let second: DataProperty<'_> = ParameterEntry {
        key: key.clone(),
        value: OpaqueBytes(vec![1, 2]),
        controls: (),
    };
    let group: DataGroup<'_> = ParameterEntry {
        key,
        value: vec![first, second],
        controls: (),
    };
    assert!(matches!(group.key.name, Cow::Borrowed("roughness")));
    assert_eq!(group.key.hash, 17);
    assert_eq!(group.value[0].key, group.value[1].key);
    assert_eq!(group.value[0].value.0, [0xff, 0]);
    assert_eq!(group.value[1].value.0, [1, 2]);
}

#[test]
fn parameter_entries_traverse_custom_key_value_and_controls_in_order() {
    use wc3::model::profiles::Universal;
    use wc3::model::{Link, Model, TextureId, ValidateLinks};
    struct Reference(TextureId);
    impl ValidateLinks for Reference {
        fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
            visitor(Link::Texture(self.0));
        }
    }
    let mut resources = Model::<Universal>::builder("textures");
    let key = resources.add_file_texture("key").unwrap();
    let value = resources.add_file_texture("value").unwrap();
    let controls = resources.add_file_texture("controls").unwrap();
    let parameter = ParameterEntry {
        key: Reference(key),
        value: Reference(value),
        controls: Reference(controls),
    };
    let mut seen = Vec::new();
    parameter.visit_links(&mut |link| match link {
        Link::Texture(id) => seen.push(id),
        _ => panic!("unexpected parameter relationship"),
    });
    assert_eq!(seen, [key, value, controls]);
}

#[test]
fn opaque_words_are_retained_while_interpreted_numbers_are_validated() {
    use wc3::model::materials::parameters::{OpaqueWord, ShaderTag, ShaderValue};
    use wc3::model::{Link, ValidateLinks};
    let opaque = ParameterEntry {
        key: ShaderTag(17),
        value: OpaqueWord(0x7fc00001),
        controls: (),
    };
    opaque.visit_links(&mut |_| panic!("opaque bits must not be interpreted as floats"));
    let interpreted = ParameterEntry {
        key: ShaderValue(17),
        value: [0., f32::NAN, 1., 2.],
        controls: (),
    };
    let mut invalid = 0;
    interpreted.visit_links(&mut |link| {
        assert!(matches!(link, Link::InvalidValue));
        invalid += 1;
    });
    assert_eq!(invalid, 1);
}
