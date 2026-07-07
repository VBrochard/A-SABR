use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use a_sabr::{
    bundle::Bundle,
    contact_plan::asabr_file_lexer::parse_from_iter,
    errors::ASABRError,
    multigraph::{Multigraph, NodeRef},
    node_manager::none::NoManagement,
    parsing::CMDynStandard,
    pathfinding::{HybridParenting, Pathfinding},
    route_storage::{Cached, cache::TreeCache},
    routing::aliases::SpsnHybridParenting,
};
use generativity::make_guard;

fn main() -> Result<(), ASABRError> {
    let cp_path = "asabr/examples/inter-regional_routing/asabr_format_dynamic.cp";
    // All nodes will have the same management approach (NoManagement) but the contacts may be of various types.
    // The manager type is Box<dyn ContactManager> through CMDynStandard, selected by contact-plan markers.
    let file = File::open(cp_path).unwrap();
    let lines = BufReader::new(file).lines().map(|l| l.unwrap());

    let contact_plan = parse_from_iter::<NoManagement, CMDynStandard>(lines).unwrap();
    println!(
        "A-SABR CP parsed (statically for nodes, dynamically for contacts), found {} nodes (no management) & {} contacts (of various types)",
        contact_plan.vnodes.len() + contact_plan.realnodes.len(),
        contact_plan.contacts.len()
    );

    make_guard!(id);
    let mut graph = Multigraph::new(id, contact_plan).unwrap();

    println!("Virtual nodes:");
    println!("{graph}");

    println!("\n---\n");

    // We create a storage for the paths and initialize SPSN with the current pathfinding API.
    let table = TreeCache::new(&graph);
    let mut spsn = SpsnHybridParenting::<1, NoManagement, CMDynStandard, _>::new(Cached::new(
        table,
        HybridParenting::new(),
    ));

    // We will route a bundle to the virtual gateway node 8.
    let bundle = Bundle {
        source: 0.into(),
        priority: 0,
        size: 1,
        expiration: 10000,
    };

    let Ok(NodeRef::R(source)) = graph.node_id_ref(0.into()) else {
        panic!("Expected RNodeRef for source node 0")
    };
    let mut destination = graph.node_id_ref(8.into())?;

    // We schedule the bundle (resource updates were conducted).
    let out = spsn.find_path(&mut graph, 0, source, &bundle, &mut destination, None)?;

    if let Some(out) = out {
        println!("{:?}", out.get_full_path(destination, &graph));
    }

    Ok(())
}
