use crate::design::composer::impl_graph::builder::{BasicGraphBuilder};
use crate::design::composer::impl_graph::misc::{FlattenStream, SequenceStream};

use crate::design::composer::GenericComponent;
use crate::design::{
    ComponentKey, IFKey, Interface, Project, Streamlet, StreamletHandle, StreamletKey,
};
use crate::{Result, UniqueKeyBuilder};
use std::borrow::Borrow;
use std::convert::TryFrom;


pub struct MapPattern {
    pub streamlet: Streamlet,
}

impl GenericComponent for MapPattern {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl MapPattern {
    pub fn try_new(name: &str, op: Streamlet, input: Interface, streamlet_handle: StreamletHandle) -> Result<Self> {
        let _op_input = op.inputs().next().unwrap().clone();
        let op_output = op.outputs().next().unwrap().clone();

        println!("Inpuffff iface: {:?}", input.clone());

        let flatten =
            FlattenStream::try_new(format!("{}_flatten", name).as_str(), input.clone())?;

        let sequence =
            SequenceStream::try_new(format!("{}_sequence", name).as_str(), op_output.clone())?;

        let mut streamlet = Streamlet::from_builder(
            StreamletKey::try_from(name).unwrap(),
            UniqueKeyBuilder::new().with_items(vec![
                flatten.inputs().next().unwrap().clone(),
                sequence.outputs().next().unwrap().clone(),
            ]),
            None,
        )
        .unwrap();

        let mut impl_builder = BasicGraphBuilder::new(streamlet.clone(), streamlet_handle);
        let op = impl_builder.instantiate(&op, format!("{}_op", name).as_str());
        let flatten = impl_builder.instantiate(flatten.streamlet(), format!("{}_flatten", name).as_str());
        let sequence = impl_builder.instantiate(sequence.streamlet(), format!("{}_sequence", name).as_str());
        let this = impl_builder.this();
        impl_builder.connect(this.io("in"), flatten.io("in"));
        impl_builder.connect(flatten.io("count"), sequence.io("count"));
        impl_builder.connect(flatten.io("element"), op.io("in"));
        impl_builder.connect(op.io("out"), sequence.io("element"));
        impl_builder.connect(sequence.io("out"), this.io("out"));
        let implementation = impl_builder.finish();

        streamlet.attach_implementation(implementation);

        Ok(MapPattern { streamlet: streamlet })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    

    
    use crate::design::{
        Streamlet, StreamletHandle, StreamletKey,
    };
    
    use crate::parser::nom::interface;
    use crate::{Name, Result, UniqueKeyBuilder};
    use std::convert::{TryFrom};

    #[test]
    fn test_map() -> Result<()> {

        let input = interface("in: in Stream<Bits<32>, d=1>")
            .unwrap()
            .1;

        println!("IUnput: {:?}", input.typ());

        let test_op = Streamlet::from_builder(
            StreamletKey::try_from("Top_level").unwrap(),
            UniqueKeyBuilder::new().with_items(vec![
                interface("in: in Stream<Bits<32>, d=0>")
                .unwrap()
                .1,
                interface("out: out Stream<Bits<32>, d=0>")
                    .unwrap()
                    .1
            ]),
            None,
        )
            .unwrap();

        let test_map = MapPattern::try_new("test", test_op, input,StreamletHandle{
            lib: Name::try_new("test")?,
            streamlet: Name::try_new("test")?
        })?;
        println!(
            "Map interface {:?}",
            test_map.streamlet().outputs().next().unwrap().typ()
        );

        Ok(())
    }
}
