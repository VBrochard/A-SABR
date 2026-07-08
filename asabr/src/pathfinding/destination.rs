extern crate alloc;
use crate::{
    bundle::Bundle,
    contact_manager::ContactManager,
    multigraph::{Multigraph, NodeRef, RNodeRef, VNodeRef},
    node_manager::NodeManager,
    pathfinding::PathFindingOutput,
    types::Date,
};
use alloc::{boxed::Box, rc::Rc};

pub trait Destination<'id> {
    /// A new pathfinding begin, reinit to a state of no reachable nodes
    fn reinit(&mut self);
    /// This node have been poped from disktra prio_queue, should we stop ?
    fn now_reached(&mut self, node: NodeRef<'id>) -> bool;
    /// Should paths to this vnode be considered ?
    fn is_useful(&self, node: VNodeRef<'id>) -> bool;
    /// Wether this path tree is still valid/usefull to pass a bundle
    /// This will be the pathfinding output so you may as well update the path times while your at it
    /// # Safety
    /// self paths should have no cycle (wich is true of any reasonable PathfindingOutput not modified by hand outside of the library)
    unsafe fn validate(
        &self,
        paths: &mut PathFindingOutput<'id, '_>,
        time: Date,
        bundle: &Bundle,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> bool;
    /// Some pathfinder can provide performance improvement if this return Some
    /// Returning the same id for two different destination may however prevent the pathfinder from finding the best path (or a path at all)
    fn into_id(
        &self,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> Option<usize>;
}

pub enum Dest<'id> {
    RNode(RNodeRef<'id>),
    VNode(VNodeRef<'id>),
    AllNodes(),
    AnyCast(Rc<[RNodeRef<'id>]>),
    MultiCast(Rc<[RNodeRef<'id>]>, Box<[bool]>, usize),
}

impl<'id> Destination<'id> for Dest<'id> {
    fn reinit(&mut self) {
        if let Self::MultiCast(_, reached, counter) = self {
            for r in reached.iter_mut() {
                *r = false
            }
            *counter = 0
        }
    }

    fn now_reached(&mut self, node: NodeRef<'id>) -> bool {
        match (self, node) {
            (Self::RNode(dest), NodeRef::R(node)) => *dest == node,
            (Self::VNode(_), NodeRef::V(_)) => true, // because the correct vnode is the only one accepted
            (Self::AllNodes(), _) => false,
            (Self::AnyCast(dests), NodeRef::R(node)) => dests.binary_search(&node).is_ok(),
            (Self::MultiCast(dests, reached, counter), NodeRef::R(node)) => {
                if let Ok(idx) = dests.binary_search(&node) {
                    if !reached[idx] {
                        reached[idx] = true;
                        *counter += 1;
                        *counter == dests.len()
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    fn is_useful(&self, node: VNodeRef<'id>) -> bool {
        match self {
            Self::VNode(dest) => *dest == node,
            Self::AllNodes() => true,
            _ => false,
        }
    }

    unsafe fn validate(
        &self,
        paths: &mut PathFindingOutput<'id, '_>,
        time: Date,
        bundle: &Bundle,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> bool {
        unsafe {
            match self {
                Dest::RNode(rnode_ref) => {
                    paths.validate(NodeRef::R(*rnode_ref), time, bundle, graph)
                }
                Dest::VNode(vnode_ref) => {
                    paths.validate(NodeRef::V(*vnode_ref), time, bundle, graph)
                }
                Dest::AllNodes() => true,
                Dest::AnyCast(rnode_refs) => rnode_refs
                    .iter()
                    .any(|dest| paths.validate(NodeRef::R(*dest), time, bundle, graph)),
                Dest::MultiCast(rnode_refs, _items, _) => rnode_refs
                    .iter()
                    // This path is not technically fully valid, but hey, it is still interesting, so we want to extract it
                    .any(|dest| paths.validate(NodeRef::R(*dest), time, bundle, graph)),
            }
        }
    }

    fn into_id(
        &self,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> Option<usize> {
        match self {
            Dest::RNode(rnode_ref) => Some((*rnode_ref).into()),
            Dest::VNode(vnode_ref) => Some(graph.into_usize((*vnode_ref).into())),
            Dest::AllNodes() => Some(graph.get_routable_count()),
            Dest::AnyCast(_rnode_refs) => None,
            Dest::MultiCast(..) => None,
        }
    }
}

impl<'id> From<RNodeRef<'id>> for Dest<'id> {
    fn from(value: RNodeRef<'id>) -> Self {
        Self::RNode(value)
    }
}
impl<'id> From<VNodeRef<'id>> for Dest<'id> {
    fn from(value: VNodeRef<'id>) -> Self {
        Self::VNode(value)
    }
}
impl<'id> From<NodeRef<'id>> for Dest<'id> {
    fn from(value: NodeRef<'id>) -> Self {
        match value {
            NodeRef::R(rnode_ref) => rnode_ref.into(),
            NodeRef::V(vnode_ref) => vnode_ref.into(),
        }
    }
}
impl<'id> From<All> for Dest<'id> {
    fn from(_value: All) -> Self {
        Self::AllNodes()
    }
}
impl<'id> Dest<'id> {
    pub fn anycast(casts: Rc<[RNodeRef<'id>]>) -> Self {
        Self::AnyCast(casts)
    }
    pub fn multicast(casts: Rc<[RNodeRef<'id>]>) -> Self {
        let bools = unsafe { Box::new_zeroed_slice(casts.len()).assume_init() };
        Self::MultiCast(casts, bools, 0)
    }
}

impl<'id> Destination<'id> for RNodeRef<'id> {
    #[inline(always)]
    fn reinit(&mut self) {}

    #[inline(always)]
    fn now_reached(&mut self, node: NodeRef<'id>) -> bool {
        node == NodeRef::R(*self)
    }

    #[inline(always)]
    fn is_useful(&self, _node: VNodeRef<'id>) -> bool {
        false
    }

    unsafe fn validate(
        &self,
        paths: &mut PathFindingOutput<'id, '_>,
        time: Date,
        bundle: &Bundle,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> bool {
        unsafe { paths.validate(NodeRef::R(*self), time, bundle, graph) }
    }

    fn into_id(
        &self,
        _graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> Option<usize> {
        Some((*self).into())
    }
}

impl<'id> Destination<'id> for VNodeRef<'id> {
    #[inline(always)]
    fn reinit(&mut self) {}

    #[inline(always)]
    fn now_reached(&mut self, node: NodeRef<'id>) -> bool {
        node == NodeRef::V(*self)
    }

    #[inline(always)]
    fn is_useful(&self, node: VNodeRef<'id>) -> bool {
        node == *self
    }

    unsafe fn validate(
        &self,
        paths: &mut PathFindingOutput<'id, '_>,
        time: Date,
        bundle: &Bundle,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> bool {
        unsafe { paths.validate(NodeRef::V(*self), time, bundle, graph) }
    }

    fn into_id(
        &self,
        _graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> Option<usize> {
        Some((*self).into())
    }
}

impl<'id> Destination<'id> for NodeRef<'id> {
    #[inline(always)]
    fn reinit(&mut self) {}

    #[inline(always)]
    fn now_reached(&mut self, node: NodeRef<'id>) -> bool {
        node == *self
    }

    #[inline(always)]
    fn is_useful(&self, node: VNodeRef<'id>) -> bool {
        match self {
            NodeRef::R(rnode_ref) => rnode_ref.is_useful(node),
            NodeRef::V(vnode_ref) => vnode_ref.is_useful(node),
        }
    }
    unsafe fn validate(
        &self,
        paths: &mut PathFindingOutput<'id, '_>,
        time: Date,
        bundle: &Bundle,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> bool {
        unsafe { paths.validate(*self, time, bundle, graph) }
    }

    fn into_id(
        &self,
        graph: &Multigraph<'id, impl NodeManager, impl ContactManager>,
    ) -> Option<usize> {
        Some(graph.into_usize(*self))
    }
}

pub struct All;

impl Destination<'_> for All {
    #[inline(always)]
    fn reinit(&mut self) {}

    #[inline(always)]
    fn now_reached(&mut self, _node: NodeRef<'_>) -> bool {
        false
    }

    #[inline(always)]
    fn is_useful(&self, _node: VNodeRef<'_>) -> bool {
        true
    }
    unsafe fn validate(
        &self,
        _paths: &mut PathFindingOutput<'_, '_>,
        _time: Date,
        _bundle: &Bundle,
        _graph: &Multigraph<'_, impl NodeManager, impl ContactManager>,
    ) -> bool {
        true
    }

    fn into_id(
        &self,
        _graph: &Multigraph<'_, impl NodeManager, impl ContactManager>,
    ) -> Option<usize> {
        None
    }
}
