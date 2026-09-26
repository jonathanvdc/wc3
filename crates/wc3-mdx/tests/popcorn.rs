use wc3_mdx::{Decodable, Encodable};
use wc3_mdx::{Model, Node, PopcornEmitter};

#[test]
fn popcorn_fixed_fields_round_trip() {
    let mut emitter =
        PopcornEmitter::new(Node::new("Spark", 3).unwrap(), "spark.mdx", "Stand").unwrap();
    emitter.set_life_span(1.0);
    emitter.set_emission_rate(20.0);
    emitter.set_speed(3.0);
    emitter.set_color([1.0, 0.5, 0.25]);
    emitter.set_alpha(0.75);
    emitter.set_replaceable_id(1);
    let mut model = Model::new(1800);
    model.set_popcorn_emitters(&[emitter]);
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let emitter = &parsed.popcorn_emitters()[0];
    assert_eq!(emitter.node().name(), "Spark");
    assert_eq!(emitter.path(), "spark.mdx");
    assert_eq!(emitter.visibility_guide(), "Stand");
    assert_eq!(emitter.life_span(), 1.0);
    assert_eq!(emitter.emission_rate(), 20.0);
    assert_eq!(emitter.speed(), 3.0);
    assert_eq!(emitter.color(), [1.0, 0.5, 0.25]);
    assert_eq!(emitter.alpha(), 0.75);
    assert_eq!(emitter.replaceable_id(), 1);
}

#[test]
fn local_popcorn_emitters_round_trip_when_available() {
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
                let mut model = Model::decode(&bytes, 800).unwrap();
                if model.chunk(*b"CORN").is_some() {
                    let emitters = model.popcorn_emitters();
                    for emitter in &emitters {
                        emitter.tracks();
                    }
                    model.set_popcorn_emitters(&emitters);
                    assert_eq!(model.encode().unwrap(), bytes);
                }
            }
        }
    }
}
