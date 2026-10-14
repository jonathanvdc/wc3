//! Native skeleton fields become graph nodes and independently typed bone controls.
use super::bindings::Bindings;
use super::materials::{anim, read_error, string};
use crate::model::conversion::Error;

use crate::model::conversion::m3::logical::GraphBuilder;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::ExplicitBinding;
use crate::model::semantic::{
    profiles::m3 as surface, Billboard, Inheritance, Node, NodeId, Property, Quaternion, Skin,
    Transform, Values,
};
use native::Envelope;
use std::array::from_fn;

pub(super) struct Skeleton<'a> {
    pub nodes: Vec<NodeId>,
    pub inverse_bind: Values<'a, [[f32; 4]; 4]>,
}
fn control<T>(value: surface::Property<T>, binding: Property<T>) -> surface::PropertyControl<T> {
    surface::PropertyControl {
        reset: value.reset,
        control: value.control,
        binding: Some(binding),
    }
}
pub(super) fn read<'a, E: Envelope>(
    source: &'a native::Model<'_, E>,
    builder: &mut GraphBuilder<'a>,
    bindings: &mut Bindings,
) -> Result<Skeleton<'a>, Error> {
    let root = source.root_record().map_err(read_error)?;
    let reference = root.bones();
    let document = source.document();
    let count = reference.entries as usize;
    let nodes: Vec<_> = (0..count).map(|_| builder.reserve_node()).collect();
    macro_rules! read_bones {
        ($revision:ty) => {{
            for (index, bone) in document
                .resolve::<nr::Bone<$revision>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let bone = bone.map_err(read_error)?;
                let id = nodes[index];
                let mut node = Node::named(string(document, bone.name)?);
                node.parent = if bone.parent_index == u16::MAX {
                    None
                } else {
                    Some(*nodes.get(bone.parent_index as usize).ok_or_else(|| {
                        Error::new("m3.bone.parent", "parent lies outside the skeleton")
                    })?)
                };
                node.transform = Transform {
                    translation: bone.position.initial_value,
                    rotation: Quaternion(bone.rotation.initial_value),
                    scale: bone.scale.initial_value,
                    ..Transform::default()
                };
                node.inheritance = Inheritance {
                    translation: bone.flags.inherit_translation(),
                    rotation: bone.flags.inherit_rotation(),
                    scale: bone.flags.inherit_scale(),
                };
                node.billboard = if bone.flags.billboard1() {
                    Billboard::Spherical
                } else {
                    Billboard::None
                };
                let rotation = anim(bone.rotation);
                let mut batching = anim(bone.batching);
                batching.binding = Some(surface::properties::BATCHING.bind(id));
                let payload = surface::BoneControls {
                    source_id: i64::from(bone.id),
                    extra_flags: surface::values::BoneFlags(bone.flags.bits() & !0x17),
                    reserved: bone.padding,
                    position: control(anim(bone.position), id.translation()),
                    rotation: control(
                        surface::Property {
                            initial: Quaternion(rotation.initial),
                            reset: Quaternion(rotation.reset),
                            control: rotation.control,
                            binding: None,
                        },
                        id.rotation(),
                    ),
                    scale: control(anim(bone.scale), id.scale()),
                    batching,
                };
                bindings.register_mode(
                    &id.translation(),
                    surface::values::Interpolation::from_raw(bone.position.interpolation.raw()),
                );
                bindings.register_mode(
                    &id.rotation(),
                    surface::values::Interpolation::from_raw(bone.rotation.interpolation.raw()),
                );
                bindings.register_mode(
                    &id.scale(),
                    surface::values::Interpolation::from_raw(bone.scale.interpolation.raw()),
                );
                bindings.register_mode(
                    payload.batching.binding.as_ref().expect("batching binding"),
                    surface::values::Interpolation::from_raw(bone.batching.interpolation.raw()),
                );
                bindings.register(bone.position.animation_id, id.translation());
                bindings.register(bone.rotation.animation_id, id.rotation());
                bindings.register(bone.scale.animation_id, id.scale());
                bindings.register(
                    bone.batching.animation_id,
                    payload
                        .batching
                        .binding
                        .clone()
                        .expect("registered batching destination"),
                );
                node.controls = payload.into();
                builder.set_node(id, node)?;
            }
        }};
    }
    if !reference.is_empty() {
        let revision = document
            .referenced_section(reference)
            .map_err(read_error)?
            .ok_or_else(|| Error::new("m3.bones", "missing bone section"))?
            .revision();
        match revision {
            0 => read_bones!(nr::V0),
            1 => read_bones!(nr::V1),
            _ => return Err(Error::new("m3.bones", "unsupported bone revision")),
        }
    }
    let initial = native::visit_m3_root!(root, |root| root.initial_reference);
    let array = document
        .resolve::<nr::InitialReference>(initial.cast())
        .map_err(read_error)?;
    // Validate once so the lazy field view remains infallible after publication.
    for item in array.iter() {
        item.map_err(read_error)?;
    }
    let count = array.len();
    let bytes = document
        .referenced_section(initial)
        .map_err(read_error)?
        .map_or(&[][..], |section| section.bytes());
    let inverse_bind = Values::view(count, move |index| {
        from_fn(|column| {
            from_fn(|row| {
                let offset = index * 64 + (column * 4 + row) * 4;
                f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
            })
        })
    });
    // Preserve the rig's declared bind transforms even when no mesh currently uses them.
    if count > nodes.len() {
        return Err(Error::new(
            "m3.initial_reference",
            "bind transform has no corresponding skeleton joint",
        ));
    }
    if count != 0 {
        builder.add_skin(Skin {
            joints: nodes[..count].to_vec(),
            binding: ExplicitBinding::new(inverse_bind.clone())?,
        })?;
    }
    Ok(Skeleton {
        nodes,
        inverse_bind,
    })
}
