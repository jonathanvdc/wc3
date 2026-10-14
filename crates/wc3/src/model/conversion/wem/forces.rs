//! Node-local influence fields retain authored dimensions and frame transforms.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::ForceChannels;
use crate::model::semantic::profiles::wem::{
    Collision, CollisionShapeKind, Force, ForceKind, ForceVolume, VertexWarp, WarpMode,
};
use crate::model::semantic::{Animated, Bounds};
use crate::model::wem::records;
use crate::model::wem::records::{
    CollisionPayload, ForceFieldPayload, ForceFieldPayloadDialect, VertexWarpPayload, V19,
};

pub(in crate::model::conversion::wem) trait ForceTransform {
    fn write_matrix(matrix: [[f32; 4]; 4]) -> Result<Self, Error>
    where
        Self: Sized;
    fn matrix(&self) -> [[f32; 4]; 4];
}
impl ForceTransform for () {
    fn write_matrix(matrix: [[f32; 4]; 4]) -> Result<Self, Error> {
        if matrix == Self::matrix(&()) {
            Ok(())
        } else {
            Err(Error::new(
                "wem.force.transform",
                "this node vocabulary omits the authored force transform",
            ))
        }
    }
    fn matrix(&self) -> [[f32; 4]; 4] {
        [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ]
    }
}
impl ForceTransform for [[f32; 4]; 4] {
    fn write_matrix(matrix: [[f32; 4]; 4]) -> Result<Self, Error> {
        Ok(matrix)
    }
    fn matrix(&self) -> [[f32; 4]; 4] {
        *self
    }
}

