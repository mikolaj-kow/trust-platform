//! Saved artifacts: a derived function block call binds inherited parameters
//! (CODESYS FB inheritance) — named inputs, omitted inputs, positional order,
//! transitive chains and inherited outputs.

use trust_runtime::harness::CompileSession;
use trust_runtime::value::Value;
use trust_runtime::Runtime;

fn saved_runtime(source: &str) -> Runtime {
    let bytes = CompileSession::from_source(source)
        .build_bytecode_module()
        .unwrap()
        .encode()
        .unwrap();
    Runtime::from_bytes(&bytes, Default::default()).expect("saved artifact constructs state")
}

fn program_var(runtime: &Runtime, name: &str) -> Value {
    let Some(Value::Instance(main)) = runtime.storage().get_global("Main") else {
        panic!("missing program root");
    };
    runtime
        .storage()
        .get_instance_var(*main, name)
        .cloned()
        .unwrap_or_else(|| panic!("missing Main.{name}"))
}

#[test]
fn saved_inherited_named_input_binds_on_call() {
    let source = r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xRst : BOOL; END_VAR
END_FUNCTION_BLOCK
FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR_OUTPUT seen : BOOL; END_VAR
seen := xRst;
END_FUNCTION_BLOCK
PROGRAM Main
VAR fb : ChildFB; out : BOOL; END_VAR
fb(xRst := TRUE);
out := fb.seen;
END_PROGRAM
"#;
    let mut runtime = saved_runtime(source);
    runtime.execute_cycle().unwrap();
    assert_eq!(program_var(&runtime, "out"), Value::Bool(true));
}

#[test]
fn saved_omitted_inherited_input_reuses_the_instance_value() {
    let source = r#"
FUNCTION_BLOCK AccumBase
VAR_INPUT amount : INT := INT#1; END_VAR
END_FUNCTION_BLOCK
FUNCTION_BLOCK Accum EXTENDS AccumBase
VAR_OUTPUT total : INT; END_VAR
total := total + amount;
END_FUNCTION_BLOCK
PROGRAM Main
VAR fb : Accum; first : INT; second : INT; END_VAR
fb(amount := INT#3, total => first);
fb(total => second);
END_PROGRAM
"#;
    let mut runtime = saved_runtime(source);
    runtime.execute_cycle().unwrap();
    assert_eq!(program_var(&runtime, "first"), Value::Int(3));
    assert_eq!(program_var(&runtime, "second"), Value::Int(6));
}

#[test]
fn saved_positional_call_binds_own_parameters_before_inherited() {
    let source = r#"
FUNCTION_BLOCK BaseFB
VAR_INPUT xBase : INT; END_VAR
END_FUNCTION_BLOCK
FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR_INPUT yOwn : INT; END_VAR
VAR_OUTPUT diff : INT; END_VAR
diff := yOwn - xBase;
END_FUNCTION_BLOCK
PROGRAM Main
VAR fb : ChildFB; out : INT; END_VAR
fb(INT#10, INT#5, diff => out);
END_PROGRAM
"#;
    let mut runtime = saved_runtime(source);
    runtime.execute_cycle().unwrap();
    assert_eq!(program_var(&runtime, "out"), Value::Int(5));
}

#[test]
fn saved_transitive_grandbase_input_binds_on_call() {
    let source = r#"
FUNCTION_BLOCK GrandFB
VAR_INPUT g : INT; END_VAR
END_FUNCTION_BLOCK
FUNCTION_BLOCK BaseFB EXTENDS GrandFB
VAR_INPUT b : INT; END_VAR
END_FUNCTION_BLOCK
FUNCTION_BLOCK LeafFB EXTENDS BaseFB
VAR_INPUT own : INT; END_VAR
VAR_OUTPUT sum : INT; END_VAR
sum := g + b + own;
END_FUNCTION_BLOCK
PROGRAM Main
VAR fb : LeafFB; out : INT; END_VAR
fb(g := INT#1, b := INT#2, own := INT#3);
out := fb.sum;
END_PROGRAM
"#;
    let mut runtime = saved_runtime(source);
    runtime.execute_cycle().unwrap();
    assert_eq!(program_var(&runtime, "out"), Value::Int(6));
}

#[test]
fn saved_inherited_output_binds_to_caller_target() {
    let source = r#"
FUNCTION_BLOCK BaseFB
VAR_OUTPUT outSeen : INT; END_VAR
END_FUNCTION_BLOCK
FUNCTION_BLOCK ChildFB EXTENDS BaseFB
VAR_INPUT y : INT; END_VAR
outSeen := y + INT#1;
END_FUNCTION_BLOCK
PROGRAM Main
VAR fb : ChildFB; out : INT; END_VAR
fb(y := INT#2, outSeen => out);
END_PROGRAM
"#;
    let mut runtime = saved_runtime(source);
    runtime.execute_cycle().unwrap();
    assert_eq!(program_var(&runtime, "out"), Value::Int(3));
}

#[test]
fn saved_child_body_reads_base_plain_var_member() {
    let source = r#"
FUNCTION_BLOCK CounterBase
VAR count : INT; END_VAR
END_FUNCTION_BLOCK
FUNCTION_BLOCK StepCounter EXTENDS CounterBase
VAR_OUTPUT doubled : INT; END_VAR
count := count + INT#1;
doubled := count * INT#2;
END_FUNCTION_BLOCK
PROGRAM Main
VAR fb : StepCounter; out : INT; END_VAR
fb();
out := fb.doubled;
END_PROGRAM
"#;
    let mut runtime = saved_runtime(source);
    runtime.execute_cycle().unwrap();
    assert_eq!(program_var(&runtime, "out"), Value::Int(2));
    runtime.execute_cycle().unwrap();
    assert_eq!(program_var(&runtime, "out"), Value::Int(4));
}
