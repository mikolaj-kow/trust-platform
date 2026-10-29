# IEC Deviations Log

Authoritative location:
- This tracked file is the repository source of truth for IEC deviations.
- Do not point tracked docs or code comments at legacy internal IEC deviation-log paths.

This file tracks known, intentional departures from normative IEC 61131-3
requirements. Every entry must cite the exact IEC section/table, state the
normative requirement, state truST's behavior, and explain the concrete
conflict, omission, or relaxation.

IEC-silent, out-of-scope, implementation-specific, platform/runtime, and
truST-only API behavior belongs in the relevant product specification, not in
this log. IEC-permitted interpretation choices belong in
`docs/IEC_DECISIONS.md`. Existing entries are not classification precedent;
re-check them against this rule when touched.

## 2026-10-15 - MOD by zero faults

- ID: DEV-023
- Area: Integer arithmetic (`MOD` function and operator)
- Normative IEC requirement: IEC 61131-3 Ed.3 Table 29 defines `MOD` as the
  result of the ST statements `IF (IN2 = 0) THEN OUT := 0; ELSE OUT := IN1 -
  (IN1/IN2)*IN2; END_IF`, so `MOD` by zero yields `0`.
- truST behavior and conflict:
  - `MOD` by zero raises `RuntimeError::ModuloByZero` (stable code
    `runtime_modulo_by_zero`) and aborts the containing assignment before
    storage, as integer division by zero raises `DivisionByZero`
    (specification 10 §6.4, specification 07 §3.1).
  - For a nonzero divisor truST computes exactly the Table 29 formula.
  - Faulting replaces the normative zero result for a zero divisor.
- Impact:
  - A program that relies on `MOD(x, 0) = 0` faults on truST instead of
    continuing with `0`.
- Mitigation:
  - A silent `0` hides a logic error in a control program, so truST reports it
    as it reports integer division by zero. Portable source that needs the
    Table 29 result guards the divisor: `IF d = 0 THEN r := 0; ELSE r := x MOD
    d; END_IF`.

## 2026-07-30 - Value-bearing RETURN statement

- ID: DEV-022
- Area: Structured Text subprogram control statements
- Normative IEC requirement: IEC 61131-3 Ed.3 section 7.3.3.2.4 and the
  `Subprog_Ctrl_Stmt` production in Annex A define the textual `RETURN`
  statement as the keyword `RETURN` without an expression. Function results
  are assigned through the function-name result variable.
- truST behavior and conflict:
  - truST additionally accepts `RETURN <expression>` in a function or
    value-returning method.
  - The expression is checked against the declared result type, supplies that
    result, and exits the POU immediately.
  - Accepting an expression after `RETURN` extends both the normative grammar
    and the normative function-result assignment model.
- Impact:
  - Source using value-bearing `RETURN` is accepted by truST but is not
    portable to a strictly conforming IEC 61131-3 implementation.
- Mitigation:
  - Portable source should assign the function-name result variable and then
    use bare `RETURN`.
  - truST rejects a value-bearing `RETURN` in a POU without a declared return
    type and type-checks the expression before execution.

## 2026-07-27 - Integer-base exponentiation

- ID: DEV-021
- Area: Structured Text arithmetic expressions
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.6.2.5.8 and Table 29
  define `EXPT`/`**` with `IN1` of generic type `ANY_REAL`, `IN2` of generic
  type `ANY_NUM`, and an output of the same type as `IN1`.
- truST behavior and conflict:
  - truST additionally accepts the reviewed integer expression
    `INT#2 ** INT#3` and evaluates it to `INT#8`.
  - Accepting an integer `IN1` extends the normative input domain beyond
    `ANY_REAL`.
- Impact:
  - Source that uses this reviewed integer-base expression is accepted by
    truST but is not portable to a strictly conforming IEC 61131-3
    implementation.
- Mitigation:
  - Portable source should convert the base to `REAL` or `LREAL` before
    exponentiation.
  - The expression and runtime specifications label that reviewed integer
    form as a truST extension. Other integer operand combinations and
    boundaries require their own specification behavior and focused proof.

