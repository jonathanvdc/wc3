//! The editable-model vocabulary retains its own mixed-family authoring domains.
use super::attachment_validation::usable_face;
use super::properties::Target as PropertySelector;
use super::{
    Appearance, ClipControls, ControllerControls, DocumentControls, Feature, Geometry, Material,
    MeshControls, ModelControls, NodeControls, System, TextureControls,
};
use crate::model::semantic::profiles::UniversalProperty;
use crate::model::semantic::profiles::{Extends, NoControls, Profile, Unavailable, Universal};
use crate::model::semantic::{Basis, LengthUnit};
use crate::model::semantic::{Error, Model, Multiple, NodeFeature, Resources};

/// Vocabulary of an editable model document, including mixed actor and surface families.
#[derive(Clone, Debug, Default)]
pub struct Wem;
impl Profile for Wem {
    type Property = PropertySelector;
    fn default_bounds() -> Option<crate::model::semantic::Bounds> {
        Some(Default::default())
    }
    type Models = Multiple;
    type Asset<'a> = DocumentControls<'a>;
    type Appearance<'a> = Appearance<'a>;
    type Binding<'a> = Unavailable;
    type Geometry<'a> = Geometry<'a>;
    type Model<'a> = ModelControls<'a>;
    type System<'a> = System<'a>;
    type Surface<'a> = Material<'a>;
    type Feature<'a> = Feature<'a>;
    type Node<'a> = NodeControls<'a>;
    type Texture<'a> = TextureControls<'a>;
    type Mesh<'a> = MeshControls;
    type Instance<'a> = NoControls;
    type Draw = NoControls;
    type Clip<'a> = ClipControls<'a>;
    type Controller<'a> = ControllerControls<'a>;

    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        if model.bounds().is_none() {
            return Err(Error::new(
                "model.bounds",
                "this profile requires an authored bounding volume",
            ));
        }

        if model.basis() != Basis::default() || model.units() != LengthUnit::Unspecified {
            return Err(Error::new(
                "wem.model.coordinates",
                "per-model axes and units have no separate declaration; document convention and unit scale belong to collection controls",
            ));
        }
        if model.nodes().any(|(_, node)| node.camera_anchored) {
            return Err(Error::new(
                "wem.node.camera_anchored",
                "the editable hierarchy has no camera-origin anchor declaration",
            ));
        }

