//! blockstrad: translate semantic blocks, never reparse translated labels.
//! Numbered placeholders identify arguments; their textual order is locale-specific.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Dictionary {
    pub texts: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub lists: HashMap<String, String>,
    pub broadcasts: HashMap<String, String>,
    pub sprites: HashMap<String, String>,
    pub costumes: HashMap<String, String>,
    pub backdrops: HashMap<String, String>,
    pub sounds: HashMap<String, String>,
    /// Explicit menu overrides keyed by "BLOCK_ID:1" (one-based slot).
    pub menus: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct Notice {
    kind: String,
    id: String,
    line: Option<u32>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct Menus {
    groups: HashMap<String, Vec<String>>,
    labels: HashMap<String, HashMap<String, String>>,
}

static MENUS: OnceLock<Menus> = OnceLock::new();
fn menus() -> &'static Menus {
    MENUS.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/menus.json")).expect("menus.json")
    })
}

fn placeholder(token: &str) -> Option<usize> {
    // blockst's legacy tokenizer can return "%2s" as one token. The
    // semantic argument is still %2; the attached unit is literal text.
    let digits: String = token
        .strip_prefix('%')?
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<usize>().ok()?.checked_sub(1)
}

fn positions(spec: &str) -> Vec<usize> {
    tokenize_spec(spec)
        .iter()
        .filter_map(|t| placeholder(t))
        .collect()
}

fn message(block: &ParsedBlock, kind: &str, value: String) -> Notice {
    Notice {
        kind: kind.into(),
        id: block.id.clone(),
        line: block.line_number,
        message: value,
    }
}

fn fail_or_keep(
    block: &ParsedBlock,
    request: &ParseRequest,
    notices: &mut Vec<Notice>,
    kind: &str,
    detail: String,
) -> Result<(), String> {
    if request.missing == "error" {
        return Err(format!("blockstrad: {detail} (block {}, block line {:?}). Use missing: \"keep\" only if mixed-language output is acceptable.", block.id, block.line_number));
    }
    notices.push(message(block, kind, detail));
    Ok(())
}

pub(super) fn apply(
    scripts: &mut [Vec<ParsedBlock>],
    request: &ParseRequest,
) -> Result<Vec<Notice>, String> {
    let Some(target) = request.target_language.as_deref() else {
        return Ok(vec![]);
    };
    if !data().languages.contains_key(target) {
        return Err(format!(
            "blockstrad: unknown target language '{target}'. Use blockstrad-languages()."
        ));
    }
    if !matches!(request.missing.as_str(), "error" | "keep") {
        return Err("blockstrad: missing must be 'error' or 'keep'.".into());
    }
    let mut notices = Vec::new();
    for script in scripts {
        for block in script {
            translate(block, target, request, &mut notices, false)?;
        }
    }
    if request.strict_menus {
        if let Some(notice) = notices
            .iter()
            .find(|n| n.kind == "untranslated-menu" || n.kind == "missing-menu-label")
        {
            return Err(format!("blockstrad: {}. Supply dictionary.menus for this slot, or set strict-menus: false and inspect blockstrad-info(...).warnings.", notice.message));
        }
    }
    Ok(notices)
}

