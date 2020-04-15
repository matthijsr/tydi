use crate::design::composer::impl_graph::NodeKey;
use crate::design::{IFKey, Interface, StreamletHandle};
use crate::Name;
use crate::{Error, Result};
use std::fmt::Debug;

pub mod impl_graph;

#[derive(Clone, Debug, PartialEq)]
pub struct NodeIFHandle {
    node: NodeKey,
    iface: IFKey,
}

impl NodeIFHandle {
    pub fn node(&self) -> NodeKey {
        self.node.clone()
    }
    pub fn iface(&self) -> NodeKey {
        self.iface.clone()
    }
}

/// Traits for components in the implementation graph
pub trait GenHDL {
    fn gen_hdl(&self) -> Result<String>;
}

pub trait GenDot {
    fn gen_dot(&self) -> Result<String>;
}

pub trait GenericNode {
    fn interfaces(&self) -> Box<dyn Iterator<Item = &Interface>>;
    fn get_interface(&self, key: IFKey) -> Result<&Interface>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct StreamletNode {
    key: NodeKey,
    streamlet: StreamletHandle,
}
