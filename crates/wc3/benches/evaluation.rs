use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::Quat;
use std::hint::black_box;
use wc3::model::profiles::Universal;
use wc3::model::{
    AnimationProgram, AnimationSample, BillboardConvention, Channel, Clock, Curve, EvaluatedPose,
    GeometryData, Mesh, Model, ModelPose, Node, PoseBehavior, PoseInputs, Quaternion, Time, Track,
    Transform,
};

fn poses(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation/pose");
    for count in [64, 256] {
        let mut builder = Model::<Universal>::builder("pose benchmark");
        let mut parent = None;
        for index in 0..count {
            let mut node = Node::named(format!("node {index}"));
            node.parent = parent;
            node.transform = Transform {
                translation: [0.1, 0.2, 0.3],
                ..Default::default()
            };
            let id = builder.add_node(node).unwrap();
            builder
                .add_timeline_channel(Channel::Rotation(Track {
                    target: id.rotation(),
                    clock: Clock::Timeline,
                    curve: Curve::linear(
                        [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                        [Quaternion::IDENTITY, Quaternion([0., 0., 0.1, 0.995])],
                    )
                    .unwrap(),
                }))
                .unwrap();
            parent = Some(id);
        }
        let model = builder.finish().unwrap();
        let layout = model.prepare_pose().unwrap();
        let animation = AnimationProgram::compile(&model);
        let sample = animation
            .sample(AnimationSample {
                timeline_time: Time::seconds(0.5).unwrap(),
                ..Default::default()
            })
            .unwrap();
        let mut pose = ModelPose::default();
        layout
            .evaluate_into(&sample, PoseInputs::default(), &mut pose)
            .unwrap();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, _| {
            b.iter(|| {
                layout
                    .evaluate_into(black_box(&sample), PoseInputs::default(), &mut pose)
                    .unwrap();
                black_box(&pose);
            });
        });
    }
    group.finish();
}

fn projection(c: &mut Criterion) {
    let mesh: Mesh<'static> = Mesh::new(
        GeometryData::triangles(
            (0..1024)
                .flat_map(|i| {
                    [
                        [i as f32, 0., 0.],
                        [i as f32 + 1., 0., 0.],
                        [i as f32, 1., 0.],
                    ]
                })
                .collect::<Vec<_>>(),
            (0..1024)
                .map(|i| [i * 3, i * 3 + 1, i * 3 + 2])
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    );
    let subset = mesh.whole_geometry().unwrap();
    c.bench_function("evaluation/project_1024_triangles", |b| {
        b.iter(|| {
            black_box(mesh.project(black_box(&subset), 0, [None; 3]).unwrap());
        })
    });
}
fn node_math(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation/node");
    group.throughput(Throughput::Elements(256));
    for (name, convention) in [
        ("generic", BillboardConvention::Generic),
        ("warcraft3", BillboardConvention::Warcraft3),
    ] {
        let behavior = PoseBehavior {
            billboard_convention: convention,
            ..Default::default()
        };
        let local = Transform {
            translation: [0.2, -0.1, 0.3],
            pivot: [0.1, 0.2, 0.3],
            rotation: Quaternion(Quat::from_rotation_z(0.01).to_array()),
            ..Default::default()
        };
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut parent = EvaluatedPose::default();
                for index in 0..256 {
                    if index % 16 == 0 {
                        parent = EvaluatedPose::default();
                    }
                    parent =
                        black_box(&behavior).evaluate(black_box(local), black_box(parent), None);
                }
                black_box(parent);
            })
        });
    }
    group.finish();
}
criterion_group!(benches, poses, projection, node_math);
criterion_main!(benches);
