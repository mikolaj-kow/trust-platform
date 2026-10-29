use crate::common::*;

#[test]
fn to_string_accepts_base_less_enum() {
    // CODESYS TO_STRING parity (IEC_DEVIATIONS DEV-025): an enumerated
    // value converts to its member name string; base-less project enums
    // included, since the runtime enum value carries its variant name.
    check_no_errors(
        r#"
TYPE eArea : (areaA, areaB); END_TYPE

PROGRAM Test
VAR
    area : eArea;
    text : STRING;
END_VAR
text := TO_STRING(area);
END_PROGRAM
"#,
    );
}

#[test]
fn to_string_accepts_bool_time_and_string_operands() {
    check_no_errors(
        r#"
PROGRAM Test
VAR
    flag : BOOL;
    elapsed : TIME;
    long : LTIME;
    text : STRING[80];
    out : STRING;
END_VAR
out := TO_STRING(flag);
out := TO_STRING(elapsed);
out := TO_STRING(long);
out := TO_STRING(text);
END_PROGRAM
"#,
    );
}

#[test]
fn vendor_time_conversions_accept_integer_and_dword_targets() {
    check_no_errors(
        r#"
PROGRAM Test
VAR
    elapsed : TIME;
    long : LTIME;
    day : TIME_OF_DAY;
    evening : LTIME_OF_DAY;
    i : DINT;
    u : UDINT;
    li : LINT;
    w : DWORD;
END_VAR
i := TIME_TO_DINT(elapsed);
u := TIME_TO_UDINT(elapsed);
li := LTIME_TO_LINT(long);
w := TOD_TO_DWORD(day);
w := LTOD_TO_DWORD(evening);
END_PROGRAM
"#,
    );
}

#[test]
fn dword_to_tod_conversion_accepts_ms_of_day_magnitude() {
    // CODESYS DWORD_TO_TOD parity (IEC_DEVIATIONS DEV-025): the raw
    // ms-of-day magnitude, the inverse of the TOD_TO_DWORD entry.
    check_no_errors(
        r#"
PROGRAM Test
VAR
    millis : DWORD;
    tea : TIME_OF_DAY;
END_VAR
tea := DWORD_TO_TOD(millis);
END_PROGRAM
"#,
    );
}

#[test]
fn numeric_to_time_conversions_accept_millisecond_magnitude() {
    // CODESYS numeric-to-duration parity (IEC_DEVIATIONS DEV-025): the
    // REAL/UINT value is the millisecond magnitude of the duration.
    check_no_errors(
        r#"
PROGRAM Test
VAR
    scaled : REAL;
    count : UINT;
    relay : TIME;
END_VAR
relay := REAL_TO_TIME(scaled);
relay := UINT_TO_TIME(count);
END_PROGRAM
"#,
    );
}

#[test]
fn string_to_bit_string_conversions_parse_unsigned_literal() {
    // CODESYS parses the string's text as an unsigned integer literal and
    // yields 0 when it is not one (IEC_DEVIATIONS DEV-025);
    // capacity-carrying instances normalize to the string family.
    check_no_errors(
        r#"
PROGRAM Test
VAR
    sValue : STRING[255];
    frame : STRING;
    bPort : BYTE;
    wMask : WORD;
    dwRaw : DWORD;
END_VAR
bPort := TO_BYTE(sValue);
bPort := STRING_TO_BYTE(frame);
wMask := STRING_TO_WORD(frame);
dwRaw := STRING_TO_DWORD(frame);
END_PROGRAM
"#,
    );
}

#[test]
fn to_string_keeps_time_of_day_strict() {
    // Scope boundary: TO_STRING over time-of-day stays rejected until its
    // CODESYS clock format is specified (IEC_DEVIATIONS DEV-025 records the
    // extension set; TOD/date string forms are deliberately not in it).
    check_has_error(
        r#"
PROGRAM Test
VAR
    day : TIME_OF_DAY;
    out : STRING;
END_VAR
out := TO_STRING(day);
END_PROGRAM
"#,
        DiagnosticCode::InvalidArgumentType,
    );
}

#[test]
fn real_to_bit_string_conversion_stays_strict() {
    // REAL_TO_BYTE/REAL_TO_WORD remain #127 territory; the downstream
    // project-side route is the explicit two-step form
    // INT_TO_BYTE(REAL_TO_INT(x)).
    check_has_error(
        r#"
PROGRAM Test
VAR
    scaled : REAL;
    out : BYTE;
END_VAR
out := REAL_TO_BYTE(scaled);
END_PROGRAM
"#,
        DiagnosticCode::InvalidArgumentType,
    );
}

#[test]
fn int_to_time_conversion_stays_strict() {
    // Scope boundary: only the REAL and UINT duration sources the vendor
    // projects use are admitted; other integer-to-TIME pairs keep the
    // strict table reading until one is needed (IEC_DEVIATIONS DEV-025).
    check_has_error(
        r#"
PROGRAM Test
VAR
    count : INT;
    relay : TIME;
END_VAR
relay := INT_TO_TIME(count);
END_PROGRAM
"#,
        DiagnosticCode::InvalidArgumentType,
    );
}

#[test]
fn string_to_bool_conversion_stays_strict() {
    // Scope boundary: the string-parse entry covers bit-string targets
    // only; TO_BOOL would need 'TRUE'/'FALSE' text semantics, which the
    // extension does not specify (IEC_DEVIATIONS DEV-025).
    check_has_error(
        r#"
PROGRAM Test
VAR
    sValue : STRING;
    flag : BOOL;
END_VAR
flag := TO_BOOL(sValue);
END_PROGRAM
"#,
        DiagnosticCode::InvalidArgumentType,
    );
}