fn translate(
    block: &mut ParsedBlock,
    target: &str,
    request: &ParseRequest,
    notices: &mut Vec<Notice>,
    parameter: bool,
) -> Result<(), String> {
    // Keep a procedure's formal parameter names separate from global variables.
    let is_definition = block.id == "procedures_definition";
    for child in &mut block.children {
        if let Child::Input(input) = child {
            if let Some(nested) = &mut input.nested {
                translate(nested, target, request, notices, is_definition)?;
            }
        }
    }
    for child in block.body.iter_mut().chain(block.else_body.iter_mut()) {
        translate(child, target, request, notices, false)?;
    }

    if parameter
        && block.id == "unrecognized"
        && block.children.iter().all(|c| matches!(c, Child::Label(_)))
    {
        block.language = target.into();
        return Ok(());
    }

    // A plain unrecognised reporter is how blockst represents user variables.
    // It is not an unknown command; preserve this role even if renamed 'answer'.
    if block.id == "unrecognized"
        && block.shape == "reporter"
        && block.children.iter().all(|c| matches!(c, Child::Label(_)))
        && block.category == "variables"
    {
        if !parameter {
            let name = block
                .children
                .iter()
                .filter_map(|c| match c {
                    Child::Label(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(translated) = request.dictionary.variables.get(&name) {
                block.children = vec![Child::Label(translated.clone())];
            }
        }
        block.language = target.into();
        return Ok(());
    }
    if block.id == "unrecognized" {
        return fail_or_keep(
            block,
            request,
            notices,
            "unrecognized-source",
            "source instruction is not recognised; no opcode is available for translation".into(),
        );
    }

    // These blocks have user-defined signatures, not a fixed argument template.
    // Do not replace procedure names or parameter labels with built-in text.
    if matches!(
        block.id.as_str(),
        "procedures_definition" | "procedures_call"
    ) {
        let locale = &data().languages[target];
        let keyword = if is_definition {
            &locale.define_spec
        } else {
            &locale.call_spec
        };
        if let Some(Child::Label(label)) = block.children.first_mut() {
            *label = keyword.clone();
        }
        block.language = target.into();
        return Ok(());
    }
    if block.category == "custom-arg" {
        block.language = target.into();
        return Ok(());
    }

    let source = &data().languages[&block.language];
    let destination = &data().languages[target];
    let Some(target_spec) = destination.native_specs.get(&block.id) else {
        return fail_or_keep(
            block,
            request,
            notices,
            "missing-template",
            format!("no blockst template for {} in '{target}'", block.id),
        );
    };
    let source_spec = source.native_specs.get(&block.id).ok_or_else(|| {
        format!(
            "blockstrad: source template missing for {} / {}",
            block.id, block.language
        )
    })?;
    let source_slots = positions(source_spec);
    let target_slots = positions(target_spec);
    let mut source_set = source_slots.clone();
    source_set.sort_unstable();
    let mut target_set = target_slots.clone();
    target_set.sort_unstable();
    if source_set != target_set {
        return Err(format!(
            "blockstrad: incompatible placeholder sets for {} ({} -> {target})",
            block.id, block.language
        ));
    }
    let inputs: Vec<InputNode> = block
        .children
        .iter()
        .filter_map(|child| match child {
            Child::Input(input) => Some(input.clone()),
            _ => None,
        })
        .collect();
    if inputs.len() != source_slots.len() {
        return Err(format!(
            "blockstrad: {} has {} inputs but source template expects {}",
            block.id,
            inputs.len(),
            source_slots.len()
        ));
    }
    let definition = data()
        .commands_by_id
        .get(&block.id)
        .ok_or_else(|| format!("blockstrad: unknown opcode {}", block.id))?;
    let mut by_slot = BTreeMap::new();
    for (slot, mut input) in source_slots.into_iter().zip(inputs) {
        if input.nested.is_none() {
            translate_input(
                &mut input, slot, definition, block, target, request, notices,
            );
        }
        by_slot.insert(slot, input);
    }
    let mut children = Vec::new();
    for token in tokenize_spec(target_spec) {
        if let Some(slot) = placeholder(&token) {
            children.push(Child::Input(
                by_slot
                    .get(&slot)
                    .ok_or_else(|| format!("blockstrad: missing slot {}", slot + 1))?
                    .clone(),
            ));
            let count = token[1..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .count();
            let suffix = &token[1 + count..];
            if !suffix.is_empty() {
                children.push(Child::Label(suffix.into()));
            }
        } else if let Some(icon) = token.strip_prefix('@') {
            children.push(Child::Icon(icon.into()));
        } else {
            children.push(Child::Label(token));
        }
    }
    // Identity, shape, category, labels and bodies are intentionally untouched.
    block.children = children;
    block.language = target.into();
    Ok(())
}

fn numeric(value: &str) -> bool {
    value.trim().parse::<f64>().is_ok()
}

fn menu_key(value: &str) -> String {
    // Numbered music menu names may be written with or without the (1) prefix.
    let mut value = value.trim();
    if value.starts_with('(') {
        if let Some((number, name)) = value[1..].split_once(')') {
            if numeric(number) {
                value = name.trim();
            }
        }
    }
    minify_hash(value).replace(" ^", "^")
}

fn translate_input(
    input: &mut InputNode,
    slot: usize,
    definition: &BlockDef,
    block: &ParsedBlock,
    target: &str,
    request: &ParseRequest,
    notices: &mut Vec<Notice>,
) {
    if input.shape == "color" {
        return;
    }
    let kind = definition
        .inputs
        .get(slot)
        .map(String::as_str)
        .unwrap_or("");
    let dictionary = &request.dictionary;
    let name_map = match kind {
        "%m.var" | "%m.attribute" => Some(&dictionary.variables),
        "%m.list" => Some(&dictionary.lists),
        "%m.broadcast" => Some(&dictionary.broadcasts),
        "%m.location" | "%m.spriteOnly" | "%m.spriteOrMouse" | "%m.spriteOrStage"
        | "%m.touching" => Some(&dictionary.sprites),
        "%m.costume" => Some(&dictionary.costumes),
        "%m.backdrop" => Some(&dictionary.backdrops),
        "%m.sound" => Some(&dictionary.sounds),
        _ => None,
    };
    if let Some(translated) = name_map.and_then(|m| m.get(&input.value)) {
        input.value = translated.clone();
        return;
    }
    if kind == "%s" && !numeric(&input.value) {
        if let Some(translated) = dictionary.texts.get(&input.value) {
            input.value = translated.clone();
        }
        return;
    }
    if !kind.starts_with("%m") && !kind.starts_with("%d") {
        return;
    }
    // These are project identifiers, never language-dependent menu enums.
    if matches!(
        kind,
        "%m.var" | "%m.list" | "%m.broadcast" | "%m.costume" | "%m.sound"
    ) {
        return;
    }
    let field = format!("{}:{}", block.id, slot + 1);
    if let Some(value) = dictionary
        .menus
        .get(&field)
        .and_then(|m| m.get(&input.value))
    {
        input.value = value.clone();
        return;
    }
    if numeric(&input.value) || input.value.is_empty() {
        return;
    }
    let tables = menus();
    let group = tables
        .groups
        .get(&field)
        .or_else(|| tables.groups.get(kind));
    if let Some(keys) = group {
        let wanted = menu_key(&input.value);
        for language in [&block.language[..], &request.language[..], "en", target] {
            let Some(source_values) = tables.labels.get(language) else {
                continue;
            };
            for key in keys {
                if source_values
                    .get(key)
                    .is_some_and(|v| menu_key(v) == wanted)
                {
                    if let Some(value) = tables
                        .labels
                        .get(target)
                        .and_then(|m| m.get(key))
                        .filter(|v| !v.is_empty())
                    {
                        input.value = value.clone();
                    } else {
                        notices.push(message(
                            block,
                            "missing-menu-label",
                            format!(
                                "{key} has no menu label for '{target}'; kept '{}'",
                                input.value
                            ),
                        ));
                    }
                    return;
                }
            }
        }
    }
    // Project object names and literal keyboard/axis/motor symbols are valid.
    if name_map.is_some()
        || input.value.chars().count() == 1
        || kind == "%d.direction"
        || kind == "%d.note"
    {
        return;
    }
    notices.push(message(block, "untranslated-menu", format!("{field}: kept menu value '{}' (no contextual official match); an explicit dictionary.menus override is possible", input.value)));
}

pub(super) fn inspect(request: &ParseRequest) -> Result<String, String> {
    let (scripts, notices) = parse_for_request(request)?;
    let target = request
        .target_language
        .as_deref()
        .unwrap_or(&request.language);
    let nodes: Vec<Vec<PublicNode>> = scripts
        .into_iter()
        .map(|s| s.into_iter().map(to_public_node).collect())
        .collect();
    serde_json::to_string(&serde_json::json!({
        "from": request.language, "to": target, "rtl": request.rtl.unwrap_or_else(|| language_is_rtl(target)),
        "scripts": nodes, "warnings": notices,
    })).map_err(|e| e.to_string())
}

pub(super) fn catalog() -> String {
    let mut languages = Vec::new();
    let ids: BTreeSet<_> = data().commands_by_id.keys().collect();
    for (code, locale) in &data().languages {
        let missing: Vec<_> = ids
            .iter()
            .filter(|id| !locale.native_specs.contains_key(**id))
            .map(|id| (*id).clone())
            .collect();
        languages.push(serde_json::json!({
            "code": code, "rtl": locale.rtl, "templates": ids.len() - missing.len(),
            "missing": missing,
        }));
    }
    languages.sort_by_key(|v| v["code"].as_str().unwrap().to_owned());
    serde_json::json!({
        "languages": languages, "block_definitions": ids.len(),
        "menu_groups": menus().groups.len(),
        "engine": "blockst 52b87da + local colour fixes + blockstrad AST translation",
    })
    .to_string()
}

/// Use official contextual menu values to disambiguate identical sound/looks
/// templates in non-English source locales. Existing parser heuristics remain
/// a fallback for legacy aliases. Never guess from arbitrary translated text.
pub(super) fn effect_candidate(
    ids: &[String],
    children: &[Child],
    language: &str,
) -> Option<String> {
    if !ids.iter().any(|id| id.starts_with("SOUND_"))
        || !ids.iter().any(|id| id.starts_with("LOOKS_"))
    {
        return None;
    }
    let value = menu_key(effect_dropdown_value(children)?);
    for lang in [language, "en"] {
        let labels = menus().labels.get(lang)?;
        for (group, prefix) in [
            ("SOUND_CHANGEEFFECTBY:1", "SOUND_"),
            ("%m.effect", "LOOKS_"),
        ] {
            for key in menus().groups.get(group)? {
                if labels.get(key).is_some_and(|text| menu_key(text) == value) {
                    return ids.iter().find(|id| id.starts_with(prefix)).cloned();
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_template_has_the_complete_numbered_slot_set() {
        let mut checked = 0;
        for locale in data().languages.values() {
            for (id, definition) in &data().commands_by_id {
                if matches!(
                    id.as_str(),
                    "procedures_definition"
                        | "procedures_call"
                        | "argument_reporter_string_number"
                        | "argument_reporter_boolean"
                ) {
                    continue; // variadic user-defined signatures
                }
                let Some(spec) = locale.native_specs.get(id) else {
                    continue;
                };
                let mut actual = positions(spec);
                actual.sort_unstable();
                assert_eq!(
                    actual,
                    (0..definition.inputs.len()).collect::<Vec<_>>(),
                    "{}/{id}",
                    locale.code
                );
                checked += 1;
            }
        }
        assert!(checked > 5000, "insufficient template coverage: {checked}");
    }
}
