//! Detach storage while retaining graph scopes, identities and authored values.
use super::Model;
use crate::model::common::{Detached, IntoOwned};
use crate::model::profiles::Profile;
use crate::model::{
    AttachedFeature, Clip, Dependencies, InstanceProperties, Material, Mesh, MeshInstance, Node,
    Skin,
};

impl<'owned> Detached<'owned> for InstanceProperties<'_> {
    type Detached = InstanceProperties<'owned>;
    fn into_detached(self) -> Self::Detached {
        match self {
            Self::Wmo(value) => InstanceProperties::Wmo(value.into_detached()),
        }
    }
}

impl<'data, 'owned, P: Profile> Detached<'owned> for Dependencies<'data, P>
where
    P::ImageResource<'data>: Detached<'owned, Detached = P::ImageResource<'owned>>,
    P::ModelResource<'data>: Detached<'owned, Detached = P::ModelResource<'owned>>,
    P::EffectResource<'data>: Detached<'owned, Detached = P::EffectResource<'owned>>,
    P::PhysicsResource<'data>: Detached<'owned, Detached = P::PhysicsResource<'owned>>,
    P::BoneOverrideResource<'data>: Detached<'owned, Detached = P::BoneOverrideResource<'owned>>,
{
    type Detached = Dependencies<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        Dependencies {
            images: self.images.into_detached(),
            models: self.models.into_detached(),
            effects: self.effects.into_detached(),
            physics: self.physics.into_detached(),
            bone_overrides: self.bone_overrides.into_detached(),
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for Node<'data, P>
where
    P::NodeProperties<'data>: Detached<'owned, Detached = P::NodeProperties<'owned>>,
{
    type Detached = Node<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        Node {
            name: self.name.into_detached(),
            parent: self.parent,
            transform: self.transform,
            inheritance: self.inheritance,
            transform_semantics: self.transform_semantics,
            billboard: self.billboard,
            visible: self.visible,
            camera_anchored: self.camera_anchored,
            properties: self.properties.into_detached(),
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for Mesh<'data, P>
where
    P::MeshProperties<'data>: Detached<'owned, Detached = P::MeshProperties<'owned>>,
    P::Geometry<'data>: Detached<'owned, Detached = P::Geometry<'owned>>,
{
    type Detached = Mesh<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        Mesh {
            subsets: self.subsets,
            properties: self.properties.into_detached(),
            name: self.name.into_detached(),
            geometry: self.geometry.into_detached(),
            bounds: self.bounds,
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for Material<'data, P>
where
    P::Surface<'data>: Detached<'owned, Detached = P::Surface<'owned>>,
{
    type Detached = Material<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        Material {
            name: self.name.into_detached(),
            surface: self.surface.into_detached(),
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for AttachedFeature<'data, P>
where
    P::Feature<'data>: Detached<'owned, Detached = P::Feature<'owned>>,
{
    type Detached = AttachedFeature<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        AttachedFeature {
            node: self.node,
            payload: self.payload.into_detached(),
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for Clip<'data, P>
where
    P::ClipProperties<'data>: Detached<'owned, Detached = P::ClipProperties<'owned>>,
{
    type Detached = Clip<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        Clip {
            properties: self.properties.into_detached(),
            animations: self.animations,
            source_start: self.source_start,
            name: self.name.into_detached(),
            duration: self.duration,
            looping: self.looping,
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for Skin<'data, P>
where
    P::Binding<'data>: Detached<'owned, Detached = P::Binding<'owned>>,
{
    type Detached = Skin<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        Skin {
            joints: self.joints,
            binding: self.binding.into_detached(),
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for MeshInstance<'data, P>
where
    P::InstanceProperties<'data>: Detached<'owned, Detached = P::InstanceProperties<'owned>>,
{
    type Detached = MeshInstance<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        MeshInstance {
            properties: self.properties.into_detached(),
            node: self.node,
            mesh: self.mesh,
            skin: self.skin,
            draws: self.draws,
            lod: self.lod,
            selection_group: self.selection_group,
        }
    }
}
impl<'data, 'owned, P: Profile> Detached<'owned> for Model<'data, P>
where
    Dependencies<'data, P>: Detached<'owned, Detached = Dependencies<'owned, P>>,
    P::ModelMetadata<'data>: Detached<'owned, Detached = P::ModelMetadata<'owned>>,
    P::System<'data>: Detached<'owned, Detached = P::System<'owned>>,
    P::EventSource<'data>: Detached<'owned, Detached = P::EventSource<'owned>>,
    P::Appearance<'data>: Detached<'owned, Detached = P::Appearance<'owned>>,
    AttachedFeature<'data, P>: Detached<'owned, Detached = AttachedFeature<'owned, P>>,
    Node<'data, P>: Detached<'owned, Detached = Node<'owned, P>>,
    Mesh<'data, P>: Detached<'owned, Detached = Mesh<'owned, P>>,
    Material<'data, P>: Detached<'owned, Detached = Material<'owned, P>>,
    Clip<'data, P>: Detached<'owned, Detached = Clip<'owned, P>>,
    Skin<'data, P>: Detached<'owned, Detached = Skin<'owned, P>>,
    MeshInstance<'data, P>: Detached<'owned, Detached = MeshInstance<'owned, P>>,
{
    type Detached = Model<'owned, P>;
    fn into_detached(self) -> Self::Detached {
        Model {
            values: self.values,
            dependencies: self.dependencies.into_detached(),
            scope: self.scope,
            metadata: self.metadata.into_detached(),
            systems: self.systems.into_detached(),
            event_sources: self.event_sources.into_detached(),
            animations: self.animations.into_detached(),
            global_animations: self.global_animations,
            features: self.features.into_detached(),
            appearance: self.appearance.into_detached(),
            nodes: self.nodes.into_detached(),
            meshes: self.meshes.into_detached(),
            materials: self.materials.into_detached(),
            clips: self.clips.into_detached(),
            skins: self.skins.into_detached(),
            instances: self.instances.into_detached(),
            clocks: self.clocks,
        }
    }
}
impl<'data, P: Profile> Model<'data, P> {
    /// Detaches borrowed buffers while preserving every graph and property identity.
    /// Already owned buffers move without re-encoding or changing authored values.
    pub fn into_owned(self) -> Model<'static, P>
    where
        Self: IntoOwned<Owned = Model<'static, P>>,
    {
        IntoOwned::into_owned(self)
    }
}
