use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::rc::Rc;

use pest::{Parser, RuleType};
use pest::iterators::Pair;

use crate::{Error, Name, Result};
use crate::design::{
    GEN_LIB, LibKey, Library, NodeIFHandle, NodeKey, Project,
    StreamletHandle, StreamletKey,
};
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::composer::impl_graph::patterns::MapStream;
use crate::design::implementation::Implementation;
use crate::design::implementation::Implementation::Structural;
use crate::error::LineErr;
use std::borrow::BorrowMut;
use std::ops::Deref;

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

fn match_rule<T>(
    pair: Pair<Rule>,
    rule: Rule,
    mut f: impl FnMut(Pair<Rule>) -> Result<T>,
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
    project: &'i mut Project,
    body: Pair<'i, Rule>,
    imp: Implementation,
}

impl<'i> ImplParser<'i> {
    pub fn try_new(project: &'i mut Project, input: &'i str) -> Result<Self> {
        let pair = ImplDef::parse(Rule::implementation, input)
            .map_err(|e| {
                Error::ImplParsingError(LineErr::new(
                    0,
                    format!("Implementation parsing error: {}", e),
                ))
            })?
            .next()
            .unwrap();

        let mut pairs = pair.into_inner();
        let streamlet_handle: StreamletHandle = pairs.next().unwrap().try_into()?;

        //let pair = pairs.next().unwrap();

        let s = project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())?
            .clone();

        let gen_lib = Library::new(LibKey::try_new(GEN_LIB).unwrap());
        project.add_lib(gen_lib)?;


