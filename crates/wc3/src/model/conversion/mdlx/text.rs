//! Explicit loss policy for native text that cannot become a logical Unicode name.
use crate::model::conversion::{Context, Error};
use crate::model::mdlx::mdx::FixedText;
use crate::model::mdlx::{Model as NativeModel, ModelDialect};
use std::str::from_utf8;
fn check<const N: usize>(
    value: &FixedText<N>,
    context: &mut Context,
    path: &str,
) -> Result<(), Error> {
    let bytes = value.as_bytes();
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if from_utf8(&bytes[..end]).is_err() {
        context.unsupported(path,"native text contains invalid UTF-8; replacement characters require an explicit semantic loss policy")?;
    }
    Ok(())
}
pub(super) fn validate<D: ModelDialect>(
    source: &NativeModel<'_, D>,
    context: &mut Context,
) -> Result<(), Error> {
    if let Some(info) = source.model_info() {
        check(&info.name, context, "model.name")?;
        check(&info.animation_file_name, context, "model.animation_file")?;
    }
    for node in source.nodes() {
        check(node.name, context, "node.name")?;
    }
    for sequence in source.sequences() {
        check(&sequence.name, context, "clip.name")?;
    }
    for material in source.materials() {
        if let Ok(value) = material.try_shader_field() {
            check(value, context, "material.shader")?;
        }
    }
    for mesh in source.geosets() {
        if let Ok(value) = mesh.try_name_field() {
            check(value, context, "mesh.name")?;
        }
    }
    for camera in source.cameras() {
        check(&camera.name, context, "camera.name")?;
    }
    for emitter in source.particle_emitters() {
        check(&emitter.path, context, "particle.resource")?;
    }
    if let Ok(emitters) = source.try_popcorn_emitters() {
        for emitter in emitters {
            check(&emitter.path, context, "popcorn.resource")?;
            check(
                &emitter.visibility_guide,
                context,
                "popcorn.visibility_guide",
            )?;
        }
    }
    for attachment in source.attachments() {
        check(&attachment.path, context, "attachment.resource")?;
    }
    if let Ok(faces) = source.try_face_fx() {
        for face in faces {
            check(&face.name, context, "face_effect.name")?;
            check(&face.path, context, "face_effect.resource")?;
        }
    }
    Ok(())
}
