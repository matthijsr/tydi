use crate::design::composer::GenericComponent;
use crate::design::{
    IFKey, Interface, NodeIFHandle, NodeKey, StreamletHandle, StreamletKey,
};
use crate::{Error, Result, Reversed};
use nom::lib::std::fmt::Formatter;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::convert::TryInto;

pub mod frontend;
pub mod generic;

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

#[derive(Clone)]
pub struct Node {
    key: NodeKey,
    item: Rc<dyn GenericComponent>,
}

impl Node {
    pub fn key(&self) -> NodeKey {
        self.key.clone()
    }

    pub fn iface(&self, key: IFKey) -> Result<Interface> {
        let _this_key = NodeKey::this();
        match &self.key {
            _this_key => self.item.get_interface(key).map(|i| i.reversed()),
            _ => self.item.get_interface(key).clone(),
        }
    }

    pub fn io<K>(&self, key: K) -> Result<NodeIFHandle>
    where
        K: TryInto<IFKey>,
        <K as TryInto<IFKey>>::Error: Into<Error>,
    {
        let key = key.try_into().map_err(Into::into)?;
        Ok(NodeIFHandle {
            node: self.key(),
            iface: key,
        })
    }

    pub fn this(&self) -> NodeKey {
        self.key.clone()
    }

    pub fn component(&self) -> Rc<dyn GenericComponent> {
        self.item.clone()
    }
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
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().map(|(_, i)| i)
    }
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }
    pub fn get_node(&self, key: NodeKey) -> Option<&Node> {
        self.nodes.get(&key)
    }
    pub fn get_edge(&self, iface: NodeIFHandle) -> Option<&Edge> {
        self.edges
            .iter()
            .find(|e| e.sink == iface || e.source == iface)
    }
    pub fn this(&self) -> &Node {
        self.nodes.get(&NodeKey::this()).unwrap()
    }
}

impl Debug for ImplementationGraph {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl PartialEq for ImplementationGraph {
    fn eq(&self, other: &ImplementationGraph) -> bool {
        self.streamlet.streamlet() == other.streamlet_key()
    }
}
