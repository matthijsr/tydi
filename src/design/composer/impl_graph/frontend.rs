use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::{NodeIFHandle, NodeKey, Project, StreamletHandle};
use crate::{Error, Result};

use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

pub struct GraphBuilder<'a> {
    project: &'a Project,
    imp: ImplementationGraph,
}

impl<'a> GraphBuilder<'a> {
    pub fn try_new(project: &'a Project, streamlet_handle: StreamletHandle) -> Result<Self> {
        match project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())
        {
            Ok(s) => Ok(GraphBuilder {
                project,
                imp: ImplementationGraph {
                    streamlet: streamlet_handle.clone(),
                    nodes: vec![(
                        NodeKey::this(),
                        Node {
                            key: NodeKey::this(),
                            item: Rc::new(s.clone()),
                        },
                    )]
                    .into_iter()
                    .collect::<HashMap<NodeKey, Node>>(),
                    edges: vec![],
                },
            }),
            Err(e) => Err(e),
        }
    }

    pub fn finish(self) -> ImplementationGraph {
        self.imp
    }

    pub fn instantiate<I>(
        &mut self,
        streamlet_handle: StreamletHandle,
        instance: I,
    ) -> Result<Node>
    where
        I: TryInto<NodeKey>,
        <I as TryInto<NodeKey>>::Error: Into<Error>,
    {
        let key = instance.try_into().map_err(Into::into).unwrap();

        match self
            .project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())
        {
            Ok(s) => {
                let node = Node {
                    key: key.clone(),
                    item: Rc::new(s.clone()),
                };
                self.imp.nodes.insert(key.clone(), node.clone());
                Ok(node)
            }
            Err(e) => Err(e),
        }

    }

    pub fn this(&self) -> Node {
        // We can unwrap safely here because the "this" node should always exist.
        self.imp.nodes.get(&NodeKey::this()).unwrap().clone()
    }

    pub fn connect(&mut self, sink: Result<NodeIFHandle>, source: Result<NodeIFHandle>) -> Result<()> {
        let sink = sink?;
        let source = source?;
        self.imp.edges.push(Edge { source, sink });
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::design::composer::impl_graph::*;
    use crate::design::composer::*;
    use crate::design::*;
    use crate::logical::LogicalType;
    use crate::{Error, Name, Result, UniqueKeyBuilder};
    use std::convert::TryFrom;

    pub(crate) fn composition_example() -> Result<Project> {
        let key = LibKey::try_new("primitives").unwrap();
        let mut lib = Library::new(key.clone());

        let mut lib_comp = Library::new(key.clone());

        //Add streamlet
        let test1 = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test1").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("a", Mode::In, LogicalType::Null, None).unwrap(),
                        Interface::try_new("b", Mode::Out, LogicalType::Null, None).unwrap(),
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let test2 = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test2").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("c", Mode::In, LogicalType::Null, None).unwrap(),
                        Interface::try_new("d", Mode::Out, LogicalType::Null, None).unwrap(),
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("e", Mode::In, LogicalType::Null, None).unwrap(),
                        Interface::try_new("f", Mode::Out, LogicalType::Null, None).unwrap(),
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let mut imp = GraphBuilder::try_new(&prj, top.clone()).unwrap();

        let this = imp.this();
        //let tet1inst = imp.instantiate(test1, "test1inst").unwrap();

        imp.connect(this.io("e"), this.io("f"));
        Ok(prj)
    }
}
