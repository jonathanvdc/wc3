use wc3::model::mdx::Write as _;
use wc3::model::NoExtensions;

use wc3::model::{DynamicModel, Model, ModelDialect};

use std::path::{Path, PathBuf};
use std::{env, fs};

#[test]
#[ignore = "requires WC3_FIXTURES"]
fn local_files_round_trip() {
    for path in fixture_paths() {
        let bytes = fs::read(&path).unwrap();
        let model = DynamicModel::<NoExtensions>::decode_mdx(&bytes, 800)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(model.encode_mdx().unwrap(), bytes, "{}", path.display());
    }
}

fn check_accessors<V: ModelDialect>(mut model: Model<V>, bytes: &[u8], path: &Path) {
    if let Some(info) = model.model_info() {
        model.set_model_info(&info);
    }
    macro_rules! round_trip_records {
        ($tag:literal, $getter:ident, $setter:ident) => {
            if model
                .chunks
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
                .chunks
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
    round_trip_records!(b"HELP", helpers, set_helpers);
    round_trip_records!(b"GLBS", global_sequences, set_global_sequences);
    round_trip_records!(b"DILG", gliders, set_gliders);
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
    for geoset in model.geosets() {
        geoset.vertices();
        geoset.normals();
        geoset.face_indices();
        geoset.vertex_groups();
        geoset.matrix_group_sizes();
        geoset.matrix_indices();
        geoset.sequence_extents.as_slice();
        let _ = geoset.try_tangents();
        let _ = geoset.try_skin_weights();
        geoset.uv_sets();
    }
    assert_eq!(model.encode_mdx().unwrap(), bytes, "{}", path.display());
}

#[test]
#[ignore = "requires WC3_FIXTURES"]
fn typed_accessors_preserve_local_files() {
    for path in fixture_paths() {
        let bytes = fs::read(&path).unwrap();
        let model = DynamicModel::<NoExtensions>::decode_mdx(&bytes, 800)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        wc3::visit_model!(model, |model| check_accessors(model, &bytes, &path));
    }
}

fn fixture_paths() -> Vec<PathBuf> {
    let directory = env::var("WC3_FIXTURES").expect("set WC3_FIXTURES to a model directory");
    let mut pending = vec![PathBuf::from(directory)];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mdx") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    assert!(!paths.is_empty(), "WC3_FIXTURES contains no MDX files");
    paths
}
