//! Deterministic model workloads, independent of downloaded assets.
use mdl::Read as _;
use std::collections::HashMap;
use std::env;
use std::fmt::Write as _;
use wc3::model::formats::mdlx::geometry::SkinWeights;
use wc3::model::formats::mdlx::materials::{LayerTextureSlot, ShaderType};
use wc3::model::formats::mdlx::ZeroVec;
use wc3::model::formats::mdlx::{mdl, ConversionOptions, Document, V1800, V800};

#[derive(Clone, Copy)]
pub struct Profile {
    pub name: &'static str,
    pub geosets: usize,
    pub vertices: usize,
    pub bones: usize,
    pub sequences: usize,
    pub keys: usize,
    pub layers: usize,
    pub depth: usize,
    pub spline: bool,
}

pub fn profiles() -> Vec<Profile> {
    let mut profiles = vec![
        Profile {
            name: "classic",
            geosets: 3,
            vertices: 192,
            bones: 24,
            sequences: 13,
            keys: 96,
            layers: 1,
            depth: 8,
            spline: true,
        },
        Profile {
            name: "unit",
            geosets: 8,
            vertices: 936,
            bones: 114,
            sequences: 27,
            keys: 768,
            layers: 2,
            depth: 17,
            spline: false,
        },
    ];
    if env::var_os("WC3_BENCH_QUICK").is_some() {
        return vec![profiles[1]];
    }
    if env::var_os("WC3_BENCH_LARGE").is_some() {
        profiles.push(Profile {
            name: "building",
            geosets: 27,
            vertices: 2370,
            bones: 35,
            sequences: 6,
            keys: 512,
            layers: 3,
            depth: 4,
            spline: false,
        });
    }
    profiles
}

pub fn classic<'data>(profile: Profile) -> Document<'data, V800> {
    let text = source(profile);
    let model =
        Document::decode_mdl(&text).unwrap_or_else(|error| panic!("{}", error.diagnostic(&text)));
    validate(&model, profile);
    model
}

fn validate<'data>(model: &Document<'data, V800>, profile: Profile) {
    let nodes = model.nodes();
    let parents: HashMap<_, _> = nodes
        .iter()
        .map(|node| (node.object_id, node.parent_id))
        .collect();
    assert_eq!(parents.len(), nodes.len(), "node IDs must be unique");
    let pivots = model.pivot_points();
    for node in nodes {
        assert!((node.object_id as usize) < pivots.len());
        let mut parent = node.parent_id;
        let mut depth = 0;
        while parent != u32::MAX {
            parent = *parents.get(&parent).expect("parent must exist");
            depth += 1;
            assert!(depth < parents.len(), "hierarchy must be acyclic");
        }
    }
    assert_eq!(model.sequences().count(), profile.sequences);
    let material_count = model.materials().count();
    for geoset in model.geosets() {
        assert!((geoset.material_id as usize) < material_count);
        assert_eq!(geoset.vertices().len(), geoset.normals().len());
        for start in (0..geoset.face_indices().len()).step_by(3) {
            let points = [start, start + 1, start + 2].map(|index| {
                geoset
                    .vertices()
                    .get(geoset.face_indices().get(index).unwrap() as usize)
                    .unwrap()
            });
            let a = [points[1][0] - points[0][0], points[1][1] - points[0][1]];
            let b = [points[2][0] - points[0][0], points[2][1] - points[0][1]];
            assert!(
                (a[0] * b[1] - a[1] * b[0]).abs() > 0.0,
                "triangles must be nondegenerate"
            );
        }
        assert!(geoset
            .matrix_indices()
            .iter()
            .all(|id| parents.contains_key(&id)));
        assert!(geoset
            .uv_sets()
            .iter()
            .all(|set| set.len() == geoset.vertices().len()));
    }
}

