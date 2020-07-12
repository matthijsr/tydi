use std::cell::{Ref, RefMut};
use std::rc::Rc;

use crate::design::implementation::Implementation;
use crate::design::{ComponentKey, IFKey, Interface, Mode, Project, Streamlet};
use crate::generator::dot::DotStyle;
use crate::{Result};

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
    fn interfaces<'a>(&'a self) -> Box<(dyn Iterator<Item = Ref<Interface>> + 'a)> {
        self.streamlet().interfaces()
    }
    fn interfaces_mut<'a>(&'a self) -> Box<(dyn Iterator<Item = RefMut<Interface>> + 'a)> {
        unimplemented!()
    }
    fn streamlet(&self) -> &Streamlet;
    fn inputs<'a>(&'a self) -> Box<(dyn Iterator<Item = Ref<Interface>> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::In))
    }
    fn outputs<'a>(&'a self) -> Box<(dyn Iterator<Item = Ref<Interface>> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::Out))
    }
    fn get_interface(&self, key: IFKey) -> Result<Ref<Interface>> {
        self.streamlet().get_interface(key)
    }
    fn get_interface_mut(&self, key: IFKey) -> Result<RefMut<Interface>> {
        self.streamlet().get_interface_mut(key)
    }
    fn get_implementation(&self) -> Option<Rc<Implementation>> {
        self.streamlet().get_implementation()
    }
    fn connect_action(&self) -> Result<()> {
        Ok(())
    }
}
