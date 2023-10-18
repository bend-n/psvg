use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use clap::Parser;
use psvg::tree::Tree;

#[derive(Parser)]

/// PSVG: the curveless aliasing svg renderer
struct Args {
    #[arg(short = 's')]
    /// Specify the size of the output png.
    /// If not supplied, will render at the svg's set width and height.
    /// Specify as: '144x124'
    size: Option<Size>,
    /// Svg to render.
    file: PathBuf,
    /// File to output rendered svg (png).
    out: PathBuf,
}

#[derive(Copy, Clone, Debug, Default)]
struct Size {
    w: u32,
    h: u32,
}

impl FromStr for Size {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (w, h) = s.split_once('x').ok_or(anyhow!(
            "please delimit width and height witih a 'x': 128x142"
        ))?;
        Ok(Size {
            w: w.parse()?,
            h: h.parse()?,
        })
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut svg = Tree::new(&std::fs::read_to_string(args.file)?)?;
    match args.size {
        Some(Size { w, h }) => {
            svg.resize(w as f32, h as f32);
        }
        None => svg.resize(svg.osize.width(), svg.osize.height()),
    }
    svg.render().save(args.out);
    Ok(())
}
