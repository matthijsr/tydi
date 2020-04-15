use crate::design::composer::impl_graph::ImplementationGraph;
use crate::design::{IFKey, Interface, Project, StreamletHandle};
use crate::{Error, Result};

use std::collections::HashMap;

pub struct GraphBuilder<'a> {
    project: &'a Project,
    imp: ImplementationGraph,
}

impl<'a> GraphBuilder<'a> {
    pub fn try_new(project: &'a Project, streamlet: StreamletHandle) -> Result<Self> {
        project.get_lib(streamlet.lib())?;

        Ok(GraphBuilder {
            project,
            imp: ImplementationGraph {
                streamlet: streamlet.clone(),
                nodes: HashMap::new(),
                edges: vec![],
            },
        })
    }
}
