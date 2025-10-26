#[derive(Copy, Clone, Debug)]
pub struct Coord3D<T>
{
	pub x : T,
	pub y : T,
	pub z : T,
}

pub struct Coord2D<T>
{
	pub x : T,
	pub y : T,
}


#[derive(Debug)]
pub struct BoundBox<T>
{
	pub x : T,
	pub y : T,
	pub w : T,
	pub h : T,
}

impl BoundBox<f32>
{
	pub(crate) const IOU : f32 = 0.47;
	pub fn bottom(&self) -> f32
	{
		self.y - self.h / 2.0
	}

	pub fn top(&self) -> f32
	{
		self.y + self.h / 2.0
	}

	pub fn left(&self) -> f32
	{
		self.x - self.w / 2.0
	}

	pub fn right(&self) -> f32
	{
		self.x + self.w / 2.0
	}

	pub fn area(&self) -> f32
	{
		self.w * self.h
	}

	/// Nice property of IOU is that two non-overlapping boxes, is that the intersection is zero.
	///
	/// Where I = 0, I / U = 0
	/// If both boxes have an area of 0, we'll die... I'm just going to assume that case will never happen...
	pub fn area_of_intersection(&self, b : &BoundBox<f32>) -> f32
	{
		let width = self.right().min(b.right()) - self.left().max(b.left());
		let height = self.top().min(b.top()) - self.bottom().max(b.bottom());
		let area = width * height;

		area
	}

	pub fn iou(&self, b : &BoundBox<f32>) -> f32
	{
		let aoi = self.area_of_intersection(b);
		let aou = self.area() + b.area() - aoi;
		aoi / aou
	}
}

impl PartialEq for BoundBox<f32>
{
	fn eq(&self, other: &Self) -> bool {
		self.iou(other) >= Self::IOU
	}
}

pub(crate) struct IndexBoundBox<T>
{
	pub(crate) idx : usize,
	pub(crate) bound_box: BoundBox<T>
}

impl PartialEq for IndexBoundBox<f32>
{
	fn eq(&self, other: &Self) -> bool {
		dbg!(self.bound_box.iou(&other.bound_box)) >= BoundBox::IOU
	}
}