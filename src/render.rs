use crate::tree::{Node, PathNode, Tree};
use fimg::Image;
use tiny_skia_path::{NormalizedF32, Path, Point};
use usvg::Color;

impl Tree {
    #[must_use]
    pub fn render(&self) -> Image<Box<[u8]>, 4> {
        let mut canvas =
            Image::alloc(self.width.round() as u32, self.height.round() as u32).boxed();
        for node in &*self.children {
            render(node, &mut canvas);
        }
        canvas
    }
}

trait Col {
    fn col(self, opacity: NormalizedF32) -> [u8; 4];
}
impl Col for Color {
    fn col(self, opacity: NormalizedF32) -> [u8; 4] {
        [self.red, self.green, self.blue, opacity.to_u8()]
    }
}

fn point(p: &Path) -> Vec<(i32, i32)> {
    let mut points = Vec::with_capacity(p.len() + 1);
    let r = |p: Point| (p.x.round() as i32, p.y.round() as i32);
    for &point in p.points() {
        points.push(r(point));
    }
    points.push(r(*p.points().first().unwrap()));
    points
}

fn render(node: &Node, img: &mut Image<Box<[u8]>, 4>) {
    match node {
        Node::Path(PathNode::Fill {
            color,
            opacity,
            path,
        }) => {
            img.points(&point(path), color.col(*opacity));
        }
        Node::Path(PathNode::Stroke {
            color,
            opacity,
            path,
            ..
            // TODO: stroek
        }) => img.points(&point(path), color.col(*opacity)),
        Node::Group {
            ..
        } => {
            // for child in &**children {
            //     render(child, img);
            // }
        }
        t => unimplemented!("{t:?}"),
    }
}