        model.controls().pose_pipeline.validate_simulations(
            |system,id|matches!(model.system(system),Some(super::System::Physics(value)) if value.rig(id).is_some()),
            |system,id|matches!(model.system(system),Some(super::System::Physics(value)) if value.cloth(id).is_some()),
        )?;
        model
            .controls()
            .channels
            .validate_simulations(&model.controls().pose_pipeline, |id| {
                match model.system(id) {
                    Some(System::Physics(value)) => Some(value),
                    _ => None,
                }
            })?;
        if model.systems().len() > 1 {
            return Err(Error::new(
                "wem.physics.systems",
                "an editable model owns one simulation domain",
            ));
        }
        if !crate::model::conversion::wem::supports_node_declarations(model) {
            return Err(Error::new(
                "wem.nodes.capabilities",
                "no shared node vocabulary preserves the authored setup and actor declarations",
            ));
        }
        if model.instances().len() != 0 {
            return Err(Error::new(
                "wem.instances",
                "WEM owns polygon groups directly rather than draw instances",
            ));
        }
        for (id, _) in model.materials() {
            if !model
                .appearance()
                .sets()
                .any(|(_, set)| set.materials.contains(&id))
            {
                return Err(Error::new(
                    "wem.material",
                    "material must belong to an authored shader convention set",
                ));
            }
        }
        let mut setup_presence = None;
        for (id, node) in model.nodes() {
            let presence = (
                node.controls.setup.skin.is_some(),
                node.controls.setup.rig.is_some(),
            );
            if setup_presence.is_some_and(|previous| previous != presence) {
                return Err(Error::new(
                    "wem.node.setup",
                    "all nodes share the same skin and rig declaration vocabulary",
                ));
            }
            setup_presence = Some(presence);

            if model.node_features(id).count() + usize::from(node.controls.bone.is_some()) > 1 {
                return Err(Error::new(
                    "wem.node.payload",
                    "an editable-model node declares one attached payload",
                ));
            }
            node.controls
                .setup
                .validate_native(&model.controls().poses)?;
        }
        super::animation::validation::playback(model.clips().map(|(_, clip)| &clip.controls))?;
        super::animation::validation::programmes(
            model.clips().map(|(_, clip)| &clip.controls),
            model.controllers().map(|(id, _)| id),
        )?;
        model
            .controls()
            .channel_selections
            .validate(&model.controls().channels)?;
        for (_, clip) in model.clips() {
            if let Some(recipe) = &clip.controls.physics {
                recipe.validate(
                    &model.controls().channels,
                    |id| {
                        model
                            .controller(id)
                            .map(|value| value.controls.tracks.len())
                    },
                    |id| match model.system(id) {
                        Some(System::Physics(value)) => Some(value),
                        _ => None,
                    },
                )?;
            }
            if !clip.channels().is_empty() || !clip.events().is_empty() || clip.alias().is_some() {
                return Err(Error::new(
                    "wem.animation.clip",
                    "editable-model clips own programme selections and typed event occurrences",
                ));
            }
            if clip.controls.selections.iter().any(|entry| {
                model
                    .controls()
                    .channel_selections
                    .get(entry.selection)
                    .is_none()
            }) {
                return Err(Error::new(
                    "wem.animation.selection",
                    "selection belongs to another model",
                ));
            }
        }
        for (_, controller) in model.controllers() {
            if !controller.channels.is_empty() || !controller.events.is_empty() {
                return Err(Error::new(
                    "wem.animation.controller",
                    "editable-model programs select declarations through their typed controls",
                ));
            }
            for track in &controller.controls.tracks {
                track.validate(&model.controls().channels)?;
            }
        }
        for (_, system) in model.systems() {
            match system {
                System::Physics(physics) => {
                    for (_, body) in physics.soft_bodies() {
                        body.validate_mesh_faces(|mesh, face| {
                            model
                                .mesh(mesh)
                                .is_some_and(|mesh| usable_face(mesh.topology(), face as usize))
                        })?;
                    }
                    for (_, cloth) in physics.cloths() {
                        for section in cloth.recipe.iter().flat_map(|recipe| {
                            recipe
                                .binding_origins
                                .iter()
                                .chain(recipe.cage_origin.iter())
                        }) {
                            if !model.mesh(section.mesh).is_some_and(|mesh| {
                                mesh.geometry.section(section.section).is_some()
                            }) {
                                return Err(Error::new(
                                    "wem.cloth.section",
                                    "cloth recipe section belongs to another mesh",
                                ));
                            }
                        }
                    }
                }
            }
        }
        for (_, feature) in model.features() {
            if let Feature::Starcraft(super::actors::starcraft::Feature::Particle(value)) =
                &feature.payload
            {
                value.validate_sections(|section| {
                    model
                        .meshes()
                        .next()
                        .is_some_and(|(_, mesh)| mesh.geometry.section(section).is_some())
                })?;
            }
        }
        Ok(())
    }
    fn validate_asset<'a>(
        controls: &Self::Asset<'a>,
        models: &[Model<'a, Self>],
        resources: &Resources<'a, Self>,
    ) -> Result<(), Error> {
        controls.validate()?;
        if !crate::model::conversion::wem::supports_model_declarations(models) {
            return Err(Error::new(
                "wem.models.capabilities",
                "collection models require incompatible hierarchy, LOD, or simulation declaration vocabularies",
            ));
        }
        super::animation::validation::playback(
            models
                .iter()
                .flat_map(|model| model.clips().map(|(_, clip)| &clip.controls)),
        )?;
        super::texture::resources(resources)?;
        for model in models {
            if model
                .controls()
                .animation_set
                .is_some_and(|id| controls.animation_set(id).is_none())
            {
                return Err(Error::new(
                    "wem.model.animation_set",
                    "animation selection program belongs to another collection",
                ));
            }
        }
        Ok(())
    }
}

