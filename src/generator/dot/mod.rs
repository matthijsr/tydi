use crate::cat;
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::composer::{GenDot, GenericComponent};
use crate::design::{Interface, Library, Mode, Project, THIS_KEY};

use crate::generator::GenerateProject;
use crate::{Identify, Result};
use std::ops::Deref;
use std::path::Path;

fn tab(n: usize) -> String {
    "\t".repeat(n)
}

//Light-dark color pairs
pub struct Color {
    l: &'static str, //light
    d: &'static str, //dark
}

pub struct Colors([Color; 7]);

impl Default for Colors {
    fn default() -> Self {
        Colors([
            Color {
                l: "#ffffff",
                d: "#f2f2f2",
            }, // 0 white
            Color {
                l: "#F5F5F5",
                d: "#607D8B",
            }, // 1 gray
            Color {
                l: "#ecd9c6",
                d: "#82c6e2",
            }, // 2 brown
            Color {
                l: "#FFF9C4",
                d: "#FBC02D",
            }, // 3 orange
            Color {
                l: "#FFC107",
                d: "#FFEB3B",
            }, // 4 yellow
            Color {
                l: "#C8E6C9",
                d: "#388E3C",
            }, // 5 green
            Color {
                l: "#BBDEFB",
                d: "#536DFE",
            }, // 6 cyan
        ])
    }
}

pub struct DotStyle {
    colors: Colors,
}

impl Default for DotStyle {
    fn default() -> Self {
        DotStyle {
            colors: Colors::default(),
        }
    }
}

impl DotStyle {
    pub fn node(&self, color: usize) -> String {
        format!(
            "fillcolor=\"{}\", color=\"{}\"",
            self.colors.0[color].l, self.colors.0[color].d,
        )
    }
    pub fn cluster(&self, color: usize, l: usize) -> String {
        format!(
            "{}{}{}",
            format!("{}style=\"rounded\";\n", tab(l)),
            format!("{}color=\"{}\" \n", tab(l), self.colors.0[color].d),
            format!("{}bgcolor=\"{}\"\n", tab(l), self.colors.0[color].l),
        )
    }

    pub fn io(&self, _color: usize, mode: Mode) -> String {
        match mode {
            Mode::In => format!("style=\"filled\", {}", self.node(4)),
            Mode::Out => format!("style=\"filled\", {}", self.node(5)),
        }
    }
}

impl GenDot for Edge {
    fn gen_dot(&self, _style: &DotStyle, _project: &Project, l:usize, prefix: &str, _label: &str) -> String {
        let src = match self.source().node().deref() {
            THIS_KEY => cat!(prefix, self.source().iface()),
            _ => cat!(prefix, "impl", self.source().node(), self.source().iface()),
        };
        let snk = match self.sink().node().deref() {
            THIS_KEY => cat!(prefix, self.sink().iface()),
            _ => cat!(prefix, "impl", self.sink().node(), self.sink().iface()),
        };
        format!("{}{} -> {};", tab(l), src, snk)
    }
}

impl GenDot for Node {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l:usize, prefix: &str, _label: &str) -> String {
        self.component().gen_dot(style, project, l, prefix, "")
    }
}

fn item_subgraph<'a, I: 'a>(
    style: &DotStyle,
    project: &Project,
    l: usize,
    prefix: &str,
    suffix: &str,
    items: impl Iterator<Item = &'a I>,
) -> String
    where
        I: GenDot,
{
    format!(
        "{}subgraph cluster_{}_{} {{\n{}\n{}}}\n",
        tab(l),
        prefix,
        suffix,
        format!(
            "{}{}{}",
            format!("{}label=\"\";\n", tab(l + 1)),
            format!("{}style=invis;\n", tab(l + 1)),
            items
                .map(|i| i.gen_dot(style, project, l + 1, prefix, ""))
                .collect::<Vec<String>>()
                .join("\n")
        ),
        tab(l),
    )
}

impl GenDot for Interface {
    fn gen_dot(&self, style: &DotStyle, _project: &Project, l:usize, prefix: &str, _label: &str) -> String {
        format!(
            "{}{} [label=\"{}\\n\", {}];",
            tab(l),
            format!("{}_{}", prefix, self.identifier()),
            self.identifier(),
            //self.typ(),
            style.io(0, self.mode()),
        )
    }
}

