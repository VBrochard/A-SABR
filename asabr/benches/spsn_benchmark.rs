use std::fs::File;

use a_sabr::{
    bundle::Bundle, contact_manager::segmentation::seg::SegmentationManager,
    contact_plan::from_tvgutil_file::TVGUtilContactPlan, node_manager::none::NoManagement,
    routing::aliases::*,
};
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

pub fn benchmark(c: &mut Criterion) {
    let ptvg_filepath = "benches/ptvg_files/sample1.json";

    let source = 0.into();
    let destinatation = 79.into();
    let bundle = Bundle {
        source,
        priority: 0,
        size: 474195330,
        expiration: 24060,
    };
    let curr_time = 60;

    let mut router_types = vec![
        "SpsnHybridParenting",
        "SpsnNodeParenting",
        "SpsnHybridParentingHop",
        "SpsnNodeParentingHop",
    ];

    router_types.extend(["SpsnContactParenting", "SpsnContactParentingHop"]);

    #[cfg(feature = "contact_suppression")]
    router_types.extend([
        "CgrFirstEndingHybridParenting",
        "CgrFirstEndingNodeParentingHop",
    ]);

    #[cfg(feature = "first_depleted")]
    router_types.extend([
        "CgrFirstDepletedHybridParenting",
        "CgrFirstDepletedNodeParenting",
        "CgrFirstDepletedHybridParentingHop",
        "CgrFirstDepletedNodeParentingHop",
    ]);

    #[cfg(feature = "contact_suppression")]
    router_types.extend([
        "CgrFirstEndingContactParenting",
        "CgrFirstEndingContactParentingHop",
    ]);
    #[cfg(feature = "first_depleted")]
    router_types.extend([
        "CgrFirstDepletedContactParenting",
        "CgrFirstDepletedContactParentingHop",
    ]);

    router_types.extend([
        "VolCgrHybridParenting",
        "VolCgrNodeParenting",
        "VolCgrHybridParentingHop",
        "VolCgrNodeParentingHop",
    ]);

    router_types.extend(["VolCgrContactParenting", "VolCgrContactParentingHop"]);
    let file = File::open(ptvg_filepath).unwrap();
    let json = serde_json::from_reader(file).unwrap();
    let contact_plan =
        TVGUtilContactPlan::parse::<NoManagement, SegmentationManager>(json).unwrap();

    let mut group = c.benchmark_group("Routers");

    for router_type in router_types {
        group.bench_function(router_type, |b| {
            b.iter_batched(
                || match unsafe {
                    build_generic_router::<3, _, _>(router_type, contact_plan.clone())
                } {
                    Ok((graph, router)) => {
                        let source = graph.node_id_ref(source).unwrap().try_into().unwrap();
                        let dest = graph.node_id_ref(destinatation).unwrap();
                        (graph, router, source, dest)
                    }
                    Err(err) => panic!("{}", err),
                },
                |(mut graph, mut router, source, mut dest)| {
                    let _ = black_box(router.find_path(
                        black_box(&mut graph),
                        black_box(curr_time),
                        black_box(source),
                        black_box(&bundle),
                        black_box(&mut dest),
                        black_box(None),
                    ));
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group! {
    name=benches;
    config=Criterion::default().sample_size(50);
    targets=benchmark
}
criterion_main!(benches);
