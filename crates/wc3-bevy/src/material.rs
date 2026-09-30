use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, CompareFunction,
    RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use wc3::model::materials::LayerFilterMode;

/// A Bevy PBR material with WC3 layer render state.
pub type Wc3LayerMaterial = ExtendedMaterial<StandardMaterial, Wc3LayerState>;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(Wc3LayerKey)]
pub struct Wc3LayerState {
    pub(crate) filter: LayerFilterMode,
    pub(crate) no_depth_test: bool,
    pub(crate) no_depth_set: bool,
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
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
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
