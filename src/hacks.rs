/// Vec<T> -> Vec<U>
///
/// This requires three things:
///
/// * The input must be have 0 len (any capacity)
///
/// * T and U must have the same memory alignment
///
/// * The number of bytes in the vector must be cleanly
/// divisible by the bytes in U
///
/// This inherently must consume the vector
/// to maintain safety.
///
/// Use this hack at your peril, make sure it works in debug, because there are no checks in release.
pub(crate) fn vec_to_vec<T, U>(mut input : Vec<T>) -> Vec<U>
{
	debug_assert!(input.is_empty());
	let available_bytes = input.capacity() * size_of::<T>();
	debug_assert_eq!(available_bytes % size_of::<U>(), 0);
	let vec_ptr = input.as_mut_ptr();
	debug_assert_eq!((vec_ptr as usize) % align_of::<U>(), 0);
	let new_length = available_bytes / size_of::<U>();
	
	std::mem::forget(input);

	unsafe { Vec::from_raw_parts(vec_ptr as *mut U, 0, new_length) }
}