#![cfg(test)]

#[test]
fn always_true() {
    use template_lib::utils::returns_true;

    assert!(returns_true());
}
