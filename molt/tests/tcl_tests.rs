extern crate molt_ng;
use molt_ng::Interp;

#[test]
fn test_tcl_tests() {
    // FIRST, create and initialize the interpreter.
    // Set the recursion limit down from its default, or the interpreter recursion
    // limit test will fail (the Rust stack will overflow).
    let mut interp = Interp::new();
    interp.set_recursion_limit(200);

    let args = vec![String::from("tests/all.tcl")];

    assert!(molt_ng::test_harness(&mut interp, &args).is_ok());
}
