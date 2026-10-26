/// CODESYS function block inheritance: after every block is lowered, fold the
/// base block's call parameters (VAR_INPUT / VAR_OUTPUT / VAR_IN_OUT members)
/// into the derived block's parameter list so argument binding, positional
/// counts and default initialization see the full inherited interface. Base
/// parameters append after the block's own in nearest-base order; a name
/// declared by the block or a nearer base shadows the base declaration. Class
/// bases contribute no call parameters. Runs once per compilation, after all
/// blocks are registered, so base definitions are available regardless of the
/// file they were declared in.
pub(crate) fn merge_inherited_function_block_parameters(
    function_blocks: &mut indexmap::IndexMap<SmolStr, FunctionBlockDef>,
) {
    let base_links: Vec<(SmolStr, Option<FunctionBlockBase>)> = function_blocks
        .iter()
        .map(|(name, fb)| (name.clone(), fb.base.clone()))
        .collect();
    for (name, base) in base_links {
        let mut current = match base {
            Some(FunctionBlockBase::FunctionBlock(base_name)) => Some(base_name),
            _ => None,
        };
        let Some(_) = &current else {
            continue;
        };
        let mut owned: std::collections::HashSet<SmolStr> = function_blocks[&name]
            .params
            .iter()
            .map(|param| SmolStr::new(param.name.to_ascii_uppercase()))
            .collect();
        owned.insert(SmolStr::new(name.to_ascii_uppercase()));
        let mut visited = std::collections::HashSet::new();
        let mut inherited: Vec<Param> = Vec::new();
        while let Some(base_name) = current.take() {
            let key = SmolStr::new(base_name.to_ascii_uppercase());
            if !visited.insert(key.clone()) {
                break;
            }
            let Some(base_def) = function_blocks.get(&key) else {
                break;
            };
            for param in &base_def.params {
                if owned.insert(SmolStr::new(param.name.to_ascii_uppercase())) {
                    inherited.push(param.clone());
                }
            }
            current = match &base_def.base {
                Some(FunctionBlockBase::FunctionBlock(next)) => Some(next.clone()),
                _ => None,
            };
        }
        if !inherited.is_empty()
            && let Some(fb) = function_blocks.get_mut(&name)
        {
            fb.params.extend(inherited);
        }
    }
}

#[cfg(test)]
mod inherited_params_tests {
    use super::*;
    use trust_hir::symbols::ParamDirection;
    use trust_hir::TypeId;

    fn param(name: &str) -> Param {
        Param {
            name: SmolStr::new(name),
            type_id: TypeId(0),
            direction: ParamDirection::In,
            address: None,
            default: None,
        }
    }

    fn fb(name: &str, base: Option<FunctionBlockBase>, params: Vec<Param>) -> FunctionBlockDef {
        FunctionBlockDef {
            name: SmolStr::new(name),
            base,
            interfaces: Vec::new(),
            params,
            vars: Vec::new(),
            temps: Vec::new(),
            using: Vec::new(),
            methods: Vec::new(),
            body: Vec::new(),
        }
    }

    fn fb_base(name: &str) -> FunctionBlockBase {
        FunctionBlockBase::FunctionBlock(SmolStr::new(name))
    }

    fn param_names(def: &FunctionBlockDef) -> Vec<String> {
        def.params.iter().map(|p| p.name.to_string()).collect()
    }

    #[test]
    fn appends_base_parameters_after_own_in_nearest_base_order() {
        let mut blocks = indexmap::IndexMap::new();
        blocks.insert(
            SmolStr::new("GRAND"),
            fb("Grand", None, vec![param("g"), param("g2")]),
        );
        blocks.insert(
            SmolStr::new("BASE"),
            fb("Base", Some(fb_base("Grand")), vec![param("b")]),
        );
        blocks.insert(
            SmolStr::new("LEAF"),
            fb("Leaf", Some(fb_base("Base")), vec![param("own")]),
        );
        merge_inherited_function_block_parameters(&mut blocks);
        assert_eq!(param_names(&blocks["LEAF"]), ["own", "b", "g", "g2"]);
        assert_eq!(param_names(&blocks["BASE"]), ["b", "g", "g2"]);
        assert_eq!(param_names(&blocks["GRAND"]), ["g", "g2"]);
    }

    #[test]
    fn resolves_bases_declared_in_any_order_and_is_idempotent_on_merged_bases() {
        // Child registered before its base and grandbase.
        let mut blocks = indexmap::IndexMap::new();
        blocks.insert(
            SmolStr::new("LEAF"),
            fb("Leaf", Some(fb_base("Base")), Vec::new()),
        );
        blocks.insert(
            SmolStr::new("GRAND"),
            fb("Grand", None, vec![param("g")]),
        );
        blocks.insert(
            SmolStr::new("BASE"),
            fb("Base", Some(fb_base("Grand")), Vec::new()),
        );
        merge_inherited_function_block_parameters(&mut blocks);
        assert_eq!(param_names(&blocks["LEAF"]), ["g"]);

        // Running the merge again must not duplicate anything.
        merge_inherited_function_block_parameters(&mut blocks);
        assert_eq!(param_names(&blocks["LEAF"]), ["g"]);
    }

    #[test]
    fn derived_declaration_shadows_the_base_parameter() {
        let mut blocks = indexmap::IndexMap::new();
        blocks.insert(
            SmolStr::new("BASE"),
            fb("Base", None, vec![param("x"), param("b")]),
        );
        blocks.insert(
            SmolStr::new("LEAF"),
            fb("Leaf", Some(fb_base("Base")), vec![param("X")]),
        );
        merge_inherited_function_block_parameters(&mut blocks);
        assert_eq!(param_names(&blocks["LEAF"]), ["X", "b"]);
    }

    #[test]
    fn class_bases_and_unknown_bases_contribute_nothing_and_cycles_terminate() {
        let mut blocks = indexmap::IndexMap::new();
        blocks.insert(
            SmolStr::new("FROMCLASS"),
            fb(
                "FromClass",
                Some(FunctionBlockBase::Class(SmolStr::new("C"))),
                vec![param("own")],
            ),
        );
        blocks.insert(
            SmolStr::new("ORPHAN"),
            fb(
                "Orphan",
                Some(fb_base("Missing")),
                vec![param("own")],
            ),
        );
        blocks.insert(
            SmolStr::new("LOOP"),
            fb("Loop", Some(fb_base("Loop")), Vec::new()),
        );
        merge_inherited_function_block_parameters(&mut blocks);
        assert_eq!(param_names(&blocks["FROMCLASS"]), ["own"]);
        assert_eq!(param_names(&blocks["ORPHAN"]), ["own"]);
        assert!(blocks["LOOP"].params.is_empty());
    }
}
