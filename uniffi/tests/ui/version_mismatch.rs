// This should fail with a version mismatch.
// *sob* - someone fixed the grammar on this output!
// normalize-stderr-test: ", which would overflow" -> "which would overflow"
uniffi::assert_compatible_version!("0.0.1"); // An error message would go here.

fn main() {}