        Ok(ImplParser {
            project,
            //Safe to unwrap, Pest guarantees that there's an implementation body.
            body: pairs.next().unwrap(),
            imp: Implementation::Structural(ImplementationGraph {
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
            })
        })
    }

    pub fn transform_body(&mut self) -> Result<()> {
        match &mut self.body.as_rule() {
            Rule::structural => {
                self.transform_structural()
            },
            _ => unimplemented!()
        }
    }

    pub fn transform_structural(&mut self) -> Result<()> {
        //Step to structural_body
        let pair = self.body.clone().into_inner().next().unwrap();
        //structural_body inner
        //let pair = pair.into_inner().next().unwrap();
        for pair in pair.into_inner() {
            match &pair.as_rule() {
                Rule::node => {
                    println!("Node!");
                    let node_p = self.transform_node(pair)?;
                    let streamlet = self.project
                        .get_lib(node_p.1.lib())?
                        .get_streamlet(node_p.1.streamlet())?;
                    let mut s_copy = streamlet.clone();
                    s_copy.set_key(node_p.0.clone());
                    let node = Node {
                        key: node_p.0.clone(),
                        item: Rc::new(s_copy.clone()),
                    };
                    match &mut self.imp {
                        Structural(ref mut s) => {
                            println!("Insert node: {:?}", node.key);
                            match s.nodes.insert(node.key().clone(), node) {
                                None => Ok(()),
                                Some(_lib) => Err(Error::ComposerError(format!(
                                    "Instance {} already exists in implementation of {:?}",
                                    node_p.0, node_p.1
                                ))),
                            }?
                        },
                        _ => unreachable!()
                    }
                },
                Rule::connection => {
                    println!("It's a connection! <3");
                    let edge = Edge::try_from(pair)?;
                    self.connect(edge)?
                },
                Rule::bulk_connection => {
                    println!("It's a BULK connection! <3");
                    self.transform_bulk_connection(pair)?;
                }
                _ => unimplemented!()
            }
        }
        Ok(())
    }

    pub fn transform_node(&mut self,pair: Pair<Rule>) -> Result<(Name, StreamletHandle)> {
        //{ ident ~ ":" ~  (pattern | streamlet_inst) }
        let mut pairs = pair.into_inner();
        //ident
        let name_pair = pairs.next().unwrap();
        let key = Name::try_from(name_pair).unwrap();
        println!("Key: {:?}", key);
        //(pattern | streamlet_inst)
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::streamlet_inst => {
                let handle = self.transform_streamlet_inst(pair, key.clone())?;
                Ok((key.clone(), handle))
            },
            Rule::pattern => {
                println!("Pattern! FInally!");
                let handle = self.transform_pattern(pair, key.clone())?;
                Ok((key.clone(), handle))
            }
            _ => {
                println!("Az baj2! {:?}", pair);
                Ok( (key.clone(),
                     StreamletHandle{
                         lib: Name::try_from("")?,
                         streamlet: Name::try_from("")?
                     }))
            }
        }

    }

    pub fn transform_streamlet_inst(&mut self, pair: Pair<Rule>, _key: Name) -> Result<StreamletHandle> {
        //{ streamlet_handle ~ ("[" ~ (parameter_assign)+ ~ "]")? }
        let mut pairs = pair.into_inner();

        //streamlet_handle
        let streamlet_handle_pair = pairs.next().unwrap();
        let streamlet_handle = StreamletHandle::try_from(streamlet_handle_pair)?;
        self
            .project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())?;
        Ok(streamlet_handle)
    }

    pub fn transform_pattern(&mut self, pair: Pair<Rule>, key: Name) -> Result<StreamletHandle> {
        //{ map_stream | filter_stream | reduce_stream }
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::map_stream => {
                println!("Map <3");
                self.transform_map_stream(pair, key)
            },
            _ => {
                println!("Different pattern {:?}", pair);
                Ok(StreamletHandle{
                    lib: Name::try_new("")?,
                    streamlet: Name::try_new("")?
                })
            }
        }
    }

    pub fn transform_map_stream(&mut self, pair: Pair<Rule>, key: Name) -> Result<StreamletHandle> {
        let op = self.transform_node(pair.into_inner().next().unwrap())?;
        println!("{:?}", op);

        let name = Name::try_from(format!("{}_gen", key.to_string()))?;
        let mut component = MapStream::try_new(self.project, name.clone(), op.1)?;
        component.with_backend(name.clone(), StreamletHandle{ lib: Name::try_new(GEN_LIB)?, streamlet: name.clone() })?;
        let streamlet = component.finish();
        self.project
            .get_lib_mut(Name::try_from(GEN_LIB).unwrap())?
            .add_streamlet(streamlet.clone())
    }

    pub fn transform_bulk_connection(&mut self, pair: Pair<Rule>) -> Result<()> {
        //{ (ident | node_if_handle_list) ~ "<=>" ~ (ident | node_if_handle_list) }
        let mut pairs = pair.into_inner();

        //(ident | node_if_handle_list)
        let pair = pairs.next().unwrap();

        let mut src = match pair.as_rule() {
            Rule::ident => Name::try_from(pair.clone()),
            _ => unimplemented!()
        }?;

        for pair in pairs {
            let dst = match pair.as_rule() {
                Rule::ident => Name::try_from(pair),
                _ => unimplemented!()
            }?;

            let src_i = match &mut self.imp {
                Structural(ref mut s) => {
                    match s.get_node(src.clone())?.component().outputs().next() {
                        Some(i) => Ok(i.clone()),
                        None =>  Err(Error::ComposerError(format!(
                            "Bulk connection left side doesn't have an output interface: {:?}",
                            src
                        )))
                    }
                },
                _ => unreachable!()
            }?;

            let dst_i = match &mut self.imp {
                Structural(ref mut s) => {
                    match s.get_node(dst.clone())?.component().inputs().next() {
                        Some(i) => Ok(i.clone()),
                        None =>  Err(Error::ComposerError(format!(
                            "Bulk connection right side doesn't have an input interface: {:?}",
                            dst
                        )))
                    }
                },
                _ => unreachable!()
            }?;

            let edge = Edge{
                source: NodeIFHandle { node: src.clone(), iface: src_i.key().clone() },
                sink: NodeIFHandle { node: dst.clone(), iface: dst_i.key().clone() }
            };

            self.connect(edge)?;

            src=dst;

        }
        Ok(())
    }

    pub fn connect(&mut self, edge: Edge) -> Result<()> {
        match &mut self.imp {
            Structural(ref mut s) => {

                //Deal with type inferences
                let src_type = s.clone().get_node(edge.clone().source().node)?.iface(edge.clone().source().iface)?.typ().clone();
                let dst_type = s.clone().get_node(edge.clone().sink().node)?.iface(edge.clone().sink().iface)?.typ().clone();
                s.get_node(edge.clone().source().node)?.iface_mut(edge.clone().source().iface)?.infer_type(dst_type)?;
                s.get_node(edge.clone().sink().node)?.iface_mut(edge.clone().sink().iface)?.infer_type(src_type)?;

                s.edges.push(edge)
            },
            _ => unreachable!()
        }
        Ok(())
    }

    pub fn this(&self) -> Node {

        match &self.imp {
            Structural(s) => {
                // We can unwrap safely here because the "this" node should always exist.
                s.nodes.get(&NodeKey::this()).unwrap().clone()
            },
            _ => unreachable!()
        }
    }

    pub fn finish(self) -> Implementation {
        self.imp
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
        match_rule(pair.clone(), Rule::ident, |pair| {
            Name::try_from(pair.clone().as_str())
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

            let sink = NodeIFHandle::try_from(pairs.next().unwrap())?;
            let source = NodeIFHandle::try_from(pairs.next().unwrap())?;

            println!("Edge: {:?}.{:?}", source, sink);
            Ok(Edge { source, sink })
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::convert::TryFrom;

    use crate::{Name, Result, UniqueKeyBuilder};
    use crate::design::*;
    use crate::logical::LogicalType;
    use crate::parser::nom::interface;

    use super::*;

    pub(crate) fn composition_test_proj() -> Result<Project> {
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

        let _top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _map = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Magic").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _map = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test3").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Group<a: Bits<32>, b: Stream<Bits<32>,d=1>>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _sqrt = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Test4").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>>").unwrap().1,
                        interface("out: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _test_op = lib.add_streamlet(Streamlet::from_builder(
            StreamletKey::try_from("test_op").unwrap(),
            UniqueKeyBuilder::new().with_items(vec![
                interface("in: in Stream<Bits<32>, d=0>")
                    .unwrap()
                    .1,
                interface("out: out Stream<Bits<32>, d=0>")
                    .unwrap()
                    .1
            ]),
            None,
        ).unwrap()
        ).unwrap();


        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib)?;
        prj.add_lib(lib_comp)?;
        Ok(prj)
    }

    pub(crate) fn impl_parser_test() -> Result<Project> {

        let mut prj = composition_test_proj()?;
        let top_impl = include_str!("../../../../tests/top.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&mut prj, &top_impl)?;
        builder.transform_body().unwrap();
        let imp = builder.finish();
        prj.add_streamlet_impl(StreamletHandle{ lib: Name::try_from("compositions")?, streamlet: Name::try_from("Top_level")? }, imp)?;
        Ok(prj)
    }

    #[test]
    fn parser() -> Result<()> {

        let mut prj = composition_test_proj()?;

        let top_impl = include_str!("../../../../tests/top.impl");
        let _map_impl = include_str!("../../../../tests/map.impl");

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&mut prj, &top_impl).unwrap();
        builder.transform_body().unwrap();
        let _imp = builder.finish();
        Ok(())
    }

    pub(crate) fn pow2_example() -> Result<Project> {
        let key1 = LibKey::try_new("primitives").unwrap();
        let key2 = LibKey::try_new("compositions").unwrap();
        let mut lib = Library::new(key1.clone());

        let mut lib_comp = Library::new(key2.clone());

        let top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _sqrt = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Pow2").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>>").unwrap().1,
                        interface("out: out Stream<Bits<32>>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();




        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib)?;
        prj.add_lib(lib_comp)?;

        let top_impl = include_str!("../../../../tests/pow2.impl");
        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&top_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;*/

        /*let mut builder = ImplementationBuilder::new(&prj);
        builder.parse_implementation(&map_impl)?;
        let imp = builder.finish();
        prj.add_streamlet_impl(map, imp)?;*/

        let mut builder = ImplParser::try_new(&mut prj, &top_impl)?;
        builder.transform_body().unwrap();
        let imp = builder.finish();
        prj.add_streamlet_impl(top, imp)?;

        Ok(prj)
    }
}
