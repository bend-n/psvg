use anyhow::{anyhow, Result};
use clap::Parser;
use log::Level as RLevel;
use psvg::tree::Tree;
use std::{path::PathBuf, str::FromStr};
mod logger;

#[derive(Parser)]

/// PSVG: the curveless aliasing svg renderer
struct Args {
    #[arg(short = 's')]
    /// Specify the size of the output png.
    /// If not supplied, will render at the svg's set width and height.
    /// Specify as: '144x124'
    size: Option<Size>,
    /// Log level.
    #[arg(default_value = "info", long = "log")]
    log: Level,
    /// Svg to render.
    file: PathBuf,
    /// File to output rendered svg (png).
    out: PathBuf,
}

#[repr(usize)]
#[derive(clap::ValueEnum, Clone, Copy)]
enum Level {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Level> for RLevel {
    fn from(value: Level) -> Self {
        // SAFETY: same
        unsafe { std::mem::transmute(value) }
    }
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
    logger::init(args.log.into());
    let (mut svg, size) = Tree::new(&std::fs::read_to_string(args.file)?)?;
    match args.size {
        Some(Size { w, h }) => {
            svg.resize(w as f32, h as f32);
        }
        None => svg.resize(size.width(), size.height()),
    }
    svg.render().save(args.out);
    Ok(())
}
