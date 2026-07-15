extern crate alloc;

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

///rexeport for macro use
pub use generativity::{Guard, Id, make_guard};

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

pub struct OptUsize(usize);

impl From<Option<usize>> for OptUsize {
    fn from(value: Option<usize>) -> Self {
        match value {
            None => OptUsize(usize::MAX),
            Some(v) => OptUsize(v),
        }
    }
}

impl From<OptUsize> for Option<usize> {
    fn from(value: OptUsize) -> Self {
        match value.0 {
            usize::MAX => None,
            v => Some(v),
        }
    }
}

/// mk_graph_pathfinding!(graphname,NODE_MANAGER,CONTACT_MANAGER,content,content_type?)
/// create a new graph with the provided names and manager types, parsing a ASABR CP from content
/// The optional content_type argument can precise how the content is handed:
///    iter: An iterator over contact plan lines [default]
///    raw: An &str over the whole file content
///    file: a file to open and parse. This require STD
#[macro_export]
macro_rules! mk_graph {
    ($graph:ident,$NM:ty,$CM:ty,$content:expr$(,iterator)?) => {
        $crate::utils::make_guard!($graph);
        let mut $graph = $crate::multigraph::Multigraph::new(
            $graph,
            $crate::contact_plan::asabr_file_lexer::parse_from_iter::<$NM, $CM>($content)?,
        )?;
    };

    ($graph:ident,$NM:ty,$CM:ty,$content:expr,raw) => {
        $crate::mk_graph!($graph, $NM, $CM, $content.lines());
    };
    ($graph:ident,$NM:ty,$CM:ty,$content:expr,file) => {
        $crate::mk_graph!($graph, $NM, $CM, {
            use std::io::{BufRead, BufReader};
            std::io::BufReader::new(match std::fs::File::open($content) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error while trying to open file: {e}");
                    return Err($crate::errors::ASABRError::ParsingError(
                        $crate::parsing::Located {
                            data: "Error while opennig file",
                            line: 0,
                            toknum: 0,
                        },
                    ));
                }
            })
            .lines()
            .map(|l| {
                l.map_err(|e| {
                    eprintln!("Error while reading file: {e}");
                    panic!();
                })
                .unwrap()
            })
        });
    };
}

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

impl<'id, NM: NodeManager, CM: ContactManager, P: Pathfinding<'id, NM, CM, D>, D: Destination<'id>>
    Deref for Router<'id, NM, CM, P, D>
{
    type Target = Multigraph<'id, NM, CM>;

    fn deref(&self) -> &Self::Target {
        &self.multigraph
    }
}
impl<'id, NM: NodeManager, CM: ContactManager, P: Pathfinding<'id, NM, CM, D>, D: Destination<'id>>
    DerefMut for Router<'id, NM, CM, P, D>
{
    fn deref_mut(&mut self) -> &mut <Router<'id, NM, CM, P, D> as Deref>::Target {
        &mut self.multigraph
    }
}
