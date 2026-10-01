use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, CompareFunction,
    RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::{ShaderDefVal, ShaderRef};
use wc3::model::materials::LayerFilterMode;

use crate::mesh::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};

/// A Bevy PBR material with WC3 layer render state.
pub type Wc3LayerMaterial = ExtendedMaterial<StandardMaterial, Wc3LayerState>;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
#[bind_group_data(Wc3LayerKey)]
pub struct Wc3LayerState {
    #[uniform(100)]
    pub(crate) hd: Wc3HdUniform,
    #[texture(101)]
    #[sampler(102)]
    pub(crate) orm: Option<Handle<Image>>,
    #[texture(103)]
    #[sampler(104)]
    pub(crate) team_color: Option<Handle<Image>>,
    #[texture(105)]
    #[sampler(106)]
    pub(crate) environment: Option<Handle<Image>>,
    pub(crate) filter: LayerFilterMode,
    pub(crate) no_depth_test: bool,
    pub(crate) no_depth_set: bool,
}

/// Continuous HD controls; flags describe bound optional maps, not pipeline variants.
#[derive(Clone, Copy, Debug, ShaderType)]
pub(crate) struct Wc3HdUniform {
    pub(crate) fresnel_color: Vec4,
    // x = opacity, y = team-color contribution, zw reserved.
    pub(crate) fresnel: Vec4,
    // x = HD enabled, y = ORM present, z = team map present, w = environment present.
    pub(crate) maps: UVec4,
}

impl Default for Wc3HdUniform {
    fn default() -> Self {
        Self {
            fresnel_color: Vec4::ONE,
            fresnel: Vec4::ZERO,
            maps: UVec4::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Wc3LayerKey {
    filter: u8,
    no_depth_test: bool,
    no_depth_set: bool,
}

impl From<&Wc3LayerState> for Wc3LayerKey {
    fn from(state: &Wc3LayerState) -> Self {
        Self {
            filter: match state.filter {
                LayerFilterMode::None => 0,
                LayerFilterMode::Transparent => 1,
                LayerFilterMode::Blend => 2,
                LayerFilterMode::Additive => 3,
                LayerFilterMode::AddAlpha => 4,
                LayerFilterMode::Modulate => 5,
                LayerFilterMode::Modulate2x => 6,
                LayerFilterMode::Unknown(_) => 2,
            },
            no_depth_test: state.no_depth_test,
            no_depth_set: state.no_depth_set,
        }
    }
}

impl MaterialExtension for Wc3LayerState {
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/wc3_material.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/wc3_material_prepass.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/wc3_mesh.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/wc3_prepass.wgsl".into()
    }

    fn deferred_vertex_shader() -> ShaderRef {
        Self::prepass_vertex_shader()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if layout.0.contains(EXTRA_JOINT_INDEX) && layout.0.contains(EXTRA_JOINT_WEIGHT) {
            descriptor
                .vertex
                .shader_defs
                .push("WC3_EXTRA_INFLUENCES".into());
            let prepass = descriptor.vertex.shader_defs.iter().any(
                |def| matches!(def, ShaderDefVal::Bool(name, true) if name == "PREPASS_PIPELINE"),
            );
            let mut attrs = Vec::new();
            let mut add = |attribute: bevy::mesh::MeshVertexAttribute, location| {
                if layout.0.contains(attribute) {
                    attrs.push(attribute.at_shader_location(location));
                }
            };
            add(Mesh::ATTRIBUTE_POSITION, 0);
            if prepass {
                add(Mesh::ATTRIBUTE_UV_0, 1);
                add(Mesh::ATTRIBUTE_UV_1, 2);
                if descriptor.vertex.shader_defs.iter().any(|def| matches!(def, ShaderDefVal::Bool(name, true) if name == "NORMAL_PREPASS_OR_DEFERRED_PREPASS")) {
                    add(Mesh::ATTRIBUTE_NORMAL, 3);
                    add(Mesh::ATTRIBUTE_TANGENT, 4);
                }
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
            descriptor.vertex.buffers = vec![layout.0.get_layout(&attrs)?];
        }
        let state = key.bind_group_data;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            if state.no_depth_test {
                depth.depth_compare = Some(CompareFunction::Always);
            }
            depth.depth_write_enabled = Some(!state.no_depth_set && state.filter <= 1);
        }
        if let Some(target) = descriptor
            .fragment
            .as_mut()
            .and_then(|fragment| fragment.targets.first_mut())
            .and_then(Option::as_mut)
        {
            let factors = match state.filter {
                2 => Some((BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha)),
                3 | 4 => Some((BlendFactor::SrcAlpha, BlendFactor::One)),
                5 => Some((BlendFactor::Zero, BlendFactor::Src)),
                6 => Some((BlendFactor::Dst, BlendFactor::Src)),
                _ => None,
            };
            if let Some((src_factor, dst_factor)) = factors {
                target.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor,
                        dst_factor,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent::OVER,
                });
            }
        }
        Ok(())
    }
}
