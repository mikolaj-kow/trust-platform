use trust_runtime::harness::TestHarness;
use trust_runtime::value::Value;

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

#[test]
#[ignore = "layer-9 remainder: the binding compiles and validates, but the runtime REFERENCE_ATTEMPT stores NULL - the segmented ProgramRoot-rooted reference needs a type-system ruling (pointee vs REF_TO typing + a pointee-vs-REF_TO arm in runtime_type_is_compatible); see s4/tmp/HANDOFF_VERIFICATION_PR117_2026-10-30.md"]
fn ref_bind_binds_live_across_programs_through_gvl_array_element() {
    // The s4 shape: each entity's REF member binds to another program's
    // FB member once at config time; reads through the reference stay
    // live afterwards (the FB member mutates every scan).
    let source = r#"
        TYPE SomeEvent : STRUCT x : BOOL; END_STRUCT; END_TYPE
        TYPE Entity : STRUCT pEvent : REF_TO SomeEvent; END_STRUCT; END_TYPE
        VAR_GLOBAL
            astEntities : ARRAY[0..1] OF Entity;
        END_VAR
        PROGRAM InputRuntime
        VAR
            fbContact : SomeEvent;
        END_VAR
            fbContact.x := TRUE;
        END_PROGRAM
        PROGRAM BinSenConfig
        VAR
            wired : BOOL;
        END_VAR
            astEntities[0].pEvent REF= InputRuntime.fbContact;
            wired := TRUE;
        END_PROGRAM
        PROGRAM Reader
        VAR
            seen : BOOL;
        END_VAR
            seen := astEntities[0].pEvent^.x;
        END_PROGRAM
    "#;

    let mut harness = TestHarness::from_source(source).unwrap();
    for _ in 0..3 {
        harness.cycle();
    }
    harness.assert_eq("seen", true);
    let entities = harness
        .get_output("astEntities")
        .expect("missing astEntities");
    assert!(matches!(entities, Value::Array(_)));
}