impl Extends<Wem> for Universal {
    fn property(value: <Wem as Profile>::Property) -> Self::Property {
        UniversalProperty::Wem(value)
    }
    fn asset<'a>(value: <Wem as Profile>::Asset<'a>) -> Self::Asset<'a> {
        Some(value)
    }
    fn appearance<'a>(value: <Wem as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        crate::model::semantic::UniversalAppearance::Wem(value)
    }
    fn binding<'a>(value: <Wem as Profile>::Binding<'a>) -> Self::Binding<'a> {
        match value {}
    }
    fn geometry<'a>(value: <Wem as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        crate::model::semantic::UniversalGeometry::Wem(value)
    }
    fn draw(_: NoControls) -> Self::Draw {
        None
    }
    fn model<'a>(value: <Wem as Profile>::Model<'a>) -> Self::Model<'a> {
        Some(crate::model::semantic::ModelControls::Wem(value))
    }
    fn system<'a>(value: <Wem as Profile>::System<'a>) -> Self::System<'a> {
        crate::model::semantic::System::Wem(value)
    }
    fn surface<'a>(value: <Wem as Profile>::Surface<'a>) -> Self::Surface<'a> {
        crate::model::semantic::MaterialSurface::Wem(Box::new(value))
    }
    fn feature<'a>(value: <Wem as Profile>::Feature<'a>) -> Self::Feature<'a> {
        NodeFeature::Wem(Box::new(value))
    }
    fn node<'a>(value: <Wem as Profile>::Node<'a>) -> Self::Node<'a> {
        Some(crate::model::semantic::NodeControls::Wem(value))
    }
    fn texture<'a>(value: <Wem as Profile>::Texture<'a>) -> Self::Texture<'a> {
        Some(crate::model::semantic::TextureControls::Wem(value))
    }
    fn mesh<'a>(value: <Wem as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        Some(crate::model::semantic::MeshControls::Wem(value))
    }
    fn instance<'a>(_: <Wem as Profile>::Instance<'a>) -> Self::Instance<'a> {
        None
    }
    fn clip<'a>(value: <Wem as Profile>::Clip<'a>) -> Self::Clip<'a> {
        Some(crate::model::semantic::ClipControls::Wem(Box::new(value)))
    }
    fn controller<'a>(value: <Wem as Profile>::Controller<'a>) -> Self::Controller<'a> {
        Some(crate::model::semantic::ControllerControls::Wem(value))
    }
}

