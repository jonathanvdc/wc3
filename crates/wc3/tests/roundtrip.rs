use wc3::model::chunks::{ModelChunk, RawChunk, UnknownChunk};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::{
    DynamicModel, Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn check_version<V: ModelVersion>() {
    let mut model = Model::<V>::new();
    model.push(ModelChunk::from_raw(RawChunk::new(*b"MODL", vec![0; 372])).unwrap());
    model.push(ModelChunk::Unknown(
        UnknownChunk::<V>::new(RawChunk::new(*b"FUTR", vec![0, 1, 2, 255])).unwrap(),
    ));
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.version(), V::NUMBER);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
}

#[test]
fn synthetic_versions_and_unknown_chunks_round_trip() {
    check_version::<V800>();
    check_version::<V900>();
    check_version::<V1000>();
    check_version::<V1100>();
    check_version::<V1200>();
    check_version::<V1300>();
    check_version::<V1400>();
    check_version::<V1600>();
    check_version::<V1800>();
}

#[test]
fn preserves_repeated_chunks_and_order() {
    let mut model = Model::<V800>::new();
    for data in [vec![1], vec![2]] {
        model.push(ModelChunk::Unknown(
            UnknownChunk::<V800>::new(RawChunk::new(*b"ABCD", data)).unwrap(),
        ));
    }
    let parsed = Model::<V800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    assert!(matches!(&parsed.chunks()[1], ModelChunk::Unknown(raw) if raw.raw().data == [1]));
    assert!(matches!(&parsed.chunks()[2], ModelChunk::Unknown(raw) if raw.raw().data == [2]));
}

#[test]
fn local_files_round_trip_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mdx") {
                let bytes = std::fs::read(&path).unwrap();
                let model = DynamicModel::decode_mdx(&bytes, 800)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                assert_eq!(model.encode_mdx().unwrap(), bytes, "{}", path.display());
            }
        }
    }
}

fn check_accessors<V: ModelVersion>(mut model: Model<V>, bytes: &[u8], path: &std::path::Path) {
    if let Some(info) = model.model_info() {
        model.set_model_info(&info);
    }
    macro_rules! round_trip_records {
        ($tag:literal, $getter:ident, $setter:ident) => {
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *$tag)
                .count()
                == 1
            {
                let records = model.$getter();
                model.$setter(&records);
            }
        };
    }
    macro_rules! round_trip_checked_records {
        ($tag:literal, $getter:ident, $setter:ident) => {
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *$tag)
                .count()
                == 1
            {
                let records = model.$getter().unwrap();
                model.$setter(&records).unwrap();
            }
        };
    }
    round_trip_records!(b"SEQS", sequences, set_sequences);
    round_trip_records!(b"TEXS", textures, set_textures);
    round_trip_records!(b"MTLS", materials, set_materials);
    round_trip_records!(b"GEOS", geosets, set_geosets);
    round_trip_records!(b"BONE", bones, set_bones);
    round_trip_records!(b"PIVT", pivot_points, set_pivot_points);
    round_trip_records!(b"EVTS", event_objects, set_event_objects);
    round_trip_records!(b"CLID", collision_shapes, set_collision_shapes);
    round_trip_records!(b"ATCH", attachments, set_attachments);
    round_trip_checked_records!(b"FAFX", try_face_fx, try_set_face_fx);
    round_trip_records!(b"GEOA", geoset_animations, set_geoset_animations);
    round_trip_records!(b"LITE", lights, set_lights);
    round_trip_records!(b"TXAN", texture_animations, set_texture_animations);
    round_trip_records!(b"CAMS", cameras, set_cameras);
    round_trip_records!(b"PREM", particle_emitters, set_particle_emitters);
    round_trip_records!(b"PRE2", particle_emitters2, set_particle_emitters2);
    round_trip_records!(b"RIBB", ribbon_emitters, set_ribbon_emitters);
    round_trip_checked_records!(b"CORN", try_popcorn_emitters, try_set_popcorn_emitters);
    round_trip_checked_records!(b"BPOS", try_bind_poses, try_set_bind_poses);
    assert_eq!(model.encode_mdx().unwrap(), bytes, "{}", path.display());
}

#[test]
fn typed_accessors_preserve_local_files_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if !path.extension().is_some_and(|extension| extension == "mdx") {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            match DynamicModel::decode_mdx(&bytes, 800).unwrap() {
                DynamicModel::V800(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V900(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V1000(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V1100(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V1200(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V1300(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V1400(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V1600(model) => check_accessors(model, &bytes, &path),
                DynamicModel::V1800(model) => check_accessors(model, &bytes, &path),
            }
        }
    }
}