## 2026-07-26 - `REF(...)` rejects CONSTANT-qualified variables

- ID: DEV-020
- Area: Structured Text reference operations
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.4.4.10.3 defines
  `REF(...)` for a variable or instance and forbids temporary variables,
  including `VAR_TEMP` and variables inside functions. Section 6.5.4 defines
  CONSTANT-qualified declarations as variables and does not exclude them from
  `REF(...)`.
- truST behavior and conflict:
  - `trust-hir` rejects `REF(c)` whenever `c` is CONSTANT-qualified.
  - This adds a restriction not present in the normative reference-operation
    rule.
- Impact:
  - IEC source may read a constant through a reference, but the equivalent
    source is rejected by truST.
- Mitigation:
  - Pass the constant value directly or copy it into non-CONSTANT storage
    before taking a reference.
  - This deviation closes when the type system can represent a read-only
    reference target without permitting mutation through the reference.

## 2026-07-26 - Non-NULL reference defaults on type and aggregate members

- ID: DEV-019
- Area: Structured Text reference initialization
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.4.4.10.2 permits a
  reference to be initialized with `NULL` or with `REF(...)` naming an already
  declared variable, function-block instance, or class instance.
- truST behavior and conflict:
  - truST accepts `NULL` as a reference type/member default.
  - During user-defined type and aggregate-member default collection,
    `trust-hir` rejects every non-`NULL` reference initializer, including
    `REF(target)`, with `InvalidOperation`.
  - Rejecting the standard-permitted `REF(target)` form omits part of the
    normative reference-initialization model.
- Impact:
  - IEC source that embeds an address-bound reference default in a type or
    aggregate member is rejected by truST.
- Mitigation:
  - Use a `NULL` default and assign the reference after an instance has been
    created.
  - This deviation closes only when HIR retains and validates safe
    address-bound reference defaults without weakening lifetime checks.

## 2026-07-26 - Mixed positional-prefix and formal-suffix calls

- ID: DEV-018
- Area: Structured Text textual call parameter binding
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.6.1.4.2 defines a
  textual parameter list as either:
  - a formal parameter list consisting of `:=` input/in-out assignments and
    `=>` output assignments, whose ordering is insignificant; or
  - a non-formal parameter list containing exactly the parameters of the
    definition in declaration order, excluding `EN` and `ENO`.
- truST behavior and conflict:
  - truST additionally accepts a hybrid call when all positional arguments
    precede all formal assignments, for example `Add(1, b := 2)`.
  - Such a list is neither the IEC formal form nor the IEC non-formal form, so
    accepting it extends the normative textual-call grammar and binding rules.
  - A formal assignment followed by a positional argument remains rejected.
- Impact:
  - Source using this convenience syntax is accepted by truST but is not
    portable to strictly conforming IEC 61131-3 implementations.
- Mitigation:
  - Portable source should use either a wholly formal or wholly non-formal
    parameter list.
  - The semantic specification labels the hybrid form as a truST extension,
    and the compiler still enforces the positional-prefix ordering.

## 2026-04-20 - Runtime STRING / WSTRING element semantics

- ID: DEV-017
- Area: Runtime string indexing, character access, and standard string operations
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.3.3 and Table 10 define
  `STRING` as a single-byte character string and `WSTRING` as a double-byte
  character string using the ISO/IEC 10646 character model.
- truST behavior and conflict:
  - truST stores both `STRING` and `WSTRING` as Rust `String` values.
  - Indexed access and `LEN`, `LEFT`, `RIGHT`, `MID`, `INSERT`, `DELETE`,
    `REPLACE`, and `FIND` operate on Unicode scalar elements rather than raw
    single-byte or UTF-16 code units.
  - Materializing a `STRING` element as `CHAR` still requires the selected
    scalar value to fit in `u8`; materializing a `WSTRING` element as `WCHAR`
    requires it to fit in `u16`.
  - `SIZEOF(STRING[n])` and `SIZEOF(WSTRING[n])` remain storage-oriented (`n`
    and `2n` respectively), while runtime value operations use scalar-element
    counts.
  - This scalar-element model conflicts with the standard's byte/code-unit
    representation for non-ASCII text.
