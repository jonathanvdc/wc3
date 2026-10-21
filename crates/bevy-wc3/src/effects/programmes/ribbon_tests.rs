use super::*;
use std::mem::size_of;

fn point(serial: u64, position: Vec3) -> RibbonSample {
    RibbonSample {
        parent: None,
        serial,
        position,
        half_width: 1.,
        color: Vec4::ONE,
        axis: Vec3::Z,
        texture_u: None,
        texture_id: 0.,
        cursor: 0.,
    }
}
fn records(points: &[RibbonSample], mode: RibbonMode) -> Vec<AgeSpriteRecord> {
    let mut scratch = RibbonSamples::default();
    for &point in points {
        assert!(scratch.push(point));
    }
    let mut records = Vec::new();
    let mut indices = Vec::new();
    assert_eq!(scratch.emit(mode, 4, &mut records, &mut indices), 0);
    assert_eq!(indices, (0..records.len() as u32).collect::<Vec<_>>());
    records
}
fn view_section(section: Vec3, point: Vec3, eye: Vec3) -> Vec3 {
    unit(section.cross(unit_or_zero(point - eye)))
}
#[test]
fn chronological_three_point_bend_reuses_identical_view_cross_sections() {
    let p = [
        point(0, Vec3::ZERO),
        point(1, 2. * Vec3::X),
        point(2, Vec3::new(2., 2., 0.)),
    ];
    let records = records(&[p[2], p[0], p[1]], RibbonMode::View);
    assert_eq!(records.len(), 2);
    let a = records[0].ribbon_parameters();
    let b = records[1].ribbon_parameters();
    assert_eq!(a.start, p[0].position);
    assert_eq!(a.end, p[1].position);
    assert_eq!(a.end, b.start);
    assert_eq!(a.sections[1], b.sections[0]);
    let eye = 10. * Vec3::Z;
    assert_eq!(view_section(a.sections[0], a.start, eye), Vec3::Y);
    assert_eq!(
        view_section(a.sections[1], a.end, eye),
        view_section(b.sections[0], b.start, eye)
    );
    assert!(view_section(b.sections[1], b.end, eye)
        .abs_diff_eq(Vec3::new(-20., 0., -4.).normalize(), 1e-6));
    for side in [-1., 1.] {
        assert_eq!(
            a.end + view_section(a.sections[1], a.end, eye) * side,
            b.start + view_section(b.sections[0], b.start, eye) * side
        );
    }
}
#[test]
fn exact_parent_incarnations_never_cross_connect() {
    let mut points = [
        point(3, 3. * Vec3::X),
        point(0, Vec3::ZERO),
        point(2, 2. * Vec3::X),
        point(1, Vec3::X),
    ];
    for p in &mut points {
        p.parent = Some(EffectParticleId {
            slot: 7,
            generation: if p.serial % 2 == 0 { 1 } else { 2 },
        });
    }
    let records = records(&points, RibbonMode::View);
    assert_eq!(records.len(), 2);
    for record in records {
        let p = record.ribbon_parameters();
        assert_eq!(p.end - p.start, 2. * Vec3::X);
    }
}
#[test]
fn packet_roundtrip_retains_endpoint_color_uv_atlas_and_authored_lut_rows() {
    let mut a = point(0, Vec3::ZERO);
    let mut b = point(1, Vec3::X);
    a.color = Vec4::new(-1., 2., 3., 0.25);
    b.color = Vec4::new(4., 5., 6., 0.75);
    a.texture_u = Some(-2.);
    b.texture_u = Some(5.);
    a.cursor = -0.25;
    b.cursor = 1.5;
    a.texture_id = -2.9;
    b.texture_id = 0.;
    a.half_width = 0.25;
    b.half_width = 2.;
    let records = records(&[a, b], RibbonMode::SideAxis);
    let p = records[0].ribbon_parameters();
    assert_eq!(size_of::<AgeSpriteRecord>(), 112);
    assert_eq!(p.colors, [a.color.max(Vec4::ZERO), b.color]);
    assert_eq!(p.texture_u, [-2., 5.]);
    assert_eq!(p.cursors, [-0.25, 1.5]);
    assert_eq!(
        p.atlas_cell, 2,
        "start TextureID selects cell independently of Cursor"
    );
    assert_eq!(p.half_widths, [0.25, 2.]);
}
#[test]
fn normal_and_side_axis_modes_reuse_endpoint_axes_at_sharp_bends() {
    let mut p = [
        point(0, Vec3::ZERO),
        point(1, Vec3::X),
        point(2, Vec3::new(1., 1., 0.)),
    ];
    p[1].axis = Vec3::new(0., 1., 1.);
    for mode in [RibbonMode::NormalAxis, RibbonMode::SideAxis] {
        let records = records(&p, mode);
        let a = records[0].ribbon_parameters();
        let b = records[1].ribbon_parameters();
        assert_eq!(a.sections[1], b.sections[0]);
        assert!(a.sections.iter().all(|v| v.is_normalized()));
    }
    p[1].axis = Vec3::ZERO;
    let records = records(&p, RibbonMode::NormalAxis);
    assert_eq!(records[0].ribbon_parameters().sections[1], Vec3::X);
}
#[test]
fn disabled_or_expired_middle_samples_bridge_survivors_without_depthsorting() {
    let records = records(
        &[point(9, 3. * Vec3::X), point(2, Vec3::ZERO)],
        RibbonMode::View,
    );
    assert_eq!(records.len(), 1);
    let p = records[0].ribbon_parameters();
    assert_eq!(p.start, Vec3::ZERO);
    assert_eq!(p.end, 3. * Vec3::X);
    assert_eq!(p.texture_u, [0., 1.]);
}
#[test]
fn collapsed_and_invalid_samples_have_explicit_bounded_omission_policy() {
    let mut scratch = RibbonSamples::default();
    let mut invalid = point(0, Vec3::ZERO);
    invalid.half_width = 0.;
    assert!(!scratch.push(invalid));
    invalid.half_width = 1.;
    invalid.cursor = f32::NAN;
    assert!(!scratch.push(invalid));
    for p in [
        point(0, Vec3::ZERO),
        point(1, Vec3::X),
        point(2, Vec3::X),
        point(3, 2. * Vec3::X),
    ] {
        assert!(scratch.push(p));
    }
    let mut records = Vec::new();
    let mut indices = Vec::new();
    assert_eq!(
        scratch.emit(RibbonMode::View, 1, &mut records, &mut indices),
        1
    );
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0].ribbon_parameters().sections[1],
        records[1].ribbon_parameters().sections[0]
    );
    scratch.begin();
    assert!(scratch.points.is_empty());
}
