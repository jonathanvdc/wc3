//! Infallible payload embeddings preserve identities and encoded storage.
use crate::model::profiles::{Extends, Profile};
use crate::model::{
    AttachedFeature, Clip, Controller, Draw, Material, Mesh, MeshInstance, Model, Node, Skin,
};
struct PayloadMap<'a, P: Profile, Q: Profile> {
    texture: fn(P::Texture<'a>) -> Q::Texture<'a>,
    property: fn(P::Property) -> Q::Property,
    appearance: fn(P::Appearance<'a>) -> Q::Appearance<'a>,
    geometry: fn(P::Geometry<'a>) -> Q::Geometry<'a>,
    binding: fn(P::Binding<'a>) -> Q::Binding<'a>,
    model: fn(P::ModelMetadata<'a>) -> Q::ModelMetadata<'a>,
    system: fn(P::System<'a>) -> Q::System<'a>,
    map_surface: fn(P::Surface<'a>) -> Q::Surface<'a>,
    feature: fn(P::Feature<'a>) -> Q::Feature<'a>,
    node: fn(P::Node<'a>) -> Q::Node<'a>,
    mesh: fn(P::Mesh<'a>) -> Q::Mesh<'a>,
    instance: fn(P::Instance<'a>) -> Q::Instance<'a>,
    draw: fn(P::Draw) -> Q::Draw,
    clip: fn(P::Clip<'a>) -> Q::Clip<'a>,
    controller: fn(P::Controller<'a>) -> Q::Controller<'a>,
}
impl<'a, P: Profile> Model<'a, P> {
    /// Embeds the complete graph into a containing vocabulary without copying array storage.
    pub fn widen<Q: Extends<P>>(self) -> Model<'a, Q> {
        self.map_payloads(PayloadMap {
            texture: Q::texture,
            property: Q::property,
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
            controller: Q::controller,
        })
    }
    fn map_payloads<Q: Profile>(self, maps: PayloadMap<'a, P, Q>) -> Model<'a, Q> {
        Model {
            resources: self.resources.map_controls(maps.texture),
            scope: self.scope,
            id: self.id,
            name: self.name,
            basis: self.basis,
            units: self.units,
            bounds: self.bounds,
            metadata: (maps.model)(self.metadata),
            systems: self
                .systems
                .into_iter()
                .map(|(id, value)| (id, (maps.system)(value)))
                .collect(),
            timeline_channels: self
                .timeline_channels
                .into_iter()
                .map(|channel| {
                    channel
                        .map_profile(|key| Ok((maps.property)(key)))
                        .expect("infallible property embedding")
                })
                .collect(),
            timeline_events: self.timeline_events,
            shared_channels: self
                .shared_channels
                .into_iter()
                .map(|channel| {
                    channel
                        .map_profile(|key| Ok((maps.property)(key)))
                        .expect("infallible property embedding")
                })
                .collect(),
            shared_events: self.shared_events,
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
            texture_transforms: self.texture_transforms,
            nodes: self
                .nodes
                .into_iter()
                .map(|(id, value)| {
                    let Node {
                        name,
                        parent,
                        transform,
                        inheritance,
                        billboard,
                        visible,
                        camera_anchored,
                        controls,
                    } = value;
                    (
                        id,
                        Node {
                            name,
                            parent,
                            transform,
                            inheritance,
                            billboard,
                            visible,
                            camera_anchored,
                            controls: (maps.node)(controls),
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
                            controls,
                            geometry: source_geometry,
                        },
                    )| {
                        (
                            id,
                            Mesh {
                                subsets,
                                name,
                                bounds,
                                controls: (maps.mesh)(controls),
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
                            payload: (maps.feature)(payload),
                        },
                    )
                })
                .collect(),
            clips: self
                .clips
                .into_iter()
                .map(|(id, value)| {
                    let Clip {
                        alias,
                        name,
                        duration,
                        looping,
                        channels,
                        events,
                        controls,
                    } = value;
                    (
                        id,
                        Clip {
                            alias,
                            name,
                            duration,
                            looping,
                            channels: channels
                                .into_iter()
                                .map(|channel| {
                                    channel
                                        .map_profile(|key| Ok((maps.property)(key)))
                                        .expect("infallible property embedding")
                                })
                                .collect(),
                            events,
                            controls: (maps.clip)(controls),
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
                        controls,
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
                                         controls,
                                     }| Draw {
                                        subset,
                                        id,
                                        controls: (maps.draw)(controls),
                                    },
                                )
                                .collect(),
                            lod,
                            selection_group,
                            controls: (maps.instance)(controls),
                        },
                    )
                })
                .collect(),
            controllers: self
                .controllers
                .into_iter()
                .map(|(id, value)| {
                    let Controller {
                        name,
                        priority,
                        concurrent,
                        channels,
                        events,
                        controls,
                    } = value;
                    (
                        id,
                        Controller {
                            name,
                            priority,
                            concurrent,
                            channels: channels
                                .into_iter()
                                .map(|channel| {
                                    channel
                                        .map_profile(|key| Ok((maps.property)(key)))
                                        .expect("infallible property embedding")
                                })
                                .collect(),
                            events,
                            controls: (maps.controller)(controls),
                        },
                    )
                })
                .collect(),
        }
    }
}
