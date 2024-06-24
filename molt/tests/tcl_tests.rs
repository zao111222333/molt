use molt_forked::prelude::*;
#[test]
fn test_tcl_tests() {
    // FIRST, create and initialize the interpreter.
    // Set the recursion limit down from its default, or the interpreter recursion
    // limit test will fail (the Rust stack will overflow).
    type YourCtx = ();
    let mut interp = Interp::new(
        (YourCtx::default(), TestCtx::new()),
        gen_command!(
            (YourCtx, TestCtx),
            // native commands
            [
                // TODO: Requires file access.  Ultimately, might go in an extension crate if
                // the necessary operations aren't available in core::).
                (_SOURCE, cmd_source),
                // TODO: Useful for entire programs written in Molt; but not necessarily wanted in
                // extension scripts).
                (_EXIT, cmd_exit),
                // TODO: Developer Tools
                (_PARSE, cmd_parse),
                (_PDUMP, cmd_pdump),
                (_PCLEAR, cmd_pclear)
            ],
            // embedded commands
            [("test", test_cmd)]
        ),
        true,
    );
    interp.set_recursion_limit(200);

    let args = vec![String::from("tests/all.tcl")];

    assert!(test_harness(&mut interp, &args).is_ok());
}
