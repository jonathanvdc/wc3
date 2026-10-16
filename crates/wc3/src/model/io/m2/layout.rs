//! Static target dialect capabilities and version-selected field families.
use super::*;
use zerovec::ZeroVec;

/// Native skin placement selected by the target core layout.
///
/// Embedded profiles stay in classic cores; modern profiles become companion
/// documents whose identities are derived from the root model identity.
pub trait ExportLayout: ModelLayout {
    /// Installs generated profiles without erasing the selected record layout.
    fn install_profiles<D: Dialect<Layout = Self>>(
        bundle: &mut Bundle<'static, D>,
        profiles: Vec<SkinProfile<'static, Self>>,
    );
}

macro_rules! embedded {
    ($($v:ty),*) => { $(impl ExportLayout for $v {
        fn install_profiles<D: Dialect<Layout = Self>>(bundle: &mut Bundle<'static,D>, profiles: Vec<SkinProfile<'static,Self>>) {
            bundle.model.core_mut().skins = profiles;
        }
    })* };
}

embedded!(V256, V257, V260, V263);

macro_rules! external {
    ($($v:ty),*) => { $(impl ExportLayout for $v {
        fn install_profiles<D: Dialect<Layout = Self>>(bundle: &mut Bundle<'static,D>, profiles: Vec<SkinProfile<'static,Self>>) {
            bundle.model.core_mut().skins = profiles.len() as u32;
            for (index, profile) in profiles.into_iter().enumerate() {
                bundle.insert(bundle.key.sibling(format!("{index:02}.skin")), File::from(Skin { profile }));
            }
        }
    })* };
}

external!(V264, V265, V272, V272PreSplit, V273, V274);

/// Target envelope's ability to carry numeric texture identities.
pub trait ExportDialect: Dialect {
    /// Installs inline physics where supported, returning a document requiring a sibling otherwise.
    fn inline_physics(
        model: &mut native::Document<'static, Self>,
        file: native::companions::PhysicsFile<'static>,
    ) -> Result<Option<native::companions::PhysicsFile<'static>>, Error>;
    /// Whether the envelope carries numeric companion identities.
    const NUMERIC_COMPANIONS: bool;
    /// Installs an external physics identity using the selected envelope.
    fn physics_id(
        model: &mut native::Document<'static, Self>,
        id: Option<u32>,
        context: &mut Context,
    ) -> Result<(), Error>;
    /// Installs customization document identities where chunk framing permits them.
    fn bone_ids(
        model: &mut native::Document<'static, Self>,
        ids: Vec<u32>,
        context: &mut Context,
    ) -> Result<(), Error>;
    /// Installs numeric texture IDs, or diagnoses an envelope without extension chunks.
    fn texture_ids(model: &mut native::Document<'static, Self>, ids: Vec<u32>)
        -> Result<(), Error>;
    /// Installs material-prefixed particle texture-program selectors where chunk framing permits them.
    fn texture_combiner_hints(
        model: &mut native::Document<'static, Self>,
        values: Vec<[u8; 2]>,
        context: &mut Context,
    ) -> Result<(), Error>;
}

impl<V: ModelLayout> ExportDialect for native::Flat<V> {
    fn texture_combiner_hints(
        _: &mut native::Document<'static, Self>,
        values: Vec<[u8; 2]>,
        context: &mut Context,
    ) -> Result<(), Error> {
        if !values.is_empty() {
            context.unsupported(
                "m2.particle.texture_program",
                "flat MD20 cannot carry TXAC particle texture programmes",
            )?;
        }
        Ok(())
    }
    fn inline_physics(
        _: &mut native::Document<'static, Self>,
        file: native::companions::PhysicsFile<'static>,
    ) -> Result<Option<native::companions::PhysicsFile<'static>>, Error> {
        Ok(Some(file))
    }
    const NUMERIC_COMPANIONS: bool = false;
    fn physics_id(
        model: &mut native::Document<'static, Self>,
        id: Option<u32>,
        context: &mut Context,
    ) -> Result<(), Error> {
        if id.is_some() {
            context.unsupported(
                "physics.id",
                "flat MD20 requires a sibling filename instead of PFID",
            )?;
        }
        model.core_mut().flags.set_external_physics(true);
        Ok(())
    }
    fn bone_ids(
        _: &mut native::Document<'static, Self>,
        ids: Vec<u32>,
        context: &mut Context,
    ) -> Result<(), Error> {
        if !ids.is_empty() {
            context.unsupported(
                "bonefiles.ids",
                "flat MD20 cannot encode BFID customization identities",
            )?;
        }
        Ok(())
    }

    fn texture_ids(_: &mut native::Document<'static, Self>, ids: Vec<u32>) -> Result<(), Error> {
        if ids.iter().any(|id| *id != 0) {
            Err(Error::new(
                "textures.id",
                "flat MD20 cannot carry numeric texture IDs; provide filenames",
            ))
        } else {
            Ok(())
        }
    }
}

impl<V: ModelLayout> ExportDialect for Chunked<V> {
    fn texture_combiner_hints(
        model: &mut native::Document<'static, Self>,
        values: Vec<[u8; 2]>,
        _: &mut Context,
    ) -> Result<(), Error> {
        if !values.is_empty() {
            model
                .insert_extension(
                    0,
                    Chunk::TextureCombinerHints(ZeroVec::alloc_from_slice(&values)),
                )
                .map_err(failure)?;
        }
        Ok(())
    }
    fn inline_physics(
        model: &mut native::Document<'static, Self>,
        file: native::companions::PhysicsFile<'static>,
    ) -> Result<Option<native::companions::PhysicsFile<'static>>, Error> {
        model
            .insert_extension(0, Chunk::Physics(file))
            .map_err(failure)?;
        Ok(None)
    }
    const NUMERIC_COMPANIONS: bool = true;
    fn physics_id(
        model: &mut native::Document<'static, Self>,
        id: Option<u32>,
        _: &mut Context,
    ) -> Result<(), Error> {
        if let Some(id) = id {
            model
                .insert_extension(0, Chunk::PhysicsId(id))
                .map_err(failure)?;
        } else {
            model.core_mut().flags.set_external_physics(true);
        }
        Ok(())
    }
    fn bone_ids(
        model: &mut native::Document<'static, Self>,
        ids: Vec<u32>,
        _: &mut Context,
    ) -> Result<(), Error> {
        if !ids.is_empty() {
            model
                .insert_extension(0, Chunk::BoneIds(ZeroVec::alloc_from_slice(&ids)))
                .map_err(failure)?;
        }
        Ok(())
    }

    fn texture_ids(
        model: &mut native::Document<'static, Self>,
        ids: Vec<u32>,
    ) -> Result<(), Error> {
        if ids.iter().any(|id| *id != 0) {
            model
                .insert_extension(0, Chunk::TextureIds(ZeroVec::alloc_from_slice(&ids)))
                .map_err(failure)?;
        }
        Ok(())
    }
}
