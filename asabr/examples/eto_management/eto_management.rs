assert_cfg!(feature = "manual_queueing");

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use a_sabr::bundle::Bundle;
use a_sabr::contact_manager::ContactManager;
use a_sabr::contact_plan::asabr_file_lexer::parse_from_iter;
use a_sabr::multigraph::{Multigraph, NodeRef};
use a_sabr::node_manager::none::NoManagement;
use a_sabr::parsing::CMDynStandard;
use a_sabr::pathfinding::Pathfinding;
use a_sabr::pathfinding::dijkstra_impl::HybridParenting;
use a_sabr::route_storage::{Cached, cache::TreeCache};
use a_sabr::routing::aliases::SpsnHybridParenting;
use generativity::make_guard;
use static_assertions::assert_cfg;

fn main() {
    // We want variations for contact management, register ETO and EVL

    // We create a lexer to retrieve tokens from a file
    let file = File::open("asabr/examples/eto_management/contact_plan_1.cp").unwrap();
    let lines = BufReader::new(file).lines().map(|l| l.unwrap());

    // We parse the contact plan (A-SABR format thanks to ASABRContactPlan) and the lexer
    let contact_plan = parse_from_iter::<NoManagement, CMDynStandard>(lines).unwrap();

    // Securely initialize the Multigraph lifecycle
    make_guard!(id);

    let mut multigraph = Multigraph::new(id, contact_plan).unwrap();

    let mut router = Box::new(
        SpsnHybridParenting::<1, NoManagement, CMDynStandard, _>::new(Cached::new(
            TreeCache::new(&multigraph, 10),
            HybridParenting::new(),
        )),
    );

    // Retrieve typed references for nodes 0 and 3
    let src_0 = match multigraph.node_id_ref(0.into()).unwrap() {
        NodeRef::I(r) => r,
        _ => panic!("Expected INodeRef for source 0"),
    };
    let mut dest_3 = multigraph
        .node_id_ref(3.into())
        .unwrap()
        .routable()
        .unwrap();

    // Scenario 1: Route the first bundle to node 3
    let bundle_1 = Bundle {
        source: 0.into(),
        priority: 0,
        size: 20,
        expiration: 10000,
    };

    // let's route with current time == 15
    let out = router
        .find_path(&mut multigraph, 15, src_0, &bundle_1, &mut dest_3, None)
        .unwrap()
        .unwrap();

    let iter_1 = out.full_path_rev(dest_3, &multigraph).unwrap();
    println!("{}", iter_1);

    let route_1 = out.get_full_path(dest_3, &multigraph).unwrap();

    // Explicitly enqueue the bundle to simulate transmission delay
    let first_hop_contact = route_1[1].via.as_ref().unwrap().contact;
    println!(
        "Enqueueing bundle_1 status : {}",
        multigraph[first_hop_contact]
            .manager
            .manual_enqueue(&bundle_1)
    );

    for frag in route_1.iter().skip(2) {
        if let Some(via) = &frag.via {
            multigraph[via.contact].manager.manual_enqueue(&bundle_1);
        }
    }

    // Scenario 2: Route a second bundle to node 3

    let mut router = Box::new(
        SpsnHybridParenting::<1, NoManagement, CMDynStandard, _>::new(Cached::new(
            TreeCache::new(&multigraph, 10),
            HybridParenting::new(),
        )),
    );

    let bundle_2 = Bundle {
        source: 0.into(),
        priority: 0,
        size: 20,
        expiration: 10000,
    };

    // let's route with current time == 15, and ensure that the queueing is taken into account
    let out_2 = router
        .find_path(&mut multigraph, 15, src_0, &bundle_2, &mut dest_3, None)
        .unwrap()
        .unwrap();

    let iter_2 = out_2.full_path_rev(dest_3, &multigraph).unwrap();
    println!("{}", iter_2);

    let route_2 = out_2.get_full_path(dest_3, &multigraph).unwrap();

    let first_hop_contact_2 = route_2[1].via.as_ref().unwrap().contact;

    // Enqueue the bundle_2
    println!(
        "Enqueueing bundle_2 status : {}",
        multigraph[first_hop_contact_2]
            .manager
            .manual_enqueue(&bundle_2)
    );

    for frag in route_2.iter().skip(2) {
        if let Some(via) = &frag.via {
            multigraph[via.contact].manager.manual_enqueue(&bundle_2);
        }
    }

    println!();
    println!(
        "Contact 0 has now 2 bundles in the queue (size: 2 x 20), unless we unqueue manually, the delay will be considered"
    );
    println!();

    // Scenario 3: Attempt to route a third bundle to node 4
    let mut router = Box::new(
        SpsnHybridParenting::<1, NoManagement, CMDynStandard, _>::new(Cached::new(
            TreeCache::new(&multigraph, 10),
            HybridParenting::new(),
        )),
    );

    let bundle_3 = Bundle {
        source: 0.into(),
        priority: 0,
        size: 20,
        expiration: 10000,
    };
    let mut dest_4 = multigraph
        .node_id_ref(4.into())
        .unwrap()
        .routable()
        .unwrap();

    // Should fail as the transmission queue is full
    let out_3 = router
        .find_path(&mut multigraph, 15, src_0, &bundle_3, &mut dest_4, None)
        .unwrap();
    println!(
        "Sending bundle 3 to node 4, the routing output should be None: {}",
        out_3.is_none()
    );

    println!();
    println!(
        "Simulate transmission success of bundle_1, Contact 0 should not be a blocker anymore"
    );

    // Free queue space and retry routing
    println!(
        "Dequeueing bundle_1, status : {}",
        multigraph[first_hop_contact]
            .manager
            .manual_dequeue(&bundle_1)
    );
    for frag in route_1.iter().skip(2) {
        if let Some(via) = &frag.via {
            multigraph[via.contact].manager.manual_dequeue(&bundle_1);
        }
    }

    println!("Retry for bundle 3");

    // Recreate the router to forcefully flush its internal TreeCache, ensuring the new queue state is considered
    let mut router = Box::new(
        SpsnHybridParenting::<1, NoManagement, CMDynStandard, _>::new(Cached::new(
            TreeCache::new(&multigraph, 10),
            HybridParenting::new(),
        )),
    );

    let out_4 = router
        .find_path(&mut multigraph, 15, src_0, &bundle_3, &mut dest_4, None)
        .unwrap()
        .unwrap();
    let iter_4 = out_4.full_path_rev(dest_4, &multigraph).unwrap();
    println!("{}", iter_4);

    // === OUTPUT ===
    // Running with contact plan location=asabr/examples/dijkstra_accuracy/contact_plan_1.cp, and destination node=3

    // Route to node 3 at t=220 with 3 hop(s):
    //         - Reach node 0 at t=15 with 0 hop(s)
    //         - Reach node 1 at t=35 with 1 hop(s)
    //         - Reach node 2 at t=120 with 2 hop(s)
    //         - Reach node 3 at t=220 with 3 hop(s)
    // Enqueueing bundle_1 status : true
    // Route to node 3 at t=240 with 3 hop(s):
    //         - Reach node 0 at t=15 with 0 hop(s)
    //         - Reach node 1 at t=55 with 1 hop(s)
    //         - Reach node 2 at t=120 with 2 hop(s)
    //         - Reach node 3 at t=240 with 3 hop(s)
    // Enqueueing bundle_2 status : true

    // Contact 0 has now 2 bundles in the queue (size: 2 x 20), unless we unqueue manually, the delay will be considered

    // Sending bundle 3 to node 4, the routing output should be None: true

    // Simulate transmission success of bundle_1, Contact 0 should not be a blocker anymore
    // Dequeueing bundle_1, status : true
    // Retry for bundle 3
    // Route to node 4 at t=75 with 2 hop(s):
    //         - Reach node 0 at t=15 with 0 hop(s)
    //         - Reach node 1 at t=55 with 1 hop(s)
    //         - Reach node 4 at t=75 with 2 hop(s)
}