- Impact:
  - Non-ASCII text behaves consistently across VM reference indexing and the
    shipped string library, but it is not raw IEC byte/code-unit indexing.
- Mitigation:
  - The runtime and specifications use one explicit element model end to end.
  - A raw-byte/raw-code-unit rewrite is a separate compatibility change because
    it requires changing the underlying runtime value representation.

## 2026-04-20 - Simplified `VAR_ACCESS` / `VAR_CONFIG` path validation

- ID: DEV-003
- Area: Structured Text access-path and configuration-variable validation
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.5.2.2 and Tables 13-16
  define access paths and configuration-variable targets across configurations,
  resources, program instances, globals, function blocks, and directly
  represented variables, including matching target types.
- truST behavior and conflict:
  - truST validates the accepted access-path shape and target-type compatibility.
  - `trust-hir` does not completely model or statically validate every
    cross-resource and cross-program-instance mapping required by those forms.
  - The missing topology validation is an omission from the normative access
    path model, not an IEC-silent product choice.
- Impact:
  - Full IEC communication-service topology is not statically proven in the
    language layer.
- Mitigation:
  - The supported subset is documented in the variables specification and
    enforced consistently for accepted forms.

## 2026-04-20 - Assignment-attempt compatibility is runtime-oriented

- ID: DEV-006
- Area: `?=` assignment-attempt semantics
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.6.6.7 and Table 52
  define `?=` as a compatibility check for interface implementation and safe
  class/function-block reference downcasts, returning a valid reference only
  for a compatible instance and `NULL` otherwise.
- truST behavior and conflict:
  - truST accepts `?=` for typed reference-style assignment attempts.
  - `trust-hir` does not fully enforce inheritance/interface compatibility for
    every source/target pair during static analysis; some incompatibility is
    deferred to runtime `NULL` behavior.
  - Deferring compatibility checks that can be derived statically is an
    incomplete implementation of the normative assignment-attempt type model.
- Impact:
  - Some compatibility failures are observed at runtime instead of being
    diagnosed statically.
- Mitigation:
  - The operator remains typed, null-producing behavior is documented, and
    callers must test the result before dereference.

## 2026-04-20 - ASCII-only identifier validation

- ID: DEV-013
- Area: Lexical identifier support
- Normative IEC requirement: IEC 61131-3 Ed.3 sections 6.1.1-6.1.2 and Tables
  1-2 define textual characters using ISO/IEC 10646 and identifiers using
  letters, digits, and permitted underscores.
- truST behavior and conflict:
  - truST accepts ASCII letters, digits, and `_` in identifiers and uses ASCII
    case folding.
  - Non-ASCII ISO/IEC 10646 letters are rejected even when they otherwise form
    a valid IEC identifier.
  - Rejecting those standard-defined characters is a supported-character-set
    omission.
- Impact:
  - IEC source using non-ASCII identifiers does not compile in truST.
- Mitigation:
  - The lexer specification and diagnostics state the current ASCII subset
    explicitly.

## 2026-04-11 - Optional `VAR_EXTERNAL` for vendor-parity global access

- Area: Structured Text global-variable access
- Normative IEC requirement: IEC 61131-3 Ed.3 sections 6.5.2.1-6.5.2.2 and
  Figure 8 require a global variable to be redeclared in the consuming POU with
  a matching `VAR_EXTERNAL` declaration before it is accessible there.
- truST behavior and conflict:
  - truST accepts direct global access without a matching `VAR_EXTERNAL`
    declaration.
  - `VAR_EXTERNAL` remains supported and type-checked when authors declare it.
  - This relaxed path applies to configuration/resource globals, file-scope
    vendor GVLs, and qualified namespaced GVL access.
  - Permitting direct access relaxes the normative explicit-linkage requirement.
- Impact:
  - CODESYS/TwinCAT-style source without injected `VAR_EXTERNAL` blocks compiles
    directly in truST.
- Mitigation:
  - Undefined names still diagnose as errors, and strict IEC export/adapter
    flows may synthesize `VAR_EXTERNAL` declarations.

