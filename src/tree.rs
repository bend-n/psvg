use anyhow::Result;
use tiny_skia_path::{NonZeroPositiveF32, NormalizedF32, Path, Rect, Size, Transform};
use usvg::{BBox, Color, Fill, Image, Opacity, Options, Paint, TreeParsing};

#[derive(Debug)]
pub struct Tree {
    pub width: f32,
    pub height: f32,
    pub children: Box<[Node]>,
    pub osize: Size,
}

#[derive(Debug)]
pub enum PathNode {
    Fill {
        color: Color,
        opacity: Opacity,
        path: Path,
    },
    Stroke {
        color: Color,
        opacity: Opacity,
        stroke_color: Color,
        stroke_opacity: Opacity,
        stroke: NonZeroPositiveF32,
        path: Path,
    },
}

#[derive(Debug)]
pub enum Node {
    Group {
        opacity: Opacity,
        bbox: BBox,
        children: Box<[Node]>,
    },
    Path(PathNode),
    Image(Image),
}

impl Node {
    fn transform(&mut self, t: Transform) {
        match self {
            Self::Path(PathNode::Fill { path, .. } | PathNode::Stroke { path, .. }) => {
                *path = path.clone().transform(t).unwrap(); // idc
            }
            // TODO
            _ => {}
        }
    }
}

impl From<usvg::Tree> for Tree {
    fn from(tree: usvg::Tree) -> Self {
        let mut children = vec![];
        collect(tree.root, &mut children);
        Self {
            osize: tree.size,
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
fn convert(node: usvg::Node, to: &mut Vec<Node>) -> Option<Rect> {
    match &*node.clone().borrow() {
        usvg::NodeKind::Group(g) => {
            let mut children = vec![];
            let bbox = collect(node, &mut children);
            let mut children = children.into_boxed_slice();
            for child in &mut *children {
                child.transform(g.transform);
            }
            to.push(Node::Group {
                opacity: g.opacity,
                bbox,
                children,
            });
            bbox.to_rect()
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
            fill: Some(Fill { opacity, paint, .. }),
            data: path,
            ..
        }) => {
            to.push(Node::Path(PathNode::Stroke {
                color: paint.col(),
                opacity: *opacity,
                stroke: *stroke,
                stroke_opacity: *stroke_opacity,
                stroke_color: stroke_paint.col(),
                path: (**path).clone().transform(*transform).unwrap(),
            }));
            Some(path.bounds())
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
            to.push(Node::Path(PathNode::Stroke {
                color: Color::black(),
                opacity: NormalizedF32::new(0.0).unwrap(),
                stroke: *stroke,
                stroke_opacity: *stroke_opacity,
                stroke_color: stroke_paint.col(),
                path: (**path).clone().transform(*transform).unwrap(),
            }));
            Some(path.bounds())
        }
        usvg::NodeKind::Path(usvg::Path {
            transform,
            stroke: None,
            fill: Some(Fill { opacity, paint, .. }),
            data: path,
            ..
        }) => {
            to.push(Node::Path(PathNode::Fill {
                color: paint.col(),
                opacity: *opacity,
                path: (**path).clone().transform(*transform).unwrap(),
            }));
            Some(path.bounds())
        }
        usvg::NodeKind::Path(usvg::Path {
            transform,
            stroke: None,
            fill: None,
            data: path,
            ..
        }) => {
            to.push(Node::Path(PathNode::Fill {
                color: Color::black(),
                opacity: NormalizedF32::new(0.0).unwrap(),
                path: (**path).clone().transform(*transform).unwrap(),
            }));
            Some(path.bounds())
        }
        usvg::NodeKind::Image(_) => todo!(),
        usvg::NodeKind::Text(_) => unimplemented!(),
    }
}

fn collect(node: usvg::Node, to: &mut Vec<Node>) -> BBox {
    let mut bbox = BBox::default();
    for child in node.children() {
        bbox = bbox.expand(collect(child.clone(), to));
        bbox = bbox.expand(convert(child, to).map_or(BBox::default(), BBox::from));
    }
    bbox
}

impl Tree {
    pub fn new(svg: &str) -> Result<Self> {
        Ok(Tree::from(usvg::Tree::from_str(svg, &Options::default())?))
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        let t = Transform::from_scale(w / self.width, h / self.height);
        for child in &mut *self.children {
            child.transform(t);
        }
        self.width = w;
        self.height = h;
    }
}
