//! Checked payload projections preserve identities and shared storage.
use crate::model::semantic::profiles::{Narrows, Profile};
use crate::model::semantic::GeometrySchema as _;
use crate::model::semantic::{
    AttachedFeature, Clip, Controller, Draw, Error, Material, Mesh, MeshInstance, Model, Node, Skin,
};
impl<'a, P: Profile> Model<'a, P> {
    fn try_map_payloads<Q: Narrows<P>>(self) -> Result<Model<'a, Q>, Error> {
        let appearance = Q::appearance;
        let geometry = Q::geometry;
        let binding = Q::binding;
        let model = Q::model;
        let system = Q::system;
        let map_surface = Q::surface;
        let feature = Q::feature;
        let node = Q::node;
        let mesh = Q::mesh;
        let instance = Q::instance;
        let draw = Q::draw;
        let clip = Q::clip;
        let controller = Q::controller;
        Ok(Model {
            scope: self.scope,
            id: self.id,
            name: self.name,
            basis: self.basis,
            units: self.units,
            bounds: self.bounds,
            controls: model(self.controls)?,
            systems: self
                .systems
                .into_iter()
                .map(|(id, value)| Ok((id, system(value)?)))
                .collect::<Result<_, Error>>()?,
            timeline_channels: self
                .timeline_channels
                .into_iter()
                .map(|channel| channel.map_profile(Q::property))
                .collect::<Result<_, Error>>()?,
            timeline_events: self.timeline_events,
            shared_channels: self
                .shared_channels
                .into_iter()
                .map(|channel| channel.map_profile(Q::property))
                .collect::<Result<_, Error>>()?,
            shared_events: self.shared_events,
            appearance: appearance(self.appearance)?,
            skins: self
                .skins
                .into_iter()
                .map(|(id, skin)| {
                    Ok((
                        id,
                        Skin {
                            joints: skin.joints,
                            binding: binding(skin.binding)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
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
                    Ok((
                        id,
                        Node {
                            name,
                            parent,
                            transform,
                            inheritance,
                            billboard,
                            visible,
                            camera_anchored,
                            controls: node(controls)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
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
                            geometry: mut source_geometry,
                        },
                    )| {
                        source_geometry.normalize_primary_domains(subsets.primary())?;
                        Ok((
                            id,
                            Mesh {
                                subsets,
                                name,
                                bounds,
                                controls: mesh(controls)?,
                                geometry: geometry(source_geometry)?,
                            },
                        ))
                    },
                )
                .collect::<Result<_, Error>>()?,
            materials: self
                .materials
                .into_iter()
                .map(|(id, value)| {
                    let Material { name, surface } = value;
                    Ok((
                        id,
                        Material {
                            name,
                            surface: map_surface(surface)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
            features: self
                .features
                .into_iter()
                .map(|(id, value)| {
                    let AttachedFeature { node, payload } = value;
                    Ok((
                        id,
                        AttachedFeature {
                            node,
                            payload: feature(payload)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
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
                    Ok((
                        id,
                        Clip {
                            alias,
                            name,
                            duration,
                            looping,
                            channels: channels
                                .into_iter()
                                .map(|channel| channel.map_profile(Q::property))
                                .collect::<Result<_, Error>>()?,
                            events,
                            controls: clip(controls)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
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
                    Ok((
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
                                         material,
                                         controls,
                                     }| {
                                        Ok(Draw {
                                            subset,
                                            material,
                                            controls: draw(controls)?,
                                        })
                                    },
                                )
                                .collect::<Result<_, Error>>()?,
                            lod,
                            selection_group,
                            controls: instance(controls)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
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
                    Ok((
                        id,
                        Controller {
                            name,
                            priority,
                            concurrent,
                            channels: channels
                                .into_iter()
                                .map(|channel| channel.map_profile(Q::property))
                                .collect::<Result<_, Error>>()?,
                            events,
                            controls: controller(controls)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
        })
    }
}
impl<'a, P: Profile> Model<'a, P> {
    /// Projects into a smaller vocabulary exactly, rejecting incompatible authored data.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<Model<'a, Q>, Error> {
        let model = self.try_map_payloads::<Q>()?;
        Q::validate_model(&model)?;
        Ok(model)
    }
}
