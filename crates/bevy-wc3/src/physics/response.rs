//! Original point-contact impulses with angular effective mass and bounded Coulomb friction.
use super::contact::Contact;
use bevy::math::{Mat3, Vec3};
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub center: Vec3,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub inverse_mass: f32,
    /// Already rotated from the body-local tensor into the simulation frame.
    pub inverse_inertia: Mat3,
}
impl Body {
    fn point_velocity(self, arm: Vec3) -> Vec3 {
        self.velocity + self.angular_velocity.cross(arm)
    }
    fn apply(&mut self, arm: Vec3, impulse: Vec3) {
        self.velocity += impulse * self.inverse_mass;
        self.angular_velocity += self.inverse_inertia * arm.cross(impulse);
    }
}
fn effective(a: Body, b: Body, arms: [Vec3; 2], direction: Vec3) -> f32 {
    a.inverse_mass
        + b.inverse_mass
        + direction.dot(
            (a.inverse_inertia * arms[0].cross(direction)).cross(arms[0])
                + (b.inverse_inertia * arms[1].cross(direction)).cross(arms[1]),
        )
}
#[derive(Clone, Copy, Debug)]
pub struct Impulse {
    pub normal: f32,
    pub tangent: Vec3,
}
/// Material mixing is an explicit caller policy; this kernel receives the chosen coefficients.
pub fn resolve(
    a: &mut Body,
    b: &mut Body,
    contact: Contact,
    restitution: f32,
    friction: f32,
) -> Option<Impulse> {
    let valid_body = |body: Body| {
        body.center.is_finite()
            && body.velocity.is_finite()
            && body.angular_velocity.is_finite()
            && body.inverse_mass.is_finite()
            && body.inverse_mass >= 0.
            && body.inverse_inertia.is_finite()
    };
    if !valid_body(*a)
        || !valid_body(*b)
        || !contact.normal.is_finite()
        || (contact.normal.length_squared() - 1.).abs() > 1e-4
        || contact.points.iter().any(|point| !point.is_finite())
    {
        return None;
    }
    let (mut next_a, mut next_b) = (*a, *b);
    let result = resolve_inner(&mut next_a, &mut next_b, contact, restitution, friction)?;
    if !valid_body(next_a) || !valid_body(next_b) {
        return None;
    }
    *a = next_a;
    *b = next_b;
    Some(result)
}
fn resolve_inner(
    a: &mut Body,
    b: &mut Body,
    contact: Contact,
    restitution: f32,
    friction: f32,
) -> Option<Impulse> {
    if !restitution.is_finite() || !friction.is_finite() || friction < 0. {
        return None;
    }
    // A common application point preserves total angular momentum even while
    // positional penetration leaves the two geometric witnesses separated.
    let point = (contact.points[0] + contact.points[1]) * 0.5;
    let arms = [point - a.center, point - b.center];
    let normal = contact.normal;
    let relative = b.point_velocity(arms[1]) - a.point_velocity(arms[0]);
    let closing = relative.dot(normal);
    if closing >= 0. {
        return Some(Impulse {
            normal: 0.,
            tangent: Vec3::ZERO,
        });
    }
    let denominator = effective(*a, *b, arms, normal);
    if !denominator.is_finite() || denominator <= 0. {
        return None;
    }
    let magnitude = -(1. + restitution.clamp(0., 1.)) * closing / denominator;
    let impulse = normal * magnitude;
    a.apply(arms[0], -impulse);
    b.apply(arms[1], impulse);
    let relative = b.point_velocity(arms[1]) - a.point_velocity(arms[0]);
    let tangent = relative - normal * relative.dot(normal);
    let tangent_impulse = if let Some(direction) = tangent.try_normalize() {
        let denominator = effective(*a, *b, arms, direction);
        if denominator <= 0. || !denominator.is_finite() {
            return None;
        }
        -direction * (tangent.length() / denominator).min(friction * magnitude)
    } else {
        Vec3::ZERO
    };
    a.apply(arms[0], -tangent_impulse);
    b.apply(arms[1], tangent_impulse);
    Some(Impulse {
        normal: magnitude,
        tangent: tangent_impulse,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn body(center: Vec3) -> Body {
        Body {
            center,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            inverse_mass: 1.,
            inverse_inertia: Mat3::IDENTITY,
        }
    }
    fn energy(a: Body) -> f32 {
        0.5 * (a.velocity.length_squared() + a.angular_velocity.length_squared())
    }
    fn angular_momentum(a: Body) -> Vec3 {
        a.center.cross(a.velocity) + a.angular_velocity
    }
    #[test]
    fn elastic_central_contact_swaps_equal_mass_velocities() {
        let mut a = body(-Vec3::X);
        let mut b = body(Vec3::X);
        a.velocity = Vec3::X;
        b.velocity = -Vec3::X;
        let contact = Contact {
            normal: Vec3::X,
            points: [Vec3::ZERO; 2],
            depth: 0.,
        };
        resolve(&mut a, &mut b, contact, 1., 0.).unwrap();
        assert!((a.velocity + Vec3::X).length() < 1e-5);
        assert!((b.velocity - Vec3::X).length() < 1e-5);
    }
    #[test]
    fn off_center_friction_conserves_momentum_and_dissipates_energy() {
        let mut a = body(-Vec3::X);
        let mut b = body(Vec3::X);
        a.velocity = Vec3::new(2., 3., 0.);
        b.velocity = Vec3::new(-1., 0., 0.);
        let before = energy(a) + energy(b);
        let linear = a.velocity + b.velocity;
        let angular = angular_momentum(a) + angular_momentum(b);
        let contact = Contact {
            normal: Vec3::X,
            points: [Vec3::Y + Vec3::X * 0.1, Vec3::Y - Vec3::X * 0.1],
            depth: 0.2,
        };
        let impulse = resolve(&mut a, &mut b, contact, 0., 0.5).unwrap();
        assert!(impulse.tangent.length() <= 0.5 * impulse.normal + 1e-6);
        assert!((a.velocity + b.velocity - linear).length() < 1e-5);
        assert!((angular_momentum(a) + angular_momentum(b) - angular).length() < 1e-5);
        assert!(a.angular_velocity.length() > 0.);
        assert!(energy(a) + energy(b) <= before + 1e-5);
    }
    #[test]
    fn moving_kinematic_surface_transfers_tangential_velocity_without_impulse_motion() {
        let mut a = body(Vec3::ZERO);
        a.inverse_mass = 0.;
        a.inverse_inertia = Mat3::ZERO;
        a.velocity = Vec3::X;
        let mut b = body(Vec3::Z);
        b.velocity = -Vec3::Z;
        let contact = Contact {
            normal: Vec3::Z,
            points: [Vec3::ZERO; 2],
            depth: 0.,
        };
        resolve(&mut a, &mut b, contact, 0., 1.).unwrap();
        assert_eq!(a.velocity, Vec3::X);
        assert!(b.velocity.x > 0.);
        assert!(b.angular_velocity.y < 0.);
        assert!(b.velocity.z.abs() < 1e-5);
    }
    #[test]
    fn anisotropic_tensor_changes_off_axis_contact_effective_mass() {
        let mut a = body(Vec3::ZERO);
        let mut b = body(Vec3::X);
        a.inverse_mass = 0.;
        a.inverse_inertia = Mat3::ZERO;
        b.velocity = -Vec3::X;
        let contact = Contact {
            normal: Vec3::X,
            points: [Vec3::Y; 2],
            depth: 0.,
        };
        let first = resolve(&mut a, &mut b, contact, 0., 0.).unwrap();
        let mut c = body(Vec3::X);
        c.inverse_inertia = Mat3::from_diagonal(Vec3::new(1., 1., 0.1));
        c.velocity = -Vec3::X;
        let second = resolve(&mut a, &mut c, contact, 0., 0.).unwrap();
        assert!(second.normal > first.normal);
    }
}
