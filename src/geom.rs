#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Eq, Ord)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    #[inline]
    pub fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }

    pub fn map<F: Fn(usize, usize) -> (usize, usize)>(self, f: F) -> Point {
        f(self.x, self.y).into()
    }
}

impl From<(usize, usize)> for Point {
    fn from((x, y): (usize, usize)) -> Point {
        Point::new(x, y)
    }
}

impl From<&Point> for Point {
    fn from(pt: &Point) -> Point {
        *pt
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Eq, Ord)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    pub fn new(width: usize, height: usize) -> Size {
        Size { width, height }
    }

    pub fn map<F: Fn(usize, usize) -> (usize, usize)>(self, f: F) -> Size {
        f(self.width, self.height).into()
    }

    /// Returns true when (x, y) is in bounds for the given image
    #[inline]
    pub fn in_bounds(&self, pt: impl Into<Point>) -> bool {
        let pt = pt.into();
        pt.x < self.width && pt.y < self.height
    }
}

impl From<(usize, usize)> for Size {
    fn from((x, y): (usize, usize)) -> Size {
        Size::new(x, y)
    }
}

impl From<&Size> for Size {
    fn from(x: &Size) -> Size {
        *x
    }
}

impl std::ops::Mul<usize> for Size {
    type Output = Size;
    fn mul(self, other: usize) -> Size {
        self.map(|x, y| (x * other, y * other))
    }
}

impl std::ops::Div<usize> for Size {
    type Output = Size;
    fn div(self, other: usize) -> Size {
        self.map(|x, y| (x / other, y / other))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Eq, Ord)]
pub struct Region {
    pub point: Point,
    pub size: Size,
}

impl<X: Into<Point>, Y: Into<Size>> From<(X, Y)> for Region {
    fn from((pt, size): (X, Y)) -> Region {
        Region::new(pt, size)
    }
}

impl From<Region> for Point {
    fn from(region: Region) -> Point {
        region.point
    }
}

impl From<Region> for Size {
    fn from(region: Region) -> Size {
        region.size
    }
}

impl Region {
    pub fn new(pt: impl Into<Point>, size: impl Into<Size>) -> Region {
        let point = pt.into();
        let size = size.into();
        Region { point, size }
    }

    pub fn in_bounds(&self, pt: impl Into<Point>) -> bool {
        let pt = pt.into();
        pt.x >= self.point.x
            && pt.x < self.point.x + self.size.width
            && pt.y >= self.point.y
            && pt.y < self.point.y + self.size.height
    }

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn points(&self) -> (Point, Point) {
        (
            self.point,
            Point::new(
                self.point.x + self.size.width,
                self.point.y + self.size.height,
            ),
        )
    }
}
