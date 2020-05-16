use crate::design::composer::impl_graph::builder::GraphBuilder;
use crate::design::composer::impl_graph::ImplementationGraph;
use crate::design::composer::GenericComponent;
use crate::design::{
    ComponentKey, IFKey, Interface, Mode, Project, Streamlet, StreamletHandle, StreamletKey,
};
use crate::logical::{LogicalSplitItem, LogicalType};
use crate::{Error, Name, NonNegative, PathName, Result, UniqueKeyBuilder};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::rc::Rc;

pub type FIFODepth = NonNegative;

pub struct StreamFIFO {
    pub streamlet: Streamlet,
}

impl GenericComponent for StreamFIFO {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl StreamFIFO {
    pub fn try_new(name: &str, data_type: LogicalType, depth: FIFODepth) -> Result<Self> {
        Ok(StreamFIFO {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    Interface::try_new("in", Mode::In, data_type.clone(), None).unwrap(),
                    Interface::try_new("out", Mode::Out, data_type.clone(), None).unwrap(),
                ]),
                None,
            )
            .unwrap(),
        })
    }
}

pub struct StreamSync {
    pub streamlet: Streamlet,
}

impl GenericComponent for StreamSync {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl StreamSync {
    pub fn try_new(name: &str, data_type: LogicalType, depth: FIFODepth) -> Result<Self> {
        Ok(StreamSync {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    Interface::try_new("in", Mode::In, data_type.clone(), None).unwrap(),
                    Interface::try_new("out", Mode::Out, data_type.clone(), None).unwrap(),
                ]),
                None,
            )
            .unwrap(),
        })
    }
}

pub struct GroupSplit {
    pub streamlet: Streamlet,
}

impl GenericComponent for GroupSplit {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl GroupSplit {
    pub fn try_new(name: &str, input: Interface, split_interfaces: Vec<PathName>) -> Result<Self> {
        let haystack: LogicalType = input.typ().clone();
        let mut ifaces: Vec<Interface> =
            vec![Interface::try_new("in", Mode::In, input.typ().clone(), None).unwrap()];

        for item in split_interfaces.iter() {
            let path_string : String = item.0.iter().next().unwrap().to_string();
            let typ: Option<LogicalSplitItem> = input.typ().clone().split().find(|i| {
                i.fields().keys().any(|i| {
                    i.as_ref()
                        .windows(item.len())
                        .any(|name| name == item.as_ref())
                })
            });

            ifaces.push(
                Interface::try_new(
                    path_string,
                    Mode::Out,
                    typ.ok_or_else(|| {
                        Error::ProjectError(format!(
                            "Element {:?} doesn't exist in interface {}",
                            item.as_ref(),
                            input.key().clone()
                        ))
                    })?
                    .logical_type()
                    .clone(),
                    None,
                )
                .unwrap(),
            );
        }
        Ok(GroupSplit {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(ifaces),
                None,
            )
            .unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::design::composer::impl_graph::*;

    use crate::design::*;
    use crate::logical::LogicalType;
    use crate::parser::nom::interface;
    use crate::{Name, Result, UniqueKeyBuilder};
    use std::convert::{TryFrom, TryInto};
    /*
        #[test]
        fn dot() {
            let tmpdir = tempfile::tempdir().unwrap();

            let prj = crate::design::project::tests::proj::single_lib_proj("test");
            let dot = DotBackend {};
            // TODO: implement actual test.

            assert!(dot.generate(&prj, tmpdir).is_ok());
        }
    */
    pub(crate) fn nulls_fifo() -> Result<StreamFIFO> {
        StreamFIFO::try_new("Null_fifo", LogicalType::Null, 0)
    }

    #[test]
    fn test_fifo() {
        assert!(nulls_fifo().is_ok())
    }

    #[test]
    fn test_split() -> Result<()> {
        let pn: PathName = "size".try_into().unwrap();
        let test_split = GroupSplit::try_new(
            "test",
            interface("a: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>").unwrap().1,
            vec![pn],
        );

        for i in test_split.unwrap().outputs() {
            println!("Split interface {}", i.key());
        }
        Ok(())
    }
}
