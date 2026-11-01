//! Regression for scratch-example.pdf: translated blocks must share opcodes
//! and paints, not merely avoid one particular fallback shade.
//! These tests deliberately use the worksheet's Arabic wording, not only the
//! canonical locale text. No category suffix is used to hide a parse failure.

use blockstrad_wasm::{parse_request_json, render_request_json};
use serde_json::{json, Value};

const PROGRAMS: &str = include_str!("../../tests/fixtures/locale-parity.json");
const CATEGORIES: &str = include_str!("../../tests/fixtures/category-parity.json");

fn parse(code: &str, language: &str) -> Value {
    let output = parse_request_json(&json!({"code": code, "language": language}).to_string())
        .expect("parse failed");
    serde_json::from_str(&output).expect("invalid AST JSON")
}

fn render(code: &str, language: &str, theme: &str) -> String {
    render_request_json(
        &json!({"code": code, "language": language, "theme": theme, "line_numbers": true})
            .to_string(),
    )
    .expect("render failed")
}

// Drop translated labels and geometry, retain semantic identity and all
// nested blocks (including sensing reporters inside arithmetic operators).
fn signature(nodes: &Value) -> Value {
    Value::Array(
        nodes
            .as_array()
            .unwrap()
            .iter()
            .map(|node| {
                let nested: Vec<Value> = node["parts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|part| part.get("nested").cloned())
                    .collect();
                json!({
                    "id": node["id"], "category": node["category"], "shape": node["shape"],
                    "nested": signature(&Value::Array(nested)),
                    "body": signature(&node["body"]),
                    "else-body": signature(&node["else-body"]),
                })
            })
            .collect(),
    )
}

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let needle = format!("{name}=\"");
    tag.find(&needle)
        .map(|at| tag[at + needle.len()..].split('"').next().unwrap())
        .unwrap_or("")
}

// Only painted outlines / input boxes, not the language-dependent count of
// text tokens. Include strokes, multiplicity and nested reporter shapes.
fn shape_paints(svg: &str) -> Vec<(String, String, String)> {
    let body = svg.split("</defs>").last().unwrap();
    let mut paints: Vec<_> = body
        .split('<')
        .filter(|tag| tag.starts_with("path ") || tag.starts_with("rect "))
        .map(|tag| {
            let tag = tag.split('>').next().unwrap();
            (
                tag.split_whitespace().next().unwrap().to_string(),
                attr(tag, "fill").to_ascii_lowercase(),
                attr(tag, "stroke").to_ascii_lowercase(),
            )
        })
        .collect();
    assert!(!paints.is_empty(), "no painted shapes in SVG");
    paints.sort();
    paints
}

fn first_block_fill(svg: &str) -> &str {
    let body = svg.split("</defs>").last().unwrap();
    let tag = body
        .split('<')
        .find(|tag| tag.starts_with("path "))
        .unwrap();
    attr(tag.split('>').next().unwrap(), "fill")
}

#[test]
fn worksheet_has_identical_french_arabic_semantics() {
    let programs: Value = serde_json::from_str(PROGRAMS).unwrap();
    let fr = parse(programs["fr"].as_str().unwrap(), "fr");
    let ar = parse(programs["ar"].as_str().unwrap(), "ar");
    assert_eq!(signature(&ar), signature(&fr));
    assert_eq!(ar[2]["id"], "DATA_SETVARIABLETO");
    assert_eq!(ar[2]["category"], "variables");
    assert_eq!(ar[3]["body"].as_array().unwrap().len(), 3);
    assert_eq!(ar[2]["parts"][3]["nested"]["id"], "SENSING_ANSWER");
}

#[test]
fn arabic_assignment_aliases_preserve_inputs_and_opcode() {
    for wording in [
        "اجعل {name} مساوية لـ (7)",
        "اجعل {name} مساوية ل (7)",
        "اجعل {name} مساويةً لـ (7)",
        "اجعل {name} مساوية (7)",
        "اجعل {name} مساويًا لـ (7)",
        "اجعل {name} مساوياً ل (7)",
        "اجعل {name} مساويًا (7)",
        "اجعل {name} مساوياً (7)",
        "اجـعل {name} مُساوِيَةً لـ (7)",
    ] {
        // Diacritics and tatweel are folded for lookup, never in user data.
        let name = "قيمةَ الطـالب";
        let code = wording.replace("{name}", &format!("[{name} v]"));
        let nodes = parse(&code, "ar");
        assert_eq!(nodes[0]["id"], "DATA_SETVARIABLETO", "{code}");
        assert_eq!(nodes[0]["category"], "variables", "{code}");
        let inputs: Vec<_> = nodes[0]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|part| part["kind"] == "input")
            .collect();
        assert_eq!(inputs.len(), 2, "{code}");
        assert_eq!(inputs[0]["shape"], "dropdown");
        assert_eq!(inputs[0]["value"], name);
        assert_eq!(inputs[1]["value"], "7");
    }
}

#[test]
fn unknown_arabic_text_is_not_guessed_to_be_an_assignment() {
    let nodes = parse("اجعل [س v] نصا غير معروف (7)", "ar");
    assert_eq!(nodes[0]["id"], "unrecognized");
    assert_eq!(nodes[0]["category"], "obsolete");
}