pub fn modern<'data>(profile: Profile) -> Document<'data, V1800> {
    let mut model = classic(profile)
        .into_convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    for (g, geoset) in model.geosets_mut().enumerate() {
        // Authored LODs, different UV arrays, and nonuniform four-joint weights.
        geoset.set_level_of_detail((g % 3) as u32);
        let uv = geoset.uv_sets()[0].clone();
        let secondary: ZeroVec<_> = uv.iter().map(|p| [p[1], 1.0 - p[0]]).collect();
        geoset.set_uv_sets(vec![uv, secondary]);
        let skin: ZeroVec<_> = (0..geoset.vertices().len())
            .map(|i| {
                let bone = (i / 12 + g) % profile.bones;
                SkinWeights {
                    bone_indices: [
                        bone as u16,
                        ((bone + 1) % profile.bones) as u16,
                        ((bone + 2) % profile.bones) as u16,
                        ((bone + 3) % profile.bones) as u16,
                    ],
                    weights: [160, 64, 24, 7],
                }
            })
            .collect();
        geoset.set_skin_weights(Some(skin)).unwrap();
    }
    for material in model.materials_mut() {
        for (i, layer) in material.layers.iter_mut().enumerate() {
            layer.coordinate_id = (i % 2) as u32;
            if i == 0 && profile.layers > 1 {
                layer.set_shader_type(ShaderType::HD_DEFAULT_UNIT);
                layer.set_texture_slots(
                    &(0..6)
                        .map(|role| LayerTextureSlot {
                            texture_type: role,
                            texture_id: (role % 2).into(),
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }
    }
    model
}

pub fn source(p: Profile) -> String {
    let mut s = String::from("Version { FormatVersion 800, }\nModel \"Synthetic workload\" { MinimumExtent { -32, -32, -32 }, MaximumExtent { 32, 32, 32 }, BoundsRadius 56, }\n");
    writeln!(s, "Sequences {} {{", p.sequences).unwrap();
    for i in 0..p.sequences {
        writeln!(
            s,
            "Anim \"Sequence {i}\" {{ Interval {{ {}, {} }}, }}",
            i * 2000,
            i * 2000 + 1800
        )
        .unwrap();
    }
    s.push_str("}\nGlobalSequences 1 { Duration 1800, }\nTextures 2 { Bitmap { Image \"synthetic/diffuse.blp\", } Bitmap { ReplaceableId 1, } }\nMaterials 1 { Material {\n");
    for l in 0..p.layers {
        writeln!(s, "Layer {{ FilterMode {}, static TextureID {}, Alpha 3 {{ Linear, 0: 1, 900: 0.5, 1800: 1, }} }}", if l == 0 { "None" } else { "Blend" }, l % 2).unwrap();
    }
    s.push_str("} }\n");
    for g in 0..p.geosets {
        writeln!(s, "Geoset {{ Vertices {} {{", p.vertices).unwrap();
        for i in 0..p.vertices {
            // Separate nondegenerate triangles with spatially varying coordinates.
            let x = (i / 3 % 24) as f32 - 12.0;
            let y = (i / 72) as f32 - 12.0;
            let dx = if i % 3 == 1 { 0.8 } else { 0.0 };
            let dy = if i % 3 == 2 { 0.8 } else { 0.0 };
            writeln!(s, "{{ {}, {}, {} }},", x + dx, y + dy, (g % 8) as f32).unwrap();
        }
        writeln!(s, "}} Normals {} {{", p.vertices).unwrap();
        for _ in 0..p.vertices {
            s.push_str("{ 0, 0, 1 },\n");
        }
        writeln!(s, "}} TVertices {} {{", p.vertices).unwrap();
        for i in 0..p.vertices {
            writeln!(s, "{{ {}, {} }},", (i % 3 == 1) as u8, (i % 3 == 2) as u8).unwrap();
        }
        s.push_str("} VertexGroup {\n");
        for i in 0..p.vertices {
            writeln!(s, "{},", (i / 12 + g) % p.bones).unwrap();
        }
        writeln!(s, "}} Faces 1 {} {{ Triangles {{ {{", p.vertices).unwrap();
        for i in 0..p.vertices {
            write!(s, "{i}, ").unwrap();
        }
        writeln!(s, "}}, }} }} Groups {} {} {{", p.bones, p.bones * 4).unwrap();
        for b in 0..p.bones {
            writeln!(
                s,
                "Matrices {{ {b}, {}, {}, {} }},",
                (b + 1) % p.bones,
                (b + 2) % p.bones,
                (b + 3) % p.bones
            )
            .unwrap();
        }
        s.push_str("} MaterialID 0, SelectionGroup 0, MinimumExtent { -32, -32, -32 }, MaximumExtent { 32, 32, 32 }, BoundsRadius 56, }\n");
    }
    for g in 0..p.geosets {
        writeln!(s, "GeosetAnim {{ GeosetId {g}, Alpha 3 {{ Linear, 0: 1, 900: 0.7, 1800: 1, }} static Color {{ 1, 0.8, 0.6 }}, }}").unwrap();
    }
    for b in 0..p.bones {
        writeln!(s, "Bone \"Joint {b}\" {{ ObjectId {b},").unwrap();
        if b > 0 {
            writeln!(
                s,
                "Parent {},",
                if b < p.depth {
                    b - 1
                } else {
                    (b - p.depth) % (p.depth - 1)
                }
            )
            .unwrap();
        }
        s.push_str("GeosetId Multiple, GeosetAnimId None,\n");
        // Uneven track sizes and timing model joints with different activity.
        let count = p.keys / (1 + b % 4);
        writeln!(
            s,
            "Rotation {count} {{ {},",
            if p.spline { "Hermite" } else { "Linear" }
        )
        .unwrap();
        for k in 0..count {
            let frame = k * (p.sequences * 2000 - 200) / (count - 1)
                + if k + 1 == count { 0 } else { k % 7 };
            let angle = ((k * 7 + b) % 31) as f32 * 0.015;
            writeln!(s, "{frame}: {{ 0, 0, {}, {} }},", angle.sin(), angle.cos()).unwrap();
            if p.spline {
                writeln!(
                    s,
                    "InTan {{ 0, 0, {}, {} }}, OutTan {{ 0, 0, {}, {} }},",
                    angle.sin(),
                    angle.cos(),
                    angle.sin(),
                    angle.cos()
                )
                .unwrap();
            }
        }
        s.push_str("} }\n");
    }
    writeln!(s, "PivotPoints {} {{", p.bones + 1).unwrap();
    for b in 0..p.bones {
        writeln!(s, "{{ {}, {}, {} }},", b % 3, (b / 3) % 3, b / 9).unwrap();
    }
    s.push_str("{ 0, 0, 0 },\n}\nEventObject \"Synthetic event\" { ObjectId ");
    writeln!(
        s,
        "{}, Parent 0, EventTrack 4 {{ 0, 450, 900, 1800, }} }}",
        p.bones
    )
    .unwrap();

    s
}
