use super::is_reserved_word;

#[test]
fn when_reserved_word() {
    assert!(is_reserved_word("end"));
}

#[test]
fn when_not_reserved_word() {
    assert!(!is_reserved_word("ruby"));
}
