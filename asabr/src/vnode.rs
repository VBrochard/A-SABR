extern crate alloc;

use alloc::vec::Vec;

use crate::{
    parse_transparent,
    types::{NodeID, NodeName},
};

/// Represents information about a vnode in the network.
#[derive(Debug, Clone)]
pub struct VirtualNodeInfo {
    /// Unique identifier of the virtual node.
    pub vid: NodeID,

    /// Human-readable name of the virtual node.
    pub name: NodeName,

    /// Identifiers of the real nodes represented by this virtual node.
    pub rids: Vec<NodeID>,
}

parse_transparent!(VirtualNodeInfo, (NodeID, NodeName, Vec<NodeID>));

impl From<(NodeID, NodeName, Vec<NodeID>)> for VirtualNodeInfo {
    fn from((vid, name, rids): (NodeID, NodeName, Vec<NodeID>)) -> Self {
        Self { vid, name, rids }
    }
}