pub(crate) fn validate_universal_model(model: &Model<'_, Universal>) -> Result<(), Error> {
    use crate::model::semantic::{ModelControls, NodeControls, System, UniversalGeometry};
    let poses = match model.controls() {
        Some(ModelControls::Wem(controls)) => {
            controls.pose_pipeline.validate_simulations(
                |system,id|matches!(model.system(system),Some(System::Wem(super::System::Physics(value))) if value.rig(id).is_some()),
                |system,id|matches!(model.system(system),Some(System::Wem(super::System::Physics(value))) if value.cloth(id).is_some()),
            )?;
            controls
                .channels
                .validate_simulations(&controls.pose_pipeline, |id| match model.system(id) {
                    Some(System::Wem(super::System::Physics(value))) => Some(value),
                    _ => None,
                })?;
            controls.channel_selections.validate(&controls.channels)?;
            Some(&controls.poses)
        }
        _ => None,
    };
    let empty = super::poses::Poses::default();
    for (_, node) in model.nodes() {
        if let Some(NodeControls::Wem(controls)) = &node.controls {
            controls.setup.validate(poses.unwrap_or(&empty))?;
        }
    }
    for (_, clip) in model.clips() {
        if let Some(crate::model::semantic::ClipControls::Wem(controls)) = &clip.controls {
            if !clip.channels().is_empty() || !clip.events().is_empty() || clip.alias().is_some() {
                return Err(Error::new(
                    "wem.animation.clip",
                    "editable-model clip has a competing common programme",
                ));
            }
            let selections = match model.controls() {
                Some(ModelControls::Wem(value)) => &value.channel_selections,
                _ => {
                    return Err(Error::new(
                        "wem.animation.selection",
                        "model selection declarations are absent",
                    ));
                }
            };
            if let Some(recipe) = &controls.physics {
                let channels = match model.controls() {
                    Some(ModelControls::Wem(value)) => &value.channels,
                    _ => {
                        return Err(Error::new(
                            "wem.physics.bake",
                            "channel declarations are absent",
                        ));
                    }
                };
                recipe.validate(
                    channels,
                    |id| {
                        model
                            .controller(id)
                            .and_then(|value| match &value.controls {
                                Some(crate::model::semantic::ControllerControls::Wem(value)) => {
                                    Some(value.tracks.len())
                                }
                                _ => None,
                            })
                    },
                    |id| match model.system(id) {
                        Some(System::Wem(super::System::Physics(value))) => Some(value),
                        _ => None,
                    },
                )?;
            }
            if controls
                .selections
                .iter()
                .any(|entry| selections.get(entry.selection).is_none())
            {
                return Err(Error::new(
                    "wem.animation.selection",
                    "selection belongs to another model",
                ));
            }
        }
    }
    for (_, controller) in model.controllers() {
        if let Some(crate::model::semantic::ControllerControls::Wem(controls)) =
            &controller.controls
        {
            let channels = match model.controls() {
                Some(ModelControls::Wem(controls)) => &controls.channels,
                _ => {
                    return Err(Error::new(
                        "wem.animation.controller",
                        "editable-model channel declarations are absent",
                    ));
                }
            };
            if !controller.channels.is_empty() || !controller.events.is_empty() {
                return Err(Error::new(
                    "wem.animation.controller",
                    "editable-model programs cannot own a second common program",
                ));
            }
            for track in &controls.tracks {
                track.validate(channels)?;
            }
        }
    }
    for (_, system) in model.systems() {
        if let System::Wem(super::System::Physics(physics)) = system {
            for (_, body) in physics.soft_bodies() {
                body.validate_mesh_faces(|mesh, face| {
                    model
                        .mesh(mesh)
                        .is_some_and(|mesh| usable_face(mesh.topology(), face as usize))
                })?;
            }
            for (_, cloth) in physics.cloths() {
                for section in cloth.recipe.iter().flat_map(|recipe| {
                    recipe
                        .binding_origins
                        .iter()
                        .chain(recipe.cage_origin.iter())
                }) {
                    if !model.mesh(section.mesh).is_some_and(|mesh|matches!(&mesh.geometry,UniversalGeometry::Wem(geometry) if geometry.section(section.section).is_some())){return Err(Error::new("wem.cloth.section","cloth recipe group belongs to another mesh"));}
                }
            }
        }
    }
    for (_, feature) in model.features() {
        if let NodeFeature::Wem(feature) = &feature.payload {
            if let Feature::Starcraft(super::actors::starcraft::Feature::Particle(value)) =
                feature.as_ref()
            {
                value.validate_sections(|section|model.meshes().next().is_some_and(|(_,mesh)|matches!(&mesh.geometry,UniversalGeometry::Wem(geometry) if geometry.section(section).is_some())))?;
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_universal_asset(
    controls: Option<&DocumentControls<'_>>,
    models: &[Model<'_, Universal>],
) -> Result<(), Error> {
    if let Some(controls) = controls {
        controls.validate()?;
    }
    for model in models {
        if let Some(crate::model::semantic::ModelControls::Wem(model_controls)) = model.controls() {
            if model_controls.animation_set.is_some_and(|id| {
                controls.is_none_or(|controls| controls.animation_set(id).is_none())
            }) {
                return Err(Error::new(
                    "wem.model.animation_set",
                    "selection program belongs to another model collection",
                ));
            }
        }
    }
    Ok(())
}