## 2026-04-11 - Missing WHILE/REPEAT termination-guarantee analysis

- Area: Structured Text iteration safety
- Normative IEC requirement: IEC 61131-3 Ed.3 section 7.3.3.4.1 requires an
  error when a `WHILE` or `REPEAT` algorithm cannot guarantee satisfaction of
  its termination condition or execution of an `EXIT` statement.
- truST behavior and conflict:
  - truST parses, type-checks, and executes `WHILE` and `REPEAT`, but
    `trust-hir` does not prove that each loop can terminate.
  - A loop whose termination cannot be guaranteed is therefore not rejected
    solely for that normative error condition.
- Impact:
  - Source can compile even when the standard requires rejection because the
    loop can act as an unbounded wait or execution loop.
- Mitigation:
  - Authors must keep termination conditions locally evident and must not use
    iteration for inter-process synchronization; a future control-flow analysis
    must diagnose the missing guarantee before this deviation can close.

## 2026-02-27 - LD contact/coil symbol subset

- Area: Ladder Diagram symbol set exposed in schema v2/editor tooling
- Normative IEC requirement: IEC 61131-3 Ed.3 Tables 75-76 define the standard
  contact and coil symbol families, including transition-sensing forms.
- truST behavior and conflict:
  - The current schema v2/editor profile implements static contacts (`NO`,
    `NC`) and coil variants (`NORMAL`, `NEGATED`, `SET`, `RESET`).
  - Transition-sensing contact/coil variants from Tables 75-76 are not
    represented in the node schema.
  - The missing standard symbols are an omission from the IEC LD surface.
- Impact:
  - Users cannot model transition-sensing LD symbols directly in the current
    profile.
- Mitigation:
  - Unsupported symbol forms are rejected explicitly rather than silently
    coerced.

## 2026-02-25 - CTUD single-input profile in LD v2 node model

- Area: Ladder Diagram counter node representation
- Normative IEC requirement: IEC 61131-3 Ed.3 section 6.6.3.5.4 and Table 45
  define `CTUD` with separate `CU`, `CD`, `R`, `LD`, and `PV` inputs and `QU`,
  `QD`, and `CV` outputs.
- truST behavior and conflict:
  - The LD schema v2 `counter` node exposes one power input.
  - `CTUD` is executed as CU-driven rising-edge increment; separate `CD`, `QD`,
    `R`, and `LD` wiring is not represented by this node profile.
  - This omits required `CTUD` inputs, outputs, and state transitions from the
    IEC-defined function-block behavior available through this LD surface.
- Impact:
  - Full dual-input `CTUD` semantics are unavailable in the current LD node
    contract.
- Mitigation:
  - The subset is explicit in tests and the LD profile; a future schema version
    can add dedicated `CU`, `CD`, `R`, and `LD` pins plus `QU`/`QD` outputs.

## 2026-10-21 - Declaration sizes from initializer-backed variables

- Area: `STRING`/`WSTRING` declared lengths and array dimension bounds.
- Normative IEC requirement: IEC 61131-3 Ed.3 requires the `STRING` declared
  length to be an integer literal (§6.3.3, Table 10) and array limits to be
  constant expressions (§6.4.4.5.1); a plain (non-`CONSTANT`) variable is
  neither. The standard text was unavailable on the authoring machine; the
  clause numbers mirror the citations already used in
  `docs/specs/02-data-types.md`.
- truST behavior and conflict:
  - Declaration sizes may name a plain variable whose own initializer is a
    literal or constant expression, including qualified names
    (`STRING(GVLMqtt.uiLenMsgState)`) and bound expressions
    (`ARRAY[0..(GVLMqtt.udiLenMsgBuffer - 1)]`). The declared initial value
    fixes the size at compile time.
  - This accepts real CODESYS/e!COCKPIT projects that size buffers from
    ordinary GVL variables, which the strict reading rejects.
- Impact: the size is fixed from the declared initial value; runtime writes
  to the variable never resize anything (CODESYS compiles sizes the same
  way).
- Mitigation: the leniency applies only to the two declaration-size
  contexts. TEMP and interface-direction sections are excluded, chains
  through other mutable variables stay undecidable with the standing
  diagnostics, and every other constant-expression context keeps the strict
  rule.

