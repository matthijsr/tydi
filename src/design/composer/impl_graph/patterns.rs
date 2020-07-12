use std::borrow::Borrow;
use std::convert::TryFrom;

use crate::{Error, Name, Result, UniqueKeyBuilder};
use crate::design::{Interface, Mode, Project, Streamlet, StreamletHandle, StreamletKey};
use crate::design::composer::GenericComponent;
use crate::design::implementation::{Implementation, ImplementationBackend};
use crate::logical::{LogicalType, Stream};
use crate::physical::Complexity;

///! MapStream construct

pub struct MapStream {
    streamlet: Streamlet,
}

impl GenericComponent for MapStream {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl MapStream {
    pub fn try_new(project: &Project, name: Name, op: StreamletHandle) -> Result<Self> {
        let op = project.get_lib(op.lib())?.get_streamlet(op.streamlet())?;

        let op_input_data_type = match op.inputs().next().unwrap().typ() {
            LogicalType::Stream(s) => Ok(s),
            _ => Err(Error::ComposerError(format!(
                "The data type for the MapStream pattern required to be be Stream!",
            ))),
        }?;

        let advanced_stream = Stream::new(
            op_input_data_type.data().clone(),
            op_input_data_type.throughput(),
            op_input_data_type.dimensionality() + 1,
            op_input_data_type.synchronicity(),
            Complexity::default(),
            op_input_data_type.direction(),
            //TODO: do we want to pass user signals?
            None,
            //TODO: ?
            false,
        );

        let mut ifaces: Vec<Interface> = vec![];
        ifaces.push(Interface::try_new(
            "in",
            Mode::In,
            advanced_stream.clone(),
            None,
        )?);
        ifaces.push(Interface::try_new(
            "out",
            Mode::Out,
            advanced_stream.clone(),
            None,
        )?);

        Ok(MapStream {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(ifaces),
                None,
            )
            .unwrap(),
        })
    }

    pub fn with_backend(&mut self, name: Name, streamlet_handle: StreamletHandle) -> Result<()> {
        //self.backend = Option::from(MapStreamBackend { name, streamlet_handle });
        self.streamlet
            .attach_implementation(Implementation::Backend(Box::new(MapStreamBackend {
                name,
                streamlet_handle,
            })))?;
        Ok(())
    }

    pub fn finish(self) -> Streamlet {
        self.streamlet
    }
}

pub struct MapStreamBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

impl ImplementationBackend for MapStreamBackend {
    fn name(&self) -> Name {
        self.name.clone()
    }

    fn streamlet_handle(&self) -> StreamletHandle {
        self.streamlet_handle.clone()
    }
}

///! ReduceStream construct
pub struct ReduceStream {
    streamlet: Streamlet,
}

impl ReduceStream {
    pub fn try_new(project: &Project, name: Name, op: StreamletHandle) -> Result<Self> {


        let input_if = Interface::try_new(
            "in",
            Mode::In,
            LogicalType::Null,
            None,
        )?.with_type_inference(|i| {
            match i.clone() {
                LogicalType::Stream(s) => Ok(s),
                _ => Err(Error::ComposerError(format!(
                    "The data type for the ReduceStream pattern required to be be Stream!",
                ))),
            }?;
            Ok(i)
        });

        let input_type = match input_if.typ() {
            LogicalType::Stream(s) => Ok(s),
            _ => Err(Error::ComposerError(format!(
                "The data type for the MapStream pattern required to be be Stream!",
            ))),
        }?;

        let output_if = Interface::try_new(
            "out",
            Mode::Out,
            LogicalType::Null,
            None,
        )?;

        let mut ifaces: Vec<Interface> = vec![];
        ifaces.push(input_if);
        ifaces.push(output_if);



        Ok(ReduceStream {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(ifaces),
                None,
            )
            .unwrap(),
        })
    }

    pub fn with_backend(&mut self, name: Name, streamlet_handle: StreamletHandle) -> Result<()> {
        //self.backend = Option::from(MapStreamBackend { name, streamlet_handle });
        self.streamlet
            .attach_implementation(Implementation::Backend(Box::new(MapStreamBackend {
                name,
                streamlet_handle,
            })))?;
        Ok(())
    }

    pub fn finish(self) -> Streamlet {
        self.streamlet
    }
}

pub struct ReduceStreamBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

impl ImplementationBackend for ReduceStreamBackend {
    fn name(&self) -> Name {
        self.name.clone()
    }

    fn streamlet_handle(&self) -> StreamletHandle {
        self.streamlet_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::{Name, Result};
    use crate::design::{StreamletHandle};
    use crate::design::composer::impl_graph::parser::tests::composition_test_proj;

    use super::*;

    #[test]
    fn test_map() -> Result<()> {
        let prj = composition_test_proj()?;

        let test_map = MapStream::try_new(
            &prj,
            Name::try_from("test")?,
            StreamletHandle {
                lib: Name::try_from("primitives")?,
                streamlet: Name::try_from("test_op")?,
            },
        )?;
        println!(
            "Map interface {:?}",
            test_map.streamlet().outputs().next().unwrap().typ()
        );

        Ok(())
    }
}
