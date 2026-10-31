use trust_runtime::harness::TestHarness;

#[test]
fn ref_bind_keyword_and_question_mark_spellings_agree() {
    // IEC_DEVIATIONS DEV-034: `REF=` (KwRefBind) lowers as a reference
    // attempt exactly like `?=` (RefAssign), and the right-hand side
    // storage reaches the runtime as a reference — the binding stays
    // live (out reads through r1 after the bind).
    let source = r#"
        PROGRAM Test
        VAR
            x : INT := INT#5;
            r1 : REF_TO INT;
            out : INT := INT#0;
        END_VAR
        r1 REF= x;
        out := r1^;
        END_PROGRAM
    "#;

    let mut harness = TestHarness::from_source(source).unwrap();
    harness.cycle();
    harness.assert_eq("out", 5i16);
}

#[test]
fn ref_assign_question_mark_still_binds_reference_values() {
    // The `?=` spelling keeps its reference-value semantics: the
    // right-hand side is an already-bound reference, not storage.
    let source = r#"
        PROGRAM Test
        VAR
            x : INT := INT#5;
            r1 : REF_TO INT;
            r2 : REF_TO INT;
            out : INT := INT#0;
        END_VAR
        r1 := REF(x);
        r2 ?= r1;
        out := r2^;
        END_PROGRAM
    "#;

    let mut harness = TestHarness::from_source(source).unwrap();
    harness.cycle();
    harness.assert_eq("out", 5i16);
}