## 2026-10-25 - Dotted declaration sizes over flattened library containers

- Area: `STRING`/`WSTRING` declared lengths and array dimension bounds whose
  size expression is a dotted constant reference.
- Normative IEC requirement: IEC 61131-3 Ed.3 resolves qualified symbolic
  variables through their declared namespace/containers (§6.2.2.2); a dotted
  reference must name an existing container path. The standard text was
  unavailable on the authoring machine; the clause numbers mirror the
  citations already used in `docs/specs/02-data-types.md`.
- truST behavior and conflict:
  - Library documentation routinely qualifies constants through a GVL level
    (`WagoTypesLog.LogParameter.MAX_LOG_MSG_LEN`) while vendor stubs flatten
    that level into namespace-level constants. In the two declaration-size
    contexts only, a dotted reference that fails strict container resolution
    falls back to a unique-leaf match: the reference's last segment must
    match exactly one constant or initializer-backed declared value
    table-wide. More than one match keeps the strict diagnostic; none keeps
    `undefined constant`.
  - Bare (non-dotted) names never take this tolerance: a constant visible
    only inside a namespace stays unimported for callers without `USING`
    (matching the unimported-namespace no-leak rule).
- Impact: documentation-faithful sizes (128/20/40-class stub constants)
  compile at their declared values instead of failing or being rewritten to
  a numeric fallback downstream.
- Mitigation: the tolerance is confined to the two declaration-size contexts
  (string capacities, array bounds); ordinary initializers and other
  constant-expression contexts keep the strict rule, references whose root
  names a scope-local value (FB instance member access) are excluded, and
  ambiguity is rejected rather than silently bound.
- PLCopen import/export (trust-plcopen): `<string length>` /
  `<wstring maxLength>` attributes and `STRING[…]` export expressions pass
  symbolic lengths through verbatim, mirroring the array-dimension
  lower/upper pass-through; the analyzer owns resolution and validation, so
  numeric-only length filtering no longer fails POUs closed at import.

## 2026-10-26 - Enum values as their base integer, and fully qualified member access

- Area: enumerated values in assignments, comparisons, call arguments, and
  CASE labels; dotted `Namespace.EnumType.Member` references of any depth.
- Normative IEC requirement: IEC 61131-3 Ed.3 treats an enumeration as a
  distinct type whose members are not integer literals (§6.4.4.2); assigning
  an enumerated value to an integer variable of the underlying base type is
  not an implicit conversion the standard defines. Qualified member access
  is spelled `EnumType#Value` (§6.4.4.2); the standard text was unavailable
  on the authoring machine and the clause numbers mirror the citations
  already used in `docs/specs/02-data-types.md`.
- truST behavior and conflict:
  - CODESYS treats an enum as its base type freely: an enumerated value
    converts to its declared integer base in assignments, comparisons, and
    call arguments, and members are reachable through fully qualified dotted
    paths (`WagoTypesLog.eSysLogPriority.LOG_ERR`). truST now follows: any
    enum source converts to its base (base-less enums are `INT`-based), and
    dotted member access accepts any namespace depth. Integer assignment
    INTO a base-less ("closed") enum and cross-enum assignment stay
    rejected, preserving the existing user-type contract.
- Impact: real CODESYS projects (the downstream tree carries ~154 such
  sites: `eSysLogPriority`, `eLedID`, `eArea` classes) stop reporting E202
  "cannot be used as a value" and IncompatibleAssignment for enum-to-base
  flows; the strict reading rejects the same code.
- Mitigation: the leniency is one-directional (enum → base only); integer
  into closed enums, cross-enum assignment, and unknown member references
  keep their diagnostics; generic `ANY` matching still does not treat an
  enum as an integer actual beyond this base conversion.

## 2026-10-28 - In-out string binding across declared capacities