fn assert_theme_parity(theme: &str, variable_fill: &str) {
    let programs: Value = serde_json::from_str(PROGRAMS).unwrap();
    let fr = render(programs["fr"].as_str().unwrap(), "fr", theme);
    let ar = render(programs["ar"].as_str().unwrap(), "ar", theme);
    assert_eq!(shape_paints(&ar), shape_paints(&fr), "worksheet / {theme}");

    let cases: Value = serde_json::from_str(CATEGORIES).unwrap();
    for case in cases.as_array().unwrap() {
        let mut paints = Vec::new();
        for language in ["fr", "ar"] {
            let code = case[language].as_str().unwrap();
            let ast = parse(code, language);
            assert_eq!(ast[0]["id"], case["id"], "{theme} / {language} / {code}");
            assert_eq!(ast[0]["category"], case["category"], "{language} / {code}");
            let svg = render(code, language, theme);
            if case["category"] == "variables" {
                assert_eq!(
                    first_block_fill(&svg),
                    variable_fill,
                    "{theme} / {language}"
                );
            }
            paints.push(shape_paints(&svg));
        }
        assert_eq!(paints[0], paints[1], "{theme} / {}", case["category"]);
    }
}

#[test]
fn normal_french_arabic_paints_match() {
    assert_theme_parity("normal", "#FF8C1A");
}

#[test]
fn grayscale_french_arabic_paints_match() {
    assert_theme_parity("grayscale", "#bfbfbf");
}

#[test]
fn print_french_arabic_paints_match() {
    assert_theme_parity("print", "#ffffff");
}

#[test]
fn high_contrast_french_arabic_paints_match() {
    assert_theme_parity("high-contrast", "#FFA54C");
}

#[test]
fn forcing_rtl_only_changes_geometry_not_paints() {
    let programs: Value = serde_json::from_str(PROGRAMS).unwrap();
    for theme in ["normal", "grayscale", "print", "high-contrast"] {
        let mut paints = Vec::new();
        for rtl in [false, true] {
            let svg = render_request_json(
                &json!({"code": programs["fr"], "language": "fr", "theme": theme, "rtl": rtl})
                    .to_string(),
            )
            .unwrap();
            paints.push(shape_paints(&svg));
        }
        assert_eq!(paints[0], paints[1], "{theme}");
    }
}

fn assert_achromatic_hex_paints(svg: &str) {
    // Check CSS and definitions as well as visible shapes: an unused coloured
    // icon must not silently reintroduce colour when a script later uses it.
    let mut count = 0;
    for rest in svg.split('#').skip(1) {
        let hex: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        let channels: Vec<u8> = match hex.len() {
            3 => hex
                .chars()
                .map(|c| c.to_digit(16).unwrap() as u8 * 17)
                .collect(),
            6 => (0..6)
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect(),
            _ => continue, // e.g. href="#sb-greenFlag", not a colour
        };
        assert!(
            channels[0] == channels[1] && channels[1] == channels[2],
            "coloured paint #{hex}"
        );
        count += 1;
    }
    assert!(count > 10, "no useful paint coverage");
}

#[test]
fn grayscale_has_no_coloured_text_icons_or_swatches() {
    let programs: Value = serde_json::from_str(PROGRAMS).unwrap();
    for language in ["fr", "ar"] {
        assert_achromatic_hex_paints(&render(
            programs[language].as_str().unwrap(),
            language,
            "grayscale",
        ));
    }
    for code in [
        "when green flag clicked\nturn @turnRight (15) degrees\nrepeat (2)\npen down\nend",
        "set pen color to [#ff0000]\nset pen color to [#00f]",
        "define sample [parameter]",
        "if <(1) < (2)> then\nsay [hello]\nelse\nsay [bye]\nend",
    ] {
        assert_achromatic_hex_paints(&render(code, "en", "grayscale"));
    }
}

#[test]
fn grayscale_short_and_long_hex_swatches_are_equivalent() {
    for (short, long) in [
        ("#f00", "#ff0000"),
        ("#0f0", "#00ff00"),
        ("#00f", "#0000ff"),
        ("#aB3", "#aaBB33"),
        ("#000", "#000000"),
        ("#fff", "#ffffff"),
    ] {
        let short_svg = render(&format!("set pen color to [{short}]"), "en", "grayscale");
        let long_svg = render(&format!("set pen color to [{long}]"), "en", "grayscale");
        assert_eq!(
            shape_paints(&short_svg),
            shape_paints(&long_svg),
            "{short} / {long}"
        );
    }
}

#[test]
fn custom_parameter_ink_follows_its_theme() {
    for (theme, expected) in [
        ("normal", "fill:#ffffff"),
        ("high-contrast", "fill:#000000"),
        ("print", "fill:#000000"),
        ("grayscale", "fill:#000000"),
    ] {
        let svg = render("define sample [parameter]", "en", theme);
        let at = svg
            .find(">parameter</text>")
            .expect("parameter was not rendered");
        let start = svg[..at].rfind("<text ").unwrap();
        assert!(
            svg[start..at].contains(expected),
            "{theme}: {}",
            &svg[start..at]
        );
    }
}

#[test]
fn normal_theme_keeps_its_original_coloured_accents() {
    let svg = render("set pen color to [#f00]", "en", "normal");
    for paint in ["#575e75", "#6b7280", "#4C97FF", "#f00"] {
        assert!(svg.contains(paint), "normal theme lost {paint}");
    }
}
