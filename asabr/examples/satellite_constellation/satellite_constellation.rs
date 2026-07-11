use a_sabr::bundle::Bundle;
use a_sabr::choices;
use a_sabr::contact_manager::legacy::evl::EVLManager;
use a_sabr::distance::sabr::SABR;
use a_sabr::errors::ASABRError;
use a_sabr::mk_graph;
use a_sabr::multigraph::{NodeRef, RoutableNodeRef};
use a_sabr::node_manager::NodeManager;
use a_sabr::node_manager::none::NoManagement;
use a_sabr::parse_transparent;
use a_sabr::parsing::LexFrom;
use a_sabr::pathfinding::{Dest, HybridParenting, Pathfinding};
use a_sabr::transparent_NM;
use a_sabr::types::Date;
use a_sabr::types::Duration;
use a_sabr::types::NodeID;
use a_sabr::types::TimeInterval;

#[derive(Debug)]
struct NoRetention {
    max_proc_time: Duration,
}

impl NodeManager for NoRetention {
    fn accept(
        &self,
        _bundle: &Bundle,
        _time: a_sabr::types::TimeInterval,
        _sender: a_sabr::types::NodeID,
    ) -> bool {
        true
    }

    fn dry_run_retention(
        &self,
        _bundle: &Bundle,
        reception: a_sabr::types::TimeInterval,
        _sender: a_sabr::types::NodeID,
        transmition: a_sabr::types::TimeInterval,
        _next: a_sabr::types::NodeID,
    ) -> bool {
        transmition.start - reception.end <= self.max_proc_time
    }

    fn dry_run_multi(
        &self,
        _bundle: &Bundle,
        reception: a_sabr::types::TimeInterval,
        _sender: a_sabr::types::NodeID,
        transmitions: &[(a_sabr::types::TimeInterval, a_sabr::types::NodeID)],
    ) -> Option<usize> {
        let r = transmitions
            .iter()
            .enumerate()
            .take_while(|(_, trans)| trans.0.start - reception.end <= self.max_proc_time)
            .last();
        Some(r.map_or(0, |(index, _)| index))
    }

    fn commit(
        &mut self,
        _bundle: &Bundle,
        _reception: a_sabr::types::TimeInterval,
        _sender: a_sabr::types::NodeID,
        _transmitions: &[(a_sabr::types::TimeInterval, a_sabr::types::NodeID)],
    ) -> Result<(), ASABRError> {
        Ok(())
    }
}

impl From<Duration> for NoRetention {
    fn from(value: Duration) -> Self {
        NoRetention {
            max_proc_time: value,
        }
    }
}

parse_transparent!(NoRetention, Duration);

#[allow(dead_code)]
struct NoRetOrNone(Box<dyn NodeManager>);

transparent_NM!(NoRetOrNone);

choices!(choice, Choice, (Void, NoManagement), (NoRet, NoRetention));

impl TryFrom<&str> for choice::Kinds {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::Void),
            "noret" => Ok(Self::NoRet),
            _ => Err(()),
        }
    }
}

impl From<choice::Choice> for NoRetOrNone {
    fn from(value: choice::Choice) -> Self {
        NoRetOrNone(match value {
            choice::Choice::Void(no_management) => Box::new(no_management),
            choice::Choice::NoRet(noret) => Box::new(noret),
        })
    }
}
parse_transparent!(NoRetOrNone, choice::Choice);
/// Implements the DispatchParser to allow dynamic parsing.
fn edge_case_example<NM: NodeManager + LexFrom<str>>(cp_path: &str) -> Result<(), ASABRError> {
    let bundle = Bundle {
        source: 0.into(),
        priority: 0,
        size: 0,
        expiration: 1000,
    };

    mk_graph!(graph, NM, EVLManager, cp_path, file);

    let mut finder = HybridParenting::<SABR, NM, EVLManager>::new();

    let source = match graph.node_id_ref(0.into()).unwrap() {
        NodeRef::I(r) => r,
        _ => panic!("Node 0 is not an internal node"),
    };

    let dest_ref = match graph.node_id_ref(2.into()).unwrap() {
        NodeRef::I(r) => r,
        _ => panic!("Node 2 is not an internal node"),
    };

    let mut destination = Dest::INode(dest_ref);
    let res = finder.find_path(&mut graph, 0, source, &bundle, &mut destination, None);

    println!("\nRunning with contact plan location={cp_path}, and destination node=2 ");

    let target_id: usize = dest_ref.into();
    let target_node: RoutableNodeRef = dest_ref.into();

    match res {
        Ok(Some(route)) if route[target_id].is_some() => {
            println!("{}", route.full_path_rev(target_node, &graph).unwrap())
        }
        _ => println!("No route found to node 2."),
    }

    Ok(())
}
fn main() -> Result<(), ASABRError> {
    edge_case_example::<NoManagement>("asabr/examples/satellite_constellation/contact_plan_1.cp")?;
    edge_case_example::<NoRetOrNone>("asabr/examples/satellite_constellation/contact_plan_2.cp")?;

    Ok(())

    // === OUTPUT ===
    // Running with contact plan location=asabr/examples/satellite_constellation/contact_plan_1.cp, and destination node=2
    // Route to node 2 at t=11 with 2 hop(s):
    //         - Reach node 0 at t=0 with 0 hop(s)
    //         - Reach node 1 at t=0 with 1 hop(s)
    //         - Reach node 2 at t=11 with 2 hop(s)

    // Running with contact plan location=asabr/examples/satellite_constellation/contact_plan_2.cp, and destination node=2
    // No route found to node 2.
}