pub(super) fn read_force<D: ForceFieldPayloadDialect>(value: &ForceFieldPayload<D>) -> Force
where
    D::Transform: ForceTransform,
{
    Force {
        kind: match value.kind {
            records::ForceKind::Directional => ForceKind::Directional,
            records::ForceKind::Radial => ForceKind::Radial,
            records::ForceKind::Drag => ForceKind::Drag,
            records::ForceKind::Vortex => ForceKind::Vortex,
            records::ForceKind::Count => ForceKind::Count,
            records::ForceKind::Unknown(code) => ForceKind::Unknown(code),
        },
        volume: match value.volume {
            records::ForceVolume::Sphere => ForceVolume::Sphere,
            records::ForceVolume::Cylinder => ForceVolume::Cylinder,
            records::ForceVolume::Box => ForceVolume::Box,
            records::ForceVolume::Hemisphere => ForceVolume::Hemisphere,
            records::ForceVolume::Cone => ForceVolume::Cone,
            records::ForceVolume::Count => ForceVolume::Count,
            records::ForceVolume::Unknown(code) => ForceVolume::Unknown(code),
        },
        falloff: value.falloff.get(),
        height_gradient: value.height_gradient.get(),
        unbounded: value.unbounded.get(),
        affects_particles: value.affects_particles.get(),
        affects_bodies: value.affects_bodies.get(),
        channels: ForceChannels::from_bits_retain(value.channels.bits()),
        scope: value.scope,
        strength: Animated::constant(value.strength),
        dimensions: [
            Animated::constant(value.width),
            Animated::constant(value.height),
            Animated::constant(value.length),
        ],
        transform: value.transform.matrix(),
    }
}
pub(super) fn write_force(value: &Force) -> ForceFieldPayload<V19> {
    ForceFieldPayload {
        kind: match value.kind {
            ForceKind::Directional => records::ForceKind::Directional,
            ForceKind::Radial => records::ForceKind::Radial,
            ForceKind::Drag => records::ForceKind::Drag,
            ForceKind::Vortex => records::ForceKind::Vortex,
            ForceKind::Count => records::ForceKind::Count,
            ForceKind::Unknown(code) => records::ForceKind::Unknown(code),
        },
        volume: match value.volume {
            ForceVolume::Sphere => records::ForceVolume::Sphere,
            ForceVolume::Cylinder => records::ForceVolume::Cylinder,
            ForceVolume::Box => records::ForceVolume::Box,
            ForceVolume::Hemisphere => records::ForceVolume::Hemisphere,
            ForceVolume::Cone => records::ForceVolume::Cone,
            ForceVolume::Count => records::ForceVolume::Count,
            ForceVolume::Unknown(code) => records::ForceVolume::Unknown(code),
        },
        falloff: value.falloff.into(),
        height_gradient: value.height_gradient.into(),
        unbounded: value.unbounded.into(),
        affects_particles: value.affects_particles.into(),
        affects_bodies: value.affects_bodies.into(),
        channels: records::ForceChannels::from_bits_retain(value.channels.bits()),
        scope: value.scope,
        strength: value.strength.default,
        width: value.dimensions[0].default,
        height: value.dimensions[1].default,
        length: value.dimensions[2].default,
        transform: value.transform,
    }
}
pub(super) fn read_warp(value: &VertexWarpPayload) -> VertexWarp {
    VertexWarp {
        mode: WarpMode::Unknown(value.type_),
        unclassified_control: value.reserved,
        radius: Animated::constant(value.radius),
        height: Animated::constant(value.height),
        strength: Animated::constant(value.strength),
        angular: Animated::constant(value.angular),
        axial: Animated::constant(value.axial),
        radial: Animated::constant(value.radial),
    }
}
pub(super) fn write_warp(value: &VertexWarp) -> VertexWarpPayload {
    let WarpMode::Unknown(type_) = value.mode;
    VertexWarpPayload {
        type_,
        reserved: value.unclassified_control,
        radius: value.radius.default,
        height: value.height.default,
        strength: value.strength.default,
        angular: value.angular.default,
        axial: value.axial.default,
        radial: value.radial.default,
    }
}
pub(super) fn read_collision(value: &CollisionPayload) -> Collision {
    let shape = &value.shape;
    Collision {
        kind: match shape.kind {
            records::CollisionShapeKind::Box => CollisionShapeKind::Box,
            records::CollisionShapeKind::Sphere => CollisionShapeKind::Sphere,
            records::CollisionShapeKind::Capsule => CollisionShapeKind::Capsule,
            records::CollisionShapeKind::Plane => CollisionShapeKind::Plane,
            records::CollisionShapeKind::Cylinder => CollisionShapeKind::Cylinder,
            records::CollisionShapeKind::Hull => CollisionShapeKind::Hull,
            records::CollisionShapeKind::Unknown(code) => CollisionShapeKind::Unknown(code),
        },
        bounds: Bounds {
            minimum: shape.bounds.minimum,
            maximum: shape.bounds.maximum,
            sphere_radius: shape.bounds.sphere_radius,
        },
        sphere_center: shape.sphere.center,
        sphere_radius: shape.sphere.radius,
        height: shape.height,
    }
}
pub(super) fn write_collision(value: &Collision) -> CollisionPayload {
    CollisionPayload {
        shape: records::CollisionShapeDesc {
            kind: match value.kind {
                CollisionShapeKind::Box => records::CollisionShapeKind::Box,
                CollisionShapeKind::Sphere => records::CollisionShapeKind::Sphere,
                CollisionShapeKind::Capsule => records::CollisionShapeKind::Capsule,
                CollisionShapeKind::Plane => records::CollisionShapeKind::Plane,
                CollisionShapeKind::Cylinder => records::CollisionShapeKind::Cylinder,
                CollisionShapeKind::Hull => records::CollisionShapeKind::Hull,
                CollisionShapeKind::Unknown(code) => records::CollisionShapeKind::Unknown(code),
            },
            bounds: records::Extent {
                minimum: value.bounds.minimum,
                maximum: value.bounds.maximum,
                sphere_radius: value.bounds.sphere_radius,
            },
            sphere: records::Sphere {
                center: value.sphere_center,
                radius: value.sphere_radius,
            },
            height: value.height,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn force_frame_channels_unknown_categories_and_edits_survive() {
        let value = ForceFieldPayload::<V19> {
            kind: records::ForceKind::Unknown(201),
            volume: records::ForceVolume::Unknown(202),
            channels: records::ForceChannels::from_bits_retain(0xa5a5_5a5a),
            scope: 71,
            strength: 3.,
            width: 2.,
            height: 4.,
            length: 6.,
            transform: [
                [1., 0., 0., 7.],
                [0., 1., 0., 8.],
                [0., 0., 1., 9.],
                [0., 0., 0., 1.],
            ],
            falloff: true.into(),
            ..Default::default()
        };
        let mut force = read_force(&value);
        force.strength.default = 12.;
        let output = write_force(&force);
        assert_eq!(output.strength, 12.);
        assert_eq!(output.channels, value.channels);
        assert_eq!(output.transform, value.transform);
        assert_eq!(output.kind, value.kind);
        assert_eq!(output.volume, value.volume);
        assert_eq!(output.scope, 71);
    }
    #[test]
    fn collision_keeps_inactive_bounds_and_warp_keeps_unclassified_control() {
        let value = CollisionPayload {
            shape: records::CollisionShapeDesc {
                kind: records::CollisionShapeKind::Sphere,
                bounds: records::Extent {
                    minimum: [-2.; 3],
                    maximum: [2.; 3],
                    sphere_radius: 4.,
                },
                sphere: records::Sphere {
                    center: [1., 2., 3.],
                    radius: 5.,
                },
                height: 6.,
            },
        };
        assert_eq!(write_collision(&read_collision(&value)), value);
        let value = VertexWarpPayload {
            type_: 0xffff_ffff,
            reserved: 0x8000_0001,
            radius: 1.,
            height: 2.,
            strength: 3.,
            angular: 4.,
            axial: 5.,
            radial: 6.,
        };
        assert_eq!(write_warp(&read_warp(&value)), value);
    }
}
