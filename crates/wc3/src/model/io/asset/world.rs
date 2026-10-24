//! Explicit modern WMO layout dispatch and required group loading.
use super::{CompanionKind, CompanionPlan, DecodeError, Error, Request};
use crate::model::common::FileKey;
use crate::model::conversion::{Conversion, ImportOptions};
use crate::model::formats::wmo::{self as native, Profile};
use crate::model::io::wmo as semantic;
use crate::model::Model;
#[cfg(test)]
use std::cell::Cell;
use std::collections::BTreeMap;

fn failure(key: &FileKey, source: native::ReadError) -> Error {
    Error::Decode {
        key: key.clone(),
        source: DecodeError::Wmo(source),
    }
}

fn decode_root<'a, D: native::Dialect>(
    bytes: &'a [u8],
    key: &FileKey,
) -> Result<native::Root<'a, D>, Error> {
    #[cfg(test)]
    ROOT_DECODES.with(|count| count.set(count.get() + 1));
    native::Root::<D>::decode(bytes).map_err(|source| failure(key, source))
}

fn prepare_typed<D: native::Dialect>(bytes: &[u8], key: &FileKey) -> Result<CompanionPlan, Error> {
    let root = decode_root::<D>(bytes, key)?;
    let requests = native::Bundle::from_root(root)
        .with_key(key.clone())
        .dependencies()
        .map_err(|source| failure(key, source))?
        .into_iter()
        .filter_map(|dependency| match dependency.kind {
            native::DependencyKind::Group { index, lod } => Some(Request {
                owner: key.clone(),
                key: dependency.key,
                kind: CompanionKind::WmoGroup { index, lod },
            }),
            _ => None,
        })
        .collect();
    Ok(CompanionPlan::new(requests))
}

fn decode_typed<'a, D: native::Dialect>(
    bytes: &'a [u8],
    key: &FileKey,
    buffers: &'a BTreeMap<FileKey, Vec<u8>>,
) -> Result<native::Bundle<'a, D>, Error> {
    let root = decode_root::<D>(bytes, key)?;
    let mut bundle = native::Bundle::from_root(root).with_key(key.clone());
    for dependency in bundle
        .dependencies()
        .map_err(|source| failure(key, source))?
    {
        let native::DependencyKind::Group { index, lod } = dependency.kind else {
            continue;
        };
        let bytes = &buffers[&dependency.key];
        let group =
            native::Group::<D>::decode(bytes).map_err(|source| failure(&dependency.key, source))?;
        bundle.insert_group(native::GroupKey { index, lod }, dependency.key, group);
    }
    bundle
        .validate_complete()
        .map_err(|source| failure(key, source))?;
    Ok(bundle)
}

macro_rules! bundles {
    ($($dialect:ident),* $(,)?) => {
        pub(super) fn prepare(
            bytes: &[u8],
            key: &FileKey,
            profile: Profile,
        ) -> Result<CompanionPlan, Error> {
            match profile {
                $(Profile::$dialect => prepare_typed::<native::$dialect>(bytes, key),)*
                Profile::Alpha14 | Profile::Legacy16 => Err(Error::UnsupportedProfile { profile }),
            }
        }
        pub(super) fn decode(
            prepared: &mut CompanionPlan,
            bytes: &[u8],
            key: &FileKey,
            profile: Profile,
            buffers: &BTreeMap<FileKey, Vec<u8>>,
            missing: &mut Option<Request>,
            options: &ImportOptions,
        ) -> Result<Option<Conversion<Model<'static>>>, Error> {
            if let Some(request) = prepared.next_missing(buffers) {
                *missing = Some(request);
                return Ok(None);
            }
            match profile {
                $(Profile::$dialect => {
                    let bundle = decode_typed::<native::$dialect>(bytes, key, buffers)?;
                    semantic::import(&bundle, options).map(|converted| Some(Conversion {
                        value: converted.value.widen().into_owned(),
                        report: converted.report,
                    })).map_err(|source| Error::Import { key: key.clone(), source })
                },)*
                Profile::Alpha14 | Profile::Legacy16 => Err(Error::UnsupportedProfile { profile }),
            }
        }
    };
}
bundles! { Classic, Legion, Bfa, ShadowlandsPre92, Shadowlands, Dragonflight, WarWithin }

#[cfg(test)]
thread_local! {
    static ROOT_DECODES: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::io::asset::{Format, Progress, Reader};
    use native::{codec::Array, records::GroupInfo, root::RootChunk, Classic, Group, Root};

    #[test]
    fn many_group_requests_decode_the_root_only_for_planning_and_completion() {
        const COUNT: usize = 128;
        let mut root = Root::<Classic>::new();
        root.header_mut().group_count = COUNT as u32;
        root.insert_chunk(
            0,
            RootChunk::Groups(Array::from_records(&vec![GroupInfo::default(); COUNT]).unwrap()),
        )
        .unwrap();
        root.insert_chunk(
            1,
            RootChunk::GroupFileIds(
                Array::from_records(&(1..=COUNT as u32).collect::<Vec<_>>()).unwrap(),
            ),
        )
        .unwrap();
        let bytes = root.encode().unwrap();
        let group = Group::<Classic>::new().encode().unwrap();
        ROOT_DECODES.with(|count| count.set(0));
        let mut reader = Reader::new(
            Format::Wmo(Profile::Classic),
            FileKey::Root,
            bytes,
            ImportOptions::default(),
        );
        for index in 0..COUNT {
            let Progress::Pending(request) = reader.poll().unwrap() else {
                panic!("group {index} must be supplied before completion");
            };
            assert_eq!(request.key, FileKey::FileId(index as u32 + 1));
            assert_eq!(request.kind, CompanionKind::WmoGroup { index, lod: 0 });
            ROOT_DECODES.with(|count| assert_eq!(count.get(), 1));
            assert_eq!(reader.world.as_ref().unwrap().examined, index * 2 + 1);
            let Progress::Pending(repeated) = reader.poll().unwrap() else {
                panic!("a pending group cannot finish without bytes");
            };
            assert_eq!(request, repeated);
            assert_eq!(reader.world.as_ref().unwrap().examined, index * 2 + 1);
            reader.provide(&request, group.clone()).unwrap();
        }
        let Progress::Ready(converted) = reader.poll().unwrap() else {
            panic!("all group files have been supplied");
        };
        ROOT_DECODES.with(|count| assert_eq!(count.get(), 2));
        assert!(reader.world.is_none());
        assert!(reader.buffers.is_empty());
        assert_eq!(reader.bytes.capacity(), 0);
        drop(reader);
        converted.value.validate().unwrap();
        assert_eq!(converted.value.meshes().len(), COUNT);
    }
}
