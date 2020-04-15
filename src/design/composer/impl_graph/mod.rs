use crate::design::composer::{GenericNode, NodeIFHandle, StreamletNode};
use crate::design::{StreamletHandle, StreamletKey};
use crate::Name;
use nom::lib::std::fmt::Formatter;
use std::collections::HashMap;
use std::fmt::Debug;

pub mod frontend;

pub type NodeKey = Name;

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    source: NodeIFHandle,
    sink: NodeIFHandle,
}
impl Edge {
    pub fn source(&self) -> NodeIFHandle {
        self.source.clone()
    }
    pub fn sink(&self) -> NodeIFHandle {
        self.sink.clone()
    }
}

pub enum Node {
    Streamlet(StreamletNode),
    Generic(Box<dyn GenericNode>),
}

pub struct ImplementationGraph {
    streamlet: StreamletHandle,
    edges: Vec<Edge>,
    nodes: HashMap<NodeKey, Node>,
}

impl ImplementationGraph {
    pub fn streamlet_key(&self) -> StreamletKey {
        self.streamlet.streamlet().clone()
    }
    pub fn nodes(&self) -> impl Iterator<Item = (&NodeKey, &Node)> {
        self.nodes.iter()
    }
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }
}

impl Debug for ImplementationGraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl PartialEq for ImplementationGraph {
    fn eq(&self, other: &ImplementationGraph) -> bool {
        self.streamlet.streamlet() == other.streamlet_key()
    }
}
