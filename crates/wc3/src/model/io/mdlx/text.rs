//! Explicit loss policy for native text that cannot become a logical Unicode name.
use crate::model::conversion::{Context, Error};
use crate::model::error::Location;
use crate::model::formats::mdlx::mdx::FixedText;
use crate::model::formats::mdlx::{Dialect, Document as NativeModel};
use std::str::from_utf8;
fn check<const N: usize>(
    value: &FixedText<N>,
    context: &mut Context,
    path: &Location,
) -> Result<(), Error> {
    let bytes = value.as_bytes();
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if let Err(error) = from_utf8(&bytes[..end]) {
        context.unsupported_source(path.clone(), error)?;
    }
    Ok(())
}
pub(super) fn validate<D: Dialect>(
    source: &NativeModel<'_, D>,
    context: &mut Context,
) -> Result<(), Error> {
    if let Some(info) = source.model_info() {
        check(
            &info.name,
            context,
            &Location::new().field("model").field("name"),
        )?;
        check(
            &info.animation_file_name,
            context,
            &Location::new().field("model").field("animation_file"),
        )?;
    }
    for node in source.nodes() {
        check(
            node.name,
            context,
            &Location::new().field("node").field("name"),
        )?;
    }
    for sequence in source.sequences() {
        check(
            &sequence.name,
            context,
            &Location::new().field("clip").field("name"),
        )?;
    }
    for material in source.materials() {
        if let Ok(value) = material.try_shader_field() {
            check(
                value,
                context,
                &Location::new().field("material").field("shader"),
            )?;
        }
    }
    for mesh in source.geosets() {
        if let Ok(value) = mesh.try_name_field() {
            check(value, context, &Location::new().field("mesh").field("name"))?;
        }
    }
    for camera in source.cameras() {
        check(
            &camera.name,
            context,
            &Location::new().field("camera").field("name"),
        )?;
    }
    if let Ok(emitters) = source.try_popcorn_emitters() {
        for emitter in emitters {
            check(
                &emitter.visibility_guide,
                context,
                &Location::new().field("popcorn").field("visibility_guide"),
            )?;
        }
    }
    if let Ok(faces) = source.try_face_fx() {
        for face in faces {
            check(
                &face.name,
                context,
                &Location::new().field("face_effect").field("name"),
            )?;
        }
    }
    Ok(())
}
