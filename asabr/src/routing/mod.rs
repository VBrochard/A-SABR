use core::marker::PhantomData;

use crate::{
    bundle::Bundle,
    contact_manager::ContactManager,
    contact_plan::ContactPlan,
    errors::ASABRError,
    multigraph::{INodeRef, Multigraph},
    node_manager::NodeManager,
    pathfinding::{PathFindingOutput, Pathfinding, destination::Destination},
    types::Date,
};

use generativity::{Guard, Id};

pub mod aliases;
pub mod cgr;
pub mod spsn;
pub mod volcgr;

pub struct Router<
    'id,
    NM: NodeManager,
    CM: ContactManager,
    P: Pathfinding<'id, NM, CM, D>,
    D: Destination<'id>,
> {
    pub multigraph: Multigraph<'id, NM, CM>,
    pub pathfinder: P,
    _phantom: PhantomData<fn(D)>,
}

impl<'id, NM: NodeManager, CM: ContactManager, P: Pathfinding<'id, NM, CM, D>, D: Destination<'id>>
    Router<'id, NM, CM, P, D>
{
    pub fn build<T>(
        guard: Guard<'id>,
        contact_plan: ContactPlan<NM, CM>,
        pathfinder_args: T,
    ) -> Result<Self, ASABRError>
    where
        for<'a> (&'a Multigraph<'id, NM, CM>, T): Into<P>,
    {
        let multigraph = Multigraph::new(guard, contact_plan)?;
        let pathfinder = (&multigraph, pathfinder_args).into();
        Ok(Self {
            multigraph,
            pathfinder,
            _phantom: PhantomData,
        })
    }
    pub fn new(multigraph: Multigraph<'id, NM, CM>, pathfinder: P) -> Self {
        Self {
            multigraph,
            pathfinder,
            _phantom: PhantomData,
        }
    }
    /// # Safety
    /// see Multigraph::new_unguarder
    pub unsafe fn build_unguarded<T>(
        contact_plan: ContactPlan<NM, CM>,
        pathfinder_args: T,
    ) -> Result<Self, ASABRError>
    where
        for<'a> (&'a Multigraph<'id, NM, CM>, T): Into<P>,
    {
        Self::build(
            unsafe { Guard::new(Id::new()) },
            contact_plan,
            pathfinder_args,
        )
    }

    pub fn find_path(
        &mut self,
        mut destination: D,
        routing_time: Date,
        source: INodeRef<'id>,
        bundle: &Bundle,
        prune_time: Option<Date>,
    ) -> Result<Option<PathFindingOutput<'id, '_>>, ASABRError> {
        self.pathfinder.find_path(
            &mut self.multigraph,
            routing_time,
            source,
            bundle,
            &mut destination,
            prune_time,
        )
    }

    pub fn route(
        &mut self,
        mut destination: D,
        routing_time: Date,
        source: INodeRef<'id>,
        bundle: &Bundle,
        prune_time: Option<Date>,
    ) -> Result<Option<D::RoutingOutput>, ASABRError> {
        let route = self.pathfinder.find_path(
            &mut self.multigraph,
            routing_time,
            source,
            bundle,
            &mut destination,
            prune_time,
        )?;
        let Some(route) = route else {
            return Ok(None);
        };
        destination.route(&mut self.multigraph, bundle, route)
    }
}
