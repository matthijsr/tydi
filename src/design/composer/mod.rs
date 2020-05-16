use crate::design::{ComponentKey, IFKey, Interface, Mode, Project, Streamlet};

use crate::Result;

use crate::design::composer::impl_graph::ImplementationGraph;
use crate::generator::dot::DotStyle;
use std::rc::Rc;

pub mod impl_graph;

/// Traits for components in the implementation graph
pub trait GenHDL {
    fn gen_hdl(&self) -> Result<String>;
}

pub trait GenDot {
    fn gen_dot(
        &self,
        style: &DotStyle,
        project: &Project,
        l: usize,
        prefix: &str,
        label: &str,
    ) -> String;
}

pub trait GenericComponent {
    fn key(&self) -> ComponentKey {
        self.streamlet().key().clone()
    }
    fn interfaces<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Interface> + 'a)> {
        self.streamlet().interfaces()
    }
    fn streamlet(&self) -> &Streamlet;
    fn inputs<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Interface> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::In))
    }
    fn outputs<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Interface> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::Out))
    }
    fn get_interface(&self, key: IFKey) -> Result<Interface> {
        self.streamlet().get_interface(key)
    }
    fn get_implementation(&self) -> Option<Rc<ImplementationGraph>> {
        self.streamlet().get_implementation()
    }
}