- ID: DEV-024
- Area: `VAR_IN_OUT` parameter binding for `STRING[n]`/`WSTRING[n]` actuals.
- Normative IEC requirement: IEC 61131-3 Ed.3 requires the actual parameter
  of a `VAR_IN_OUT` formal to be a variable of the same data type
  (§6.6.2.2). String capacity is part of a string type, so binding a
  `STRING[255]` actual to a `STRING[1]` formal is not a same-type binding
  and the strict reading rejects it. The standard text was unavailable on
  the authoring machine; the clause number mirrors the citation style
  already used in `docs/specs/02-data-types.md`.
- truST behavior and conflict: truST requires only the same string family
  (`STRING` vs `WSTRING`) and accepts in-out binding across declared
  capacities, matching CODESYS. CODESYS accepts the vendor buffer idiom — a
  `STRING(1)` in-out formal (used by every `WagoAppString` function) whose
  real capacity arrives in a separate size argument
  (`StrAppend(sBuffer, SIZEOF(sBuffer), sAppendee)`) — so the
  capacity-strict reading rejects library calls that compile clean in the
  vendor toolchain.
- Impact: vendor-library call sites with in-out string formals (the
  downstream tree carries several hundred `WagoAppString` in-out and size
  call arguments) now pass the compile gate; the previous
  capacity-equality rule rejected the same code.
- Mitigation: nothing is lost at the boundary — the callee still sees the
  actual variable through the reference and owns the capacity contract
  through its formal capacity or size argument; the caller's variable keeps
  its own storage. Cross-family binding and non-string in-out type
  mismatches keep their diagnostics.
- Open note (2026-10-29): the compile-side contract is capacity-agnostic,
  but the runtime's in-out reference binding still width-checks under the
  `hir`-feature store/binding policy — a cycle that calls through a
  capacity-mismatched in-out binding faults with `TypeMismatch` there
  (the non-hir fallback binds). Making the runtime reference binding
  capacity-agnostic (or documenting the fault as the vendor-parity
  behavior) is a spec 10 owner decision tracked with DEV-026's narrowed
  store note; until then downstream trace cases assert the compile
  contract for capacity-mismatched bindings.

## 2026-10-28 - Vendor conversion extensions: enums, BOOL, durations, strings

- ID: DEV-025
- Area: typed `*_TO_*`/`TO_*` standard conversion functions.
- Normative IEC requirement: IEC 61131-3 Ed.3 Tables 22-27 enumerate the
  typed conversions; enumerated source types appear in no conversion table
  (`§6.4.4.2` treats an enumeration as a distinct type), `BOOL`/`TIME`
  string forms are not listed, duration-to-narrow-integer forms exceed the
  listed `TIME`/`DWORD` pair, `TOD`/`LTOD` to `DWORD` is not listed, and
  string sources to bit-string targets are not listed. The standard text
  was unavailable on the authoring machine; the clause numbers mirror the
  citations already used in `docs/specs/07-standard-functions.md`.
