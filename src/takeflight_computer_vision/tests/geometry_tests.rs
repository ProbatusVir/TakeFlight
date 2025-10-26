use rstest::rstest;
use takeflight_computer_vision::geometry::BoundBox;

#[rstest]
fn area_of_intersection_test()
{
	let a_lar : f32 = 1.0; // larboard
	let a_btm : f32 = 2.0;
	let a_str : f32 = 4.0; // starboard
	let a_top : f32 = 6.0;

	let b_lar : f32 = 3.0; // larboard
	let b_btm : f32 = 4.0;
	let b_str : f32 = 5.0; // starboard
	let b_top : f32 = 8.0;

	let a = BoundBox::<f32> {
		x: a_lar.midpoint(a_str),
		y: a_btm.midpoint(a_top),
		w: a_str - a_lar,
		h: a_top - a_btm,
	};


	let b = BoundBox::<f32> {
		x: b_lar.midpoint(b_str),
		y: b_btm.midpoint(b_top),
		w: b_str - b_lar,
		h: b_top - b_btm,
	};

	let aoi = a.area_of_intersection(&b);
	assert_eq!(aoi, 2.0)
}


fn area_of_union_test()
{
	let a_lar : f32 = 1.0; // larboard
	let a_btm : f32 = 2.0;
	let a_str : f32 = 4.0; // starboard
	let a_top : f32 = 6.0;

	let b_lar : f32 = 3.0; // larboard
	let b_btm : f32 = 4.0;
	let b_str : f32 = 5.0; // starboard
	let b_top : f32 = 8.0;

	let a = BoundBox::<f32> {
		x: a_lar.midpoint(a_str),
		y: a_btm.midpoint(a_top),
		w: a_str - a_lar,
		h: a_top - a_btm,
	};


	let b = BoundBox::<f32> {
		x: b_lar.midpoint(b_str),
		y: b_btm.midpoint(b_top),
		w: b_str - b_lar,
		h: b_top - b_btm,
	};

}