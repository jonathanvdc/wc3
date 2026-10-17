use super::*;
use crate::model::io::ExportOptions;
use crate::model::profiles::Universal;
use crate::model::Model;

fn texture() -> semantic::Texture {
    Model::<Universal>::builder("particle geometry controls")
        .add_file_texture("particle.blp")
        .unwrap()
}

#[test]
fn modern_appearance_timestamps_span_the_complete_particle_lifetime() {
    let times = [0, 16384, 32767];
    let native = native::records::ModernParticleLifetime {
        color: native::records::ParticleTrack {
            timestamps: ZeroVec::alloc_from_slice(&times),
            values: ZeroVec::alloc_from_slice(&[[255., 0., 0.], [255., 255., 0.], [0.; 3]]),
        },
        alpha: native::records::ParticleTrack {
            timestamps: ZeroVec::alloc_from_slice(&times),
            values: ZeroVec::alloc_from_slice(&[0, 32767, 0]),
        },
        scale: native::records::ParticleTrack {
            timestamps: ZeroVec::alloc_from_slice(&[0, 32767]),
            values: ZeroVec::alloc_from_slice(&[[1.; 2], [2.; 2]]),
        },
        head_uv: native::records::ParticleTrack {
            timestamps: ZeroVec::alloc_from_slice(&[0, 32767]),
            values: ZeroVec::alloc_from_slice(&[0, 7]),
        },
        tail_uv: native::records::ParticleTrack {
            timestamps: ZeroVec::alloc_from_slice(&[0, 32767]),
            values: ZeroVec::alloc_from_slice(&[1, 8]),
        },
        scale_variation: [0.; 2],
    };
    let appearance = native.logical().unwrap();
    let alpha = appearance.alpha.as_ref().unwrap();
    assert_eq!(alpha.times().get(2).unwrap().as_seconds(), 1.);
    assert_eq!(alpha.values().get(1), Some(1.));
    assert!(alpha.sample(Time::seconds(0.75).unwrap()).unwrap() > 0.49);
    assert_eq!(
        appearance
            .color
            .as_ref()
            .unwrap()
            .times()
            .get(2)
            .unwrap()
            .as_seconds(),
        1.
    );
    assert_eq!(
        appearance
            .scale
            .as_ref()
            .unwrap()
            .times()
            .get(1)
            .unwrap()
            .as_seconds(),
        1.
    );
    assert_eq!(
        appearance
            .head_uv
            .as_ref()
            .unwrap()
            .times()
            .get(1)
            .unwrap()
            .as_seconds(),
        1.
    );
    assert_eq!(
        appearance
            .tail_uv
            .as_ref()
            .unwrap()
            .times()
            .get(1)
            .unwrap()
            .as_seconds(),
        1.
    );
    let options = ExportOptions::default();
    let restored =
        native::records::ModernParticleLifetime::make(&appearance, &mut Context::export(&options))
            .unwrap();
    assert_eq!(restored, native);
}

#[test]
fn modern_geometry_selection_round_trips_style_flags_including_disabled() {
    let textures = [texture()];
    let options = ExportOptions::default();
    for (head, tail, selection) in [
        (true, false, GeometrySelection::Head),
        (false, true, GeometrySelection::Tail),
        (true, true, GeometrySelection::Both),
        (false, false, GeometrySelection::Disabled),
    ] {
        let mut native = native::records::ParticleEmitter::<V274>::new(ModelFlags::default());
        native.flags.set_head(head);
        native.flags.set_tail(tail);
        native.flags.set_packed_gravity(true);
        let logical = read(
            &native,
            &textures,
            &mut Model::<semantic::profiles::M2>::builder("particles"),
        )
        .unwrap();
        assert_eq!(logical.details.rendering.head_tail, selection);
        let restored = write::<V274>(
            &logical,
            &textures,
            &mut Context::export(&options),
            &Model::<semantic::profiles::M2>::empty("particles").unwrap(),
        )
        .unwrap();
        assert_eq!(restored.flags.head(), head);
        assert_eq!(restored.flags.tail(), tail);
        assert!(restored.flags.packed_gravity());
        assert_eq!(
            read(
                &restored,
                &textures,
                &mut Model::<semantic::profiles::M2>::builder("particles")
            )
            .unwrap()
            .details
            .rendering
            .head_tail,
            selection
        );
    }
}

#[test]
fn edited_modern_geometry_selection_overrides_retained_style_bits() {
    let textures = [texture()];
    let options = ExportOptions::default();
    let mut native = native::records::ParticleEmitter::<V274>::new(ModelFlags::default());
    native.flags.set_head(true);
    let mut logical = read(
        &native,
        &textures,
        &mut Model::<semantic::profiles::M2>::builder("particles"),
    )
    .unwrap();
    logical.details.rendering.head_tail = GeometrySelection::Tail;
    let restored = write::<V274>(
        &logical,
        &textures,
        &mut Context::export(&options),
        &Model::<semantic::profiles::M2>::empty("particles").unwrap(),
    )
    .unwrap();
    assert!(!restored.flags.head());
    assert!(restored.flags.tail());
    logical.details.rendering.head_tail = GeometrySelection::Unknown(77);
    assert!(write::<V274>(
        &logical,
        &textures,
        &mut Context::export(&options),
        &Model::<semantic::profiles::M2>::empty("particles").unwrap()
    )
    .is_err());
}

#[test]
fn legacy_geometry_byte_remains_authoritative_and_cannot_encode_disabled() {
    let textures = [texture()];
    let options = ExportOptions::default();
    let mut native = native::records::ParticleEmitter::<V256>::new(ModelFlags::default());
    native.texture_scale.head_or_tail = ParticleHeadTail::Tail;
    native.flags.set_head(true);
    let mut logical = read(
        &native,
        &textures,
        &mut Model::<semantic::profiles::M2>::builder("particles"),
    )
    .unwrap();
    assert_eq!(logical.details.rendering.head_tail, GeometrySelection::Tail);
    let restored = write::<V256>(
        &logical,
        &textures,
        &mut Context::export(&options),
        &Model::<semantic::profiles::M2>::empty("particles").unwrap(),
    )
    .unwrap();
    assert_eq!(restored.texture_scale.head_or_tail, ParticleHeadTail::Tail);
    logical.details.rendering.head_tail = GeometrySelection::Disabled;
    let error = write::<V256>(
        &logical,
        &textures,
        &mut Context::export(&options),
        &Model::<semantic::profiles::M2>::empty("particles").unwrap(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("cannot represent disabled"));
    assert_eq!(GeometrySelection::Disabled.legacy_code(), None);
}
