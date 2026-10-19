//! Checked payload projections preserve identities and encoded storage.
use crate::model::profiles::{Narrows, Profile};
use crate::model::GeometrySchema as _;
use crate::model::{
    AttachedFeature, Clip, Dependencies, Draw, Error, Material, Mesh, MeshInstance, Model, Node,
    Skin,
};
impl<'a, P: Profile> Model<'a, P> {
    fn try_map_payloads<Q: Narrows<P>>(self) -> Result<Model<'a, Q>, Error> {
        let appearance = Q::appearance;
        let geometry = Q::geometry;
        let binding = Q::binding;
        let model = Q::model_metadata;
        let system = Q::system;
        let map_surface = Q::surface;
        let feature = Q::feature;
        let node = Q::node;
        let mesh = Q::mesh;
        let instance = Q::instance;
        let draw = Q::draw;
        let clip = Q::clip;
        Ok(Model {
            dependencies: Dependencies {
                images: self
                    .dependencies
                    .images
                    .into_iter()
                    .map(|(id, value)| Ok((id, Q::image_resource(value)?)))
                    .collect::<Result<_, Error>>()?,
                models: self
                    .dependencies
                    .models
                    .into_iter()
                    .map(|(id, value)| Ok((id, Q::model_resource(value)?)))
                    .collect::<Result<_, Error>>()?,
                effects: self
                    .dependencies
                    .effects
                    .into_iter()
                    .map(|(id, value)| Ok((id, Q::effect_resource(value)?)))
                    .collect::<Result<_, Error>>()?,
                physics: self
                    .dependencies
                    .physics
                    .into_iter()
                    .map(|(id, value)| Ok((id, Q::physics_resource(value)?)))
                    .collect::<Result<_, Error>>()?,
                bone_overrides: self
                    .dependencies
                    .bone_overrides
                    .into_iter()
                    .map(|(id, value)| Ok((id, Q::bone_override_resource(value)?)))
                    .collect::<Result<_, Error>>()?,
            },
            scope: self.scope,
            metadata: model(self.metadata)?,
            systems: self
                .systems
                .into_iter()
                .map(|(id, value)| Ok((id, system(value)?)))
                .collect::<Result<_, Error>>()?,
            event_sources: self
                .event_sources
                .into_iter()
                .map(|(id, value)| Ok((id, Q::event_source(value)?)))
                .collect::<Result<_, Error>>()?,
            animations: self.animations,
            global_animations: self.global_animations,
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
                    Ok((
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
                            properties: node(properties)?,
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
                            properties,
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
                                properties: mesh(properties)?,
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
                            payload: feature(payload, &self.values)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
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
                    Ok((
                        id,
                        Clip {
                            animations,
                            source_start,
                            name,
                            duration,
                            looping,

                            properties: clip(properties)?,
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
                        properties,
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
                                         id,
                                         properties,
                                     }| {
                                        Ok(Draw {
                                            subset,
                                            id,
                                            properties: draw(properties)?,
                                        })
                                    },
                                )
                                .collect::<Result<_, Error>>()?,
                            lod,
                            selection_group,
                            properties: instance(properties)?,
                        },
                    ))
                })
                .collect::<Result<_, Error>>()?,
            values: self.values,
        })
    }
}
impl<'a, P: Profile> Model<'a, P> {
    /// Projects into a smaller vocabulary exactly, rejecting incompatible authored data.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<Model<'a, Q>, Error> {
        Q::validate_projection(&self)?;
        let model = self.try_map_payloads::<Q>()?;
        model.validate()?;
        Ok(model)
    }
}
