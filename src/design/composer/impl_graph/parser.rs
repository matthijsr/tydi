use crate::design::composer::impl_graph::builder::*;
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::{IFKey, LibKey, NodeIFHandle, NodeKey, Project, StreamletHandle, StreamletKey};
use crate::{Error, Name, Result};

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

use nom::sequence::pair;
use pest::error::ErrorVariant::ParsingError;
use pest::Parser;

#[derive(Parser)]
#[grammar = "design/composer/impl_graph/impl.pest"]
pub struct ImplParser;

pub struct ImplementationBuilder<'a> {
    project: &'a Project,
    imp: ImplementationGraph,
}

impl<'a> ImplementationBuilder<'a> {
    pub fn new(project: &'a Project) -> Self {
        ImplementationBuilder {
            project,
            imp: ImplementationGraph {
                streamlet: StreamletHandle {
                    lib: Name("".parse().unwrap()),
                    streamlet: Name("".parse().unwrap()),
                },
                edges: vec![],
                nodes: Default::default(),
            },
        }
    }

    pub fn parse_Implementation(&mut self, input: &str) -> Result<()> {
        let parsed = ImplParser::parse(Rule::Implementation, input)
            .expect("unsuccessful parse") // unwrap the parse result
            .next()
            .unwrap();

        println!("input: {}", input);

        for elem in parsed.into_inner() {
            match elem.as_rule() {
                //Streamlet identifier, builder can be crated now
                Rule::StreamletHandle => {
                    let handle = parse_StreamletHandle(elem.into_inner())?;
                    let builder = GraphBuilder::try_new(self.project, handle)?;
                    self.imp = builder.finish();
                }
                Rule::Instantiation => {
                    println!("That's an Instantiation!");
                    let parsed = parse_Instantiation(elem.into_inner())?;
                    self.instantiate(parsed.1.clone(), parsed.0.clone())?;
                    let edges = parsed.2.clone().into_iter().map(|c| {
                        (
                            NodeIFHandle {
                                node: parsed.0.clone(),
                                iface: c.0,
                            },
                            c.1,
                        )
                    });
                    for edge in edges {
                        self.connect(edge.0, edge.1)?
                    }
                }
                _ => {
                    println!("Bad news! Supposedly unreachable: {}", elem.as_str());
                }
            }
        }
        Ok(())
    }

    pub fn instantiate(&mut self, streamlet_handle: StreamletHandle, key: NodeKey) -> Result<()> {
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
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn finish(self) -> ImplementationGraph {
        self.imp
    }

    pub fn connect(&mut self, sink: NodeIFHandle, source: NodeIFHandle) -> Result<()> {
        let sink = sink;
        let source = source;
        self.imp.edges.push(Edge { source, sink });
        Ok(())
    }
}

fn parse_StreamletHandle(mut pair: pest::iterators::Pairs<Rule>) -> Result<StreamletHandle> {
    let lib = pair.next().unwrap().as_str();
    let streamlet = pair.next().unwrap().as_str();
    let lib_key = LibKey::try_from(lib)?;
    let streamlet_key = StreamletKey::try_from(streamlet)?;
    println!("StreamletHandle: {}.{}", lib, streamlet);
    Ok(StreamletHandle {
        lib: lib_key,
        streamlet: streamlet_key,
    })
}

fn parse_NodeIFHandle(mut pair: pest::iterators::Pairs<Rule>) -> Result<NodeIFHandle> {
    let node = pair.next().unwrap().as_str();
    let iface = pair.next().unwrap().as_str();
    let nodekey = LibKey::try_from(node)?;
    let ifkey = StreamletKey::try_from(iface)?;
    println!("NodeIFGHandle: {}.{}", nodekey, ifkey);
    Ok(NodeIFHandle {
        node: nodekey,
        iface: ifkey,
    })
}

fn parse_StreamletInst(
    pairs: pest::iterators::Pairs<Rule>,
) -> Result<(StreamletHandle, Vec<(IFKey, NodeIFHandle)>)> {
    let mut edges = vec![];
    let mut handle: Option<StreamletHandle> = None;
    for elem in pairs {
        match elem.as_rule() {
            Rule::StreamletHandle => {
                let parsed = parse_StreamletHandle(elem.into_inner())?;
                println!("Lolza: {:?}", parsed);
                handle = Option::from(parsed);
            }
            Rule::Connection => {
                let connection = parse_Connection(elem.into_inner())?;
                edges.push(connection);
            }
            _ => {
                println!("Bad news! Supposedly unreachable: {}", elem.as_str());
            }
        }
    }
    Ok((handle.unwrap(), edges))
}

fn parse_GenericInst(pair: pest::iterators::Pairs<Rule>) {}

fn parse_Connection(mut pairs: pest::iterators::Pairs<Rule>) -> Result<(IFKey, NodeIFHandle)> {
    let source = pairs.next().unwrap().as_str();
    let sink = parse_NodeIFHandle(pairs.next().unwrap().into_inner())?;
    println!("Connection: {} => {:?}", source, sink);
    let src = IFKey::try_from(source)?;
    let dst = sink;

    Ok((src, dst))
}

fn parse_Instantiation(
    mut pair: pest::iterators::Pairs<Rule>,
) -> Result<(NodeKey, StreamletHandle, Vec<(IFKey, NodeIFHandle)>)> {
    let nodekey = pair.next().unwrap().as_str();
    let nodekey = NodeKey::try_from(nodekey)?;

    let pair = pair.next().unwrap();
    match pair.as_rule() {
        Rule::StreamletInst => {
            let parsed = parse_StreamletInst(pair.into_inner())?;
            Ok((nodekey, parsed.0, parsed.1))
        }
        _ => {
            println!("Bad news! Supposedly unreachable: {}", pair.as_str());
            Err(Error::ParsingError("Jaj!".parse().unwrap()))
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::design::composer::impl_graph::*;

    use crate::design::*;
    use crate::logical::LogicalType;
    use crate::{Name, Result, UniqueKeyBuilder};
    use std::convert::TryFrom;
    use std::fs;

    #[test]
    pub(crate) fn impl_parser_test() -> Result<Project> {
        /*let key1 = LibKey::try_new("primitives")?;
        let key2 = LibKey::try_new("compositions")?;
        let mut lib = Library::new(key1.clone());
        let mut lib_comp = Library::new(key2.clone());

        //Add streamlet
        let test1 = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test1")?,
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("a", Mode::In, LogicalType::Null, None)?,
                        Interface::try_new("b", Mode::Out, LogicalType::Null, None)?,
                    ]),
                    None,
                )
                ?,
            )
            ?;

        let test2 = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test2")?,
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("c", Mode::In, LogicalType::Null, None)?,
                        Interface::try_new("d", Mode::Out, LogicalType::Null, None)?,
                    ]),
                    None,
                )
                ?,
            )
            ?;

        let top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level")?,
                    UniqueKeyBuilder::new().with_items(vec![
                        Interface::try_new("e", Mode::In, LogicalType::Null, None)?,
                        Interface::try_new("f", Mode::Out, LogicalType::Null, None)?,
                    ]),
                    None,
                )
                ?,
            )
            ?;*/

        let mut prj = Project::new(Name::try_new("TestProj")?);
        /*prj.add_lib(lib)?;
        prj.add_lib(lib_comp)?;

        let unparsed_file = include_str!("../../../../tests/test.imp");

        let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_Implementation(&unparsed_file)?;

        let imp = builder.finish();

        prj.add_streamlet_impl(top, imp)?;*/
        Ok(prj)
    }
}
