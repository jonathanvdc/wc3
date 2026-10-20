use super::*;
#[test]
fn unit_vectors_use_y_polar_axis_and_shells_use_uniform_volume() {
    assert_eq!(vector([0., 0., 0.], None).unwrap(), Vec3::Y);
    assert_eq!(vector([0., 1., 0.], None).unwrap(), -Vec3::Y);
    assert!((vector([0., 0.5, 0.], None).unwrap() - Vec3::X).length() < 1e-6);
    let expected = ((8. + 1.) * 0.5f32).cbrt();
    assert!((vector([0., 0.5, 0.5], Some((2., 1.))).unwrap().length() - expected).abs() < 1e-6);
    assert_eq!(
        vector([0.2, 0.7, 0.9], Some((-5., 0.))).unwrap(),
        Vec3::ZERO
    );
    assert!(vector([0.3, 0.4, 0.5], Some((f32::MAX, f32::MAX)))
        .unwrap()
        .is_finite());
    assert!(vector([0.3, 0.4, 0.5], Some((f32::NAN, 1.))).is_err());
    assert!(vector([f32::INFINITY, 0., 0.], None).is_err());
}
#[test]
fn angular_and_cubic_radial_moments_match_isotropic_shell_sampling() {
    let mut sum = Vec3::ZERO;
    let mut radial = 0.;
    let mut count = 0.;
    for azimuth in 0..24 {
        for polar in 0..24 {
            for radius in 0..24 {
                let draws = [
                    (azimuth as f32 + 0.5) / 24.,
                    (polar as f32 + 0.5) / 24.,
                    (radius as f32 + 0.5) / 24.,
                ];
                let value = vector(draws, Some((1., 2.))).unwrap();
                sum += value.normalize();
                radial += value.length().powi(3);
                count += 1.;
            }
        }
    }
    assert!((sum / count).length() < 1e-5);
    assert!((radial / count - 4.5).abs() < 1e-4);
}
