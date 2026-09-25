use wc3_mdx::{Light, Model, Node};

#[test]
fn light_fields_round_trip() {
    let mut light = Light::new(Node::new("Torch", 2).unwrap(), 1);
    light.set_attenuation_start(100.0);
    light.set_attenuation_end(500.0);
    light.set_color([1.0, 0.5, 0.25]);
    light.set_intensity(2.0);
    light.set_ambient_color([0.1, 0.2, 0.3]);
    light.set_ambient_intensity(0.5);
    let mut model = Model::new(1200);
    model.set_lights(&[light]).unwrap();
    let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let light = &parsed.lights().unwrap()[0];
    assert_eq!(light.node().name(), "Torch");
    assert_eq!(light.light_type(), 1);
    assert_eq!(light.attenuation_start(), 100.0);
    assert_eq!(light.attenuation_end(), 500.0);
    assert_eq!(light.color(), [1.0, 0.5, 0.25]);
    assert_eq!(light.intensity(), 2.0);
    assert_eq!(light.ambient_color(), [0.1, 0.2, 0.3]);
    assert_eq!(light.ambient_intensity(), 0.5);
}
