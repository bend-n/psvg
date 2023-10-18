use anyhow::Result;
use std::rc::Rc;
use tiny_skia_path::{NonZeroPositiveF32, NormalizedF32, Path, Point, Size, Transform};
use usvg::{Color, Fill, Opacity, Options, Paint, TreeParsing};
use vecto::Vector2;

#[derive(Debug)]
pub struct Tree {
    pub width: f32,
    pub height: f32,
    pub children: Box<[Node]>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn pack(Color { red, green, blue }: Color, a: Opacity) -> u64 {
            ((red as u64) << 24)
                | ((green as u64) << 16)
                | ((blue as u64) << 8)
                | (a.to_u8() as u64)
        }

        macro_rules! hex {
            ($c:expr, $a:expr) => {
                &format_args!("#{:x}", pack($c, $a))
            };
        }
        // can be transmuted etc but takes much effort
        fn vec(points: &[Point]) -> Vec<Vector2<u64>> {
            points
                .iter()
                .map(|&Point { x, y }| Vector2::new(x.round() as u64, y.round() as u64))
                .collect()
        }
        match self {
            Self::Fill {
                color,
                opacity,
                path,
            } => f
                .debug_struct("Fill")
                .field("color", hex!(*color, *opacity))
                .field("path", &format_args!("{:?}", vec(path)))
                .finish(),
            Self::Stroke {
                color,
                opacity,
                stroke_color,
                stroke_opacity,
                stroke,
                path,
            } => f
                .debug_struct("Stroke")
                .field("color", hex!(*color, *opacity))
                .field("stroke_color", hex!(*stroke_color, *stroke_opacity))
                .field("stroke", &stroke.get())
                .field("path", &format_args!("{:?}", vec(path)))
                .finish(),
        }
    }
}

pub enum Node {
    Fill {
        color: Color,
        opacity: Opacity,
        path: Box<[Point]>,
    },
    Stroke {
        color: Color,
        opacity: Opacity,
        stroke_color: Color,
        stroke_opacity: Opacity,
        stroke: NonZeroPositiveF32,
        path: Box<[Point]>,
    },
}

fn pointify(p: &Rc<Path>, t: Transform) -> Box<[Point]> {
    let mut p: Box<[Point]> = p.points().into();
    t.map_points(&mut p);
    p
}

impl Node {
    fn transform(&mut self, t: Transform) {
        let (Self::Fill { path, .. } | Self::Stroke { path, .. }) = self;
        if t.is_identity() {
            return;
        }
        t.map_points(path);
    }
}

impl From<usvg::Tree> for Tree {
    fn from(tree: usvg::Tree) -> Self {
        let mut children = vec![];
        collect(tree.root, &mut children);
        Self {
            width: tree.view_box.rect.width(),
            height: tree.view_box.rect.height(),
            children: children.into(),
        }
    }
}

trait PColor {
    fn col(&self) -> Color;
}

impl PColor for Paint {
    fn col(&self) -> Color {
        match self {
            Self::Color(c) => *c,
            _ => unimplemented!(),
        }
    }
}
fn convert(node: usvg::Node, to: &mut Vec<Node>) {
    match &*node.clone().borrow() {
        usvg::NodeKind::Path(usvg::Path {
            stroke:
                Some(usvg::Stroke {
                    paint: stroke_paint,
                    opacity: stroke_opacity,
                    width: stroke,
                    ..
                }),
            transform,
            fill: Some(Fill { opacity, paint, .. }),
            data: path,
            ..
        }) => {
            to.push(Node::Stroke {
                color: paint.col(),
                opacity: *opacity,
                stroke: *stroke,
                stroke_opacity: *stroke_opacity,
                stroke_color: stroke_paint.col(),
                path: pointify(path, *transform),
            });
        }
        usvg::NodeKind::Path(usvg::Path {
            stroke:
                Some(usvg::Stroke {
                    paint: stroke_paint,
                    opacity: stroke_opacity,
                    width: stroke,
                    ..
                }),
            transform,
            fill: None,
            data: path,
            ..
        }) => {
            to.push(Node::Stroke {
                color: Color::black(),
                opacity: NormalizedF32::new(0.0).unwrap(),
                stroke: *stroke,
                stroke_opacity: *stroke_opacity,
                stroke_color: stroke_paint.col(),
                path: pointify(path, *transform),
            });
        }
        usvg::NodeKind::Path(usvg::Path {
            transform,
            stroke: None,
            fill: Some(Fill { opacity, paint, .. }),
            data: path,
            ..
        }) => {
            to.push(Node::Fill {
                color: paint.col(),
                opacity: *opacity,
                path: pointify(path, *transform),
            });
        }
        usvg::NodeKind::Path(usvg::Path {
            transform,
            stroke: None,
            fill: None,
            data: path,
            ..
        }) => {
            to.push(Node::Fill {
                color: Color::black(),
                opacity: NormalizedF32::new(0.0).unwrap(),
                path: pointify(path, *transform),
            });
        }
        usvg::NodeKind::Group(_) => {}
        usvg::NodeKind::Image(_) => todo!(),
        usvg::NodeKind::Text(_) => unimplemented!(),
    }
}

fn collect(node: usvg::Node, to: &mut Vec<Node>) {
    for child in node.children() {
        collect(child.clone(), to);
        convert(child, to);
    }
}

impl Tree {
    pub fn new(svg: &str) -> Result<(Self, Size)> {
        let t = usvg::Tree::from_str(svg, &Options::default())?;
        let ts = t.size;
        Ok((Tree::from(t), ts))
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        let t = Transform::from_scale(w / self.width, h / self.height);
        for child in &mut *self.children {
            child.transform(t);
        }
        self.width = w;
        self.height = h;
        log::debug!("changed size to {w}x{h}");
    }
}
