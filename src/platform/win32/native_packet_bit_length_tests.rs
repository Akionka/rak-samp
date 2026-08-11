use super::native_bit_length;
use crate::SendError;

#[test]
fn rejects_bit_lengths_that_overflow_native_i32() {
    assert_eq!(native_bit_length(i32::MAX as usize), Ok(i32::MAX));
    assert_eq!(
        native_bit_length(i32::MAX as usize + 1),
        Err(SendError::PayloadTooLarge)
    );
}