impl GenDot for ImplementationGraph {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l:usize, prefix: &str, _label: &str) -> String {
        format!(
            "{}subgraph cluster_{} {{\n{}\n{}}}",
            tab(l),
            format!("{}_{}", prefix, "impl"),
            format!(
                "{}{}{}\n{}",
                format!("{}label=\"Arch\";\n", tab(l + 1)),
                style.cluster(6, l + 1),
                //nodes,
                item_subgraph(
                    style,
                    project,
                    l + 1,
                    format!("{}_{}", prefix, "impl").as_ref(),
                    "nodes",
                    self.nodes().filter(|n| n.key().deref() != THIS_KEY)
                ),
                //edges
                self.edges()
                    .map(|e| e.gen_dot(style, project, l + 1, prefix, ""))
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            tab(l)
        )
    }
}

impl GenDot for dyn GenericComponent {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l:usize, prefix: &str, _label: &str) -> String {
        format!(
            "{}subgraph cluster_{} {{ \n {}{}}}",
            tab(l),
            self.key(),
            format!(
                "\n{}{}{}{}{}\n",
                format!("{}label = \"{}\";\n", tab(l + 1), self.key()),
                style.cluster(1, l + 1),
                item_subgraph(
                    style,
                    project,
                    l + 1,
                    format!("{}_{}", prefix, self.key()).as_ref(),
                    "inputs",
                    self.inputs()
                ),
                item_subgraph(
                    style,
                    project,
                    l + 1,
                    format!("{}_{}", prefix, self.key()).as_ref(),
                    "outputs",
                    self.outputs()
                ),
                // implementation
                if self.get_implementation().is_some() {
                    self.get_implementation().unwrap().gen_dot(
                        style,
                        project,
                        l + 1,
                        format!("{}_{}", prefix, self.key()).as_ref(),
                        "",
                    )
                } else {
                    String::new()
                }
            ),
            tab(l),
        )
    }
}

impl GenDot for Library {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l:usize, _prefix: &str, _label: &str) -> String {
        format!(
            "digraph  {{\n{}\n{}}}",
            format!(
                "{}{}{}{}{}{}{}",
                format!("{}rankdir=LR;\n", tab(l + 1)),
                format!("{}graph [fontname=\"Bitstream Charter\"];\n", tab(l + 1)),
                format!("{}node [fontname=\"Bitstream Charter\"];\n", tab(l + 1)),
                format!("{}node [shape=box, style=\"rounded, filled\"]\n", tab(l + 1)),
                format!("{}edge [fontname=\"Bitstream Charter\"];\n", tab(l + 1)),
                format!("{}splines=compound;\n", tab(l + 1)),
                self.streamlets()
                    .map(|s| s as &dyn GenericComponent)
                    .map(|s| s.gen_dot(style, project, l + 1, self.identifier(), ""))
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            tab(l)
        )
    }
}

/// A configurable VHDL back-end entry point.
#[derive(Default)]
pub struct DotBackend {}

impl GenerateProject for DotBackend {
    fn generate(&self, project: &Project, path: impl AsRef<Path>) -> Result<()> {
        // Create the project directory.
        let dir = path.as_ref().to_path_buf();

        for lib in project.libraries() {
            // Create sub-directory for each lib
            let mut lib_dir = dir.clone();
            lib_dir.push(project.identifier());
            std::fs::create_dir_all(lib_dir.as_path())?;

            // Determine output file
            let mut lib_path = lib_dir.clone();
            lib_path.push(lib.identifier());
            lib_path.set_extension("dot");

            let dot = lib.gen_dot(&DotStyle::default(), project, 0, "", "");

            // TODO: remove this
            println!("{}", dot);

            std::fs::write(lib_path, dot)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use crate::design::composer::impl_graph::parser::tests::impl_parser_test;
    use crate::design::composer::impl_graph::parser::tests::impl_parser_test;
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
    /*#[test]
    fn dot_impl() {
        let tmpdir = tempfile::tempdir().unwrap();

        //let prj = impl_parser_test().unwrap();
        let prj = composition_example().unwrap();
        let dot = DotBackend {};
        // TODO: implement actual test.

        assert!(dot.generate(&prj, tmpdir).is_ok());
    }*/

    #[test]
    fn dot_impl() {
        let tmpdir = tempfile::tempdir().unwrap();

        //let prj = impl_parser_test().unwrap();
        let prj = impl_parser_test().unwrap();
        let dot = DotBackend {};
        // TODO: implement actual test.

        assert!(dot.generate(&prj, tmpdir).is_ok());
    }
}
