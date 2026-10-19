//! Infallible payload embeddings preserve identities and encoded storage.
use crate::model::profiles::{Extends, Profile};
use crate::model::PropertyValues;
use crate::model::{
    AttachedFeature, Clip, Dependencies, Draw, Material, Mesh, MeshInstance, Model, Node, Skin,
};
struct PayloadMap<'a, P: Profile, Q: Profile> {
    appearance: fn(P::Appearance<'a>) -> Q::Appearance<'a>,
    geometry: fn(P::Geometry<'a>) -> Q::Geometry<'a>,
    binding: fn(P::Binding<'a>) -> Q::Binding<'a>,
    model: fn(P::ModelMetadata<'a>) -> Q::ModelMetadata<'a>,
    system: fn(P::System<'a>) -> Q::System<'a>,
    map_surface: fn(P::Surface<'a>) -> Q::Surface<'a>,
    feature: fn(P::Feature<'a>, &mut PropertyValues) -> Q::Feature<'a>,
    node: fn(P::NodeProperties<'a>) -> Q::NodeProperties<'a>,
    mesh: fn(P::MeshProperties<'a>) -> Q::MeshProperties<'a>,
    instance: fn(P::InstanceProperties<'a>) -> Q::InstanceProperties<'a>,
    draw: fn(P::DrawProperties) -> Q::DrawProperties,
    clip: fn(P::ClipProperties<'a>) -> Q::ClipProperties<'a>,
}
impl<'a, P: Profile> Model<'a, P> {
    /// Embeds the complete graph into a containing vocabulary without copying array storage.
    pub fn widen<Q: Extends<P>>(self) -> Model<'a, Q> {
        self.map_payloads(PayloadMap {
            appearance: Q::appearance,
            geometry: Q::geometry,
            binding: Q::binding,
            model: Q::model_metadata,
            system: Q::system,
            map_surface: Q::surface,
            feature: Q::feature,
            node: Q::node,
            mesh: Q::mesh,
            instance: Q::instance,
            draw: Q::draw,
            clip: Q::clip,
        })
    }
    fn map_payloads<Q: Extends<P>>(mut self, maps: PayloadMap<'a, P, Q>) -> Model<'a, Q> {
        Model {
            dependencies: Dependencies {
                images: self
                    .dependencies
                    .images
                    .into_iter()
                    .map(|(id, value)| (id, Q::image_resource(value)))
                    .collect(),
                models: self
                    .dependencies
                    .models
                    .into_iter()
                    .map(|(id, value)| (id, Q::model_resource(value)))
                    .collect(),
                effects: self
                    .dependencies
                    .effects
                    .into_iter()
                    .map(|(id, value)| (id, Q::effect_resource(value)))
                    .collect(),
                physics: self
                    .dependencies
                    .physics
                    .into_iter()
                    .map(|(id, value)| (id, Q::physics_resource(value)))
                    .collect(),
                bone_overrides: self
                    .dependencies
                    .bone_overrides
                    .into_iter()
                    .map(|(id, value)| (id, Q::bone_override_resource(value)))
                    .collect(),
            },
            scope: self.scope,
            metadata: (maps.model)(self.metadata),
            systems: self
                .systems
                .into_iter()
                .map(|(id, value)| (id, (maps.system)(value)))
                .collect(),
            event_sources: self
                .event_sources
                .into_iter()
                .map(|(id, value)| (id, Q::event_source(value)))
                .collect(),
            animations: self.animations,
            global_animations: self.global_animations,
            appearance: (maps.appearance)(self.appearance),
            skins: self
                .skins
                .into_iter()
                .map(|(id, skin)| {
                    (
                        id,
                        Skin {
                            joints: skin.joints,
                            binding: (maps.binding)(skin.binding),
                        },
                    )
                })
                .collect(),
            clocks: self.clocks,
            nodes: self
                .nodes
                .into_iter()
                .map(|(id, value)| {
                    let Node {
                        name,
                        parent,
                        transform,
                        inheritance,
                        transform_semantics,
                        billboard,
                        visible,
                        camera_anchored,
                        properties,
                    } = value;
                    (
                        id,
                        Node {
                            name,
                            parent,
                            transform,
                            inheritance,
                            transform_semantics,
                            billboard,
                            visible,
                            camera_anchored,
                            properties: (maps.node)(properties),
                        },
                    )
                })
                .collect(),
            meshes: self
                .meshes
                .into_iter()
                .map(
                    |(
                        id,
                        Mesh {
                            subsets,
                            name,
                            bounds,
                            properties,
                            geometry: source_geometry,
                        },
                    )| {
                        (
                            id,
                            Mesh {
                                subsets,
                                name,
                                bounds,
                                properties: (maps.mesh)(properties),
                                geometry: (maps.geometry)(source_geometry),
                            },
                        )
                    },
                )
                .collect(),
            materials: self
                .materials
                .into_iter()
                .map(|(id, value)| {
                    let Material { name, surface } = value;
                    (
                        id,
                        Material {
                            name,
                            surface: (maps.map_surface)(surface),
                        },
                    )
                })
                .collect(),
            features: self
                .features
                .into_iter()
                .map(|(id, value)| {
                    let AttachedFeature { node, payload } = value;
                    (
                        id,
                        AttachedFeature {
                            node,
                            payload: (maps.feature)(payload, &mut self.values),
                        },
                    )
                })
                .collect(),
            clips: self
                .clips
                .into_iter()
                .map(|(id, value)| {
                    let Clip {
                        animations,
                        source_start,
                        name,
                        duration,
                        looping,

                        properties,
                    } = value;
                    (
                        id,
                        Clip {
                            animations,
                            source_start,
                            name,
                            duration,
                            looping,

                            properties: (maps.clip)(properties),
                        },
                    )
                })
                .collect(),
            instances: self
                .instances
                .into_iter()
                .map(|(id, value)| {
                    let MeshInstance {
                        node,
                        mesh,
                        skin,
                        draws,
                        lod,
                        selection_group,
                        properties,
                    } = value;
                    (
                        id,
                        MeshInstance {
                            node,
                            mesh,
                            skin,
                            draws: draws
                                .into_iter()
                                .map(
                                    |Draw {
                                         subset,
                                         id,
                                         properties,
                                     }| Draw {
                                        subset,
                                        id,
                                        properties: (maps.draw)(properties),
                                    },
                                )
                                .collect(),
                            lod,
                            selection_group,
                            properties: (maps.instance)(properties),
                        },
                    )
                })
                .collect(),
            values: self.values,
        }
    }
}
