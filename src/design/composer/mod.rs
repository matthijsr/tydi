use crate::design::{IFKey, Interface, ComponentKey, Project, Mode};

use crate::{Result};

use crate::design::composer::impl_graph::ImplementationGraph;
use std::rc::Rc;
use crate::generator::dot::DotStyle;

pub mod impl_graph;

/// Traits for components in the implementation graph
pub trait GenHDL {
    fn gen_hdl(&self) -> Result<String>;
}

pub trait GenDot{
    fn gen_dot(&self, style: &DotStyle, project: &Project, l:usize, prefix: &str) -> String;
}

pub trait GenericComponent {
    fn key(&self) -> ComponentKey;
    fn interfaces<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Interface> + 'a)>;
    fn inputs<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Interface> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::In))
    }
    fn outputs<'a>(&'a self) -> Box<(dyn Iterator<Item = &'a Interface> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::Out))
    }
    fn get_interface(&self, key: IFKey) -> Result<Interface>;
    fn get_implementation(&self) -> Option<Rc<ImplementationGraph>>;
}