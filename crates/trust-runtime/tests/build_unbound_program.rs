use trust_runtime::harness::{CompileSession, SourceFile, TestHarness};

fn unbound_program_source() -> &'static str {
    r#"
CONFIGURATION Conf
VAR_GLOBAL
    g_flag : BOOL;
END_VAR
END_CONFIGURATION

PROGRAM Main
VAR
    arr : ARRAY[1..2] OF INT;
END_VAR
arr[1] := INT#10;
END_PROGRAM
"#
}

#[test]
fn configuration_without_program_instance_errors_by_default() {
    let error = match TestHarness::from_source(unbound_program_source()) {
        Ok(_) => panic!("unbound PROGRAM under CONFIGURATION must fail load"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("unbound PROGRAM declaration"),
        "unexpected error: {error}"
    );
    assert!(
        error.to_string().contains("Main"),
        "error should name unbound program: {error}"
    );
}

#[test]
fn explicit_extra_program_instance_keeps_test_builder_opt_in() {
    let source = SourceFile::new(unbound_program_source());
    let runtime = CompileSession::from_sources(vec![source])
        .with_extra_program_instances(["Main"])
        .build_runtime()
        .expect("explicit extra program instance should opt in");

    assert!(runtime.storage().get_global("Main").is_some());
}

#[test]
fn referenced_unbound_program_is_admitted_without_a_binding() {
    // IEC_DEVIATIONS DEV-032: the vendor toolchain compiles a declared
    // PROGRAM that is never bound under a RESOURCE task but is invoked
    // from another POU (the s4 tree's `MqttAutodiscovery()` call from
    // MqttSendQueue). The binding guard exempts programs the analyzer's
    // used-symbol pass did not flag unused.
    let source = SourceFile::new(
        r#"
CONFIGURATION Conf
RESOURCE Res ON PLC
    TASK Scan(INTERVAL := T#10MS, PRIORITY := 1);
    PROGRAM Main WITH Scan : Main;
END_RESOURCE
END_CONFIGURATION

PROGRAM Main
VAR
    runs : INT;
END_VAR
Unbound();
runs := runs + INT#1;
END_PROGRAM

PROGRAM Unbound
VAR
    touched : BOOL;
END_VAR
touched := TRUE;
END_PROGRAM
"#,
    );
    let runtime = CompileSession::from_sources(vec![source])
        .build_runtime()
        .expect("a referenced unbound PROGRAM must compile");

    assert!(runtime.storage().get_global("Main").is_some());
}

#[test]
fn unbound_program_referenced_only_by_comment_stays_rejected() {
    // The admission keys on the analyzer's used-symbol pass, not on text
    // presence: a program mentioned only in comments is still forgotten.
    let error = match TestHarness::from_source(
        r#"
CONFIGURATION Conf
END_CONFIGURATION

PROGRAM Main
VAR
    runs : INT;
END_VAR
runs := runs + INT#1;
END_PROGRAM

PROGRAM Forgotten
VAR
    touched : BOOL;
END_VAR
touched := TRUE;
END_PROGRAM
"#,
    ) {
        Ok(_) => panic!("an unreferenced unbound PROGRAM must still fail load"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("Forgotten"),
        "error should name the forgotten program: {error}"
    );
}
