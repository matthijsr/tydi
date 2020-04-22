use crate::cat;
use crate::design::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::composer::{GenDot, GenericComponent};
use crate::design::{Interface, Library, Mode, Project, Streamlet, THIS_KEY};
use crate::generator::GenerateProject;
use crate::{Identify, Result};
use std::ops::Deref;
use std::path::Path;

// To be added later for Dot configuration from CLI:
// #[cfg(feature = "cli")]
// use structopt::StructOpt;

fn tab(n: usize) -> String {
    "\t".repeat(n)
}

//Light-dark color pairs
pub struct Color {
    l: &'static str, //light
    d: &'static str, //dark
}

pub struct Colors([Color; 10]);

impl Default for Colors {
    fn default() -> Self {
        Colors([
            Color {
                l: "#ffffff",
                d: "#c4c4c4",
            }, // 0 white
            Color {
                l: "#c4c4c4",
                d: "#808080",
            }, // 1 gray
            Color {
                l: "#d65f5f",
                d: "#8c0800",
            }, // 2 red
            Color {
                l: "#ee854a",
                d: "#b1400d",
            }, // 3 orange
            Color {
                l: "#d5bb67",
                d: "#b8850a",
            }, // 4 yellow
            Color {
                l: "#6acc64",
                d: "#12711c",
            }, // 5 green
            Color {
                l: "#82c6e2",
                d: "#006374",
            }, // 6 cyan
            Color {
                l: "#4878d0",
                d: "#001c7f",
            }, // 7 blue
            Color {
                l: "#956cb4",
                d: "#591e71",
            }, // 8 purple
            Color {
                l: "#dc7ec0",
                d: "#a23582",
            }, // 9 pink
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
            format!("{} color=\"{}\" \n", tab(l), self.colors.0[color].d),
            format!("{} bgcolor=\"{}\"\n", tab(l), self.colors.0[color].l),
        )
    }

    pub fn io(&self, color: usize, mode: Mode) -> String {
        match mode {
            Mode::In => format!("style=\"filled\", {}", self.node(5)),
            Mode::Out => format!("style=\"filled\", {}", self.node(6)),
        }
    }
}

impl GenDot for Edge {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l: usize, prefix: &str) -> String {
        let src = match self.source().node().deref() {
            THIS_KEY => cat!(prefix, self.source().iface()),
            _ => cat!(
                prefix,
                "impl",
                self.source().node(),
                self.source().iface()
            ),
        };
        let snk = match self.sink().node().deref() {
            THIS_KEY => cat!(prefix, self.sink().iface()),
            _ => cat!(prefix, "impl", self.sink().node(), self.sink().iface()),
        };
        format!("{}{} -> {};", tab(l), src, snk)
    }
}

impl GenDot for Node {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l: usize, prefix: &str) -> String {
        self.component().gen_dot(style, project, l, prefix)
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
                .map(|i| i.gen_dot(style, project, l + 1, prefix))
                .collect::<Vec<String>>()
                .join("\n")
        ),
        tab(l),
    )
}

impl GenDot for Interface {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l: usize, prefix: &str) -> String {
        format!(
            "{}{} [label=\"{}\", {:?}]",
            tab(l),
            self.identifier(),
            self.identifier(),
            self.mode()
        )
    }
}

impl GenDot for ImplementationGraph {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l: usize, prefix: &str) -> String {
        let prefix = format!("{}_{}", prefix, "impl");
        format!(
            "{}subgraph cluster_{} {{\n{}\n{}}}",
            tab(l),
            prefix,
            format!(
                "{}{}{}\n{}",
                format!("{}label=\"Implementation\";\n", tab(l + 1)),
                style.cluster(0, l + 1),
                //nodes,
                item_subgraph(
                    style,
                    project,
                    l + 1,
                    prefix.as_ref(),
                    "nodes",
                    self.nodes().filter(|n| n.key().deref() != THIS_KEY)
                ),
                //edges
                self.edges()
                    .map(|e| e.gen_dot(style, project, l + 1, prefix.as_ref()))
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            tab(l)
        )
    }
}

impl GenDot for dyn GenericComponent {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l: usize, prefix: &str) -> String {
        format!(
            "{} subgraph cluster_{} {{ \n {} }}",
            tab(l),
            self.key(),
            format!(
                "\n {} {} {} {} {} \n",
                format!("{} label = \"{}\";\n", tab(l + 1), self.key()),
                style.cluster(1, l + 1),
                item_subgraph(
                    style,
                    project,
                    l + 1,
                    self.key().as_ref(),
                    "inputs",
                    self.inputs()
                ),
                item_subgraph(
                    style,
                    project,
                    l + 1,
                    self.key().as_ref(),
                    "outputs",
                    self.outputs()
                ),
                // implementation
                if self.get_implementation().is_some() {
                    self.get_implementation().unwrap().gen_dot(style, project, l + 1, prefix)
                } else {
                    String::new()
                }

            )
        )
    }
}

impl GenDot for Library {
    fn gen_dot(&self, style: &DotStyle, project: &Project, l: usize, prefix: &str) -> String {
        format!(
            "digraph  {{\n{}\n{}}}",
            format!(
                "{}{}{}",
                format!("{}rankdir=LR;\n", tab(l + 1)),
                format!("{}splines=compound;\n", tab(l + 1)),
                self.streamlets()
                    .map(|s | s as &GenericComponent )
                    .map(|s|s.gen_dot(style, project, l + 1, self.identifier()))
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

            let dot = lib.gen_dot(&DotStyle::default(), project, 0, "");

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
    use crate::design::composer::impl_graph::frontend::tests::composition_example;
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
    #[test]
    fn dot_impl() {
        let tmpdir = tempfile::tempdir().unwrap();

        let prj = composition_example().unwrap();
        let dot = DotBackend {};
        // TODO: implement actual test.

        assert!(dot.generate(&prj, tmpdir).is_ok());
    }
}
