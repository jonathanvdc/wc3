use std::borrow::Cow;
use wc3::model::profiles::m3;
use wc3::model::FeatureSchema;
use wc3::model::{Camera, EventSource, FeatureView, Light, NodeFeature};
use wc3::model::{Construct as SlotConstruct, Values as SlotValues};
use wc3::model::{FromFeature as SlotFromFeature, TryFromFeature as SlotTryFromFeature};

#[test]
fn optical_programs_keep_initial_reset_and_reference_controls() {
    let mut slot_values = SlotValues::default();
    let authored = m3::Property {
        value: slot_values.add(0.6),
        reset: 0.9,
        control: m3::AnimationControl {
            source_id: 123,
            reserved: -17,
            ..SlotConstruct::construct(&mut slot_values)
        },
    };
    let source = m3::Feature::Camera(Box::new(m3::Camera {
        name: Cow::Borrowed("Portrait"),
        field_of_view: authored.clone(),
        use_vertical_f_o_v: 7,
        optics: m3::CameraOptics::Bokeh {
            clipping: m3::CameraClipping {
                near: authored.clone(),
                far: authored.clone(),
            },
            falloff: m3::CameraFalloff {
                start: authored.clone(),
                end: authored.clone(),
            },
            bokeh: m3::CameraBokeh {
                mode: 42,
                f_stop: authored.clone(),
                maximum_circle: authored.clone(),
            },
        },
        shadow_clip_distance: authored.clone(),
        focus_distance: authored.clone(),
        far_focus_range: authored.clone(),
        near_focus_range: authored.clone(),
        dof_amount: authored,
    }));
    let universal = NodeFeature::from_feature(source.clone(), &mut slot_values);
    let NodeFeature::Camera(Camera::Optical(camera)) = &universal else {
        panic!("camera interpretation changed")
    };
    assert!(matches!(camera.name, Cow::Borrowed(_)));
    assert!(matches!(universal.feature(), FeatureView::Camera(_)));
    assert_eq!(
        m3::Feature::try_from_feature(universal, &slot_values).unwrap(),
        source
    );
}

#[test]
fn specular_light_and_command_options_round_trip_without_flattening() {
    let mut slot_values = SlotValues::default();
    let source = m3::Feature::Light(Box::new(m3::Light {
        light_type: SlotConstruct::construct(&mut slot_values),
        flags: SlotConstruct::construct(&mut slot_values),
        lod_cut: 2,
        shadow_lod_cut: 5,
        diffuse_color: m3::Property {
            value: slot_values.add([0.1, 0.2, 0.3]),
            reset: [0.7, 0.8, 0.9],
            ..SlotConstruct::construct(&mut slot_values)
        },
        intensity_multiplier: SlotConstruct::construct(&mut slot_values),
        specular_color: m3::Property {
            value: slot_values.add([0.9, 0.1, 0.8]),
            ..SlotConstruct::construct(&mut slot_values)
        },
        specular_multiplier: SlotConstruct::construct(&mut slot_values),
        decay: SlotConstruct::construct(&mut slot_values),
        attenuation_end: 1024.,
        attenuation_start: SlotConstruct::construct(&mut slot_values),
        hot_spot: SlotConstruct::construct(&mut slot_values),
        falloff: SlotConstruct::construct(&mut slot_values),
    }));
    let universal = NodeFeature::from_feature(source.clone(), &mut slot_values);
    assert!(matches!(universal, NodeFeature::Light(Light::Specular(_))));
    assert_eq!(
        m3::Feature::try_from_feature(universal, &slot_values).unwrap(),
        source
    );

    let source = m3::Feature::Event(Box::new(m3::EventCommand {
        event_type: 0x8000_0001,
        transform: [[1., 2., 3., 4.]; 4],
        options: Cow::Borrowed("authored-command-options"),
        render_target_channel: Some(19),
        parameter: Some(31),
        control: 0x8000_0020,
    }));
    let universal = NodeFeature::from_feature(source.clone(), &mut slot_values);
    let NodeFeature::Event(EventSource::Command(command)) = &universal else {
        panic!("command interpretation changed")
    };
    assert!(matches!(command.options, Cow::Borrowed(_)));
    assert_eq!(
        m3::Feature::try_from_feature(universal, &slot_values).unwrap(),
        source
    );
}
