//! Rigid-body companions, convex geometry, joints, and sparse bone overrides.
use super::*;
use native::companions::{PhysicsChunk as Pc, PhysicsFile};
mod bodies;
mod export_joints;
mod export_shapes;
mod shapes;
use export_joints::Constraints;
use export_shapes::Geometry;
mod export;
mod hull;
mod import;
mod joints;
mod overrides;
use bodies::*;
pub(super) use export::export_physics;
use hull::hull;
pub(super) use import::import_physics;
use joints::*;
pub(super) use overrides::{export_overrides, import_overrides};
use shapes::*;

fn identity() -> [[f32; 4]; 4] {
    from_fn(|row| from_fn(|column| if row == column { 1. } else { 0. }))
}
fn frame(value: &native::companions::PhysicsFrame) -> [[f32; 4]; 4] {
    let [x, y, z, t] = [value.axis_x, value.axis_y, value.axis_z, value.origin];
    [
        [x[0], x[1], x[2], 0.],
        [y[0], y[1], y[2], 0.],
        [z[0], z[1], z[2], 0.],
        [t[0], t[1], t[2], 1.],
    ]
}
fn native_frame(value: [[f32; 4]; 4]) -> native::companions::PhysicsFrame {
    native::companions::PhysicsFrame {
        axis_x: value[0][..3].try_into().unwrap(),
        axis_y: value[1][..3].try_into().unwrap(),
        axis_z: value[2][..3].try_into().unwrap(),
        origin: value[3][..3].try_into().unwrap(),
    }
}
fn translate(position: [f32; 3]) -> [[f32; 4]; 4] {
    let mut value = identity();
    value[3] = [position[0], position[1], position[2], 1.];
    value
}
fn append_bag(
    doc: &mut wem::RawDocument<'static>,
    native: &mut wr::NativeBag,
    values: &[(&str, i64)],
) -> Result<(), Error> {
    let mut existing: Vec<_> = doc
        .resolve::<wr::NativeEntry<wr::V2>>(native.entries.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let extra = bag(doc, values)?;
    existing.extend(
        doc.resolve::<wr::NativeEntry<wr::V2>>(extra.entries.cast())?
            .iter()
            .collect::<Result<Vec<_>, _>>()?,
    );
    native.entries = put(doc, &existing)?.cast();
    Ok(())
}
