//! Standalone material-resource conversion retains shader inputs and owner-local declarations.
use super::materials::{read_shader, write_shader};
use crate::model::conversion::{Context, Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::diablo::d4::{self, records as native};
use crate::model::semantic::profiles::diablo::d4::{
    Material as Surface, MaterialAssetControls, D4,
};
use crate::model::semantic::{Material, MaterialId, Model, Resources};

/// Imports one standalone material resource as a checked material-only graph.
pub fn import_material<'a>(
    source: &'a d4::Material<'_>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, D4>>, Error> {
    let root = &source.root;
    let mut context = Context::import(options);
    if !root.decal_look_variant_map.is_empty()
        || root
            .biome_settings
            .iter()
            .any(|setting| *setting != Default::default())
        || root.parametric_clutter_index != Default::default()
        || root.path_playback_multiplier != Default::default()
        || root.path_playback_controller != Default::default()
    {
        context.unsupported("d4.material.programs", "decal variants, biome/clutter programs, and playback controller paths require additional semantic declarations")?;
    }
    let mut resources = Resources::<D4>::builder();
    let mut shader = read_shader(&root.uber_material, &mut resources, &mut context)?;
    shader.asset = Some(MaterialAssetControls {
        sno_id: root.sno_id,
        record_flags: root.record_flags,
        flags: root.flags,
        surface: root.surface,
        unclassified_scalars: [root.unk_93ea20a, root.unk_a08119e, root.unk_5b1ce37],
        unclassified_unsigned: root.unk_ce6fe3b,
        unclassified_signed: [root.unk_245935, root.unk_6d58226],
        duration: root.duration,
        outro_duration: root.outro_duration,
        stiffness: root.stiffness,
        gravity_magnitude: root.gravity_magnitude,
        animated_influences: root.fx_animated_influences.indices,
    });
    let mut model = Model::<D4>::builder("");
    model.set_resources(resources.finish());
    model.add_material(Material {
        name: "".into(),
        surface: Surface {
            shader: Some(shader),
            ..Default::default()
        },
    })?;
    Ok(Conversion {
        value: model.finish()?,
        report: context.report,
    })
}
/// Plans one actual material resource independently of appearance document output.
pub fn plan_material(
    asset: &Model<'_, D4>,
    material: MaterialId,
    options: &ExportOptions,
) -> Result<Plan<d4::Material<'static>>, Error> {
    asset.validate()?;
    let model = asset;
    let surface = &model
        .material(material)
        .ok_or_else(|| Error::new("d4.material", "material belongs to another model"))?
        .surface;
    let shader = surface.shader.as_ref().ok_or_else(|| {
        Error::new(
            "d4.material.shader",
            "standalone material output requires a supplied shader definition",
        )
    })?;
    let controls = shader.asset.as_ref().ok_or_else(|| {
        Error::new(
            "d4.material.asset",
            "standalone output requires material-resource declarations",
        )
    })?;
    let mut context = Context::export(options);
    if options.look.is_some() {
        return Err(Error::new(
            "d4.material.look",
            "standalone material output has no draw-based look selection",
        ));
    }

    let mut output = d4::Material::default();
    output.root = native::MaterialRecord {
        sno_id: controls.sno_id,
        record_flags: controls.record_flags,
        flags: controls.flags,
        surface: controls.surface,
        uber_material: write_shader(shader, asset.resources(), &mut context)?,
        unk_93ea20a: controls.unclassified_scalars[0],
        unk_a08119e: controls.unclassified_scalars[1],
        unk_5b1ce37: controls.unclassified_scalars[2],
        unk_ce6fe3b: controls.unclassified_unsigned,
        unk_245935: controls.unclassified_signed[0],
        unk_6d58226: controls.unclassified_signed[1],
        duration: controls.duration,
        outro_duration: controls.outro_duration,
        stiffness: controls.stiffness,
        gravity_magnitude: controls.gravity_magnitude,
        fx_animated_influences: native::FxAnimatedInfluences {
            indices: controls.animated_influences,
        },
        ..Default::default()
    };
    Ok(Plan::new(output, context.report))
}
