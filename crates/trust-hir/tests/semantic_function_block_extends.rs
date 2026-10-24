//! Function block inheritance: call-site and body-side resolution of
//! inherited members through `EXTENDS` (CODESYS vendor parity).

mod common;

use common::*;

#[test]
fn call_site_accepts_inherited_input_parameter() {
    check_no_errors(
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR_INPUT uiOwn : INT; END_VAR
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(xRst := TRUE, uiOwn := INT#3);
END_PROGRAM
"#,
    );
}

#[test]
fn call_site_accepts_inherited_output_and_in_out_parameters() {
    check_no_errors(
        r#"
FUNCTION_BLOCK BaseFB
VAR_OUTPUT outSeen : DINT; END_VAR
VAR_IN_OUT ioLink : DINT; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; sink : DINT; ioVar : DINT; END_VAR
    fb(outSeen => sink, ioLink := ioVar);
END_PROGRAM
"#,
    );
}

#[test]
fn call_site_accepts_transitive_grandbase_parameters() {
    check_no_errors(
        r#"
FUNCTION_BLOCK RootFB
VAR_INPUT xRoot : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK MiddleFB EXTENDS RootFB
VAR_INPUT xMiddle : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK LeafFB EXTENDS MiddleFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : LeafFB; END_VAR
    fb(xRoot := TRUE, xMiddle := FALSE);
END_PROGRAM
"#,
    );
}

#[test]
fn call_site_inherited_parameters_resolve_cross_file() {
    check_no_errors_multi(&[
        r#"
FUNCTION_BLOCK ChildFB EXTENDS BaseFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(xRst := TRUE);
END_PROGRAM
"#,
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
END_FUNCTION_BLOCK
"#,
    ]);
}

#[test]
fn unknown_parameter_on_derived_block_still_errors() {
    check_has_error(
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(bogus := TRUE);
END_PROGRAM
"#,
        DiagnosticCode::CannotResolve,
    );
}

#[test]
fn redeclared_input_shadows_base_parameter() {
    check_no_errors(
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR_INPUT xRst : INT; END_VAR
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(xRst := INT#5);
END_PROGRAM
"#,
    );
}

#[test]
fn positional_call_binds_own_parameters_before_inherited() {
    check_no_errors(
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xBase : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR_INPUT uiOwn : INT; END_VAR
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(INT#3, TRUE);
END_PROGRAM
"#,
    );

    check_has_error(
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xBase : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR_INPUT uiOwn : INT; END_VAR
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(INT#3);
END_PROGRAM
"#,
        DiagnosticCode::WrongArgumentCount,
    );
}

#[test]
fn inherited_method_call_checks_arguments() {
    check_no_errors(
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
METHOD PUBLIC Reset
VAR_INPUT times : INT; END_VAR
    xRst := FALSE;
END_METHOD
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb.Reset(INT#2);
END_PROGRAM
"#,
    );

    check_has_error(
        r#"
FUNCTION_BLOCK BaseFB
METHOD PUBLIC Reset
VAR_INPUT times : INT; END_VAR
END_METHOD
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb.Reset(TRUE);
END_PROGRAM
"#,
        DiagnosticCode::InvalidArgumentType,
    );
}

#[test]
fn inherited_plain_var_member_usable_in_child_body() {
    check_no_errors(
        r#"
FUNCTION_BLOCK BaseFB
VAR
    stepCount : DINT;
END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
    stepCount := stepCount + INT#1;
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb();
END_PROGRAM
"#,
    );
}

#[test]
fn array_of_derived_blocks_accept_inherited_named_args() {
    check_no_errors(
        r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
VAR_INPUT uiEntityIdx : UINT; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR afb : ARRAY[0..3] OF ChildFB; END_VAR
    afb[0](xRst := TRUE, uiEntityIdx := UINT#1);
END_PROGRAM
"#,
    );
}

#[test]
fn derived_block_implements_interface_and_extends() {
    check_no_errors(
        r#"
INTERFACE ILight
METHOD Apply
END_METHOD
END_INTERFACE

FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB IMPLEMENTS ILight
METHOD PUBLIC Apply
END_METHOD
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(xRst := TRUE);
END_PROGRAM
"#,
    );
}

#[test]
fn unresolvable_base_still_diagnoses_cleanly() {
    let source = r#"
FUNCTION_BLOCK ChildFB EXTENDS MissingBase
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : ChildFB; END_VAR
    fb(xRst := TRUE);
END_PROGRAM
"#;
    let errors = check_errors(source);
    assert!(
        errors.contains(&DiagnosticCode::UndefinedType),
        "expected the unresolvable EXTENDS base diagnostic, got: {:?}",
        errors
    );
    assert!(
        errors.contains(&DiagnosticCode::WrongArgumentCount),
        "expected the call site diagnostic, got: {:?}",
        errors
    );
}

#[test]
fn inherited_member_as_array_index_stays_resolvable() {
    check_no_errors(
        r#"
VAR_GLOBAL
    a : ARRAY[0..3] OF UINT;
END_VAR

FUNCTION_BLOCK BaseFB
VAR_INPUT uiEntityIdx : UINT; END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR
    uiKelvin : UINT := 3000;
END_VAR
    IF (a[uiEntityIdx] >= UINT#1) THEN
        uiKelvin := a[uiEntityIdx];
    END_IF
END_FUNCTION_BLOCK

PROGRAM Main
VAR c : ChildFB; END_VAR
    c();
END_PROGRAM
"#,
    );
}

#[test]
fn self_extending_base_chain_terminates() {
    let source = r#"
FUNCTION_BLOCK LoopFB EXTENDS LoopFB
END_FUNCTION_BLOCK

PROGRAM Main
VAR fb : LoopFB; END_VAR
    fb(xRst := TRUE);
END_PROGRAM
"#;
    let errors = check_errors(source);
    assert!(
        errors.contains(&DiagnosticCode::CyclicDependency),
        "expected the inheritance cycle diagnostic, got: {:?}",
        errors
    );
}
