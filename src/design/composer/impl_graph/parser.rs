use crate::design::composer::impl_graph::builder::*;
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::{
    IFKey, Interface, LibKey, Mode, NodeIFHandle, NodeKey, Project, Streamlet, StreamletHandle,
    StreamletKey,
};
use crate::{Error, Name, Result, UniqueKeyBuilder};

use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

use crate::error::LineErr;
use crate::logical::LogicalType;
use pest::iterators::Pair;
use pest::{Parser, RuleType};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "design/composer/impl_graph/impl.pest"]
pub struct ImplDef;

pub trait LineNum {
    fn line_num(&self) -> usize;
}
impl<'i, R: RuleType> LineNum for Pair<'i, R> {
    fn line_num(&self) -> usize {
        self.as_span().start_pos().line_col().0
    }
}

fn match_rule<'i, T>(
    pair: Pair<'i, Rule>,
    rule: Rule,
    f: impl Fn(Pair<Rule>) -> Result<T>,
) -> Result<T> {
    if pair.as_rule() == rule {
        f(pair)
    } else {
        Err(Error::ImplParsingError(LineErr::new(
            pair.line_num(),
            format!("Expected: \"{:?}\", Actual: \"{:?}\"", rule, pair),
        )))
    }
}

pub struct ImplParser<'i> {
    project: &'i Project,
    body: String,
    imp: ImplementationGraph,
}

impl<'i> ImplParser<'i> {
    pub fn try_new(project: &'i Project, input: &'i str) -> Result<Self> {
        let pair = ImplDef::parse(Rule::implementation, input)
            .map_err(|e| {
                Error::ImplParsingError(LineErr::new(
                    0,
                    format!("Implementation parsing error: {}", e),
                ))
            })?
            .next()
            .unwrap();

        match_rule(pair, Rule::implementation, |pair| {
            let mut pairs = pair.into_inner();
            let streamlet_handle: StreamletHandle = pairs.next().unwrap().try_into()?;
            match project
                .get_lib(streamlet_handle.lib())?
                .get_streamlet(streamlet_handle.streamlet())
            {
                Ok(s) => Ok(ImplParser {
                    project,
                    body: pairs.next().unwrap().as_str().to_string(),
                    imp: ImplementationGraph {
                        streamlet: streamlet_handle,
                        edges: vec![],
                        nodes: vec![(
                            NodeKey::this(),
                            Node {
                                key: NodeKey::this(),
                                item: Rc::new(s.clone()),
                            },
                        )]
                        .into_iter()
                        .collect::<HashMap<NodeKey, Node>>(),
                    },
                }),
                Err(e) => Err(e),
            }
        })
    }

    pub fn parse_node(&self, pair: Pair<Rule>) -> Result<Node> {
        match_rule(pair, Rule::node, |pair| {
            let mut pairs = pair.into_inner();
            let key = Name::try_from(pairs.next().unwrap().as_str())?;
            //println!("Pairs: {:?}", pairs.next());
            let component = pairs.next().unwrap();
            match component.as_rule() {
                Rule::streamlet_inst => {
                    let mut pairs = component.into_inner();
                    let streamlet_handle = StreamletHandle::try_from(pairs.next().unwrap())?;
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
                            Ok((node))
                        }
                        Err(e) => Err(e),
                    }
                }
                _ => {
                    println!("Not implemented yet :( {:?}", pairs);
                    unimplemented!();
                }
            }
        })
    }

    pub fn parse_body(&mut self) -> Result<()> {
        let body = self.body.clone();
        let pair = ImplDef::parse(Rule::implementation_body, body.as_str())
            .map_err(|e| {
                Error::ImplParsingError(LineErr::new(
                    0,
                    format!("Implementation body parsing error: {}", e),
                ))
            })?
            .next()
            .unwrap();

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::connection => {
                    println!("It's a connection! <3");
                    let edge = Edge::try_from(pair)?;
                    self.connect(edge);
                }
                Rule::node => {
                    println!("It's a instantiation! <3");
                    let node = self.parse_node(pair);
                }
                _ => {
                    println!("Not implemented yet :( : {:?}", pair);
                }
            }
        }

        Ok(())
    }



    pub fn connect(&mut self, edge: Edge) -> Result<()> {
        self.imp.edges.push(edge);
        Ok(())
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for StreamletHandle {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::streamlet_handle, |pair| {
            let mut pairs = pair.into_inner();
            let lib = pairs.next().unwrap().as_str();
            let streamlet = pairs.next().unwrap().as_str();
            let lib_key = LibKey::try_from(lib)?;
            let streamlet_key = StreamletKey::try_from(streamlet)?;
            println!("StreamletHandle: {}.{}", lib, streamlet);
            Ok(StreamletHandle {
                lib: lib_key,
                streamlet: streamlet_key,
            })
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Name {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::ident, |pair| {
            let mut pairs = pair.into_inner();
            Name::try_from(pairs.next().unwrap().as_str())
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for NodeIFHandle {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::node_if_handle, |pair| {
            let mut pairs = pair.into_inner();
            let node = pairs.next().unwrap().as_str();
            let node = Name::try_from(node)?;

            let iface = pairs.next().unwrap().as_str();
            let iface = Name::try_from(iface)?;
            println!("NodeIfHandle: {}.{}", node, iface);
            Ok(NodeIFHandle { node, iface })
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Edge {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::connection, |pair| {
            let mut pairs = pair.into_inner();

            let source = NodeIFHandle::try_from(pairs.next().unwrap())?;
            let sink = NodeIFHandle::try_from(pairs.next().unwrap())?;

            println!("Edge: {:?}.{:?}", source, sink);
            Ok(Edge { source, sink })
        })
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
                        interface(
                            "data_in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>",
                        )
                        .unwrap()
                        .1,
                        interface(
                            "data_out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>",
                        )
                        .unwrap()
                        .1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let map = lib_comp
            .add_streamlet(
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
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Sqrt").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>>").unwrap().1,
                        interface("out: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Sequence").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("length: in Stream<Bits<32>>").unwrap().1,
                        interface("elem: in Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Split").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("length: out Stream<Bits<32>>").unwrap().1,
                        interface("elem: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let top_impl = include_str!("../../../../tests/top.impl");
        let map_impl = include_str!("../../../../tests/map.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&prj, &top_impl);
        //let imp = builder.finish();

        Ok(prj)
    }

    #[test]
    fn parser() {
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
                        interface(
                            "data_in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>",
                        )
                        .unwrap()
                        .1,
                        interface(
                            "data_out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>",
                        )
                        .unwrap()
                        .1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let map = lib_comp
            .add_streamlet(
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
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Sqrt").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>>").unwrap().1,
                        interface("out: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Sequence").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("out: out Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("length: in Stream<Bits<32>>").unwrap().1,
                        interface("elem: in Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Split").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                            .unwrap()
                            .1,
                        interface("length: out Stream<Bits<32>>").unwrap().1,
                        interface("elem: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib);
        prj.add_lib(lib_comp);

        let top_impl = include_str!("../../../../tests/top.impl");
        let map_impl = include_str!("../../../../tests/map.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&prj, &top_impl).unwrap();
        builder.parse_body().unwrap();
        //let imp = builder.finish();
    }
}
