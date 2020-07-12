use crate::design::{ComponentKey, IFKey, Interface, Mode, Project, Streamlet, StreamletHandle};

use crate::{Result, Name};

use crate::design::composer::impl_graph::ImplementationGraph;
use crate::generator::dot::DotStyle;
use std::rc::Rc;
use std::fmt::{Debug, Formatter};


///Trait for general implementation backends
pub trait ImplementationBackend {
    fn name(&self) -> Name;
    fn streamlet_handle(&self) -> StreamletHandle;
}

impl Debug for ImplementationBackend {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl PartialEq for Implementation {
    fn eq(&self, other: &Implementation) -> bool {
        PartialEq::eq(&self.streamlet(), &other.streamlet())
    }
}

/// An implementation variant.
#[derive(Debug)]
pub enum Implementation {
    Structural(ImplementationGraph),
    Backend(Box<dyn ImplementationBackend>)
}

impl Implementation {
    /// Returns a reference to the streamlet this implementation implements.
    pub fn streamlet(&self) -> StreamletHandle {
        self.streamlet()
    }
}

