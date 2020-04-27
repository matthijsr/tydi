use crate::design::composer::impl_graph::builder::*;
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::{IFKey, LibKey, NodeIFHandle, NodeKey, Project, StreamletHandle, StreamletKey, Interface, Mode};
use crate::{Error, Name, Result};


use std::convert::{TryFrom};
use std::rc::Rc;



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

    pub fn parse_implementation(&mut self, input: &str) -> Result<()> {
        let parsed = ImplParser::parse(Rule::Implementation, input)
            .expect("unsuccessful parse") // unwrap the parse result
            .next()
            .unwrap();

        println!("input: {}", input);

        for elem in parsed.into_inner() {
            match elem.as_rule() {
                //Streamlet identifier, builder can be crated now
                Rule::StreamletHandle => {
                    let handle = parse_streamlet_handle(elem.into_inner())?;
                    let builder = GraphBuilder::try_new(self.project, handle)?;
                    self.imp = builder.finish();
                }
                Rule::Instantiation => {
                    println!("That's an Instantiation!");
                    let parsed = parse_instantiation(elem.into_inner())?;
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

    fn parse_instantiation(
        mut pair: pest::iterators::Pairs<Rule>,
    ) -> Result<(NodeKey, StreamletHandle, Vec<(IFKey, NodeIFHandle)>)> {
        let nodekey = pair.next().unwrap().as_str();
        let nodekey = NodeKey::try_from(nodekey)?;

        let pair = pair.next().unwrap();
        match pair.as_rule() {
            Rule::StreamletInst => {
                let parsed = parse_streamlet_inst(pair.into_inner())?;
                println!("Inst: {:?}", nodekey);
                Ok((nodekey, parsed.0, parsed.1))
            }
            _ => {
                println!("Bad news! Supposedly unreachable: {}", pair.as_str());
                Err(Error::ParsingError("Jaj!".parse().unwrap()))
            }
        }
    }

    pub fn instantiate(&mut self, streamlet_handle: StreamletHandle, key: NodeKey) -> Result<()> {
        match self
            .project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())
        {
            Ok(s) => {
                let mut s_copy = s.clone();
                s_copy.set_key(key.clone());
                let node = Node {
                    key: key.clone(),
                    item: Rc::new(s_copy.clone()),
                };
                self.imp.nodes.insert(key.clone(), node.clone());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn get_interface(&self, io: NodeIFHandle) -> Result<Interface> {
        self.imp
            .get_node(io.node()).unwrap()
            .iface(io.iface())
    }

    pub fn finish(self) -> ImplementationGraph {
        self.imp
    }

    pub fn connect(&mut self, sink: NodeIFHandle, source: NodeIFHandle) -> Result<()> {

        let mut sink = sink;
        let mut source = source;

        match self.get_interface(source.clone())?.mode() {
            Mode::In => {
                let swap = source;
                source = sink;
                sink = swap;
            }
            Mode::Out => {}
        }

        self.imp.edges.push(Edge { source, sink });
        Ok(())
    }
}

fn parse_streamlet_handle(mut pair: pest::iterators::Pairs<Rule>) -> Result<StreamletHandle> {
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

fn parse_node_if_handle(mut pair: pest::iterators::Pairs<Rule>) -> Result<NodeIFHandle> {
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

fn parse_streamlet_inst(
    pairs: pest::iterators::Pairs<Rule>,
) -> Result<(StreamletHandle, Vec<(IFKey, NodeIFHandle)>)> {
    let mut edges = vec![];
    let mut handle: Option<StreamletHandle> = None;
    for elem in pairs {
        match elem.as_rule() {
            Rule::StreamletHandle => {
                let parsed = parse_streamlet_handle(elem.into_inner())?;
                println!("Lolza: {:?}", parsed);
                handle = Option::from(parsed);
            }
            Rule::Connection => {
                let connection = parse_connection(elem.into_inner())?;
                edges.push(connection);
            }
            _ => {
                println!("Bad news! Supposedly unreachable: {}", elem.as_str());
            }
        }
    }
    Ok((handle.unwrap(), edges))
}

fn parse_generic_inst(_pair: pest::iterators::Pairs<Rule>) {}

fn parse_connection(mut pairs: pest::iterators::Pairs<Rule>) -> Result<(IFKey, NodeIFHandle)> {
    let source = pairs.next().unwrap().as_str();
    let sink = parse_node_if_handle(pairs.next().unwrap().into_inner())?;
    println!("Connection: {} => {:?}", source, sink);
    let src = IFKey::try_from(source)?;
    let dst = sink;

    Ok((src, dst))
}

fn parse_instantiation(
    mut pair: pest::iterators::Pairs<Rule>,
) -> Result<(NodeKey, StreamletHandle, Vec<(IFKey, NodeIFHandle)>)> {
    let nodekey = pair.next().unwrap().as_str();
    let nodekey = NodeKey::try_from(nodekey)?;

    let pair = pair.next().unwrap();
    match pair.as_rule() {
        Rule::StreamletInst => {
            let parsed = parse_streamlet_inst(pair.into_inner())?;
            println!("Inst: {:?}", nodekey);
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

    

    use crate::design::*;
    use crate::logical::LogicalType;
    use crate::parser::nom::interface;
    use crate::{Name, Result, UniqueKeyBuilder};
    use std::convert::TryFrom;
    

    pub(crate) fn impl_parser_test() -> Result<Project> {
        let key1 = LibKey::try_new("primitives").unwrap();
        let key2 = LibKey::try_new("compositions").unwrap();
        let mut lib = Library::new(key1.clone());

        let mut lib_comp = Library::new(key2.clone());

        //Add streamlet
        let _test1 = lib
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

        let _test2 = lib
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
                        interface("data_in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("data_out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let map = lib_comp.add_streamlet(
            Streamlet::from_builder(
                StreamletKey::try_from("Map").unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    interface("in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                        .unwrap()
                        .1,
                    interface("out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                        .unwrap()
                        .1,
                ]),
                None,
            ).unwrap(),
        ).unwrap();

        lib_comp.add_streamlet(
            Streamlet::from_builder(
                StreamletKey::try_from("Sqrt").unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    interface("in: in Stream<Bits<32>>")
                        .unwrap()
                        .1,
                    interface("out: out Stream<Bits<32>>")
                        .unwrap()
                        .1,
                ]),
                None,
            ).unwrap(),
        ).unwrap();

        lib_comp.add_streamlet(
            Streamlet::from_builder(
                StreamletKey::try_from("Sequence").unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    interface("out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                        .unwrap()
                        .1,
                    interface("length: in Stream<Bits<32>>")
                        .unwrap()
                        .1,
                    interface("elem: in Stream<Bits<32>>")
                        .unwrap()
                        .1,
                ]),
                None,
            ).unwrap(),
        ).unwrap();

        lib_comp.add_streamlet(
            Streamlet::from_builder(
                StreamletKey::try_from("Split").unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    interface("in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                        .unwrap()
                        .1,
                    interface("length: out Stream<Bits<32>>")
                        .unwrap()
                        .1,
                    interface("elem: out Stream<Bits<32>>")
                        .unwrap()
                        .1,
                ]),
                None,
            ).unwrap(),
        ).unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let top_impl = include_str!("../../../../tests/top.impl");
        let map_impl = include_str!("../../../../tests/map.impl");

        let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;

        let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;

        Ok(prj)
    }
}
