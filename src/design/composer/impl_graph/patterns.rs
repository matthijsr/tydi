use crate::design::composer::impl_graph::ImplementationGraph;
use crate::design::composer::GenericComponent;
use crate::design::{ComponentKey, IFKey, Interface, Project, Streamlet, StreamletHandle};
use crate::{Error, Result};
use std::rc::Rc;
use crate::design::composer::impl_graph::builder::GraphBuilder;
use std::borrow::Borrow;

pub struct MapPattern {
    pub streamlet: Streamlet,
}

pub struct MapBuilder<'a> {
    project: &'a Project,
    inner: MapPattern,
}

impl GenericComponent for MapPattern {
    fn key(&self) -> ComponentKey {
        self.streamlet.key().clone()
    }
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
    fn interfaces<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Interface> + 'a)> {
        self.streamlet.interfaces()
    }
    fn get_interface(&self, key: IFKey) -> Result<Interface> {
        self.streamlet.get_interface(key)
    }
    fn get_implementation(&self) -> Option<Rc<ImplementationGraph>> {
        self.streamlet.get_implementation()
    }
}
/*
impl<'a> MapPattern<'a> {
    pub fn try_new(project: &'a Project, op: StreamletHandle) -> Result<Self> {
        let mut impl_builder = GraphBuilder::try_new()
        match project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())
        {
            Ok(s) => Ok(
                MapPattern {
                    streamlet: 
                }
            ),
            Err(e) => Err(e),
        }
    }
}*/