- truST behavior and conflict: truST accepts — matching the CODESYS /
  TwinCAT vendor toolchain — (a) `TO_STRING`/`*_TO_STRING` over an
  enumerated value (yields the variant name), over `BOOL`
  (`'TRUE'`/`'FALSE'`), over `TIME`/`LTIME` (`T#`/`LT#` duration text),
  and over any same-family string (identity); (b) `TIME_TO_<int>` /
  `LTIME_TO_<int>` (ms / ns magnitude truncated into the destination,
  negative durations overflow); (c) `TOD_TO_DWORD` / `LTOD_TO_DWORD`
  (ms-of-day magnitude); (d) `REAL_TO_TIME` / `UINT_TO_TIME` (the value is
  the millisecond magnitude of the duration, truncated; a negative or
  non-finite value, or a magnitude beyond the `DWORD`-ms range, overflows);
  (e) `DWORD_TO_TOD` (the raw ms-of-day magnitude without 24-hour
  validation, mirroring `TOD`'s `DWORD` representation); (f) `STRING` /
  `WSTRING` to `BYTE`/`WORD`/`DWORD`/`LWORD` (the string's text is parsed
  as an unsigned integer literal — radix-prefixed forms included — and
  anything that is not one yields `0`; the magnitude wraps into the
  destination width). The strict table reading rejects vendor-project
  call sites that compile clean in the vendor toolchain (the downstream
  tree carries ~130 such sites: `TO_STRING(eArea)` families, boot-report
  `TIME_TO_DINT` chains, `TOD_TO_DWORD` day-phase math, MQTT payload
  `TO_BYTE(sValue)` fields, and `REAL_TO_TIME`/`UINT_TO_TIME` scaling
  chains).
- Impact: those call sites pass the compile gate; the runtime conversion
  dispatch (`trust-runtime-core`) implements the same entries so executed
  conversions produce the documented values.
- Mitigation: the extension set is deliberately bounded — time-of-day and
  date string forms, REAL-to-bitstring conversions (upstream #127
  territory), other integer-to-`TIME` pairs (only the `REAL`/`UINT`
  duration sources vendor projects use are admitted; `INT_TO_TIME` and
  friends stay rejected), and `STRING`/`WSTRING` to `BOOL` (`'TRUE'`/
  `'FALSE'` text semantics are not specified here) stay rejected; integer
  and bit-string destinations wrap rather than fault on magnitude overflow,
  matching the vendor toolchain's silent narrowing documented in DEV-023's
  neighborhood, and string conversions to numeric destinations keep the
  strict fault behavior they already had.
- Open note (2026-10-29): extension round — (d)-(f) were added after the
  downstream runtime-check campaign enumerated its remaining E205 residue;
  they ride the same compiler accept table, runtime conversion dispatch,
  and spec 07 sections as (a)-(c).

## 2026-10-28 - Integer/bit-string narrowing bindings as warnings

- ID: DEV-026
- Area: implicit integer↔bit-string conversions at assignments and call
  arguments (`E203` "cannot assign", `E205` "expected '<int>' for parameter").
- Normative IEC requirement: IEC 61131-3 Ed.3 defines implicit assignments
  only where value and accuracy are preserved (`§6.6.1.6`); a signed to
  narrower-unsigned store (`SINT` into `BYTE`, `INT` into `BYTE`,
  `UDINT` into `DWORD`) is not an implicit conversion the standard defines.
  The standard text was unavailable on the authoring machine; the clause
  mirrors the citation style already used in `docs/specs/02-data-types.md`.
- truST behavior and conflict: a binding whose target and source are both
  integer/bit-string types is diagnosed as a **warning** with the same
  message and code, not an error — matching CODESYS, which compiles the
  store silently with a compiler warning. Diagnostics outside that type
  class (REAL targets, enums, strings, aggregates) stay errors. `SIZEOF`
  additionally yields its natural unsigned magnitude (`UDINT`) like the
  vendor toolchain, so vendor `udiSize` parameters bind directly.
- Impact: vendor-idiom call sites and configuration wiring tables (the
  downstream tree carries 57 narrowing assignments, 7 port arguments, and
  ~163 `SIZEOF`-into-`UDINT` sites) pass the compile gate while every
  narrowed store remains individually visible.
- Mitigation: the demotion is type-class-scoped, not blanket — mixed
  (non-integer) mismatches keep their errors, and each narrowed binding
  still produces one warning so recurrence is loud.
- Open note (2026-10-29): the narrowed store's runtime write semantics are
  build-configuration dependent — the `hir`-feature runtime store policy
  range-rejects an out-of-range narrowed store (caller variable keeps its
  prior value, per the 2026-07-05 subrange write-enforcement decision),
  while the non-hir fallback wraps. Which semantics is canonical for
  narrowed stores is a spec 10 owner decision; until then downstream pins
  assert only the compile contract.

## 2026-10-28 - Mixed-family comparisons and bit-string arithmetic

- ID: DEV-027
- Area: comparison operators (`= <> < <= > >=`) over mixed integer
  families, and `+ - * /` over bit-string operands.
- Normative IEC requirement: IEC 61131-3 Ed.3 `§6.6.1.6` admits implicit
  operations only where value and accuracy are preserved; signed/unsigned
  pairs have no accuracy-preserving common type in the reviewed matrix,
  and bit-strings (`ANY_BIT` minus `BOOL`) are not `ANY_NUM` operands at
  all, so `DWORD - DWORD` and `INT < UINT` are not defined operations.
  The standard text was unavailable on the authoring machine; the clause
  mirrors the citation style already used in `docs/specs/05-expressions.md`.
- truST behavior and conflict: comparisons over any mix of
  signed/unsigned/bit-string integer operands are admitted and evaluate
  through one mathematical magnitude (i128 promotion at runtime); two
  bit-string operands participate in `+ - * /` as unsigned magnitudes
  whose result carries the wider operand's width and wraps like the
  vendor toolchain (`Div`/`Mod` keep the unsigned zero-divisor faults;
  `Pow` stays rejected). This matches CODESYS, which compiles the
  downstream tree's `UINT` vs division-result comparisons and
  `DWORD - DWORD` energy-meter arithmetic silently. The strict reading
  rejects both families (62 downstream sites).
- Impact: mixed-sign comparisons, integer/bit-string comparisons, and
  bit-string arithmetic pass the compile gate and execute correctly in
  the runtime (the VM promotes instead of faulting); MOD keeps its
  integer-only gate, mixed integer/bit-string *arithmetic* (`INT +
  UINT`, `WORD + INT`), and integer/REAL accuracy-loss combinations keep
  their diagnostics. The compile-time constant evaluator promotes true
  mixed signed×unsigned pairs the same way (exact i128 math, materialized
  LINT/ULINT) so array bounds and initializers like
  `ARRAY[0..SIZEOF(x)-1]` keep working with an unsigned SIZEOF; the VM
  execution path keeps its stricter contract.
- Mitigation: admission is silent (matches the vendor toolchain and the
  existing logical-operator precedent) but type-class-scoped — BOOL and
  non-scalar pairs, REAL accuracy-loss pairs, and mixed-family
  arithmetic keep their errors; the runtime promotion lives in the
  compare/arithmetic paths, leaving the value-helper rejection pins
  intact.

## 2026-10-29 - Pointer-target lenient bindings

- ID: DEV-028
- Area: pointer bindings at call arguments and plain assignments
  (`E205` "pointer target mismatch for parameter ...", `E203` "cannot
  assign") where the two pointer targets differ by a 1-byte scalar class
  (`CHAR` ↔ `BYTE`) or by the vendor byte-buffer idiom (`ADR(STRING[n])`
  against a `POINTER TO ARRAY[i..j] OF BYTE` formal).
- Normative IEC requirement: IEC 61131-3 Ed.3 defines no implicit
  conversion between pointer targets; `§6.6.1.6` admits implicit
  assignments only where value and accuracy are preserved, and strict
  reading requires identical pointee types. The standard text was
  unavailable on the authoring machine; the clause mirrors the citation
  style already used in `docs/specs/09-semantic-rules.md`.
- truST behavior and conflict: those two binding forms are diagnosed as
  a **warning** with the same code and message, not an error — matching
  CODESYS, which compiles them as implicit pointer conversions with a
  compiler warning. `ADR` keeps its pointee typing (`ADR(str[1])` stays
  `POINTER TO CHAR`), so the vendor `MemCopy`/`MemCopySecure`
  `POINTER TO BYTE` formals and the vendor `Fb_JSON_ParseAndModify`
  `pData : POINTER TO ARRAY[0..65535] OF BYTE` formal bind directly.
  Every other pointee mismatch stays an error (wider scalar classes,
  array bound widening, records, other families).
- Impact: the downstream tree's 3 remaining pointer sites (2×
  `MemCopy`/`MemCopySecure` string-offset byte-pointer bindings, 1×
  JSON-parser data binding) pass the compile gate while each binding
  stays individually visible. No runtime mirror is needed: no
  conversion dispatch is involved and the pointer value is copied
  verbatim, so only the compile contract changes.
- Mitigation: the demotion is form-scoped, not blanket — element/record
  boundary mismatches keep their errors, and each lenient binding still
  produces one warning so recurrence is loud. Function/method
  `VAR_OUT`/`VAR_IN_OUT` pointer arms and variable initializers are not
  wired (mirroring DEV-026's scoping) and keep the error until a
  downstream need appears.
