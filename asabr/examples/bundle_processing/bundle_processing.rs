// use std::fs::File;
// use std::io::{BufRead, BufReader};

// use a_sabr::bundle::Bundle;
// use a_sabr::contact_manager::legacy::evl::EVLManager;
// use a_sabr::distance::sabr::SABR;
use a_sabr::errors::ASABRError;
// use a_sabr::node_manager::NodeManager;
// use a_sabr::node_manager::none::NoManagement;
// use a_sabr::parsing::LexFrom;
// use a_sabr::pathfinding::Pathfinding;
// use a_sabr::pathfinding::hybrid_parenting::HybridParentingPath;
// use a_sabr::types::Date;
// use a_sabr::types::Priority;
// use a_sabr::utils::init_pathfinding;
// use a_sabr::{choices, mk_graph, parse_transparent, transparent_NM};

// #[derive(Debug)]
// struct Compressing {
//     max_priority: Priority,
// }

// impl NodeManager for Compressing {
//     fn accept(
//         &self,
//         bundle: &Bundle,
//         time: a_sabr::types::TimeInterval,
//         sender: a_sabr::types::NodeID,
//     ) -> bool {
//         todo!()
//     }

//     fn dry_run_retention(
//         &self,
//         bundle: &Bundle,
//         reception: a_sabr::types::TimeInterval,
//         sender: a_sabr::types::NodeID,
//         transmition: a_sabr::types::TimeInterval,
//         next: a_sabr::types::NodeID,
//     ) -> bool {
//         todo!()
//     }

//     fn dry_run_multi(
//         &self,
//         bundle: &Bundle,
//         reception: a_sabr::types::TimeInterval,
//         sender: a_sabr::types::NodeID,
//         transmitions: &[(a_sabr::types::TimeInterval, a_sabr::types::NodeID)],
//     ) -> Option<usize> {
//         todo!()
//     }

//     fn commit(
//         &mut self,
//         bundle: &Bundle,
//         reception: a_sabr::types::TimeInterval,
//         sender: a_sabr::types::NodeID,
//         transmitions: &[(a_sabr::types::TimeInterval, a_sabr::types::NodeID)],
//     ) -> Result<(), ASABRError> {
//         todo!()
//     }
// }

// impl From<Priority> for Compressing {
//     fn from(value: Priority) -> Self {
//         Compressing {
//             max_priority: value,
//         }
//     }
// }

// parse_transparent!(Compressing, Priority);

// struct CompressingOrNone(Box<dyn NodeManager>);

// transparent_NM!(CompressingOrNone);

// choices!(
//     choice,
//     Choice,
//     (Void, NoManagement),
//     (Compress, Compressing)
// );

// impl TryFrom<&str> for choice::Kinds {
//     type Error = ();
//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         match value {
//             "none" => Ok(Self::Void),
//             "compress" => Ok(Self::Compress),
//             _ => Err(()),
//         }
//     }
// }

// impl From<choice::Choice> for CompressingOrNone {
//     fn from(value: choice::Choice) -> Self {
//         CompressingOrNone(match value {
//             choice::Choice::Void(no_management) => Box::new(no_management),
//             choice::Choice::Compress(compressing) => Box::new(compressing),
//         })
//     }
// }
// parse_transparent!(CompressingOrNone, choice::Choice);


fn main() -> Result<(), ASABRError> {
    Ok(())
}