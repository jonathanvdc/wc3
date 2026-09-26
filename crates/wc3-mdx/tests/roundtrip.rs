use wc3_mdx::chunks::{ModelChunk, RawChunk};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::Model;

#[test]
fn synthetic_versions_and_unknown_chunks_round_trip() {
    for version in [800, 900, 1000, 1100, 1200, 1800] {
        let mut model = Model::new(version);
        model.push(ModelChunk::from_raw(
            RawChunk::new(*b"MODL", vec![0; 372]),
            version,
        ));
        model.push(ModelChunk::from_raw(
            RawChunk::new(*b"FUTR", vec![0, 1, 2, 255]),
            version,
        ));
        let bytes = model.encode().unwrap();
        let parsed = Model::decode(&bytes, 800).unwrap();
        assert_eq!(parsed.version(), version);
        assert_eq!(parsed.encode().unwrap(), bytes);
    }
}

#[test]
fn preserves_repeated_chunks_and_order() {
    let mut model = Model::new(800);
    model.push(ModelChunk::from_raw(RawChunk::new(*b"ABCD", vec![1]), 800));
    model.push(ModelChunk::from_raw(RawChunk::new(*b"ABCD", vec![2]), 800));
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    assert!(matches!(&parsed.chunks()[1], ModelChunk::Unknown(raw) if raw.data == [1]));
    assert!(matches!(&parsed.chunks()[2], ModelChunk::Unknown(raw) if raw.data == [2]));
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
                let model = Model::decode(&bytes, 800)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                assert_eq!(model.encode().unwrap(), bytes, "{}", path.display());
            }
        }
    }
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
            let mut model = Model::decode(&bytes, 800).unwrap();
            if let Some(info) = model.model_info() {
                model.set_model_info(&info);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"SEQS")
                .count()
                == 1
            {
                let records = model.sequences();
                model.set_sequences(&records);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"TEXS")
                .count()
                == 1
            {
                let records = model.textures();
                model.set_textures(&records);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"MTLS")
                .count()
                == 1
            {
                let records = model.materials();
                model.set_materials(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"GEOS")
                .count()
                == 1
            {
                let records = model.geosets();
                model.set_geosets(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"BONE")
                .count()
                == 1
            {
                let records = model.bones();
                model.set_bones(&records);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"PIVT")
                .count()
                == 1
            {
                let points = model.pivot_points();
                model.set_pivot_points(&points);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"EVTS")
                .count()
                == 1
            {
                let records = model.event_objects();
                model.set_event_objects(&records);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"CLID")
                .count()
                == 1
            {
                let records = model.collision_shapes();
                model.set_collision_shapes(&records);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"ATCH")
                .count()
                == 1
            {
                let records = model.attachments();
                model.set_attachments(&records);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"FAFX")
                .count()
                == 1
            {
                let records = model.face_fx();
                model.set_face_fx(&records);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"GEOA")
                .count()
                == 1
            {
                let records = model.geoset_animations();
                model.set_geoset_animations(&records);
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
            round_trip_records!(b"LITE", lights, set_lights);
            round_trip_records!(b"TXAN", texture_animations, set_texture_animations);
            round_trip_records!(b"CAMS", cameras, set_cameras);
            round_trip_records!(b"PREM", particle_emitters, set_particle_emitters);
            round_trip_records!(b"PRE2", particle_emitters2, set_particle_emitters2);
            round_trip_records!(b"RIBB", ribbon_emitters, set_ribbon_emitters);
            round_trip_records!(b"CORN", popcorn_emitters, set_popcorn_emitters);
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag() == *b"BPOS")
                .count()
                == 1
            {
                let pose = model.bind_poses().remove(0);
                model.set_bind_pose(&pose);
            }
            assert_eq!(model.encode().unwrap(), bytes, "{}", path.display());
        }
    }
}
