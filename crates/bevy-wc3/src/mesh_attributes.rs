//! Vertex layouts for extended skinning and generated texture coordinates.
use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::prelude::Mesh;
use bevy::render::render_resource::{
    RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexFormat,
};
use bevy::shader::ShaderDefVal;

pub(crate) const EXTRA_JOINT_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("Model_ExtraJointIndex", 138_000_001, VertexFormat::Uint16x4);
pub(crate) const EXTRA_JOINT_WEIGHT: MeshVertexAttribute = MeshVertexAttribute::new(
    "Model_ExtraJointWeight",
    138_000_002,
    VertexFormat::Float32x4,
);

/// Optional third coordinate stream, independent of both ordinary mesh UV sets.
pub(crate) const EXTRA_TEXCOORD: MeshVertexAttribute =
    MeshVertexAttribute::new("Model_Uv2", 138_000_003, VertexFormat::Float32x2);

/// Second authored color stream; world surfaces use its alpha for layer blending.
pub(crate) const SECOND_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("Model_Color1", 138_000_004, VertexFormat::Float32x4);

pub(crate) fn specialize_mesh_attributes(
    descriptor: &mut RenderPipelineDescriptor,
    layout: &MeshVertexBufferLayoutRef,
    generated_coordinates: bool,
    third_uv: bool,
    second_color: bool,
) -> Result<(), SpecializedMeshPipelineError> {
    let extra = layout.0.contains(EXTRA_JOINT_INDEX) && layout.0.contains(EXTRA_JOINT_WEIGHT);
    let third_uv = third_uv && layout.0.contains(EXTRA_TEXCOORD);
    let second_color = second_color && layout.0.contains(SECOND_COLOR);
    if !extra && !generated_coordinates && !third_uv && !second_color {
        return Ok(());
    }
    if second_color {
        descriptor
            .vertex
            .shader_defs
            .push("WORLD_LAYER_ALPHA".into());
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.shader_defs.push("WORLD_LAYER_ALPHA".into());
        }
    }
    if third_uv {
        descriptor.vertex.shader_defs.push("VERTEX_UVS_C".into());
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.shader_defs.push("VERTEX_UVS_C".into());
        }
    }
    if extra {
        descriptor
            .vertex
            .shader_defs
            .push("EXTRA_SKIN_INFLUENCES".into());
    }
    if generated_coordinates {
        if layout.0.contains(Mesh::ATTRIBUTE_NORMAL)
            && !descriptor
                .vertex
                .shader_defs
                .iter()
                .any(|def| matches!(def, ShaderDefVal::Bool(name,true) if name=="VERTEX_NORMALS"))
        {
            descriptor.vertex.shader_defs.push("VERTEX_NORMALS".into());
        }
        descriptor
            .vertex
            .shader_defs
            .push("MODEL_GENERATED_COORDS".into());
    }
    let prepass = descriptor
        .vertex
        .shader_defs
        .iter()
        .any(|def| matches!(def,ShaderDefVal::Bool(name,true) if name=="PREPASS_PIPELINE"));
    let mut attributes = Vec::new();
    let mut add = |attribute: MeshVertexAttribute, location| {
        if layout.0.contains(attribute) {
            attributes.push(attribute.at_shader_location(location));
        }
    };
    add(Mesh::ATTRIBUTE_POSITION, 0);
    if prepass {
        add(Mesh::ATTRIBUTE_UV_0, 1);
        add(Mesh::ATTRIBUTE_UV_1, 2);
        if generated_coordinates || descriptor.vertex.shader_defs.iter().any(|def| matches!(def,ShaderDefVal::Bool(name,true) if name=="NORMAL_PREPASS_OR_DEFERRED_PREPASS")) { add(Mesh::ATTRIBUTE_NORMAL,3); add(Mesh::ATTRIBUTE_TANGENT,4); }
        add(Mesh::ATTRIBUTE_JOINT_INDEX, 5);
        add(Mesh::ATTRIBUTE_JOINT_WEIGHT, 6);
        add(Mesh::ATTRIBUTE_COLOR, 7);
    } else {
        add(Mesh::ATTRIBUTE_NORMAL, 1);
        add(Mesh::ATTRIBUTE_UV_0, 2);
        add(Mesh::ATTRIBUTE_UV_1, 3);
        add(Mesh::ATTRIBUTE_TANGENT, 4);
        add(Mesh::ATTRIBUTE_COLOR, 5);
        add(Mesh::ATTRIBUTE_JOINT_INDEX, 6);
        add(Mesh::ATTRIBUTE_JOINT_WEIGHT, 7);
    }
    add(EXTRA_JOINT_INDEX, 8);
    add(EXTRA_JOINT_WEIGHT, 9);
    if third_uv {
        add(EXTRA_TEXCOORD, 10);
    }
    if second_color {
        add(SECOND_COLOR, 11);
    }
    descriptor.vertex.buffers = vec![layout.0.get_layout(&attributes)?];
    Ok(())
}
