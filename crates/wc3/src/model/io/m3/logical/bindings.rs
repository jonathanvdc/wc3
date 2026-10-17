//! Logical material properties bind to typed graph destinations, never section addresses.
use crate::model::materials::m3::properties::{MaterialProperties, PropertyMut};
use crate::model::profiles::m3::properties::{Field, FieldPath};
use crate::model::Animatable;
use crate::model::{
    profiles::m3 as surface, Animated as Destination, OwnerId, Quaternion, ValueSlot,
};
use std::collections::{BTreeMap, HashMap};
#[derive(Clone, Debug)]
pub(super) enum Target {
    Scalar(Destination<f32>),
    Vector2(Destination<[f32; 2]>),
    Vector3(Destination<[f32; 3]>),
    Vector4(Destination<[f32; 4]>),
    Unsigned(Destination<u32>),
    Unsigned16(Destination<u16>),
    Rotation(Destination<Quaternion>),
    Signed(Destination<i32>),
    Byte(Destination<u8>),
    Bytes4(Destination<[u8; 4]>),
    Unsigned4(Destination<[u32; 4]>),
}
#[derive(Default)]
pub(super) struct Bindings {
    pub entries: BTreeMap<u32, Vec<Target>>,
    pub modes: HashMap<ValueSlot, surface::values::Interpolation>,
}
pub(super) trait Value: Animatable {
    fn target(value: Destination<Self>) -> Target
    where
        Self: Sized;
}
impl Value for f32 {
    fn target(value: Destination<Self>) -> Target {
        Target::Scalar(value)
    }
}
impl Value for [f32; 2] {
    fn target(value: Destination<Self>) -> Target {
        Target::Vector2(value)
    }
}
impl Value for [f32; 3] {
    fn target(value: Destination<Self>) -> Target {
        Target::Vector3(value)
    }
}
impl Value for [f32; 4] {
    fn target(value: Destination<Self>) -> Target {
        Target::Vector4(value)
    }
}
impl Value for u32 {
    fn target(value: Destination<Self>) -> Target {
        Target::Unsigned(value)
    }
}
impl Value for u16 {
    fn target(value: Destination<Self>) -> Target {
        Target::Unsigned16(value)
    }
}
impl Value for Quaternion {
    fn target(value: Destination<Self>) -> Target {
        Target::Rotation(value)
    }
}
impl Value for i32 {
    fn target(value: Destination<Self>) -> Target {
        Target::Signed(value)
    }
}
impl Value for u8 {
    fn target(value: Destination<Self>) -> Target {
        Target::Byte(value)
    }
}
impl Value for [u8; 4] {
    fn target(value: Destination<Self>) -> Target {
        Target::Bytes4(value)
    }
}
impl Value for [u32; 4] {
    fn target(value: Destination<Self>) -> Target {
        Target::Unsigned4(value)
    }
}
impl Bindings {
    pub fn register_mode<T: Animatable>(
        &mut self,
        destination: &Destination<T>,
        interpolation: surface::values::Interpolation,
    ) {
        self.modes.insert(destination.slot(), interpolation);
    }
    pub fn register<T: Value>(&mut self, source_id: u32, destination: Destination<T>) {
        self.entries
            .entry(source_id)
            .or_default()
            .push(T::target(destination));
    }
}
trait Bind {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings);
}
impl<T: Value> Bind for surface::Property<T> {
    fn bind(&mut self, _owner: OwnerId, _path: &FieldPath, bindings: &mut Bindings) {
        let destination = self.value;
        bindings.register_mode(&destination, self.control.interpolation);
        bindings
            .entries
            .entry(self.control.source_id)
            .or_default()
            .push(T::target(destination));
    }
}
pub(super) fn material(
    value: &mut surface::MaterialStep<'_>,
    owner: OwnerId,
    bindings: &mut Bindings,
) {
    value.visit_properties_mut(
        &FieldPath::root(Field::Material),
        &mut |value, path| match value {
            PropertyMut::Scalar(v) => v.bind(owner, path, bindings),
            PropertyMut::Vector2(v) => v.bind(owner, path, bindings),
            PropertyMut::Vector3(v) => v.bind(owner, path, bindings),
            PropertyMut::Vector4(v) => v.bind(owner, path, bindings),
            PropertyMut::Unsigned(v) => v.bind(owner, path, bindings),
            PropertyMut::Unsigned16(v) => v.bind(owner, path, bindings),
        },
    );
}
pub(super) fn field<T: Value>(
    value: &mut surface::Property<T>,
    owner: OwnerId,
    path: &FieldPath,
    bindings: &mut Bindings,
) {
    value.bind(owner, path, bindings)
}

pub(super) use super::property_ids::collect;

impl Bind for surface::Body<'_> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(dynamic) = &mut self.dynamic {
            dynamic
                .state
                .bind(owner, &path.child(Field::State), bindings);
        }
    }
}
impl Bind for surface::Cloth<'_> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        if let Some(active) = &mut self.active {
            active.bind(owner, &path.child(Field::Active), bindings);
        }
    }
}
impl Bind for surface::System<'_> {
    fn bind(&mut self, owner: OwnerId, path: &FieldPath, bindings: &mut Bindings) {
        match self {
            surface::System::AnimationPartitions(_)
            | surface::System::Collision(_)
            | surface::System::TurretGroups(_)
            | surface::System::ControllerSelections(_) => {}
            surface::System::Physics(system) => {
                for (index, body) in system.bodies.iter_mut().enumerate() {
                    body.bind(owner, &path.child(Field::Body).indexed(index), bindings);
                }
            }
            surface::System::Cloth(system) => {
                for (index, cloth) in system.cloths.iter_mut().enumerate() {
                    cloth.bind(owner, &path.child(Field::Cloth).indexed(index), bindings);
                }
            }
        }
    }
}

pub(super) fn system(value: &mut surface::System<'_>, owner: OwnerId, bindings: &mut Bindings) {
    value.bind(owner, &FieldPath::root(Field::Physics), bindings);
}
