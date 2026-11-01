use blockstrad_wasm::{
    catalog_json, extract_texts_json, inspect_request_json, parse_request_json, render_request_json,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn inspect(code: &str, from: &str, to: &str, dictionary: Value) -> Value {
    let output = inspect_request_json(
        &json!({
            "code": code, "language": from, "target_language": to, "dictionary": dictionary,
        })
        .to_string(),
    )
    .unwrap_or_else(|e| panic!("{from}->{to}: {e}\n{code}"));
    serde_json::from_str(&output).unwrap()
}
fn nodes(info: &Value) -> &Vec<Value> {
    info["scripts"][0].as_array().unwrap()
}
fn inputs(node: &Value) -> Vec<&Value> {
    node["parts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["kind"] == "input")
        .collect()
}
fn variable_name(node: &Value) -> String {
    node["parts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| {
            if p["kind"] == "label" {
                p["value"].as_str()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_users_french_program_renders_as_canonical_arabic() {
    let programs: Value =
        serde_json::from_str(include_str!("../../tests/fixtures/locale-parity.json")).unwrap();
    let info = inspect(programs["fr"].as_str().unwrap(), "fr", "ar", json!({}));
    assert!(info["warnings"].as_array().unwrap().is_empty());
    let assignment = &nodes(&info)[2];
    assert_eq!(assignment["id"], "DATA_SETVARIABLETO");
    assert_eq!(assignment["category"], "variables");
    assert_eq!(assignment["parts"][2]["value"], "مساويًا");
    assert_eq!(inputs(assignment)[0]["value"], "réponse");
    assert_eq!(inputs(assignment)[1]["nested"]["id"], "SENSING_ANSWER");
    assert_eq!(
        inputs(assignment)[1]["nested"]["parts"][0]["value"],
        "الإجابة"
    );
    assert_eq!(
        inputs(&nodes(&info)[1])[0]["value"],
        "Quel est le nombre de départ ?"
    );
    assert_eq!(info["rtl"], true);
}

#[test]
fn optional_dictionary_translates_data_not_builtins_or_numbers() {
    let info = inspect("demander [Quel nombre ?] et attendre\nmettre [réponse v] à (réponse)\nmettre [n v] à (10)\ndire (n)", "fr", "ar", json!({
        "texts": {"Quel nombre ?":"أي عدد؟", "10":"ten"},
        "variables": {"réponse":"النتيجة", "n":"الإجابة"},
    }));
    let n = nodes(&info);
    assert_eq!(inputs(&n[0])[0]["value"], "أي عدد؟");
    assert_eq!(inputs(&n[1])[0]["value"], "النتيجة");
    assert_eq!(inputs(&n[1])[1]["nested"]["id"], "SENSING_ANSWER");
    assert_eq!(inputs(&n[2])[1]["value"], "10");
    let variable = &inputs(&n[3])[0]["nested"];
    assert_eq!(variable["id"], "unrecognized");
    assert_eq!(variable["category"], "variables");
    assert_eq!(variable_name(variable), "الإجابة"); // must NOT become sensing/answer
}

#[test]
fn user_text_cannot_inject_scratch_syntax_into_the_ast() {
    let info = inspect(
        "dire [bonjour]",
        "fr",
        "en",
        json!({"texts":{"bonjour":"] ::motion\nstop [all v] <&>"}}),
    );
    assert_eq!(nodes(&info).len(), 1);
    assert_eq!(nodes(&info)[0]["id"], "LOOKS_SAY");
    assert_eq!(
        inputs(&nodes(&info)[0])[0]["value"],
        "] ::motion\nstop [all v] <&>"
    );
    let svg = render_request_json(&json!({"code":"dire [bonjour]", "language":"fr", "target_language":"en", "dictionary":{"texts":{"bonjour":"<&>"}}}).to_string()).unwrap();
    assert!(svg.contains("&lt;&amp;&gt;"));
}

#[test]
fn numbered_slots_are_reordered_not_zipped_by_position() {
    let info = inspect(
        "remplacer l'élément (1) de la liste [items v] par [NEW]",
        "fr",
        "ar",
        json!({"lists":{"items":"عناصر"},"texts":{"NEW":"جديد"}}),
    );
    let values: Vec<_> = inputs(&nodes(&info)[0])
        .iter()
        .map(|v| v["value"].as_str().unwrap())
        .collect();
    assert_eq!(values, vec!["جديد", "1", "عناصر"]); // Arabic %3, %1, %2
    let back = inspect(
        "استبدل [جديد] بالعنصر (1) من [عناصر v]",
        "ar",
        "fr",
        json!({"lists":{"عناصر":"items"},"texts":{"جديد":"NEW"}}),
    );
    let values: Vec<_> = inputs(&nodes(&back)[0])
        .iter()
        .map(|v| v["value"].as_str().unwrap())
        .collect();
    assert_eq!(values, vec!["1", "items", "NEW"]);
}

#[test]
fn unknown_commands_are_errors_but_plain_variable_reporters_are_valid() {
    assert!(inspect_request_json(
        &json!({"code":"ceci est inconnu", "language":"fr", "target_language":"ar"}).to_string()
    )
    .is_err());
    let info = inspect("(ma variable)", "fr", "ar", json!({}));
    assert_eq!(nodes(&info)[0]["category"], "variables");
    assert_eq!(variable_name(&nodes(&info)[0]), "ma variable");
}

#[test]
fn explicit_keep_mode_returns_diagnostics_instead_of_guessing() {
    let info: Value = serde_json::from_str(&inspect_request_json(&json!({"code":"ceci est inconnu", "language":"fr", "target_language":"ar", "missing":"keep"}).to_string()).unwrap()).unwrap();
    assert_eq!(info["warnings"][0]["kind"], "unrecognized-source");
    assert_eq!(nodes(&info)[0]["id"], "unrecognized");
}

#[test]
fn missing_target_templates_are_not_silently_english() {
    let info: Value = serde_json::from_str(&catalog_json()).unwrap();
    let ar = info["languages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["code"] == "ar")
        .unwrap();
    assert!(!ar["missing"].as_array().unwrap().is_empty());
    // The face-sensing extension exists in en but not ar.
    let code = "(face size)";
    let parsed: Value = serde_json::from_str(
        &parse_request_json(&json!({"code":code,"language":"en"}).to_string()).unwrap(),
    )
    .unwrap();
    assert_ne!(parsed[0]["id"], "unrecognized");
    assert!(inspect_request_json(
        &json!({"code":code,"language":"en","target_language":"ar"}).to_string()
    )
    .is_err());
}

#[test]
fn official_menu_values_are_contextual_and_user_names_are_not_menus() {
    let menu_data: Value = serde_json::from_str(include_str!("../data/menus.json")).unwrap();
    let info = inspect(
        "mettre [tout v] à (0)\narrêter [tout v]",
        "fr",
        "ar",
        json!({}),
    );
    assert_eq!(inputs(&nodes(&info)[0])[0]["value"], "tout");
    assert_eq!(
        inputs(&nodes(&info)[1])[0]["value"],
        menu_data["labels"]["ar"]["CONTROL_STOP_ALL"]
    );
    let info = inspect(
        "fixer le sens de rotation [gauche-droite v]",
        "fr",
        "ja",
        json!({}),
    );
    assert_eq!(
        inputs(&nodes(&info)[0])[0]["value"],
        menu_data["labels"]["ja"]["MOTION_SETROTATIONSTYLE_LEFTRIGHT"]
    );
}

#[test]
fn menu_overrides_are_scoped_to_a_block_and_slot() {
    let info = inspect(
        "mettre [custom v] à (0)\narrêter [custom v]",
        "fr",
        "ar",
        json!({"menus":{"CONTROL_STOP:1":{"custom":"مخصص"}}}),
    );
    assert_eq!(inputs(&nodes(&info)[0])[0]["value"], "custom");
    assert_eq!(inputs(&nodes(&info)[1])[0]["value"], "مخصص");
}

#[test]
fn unsupported_menus_are_reported_and_preserved() {
    let info = inspect("arrêter [custom v]", "fr", "ar", json!({}));
    assert_eq!(info["warnings"][0]["kind"], "untranslated-menu");
    assert_eq!(inputs(&nodes(&info)[0])[0]["value"], "custom");
}

#[test]
fn nested_control_else_and_line_labels_survive() {
    let code =
        "répéter (2) fois #loop\nsi <(1) < (2)> alors\ndire [yes]\nsinon\ndire [no]\nfin\nfin";
    let info = inspect(code, "fr", "ar", json!({"texts":{"yes":"نعم","no":"لا"}}));
    assert_eq!(nodes(&info)[0]["label"], "loop");
    let conditional = &nodes(&info)[0]["body"][0];
    assert_eq!(conditional["id"], "CONTROL_IF");
    assert_eq!(conditional["else-body"].as_array().unwrap().len(), 1);
    let texts: Value = serde_json::from_str(
        &extract_texts_json(
            &json!({"code":code,"language":"fr","target_language":"ar"}).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(texts.as_array().unwrap().iter().any(|s| s == "وإلا"));
}

#[test]
fn procedures_and_formal_parameter_names_are_not_builtin_templates() {
    let info = inspect(
        "définir carre (n) <ok>\nappeler carre (n) <(1) = (1)>",
        "fr",
        "ar",
        json!({"variables":{"n":"global"}}),
    );
    assert_eq!(nodes(&info)[0]["id"], "procedures_definition");
    assert_eq!(nodes(&info)[0]["parts"][0]["value"], "تعريف");
    assert_eq!(nodes(&info)[0]["parts"][1]["value"], "carre");
    let formal = &inputs(&nodes(&info)[0])[0]["nested"];
    assert_eq!(variable_name(formal), "n");
    assert_eq!(nodes(&info)[1]["parts"][0]["value"], "استدعاء");
    let argument = &inputs(&nodes(&info)[1])[0]["nested"];
    assert_eq!(variable_name(argument), "global");
}

#[test]
fn translated_strings_are_what_the_font_measurement_sees() {
    let tokens: Value = serde_json::from_str(
        &extract_texts_json(
            &json!({
                "code":"dire [hi]", "language":"fr", "target_language":"ar",
                "dictionary":{"texts":{"hi":"مرحبا بكم في هذا المثال الطويل"}},
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(tokens.as_array().unwrap().iter().any(|s| s == "قل"));
    assert!(tokens
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s == "مرحبا بكم في هذا المثال الطويل"));
    assert!(!tokens
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s == "dire" || s == "hi"));
}

#[test]
fn unknown_languages_and_misspelled_dictionary_domains_fail_clearly() {
    for request in [
        json!({"code":"dire [hi]","language":"fr","target_language":"xx"}),
        json!({"code":"dire [hi]","language":"xx","target_language":"ar"}),
        json!({"code":"dire [hi]","language":"fr","target_language":"ar","dictionary":{"variable":{}}}),
    ] {
        assert!(inspect_request_json(&request.to_string()).is_err());
    }
}

// Normalize input order using the locale's %n identifiers, not visual order.
fn specs(lang: &str) -> BTreeMap<String, String> {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("data/locales/{lang}.toml"));
    let data: toml::Value = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    data["specs"]
        .as_table()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect()
}
fn slots(spec: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut chars = spec.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut number = String::new();
            while chars.peek().is_some_and(char::is_ascii_digit) {
                number.push(chars.next().unwrap());
            }
            if !number.is_empty() {
                out.push(number.parse::<usize>().unwrap());
            }
        }
    }
    out
}
fn semantic(node: &Value, specs: &BTreeMap<String, String>) -> Value {
    let id = node["id"].as_str().unwrap();
    let ord = specs.get(id).map(|s| slots(s)).unwrap_or_default();
    let args: BTreeMap<usize, Value> = ord
        .into_iter()
        .zip(inputs(node))
        .map(|(slot, input)| {
            (
                slot,
                if let Some(nested) = input.get("nested") {
                    semantic(nested, specs)
                } else {
                    json!({"value":input["value"], "shape":input["shape"]})
                },
            )
        })
        .collect();
    json!({"id":node["id"],"category":node["category"],"shape":node["shape"],"args":args,
        "name":if id=="unrecognized" {variable_name(node)} else {String::new()},
        "body":node["body"].as_array().unwrap().iter().map(|n|semantic(n,specs)).collect::<Vec<_>>(),
        "else":node["else-body"].as_array().unwrap().iter().map(|n|semantic(n,specs)).collect::<Vec<_>>()})
}

#[test]
fn all_676_language_pairs_preserve_program_semantics() {
    let programs: BTreeMap<String, String> =
        serde_json::from_str(include_str!("../../tests/fixtures/all-languages.json")).unwrap();
    assert_eq!(programs.len(), 26);
    let all_specs: BTreeMap<_, _> = programs.keys().map(|l| (l.clone(), specs(l))).collect();
    let reference = inspect(&programs["en"], "en", "en", json!({}));
    let expected: Vec<_> = nodes(&reference)
        .iter()
        .map(|n| semantic(n, &all_specs["en"]))
        .collect();
    for (from, code) in &programs {
        for to in programs.keys() {
            let info = inspect(code, from, to, json!({}));
            assert!(
                info["warnings"].as_array().unwrap().is_empty(),
                "{from}->{to}: {}",
                info["warnings"]
            );
            let actual: Vec<_> = nodes(&info)
                .iter()
                .map(|n| semantic(n, &all_specs[to]))
                .collect();
            assert_eq!(actual, expected, "{from}->{to}");
        }
    }
}

#[test]
fn all_four_themes_and_rtl_directions_render_without_reclassification() {
    let code = "mettre [réponse v] à (réponse)";
    for (theme, fill) in [
        ("normal", "#FF8C1A"),
        ("grayscale", "#bfbfbf"),
        ("print", "#ffffff"),
        ("high-contrast", "#FFA54C"),
    ] {
        for lang in ["ar", "he", "fa", "en", "ja", "hi"] {
            let svg = render_request_json(
                &json!({"code":code,"language":"fr","target_language":lang,"theme":theme})
                    .to_string(),
            )
            .unwrap();
            assert!(
                svg.split("</defs>")
                    .last()
                    .unwrap()
                    .contains(&format!("fill=\"{fill}\"")),
                "{lang}/{theme}"
            );
            assert_eq!(
                svg.contains("scale(-1 1)"),
                ["ar", "he", "fa"].contains(&lang)
            );
        }
    }
}

#[test]
fn official_sound_effects_disambiguate_source_locales_too() {
    let data: Value = serde_json::from_str(include_str!("../data/menus.json")).unwrap();
    for (lang, labels) in data["labels"].as_object().unwrap() {
        let template = &specs(lang)["SOUND_SETEFFECTO"];
        let value = labels["SOUND_EFFECTS_PITCH"].as_str().unwrap();
        let code = template
            .replace("%1", &format!("[{value} v]"))
            .replace("%2", "(7)");
        let result = inspect(&code, lang, "ar", json!({}));
        assert_eq!(
            nodes(&result)[0]["id"],
            "SOUND_SETEFFECTO",
            "{lang}: {code}"
        );
        assert_eq!(nodes(&result)[0]["category"], "sound", "{lang}");
        assert_eq!(
            inputs(&nodes(&result)[0])[0]["value"],
            data["labels"]["ar"]["SOUND_EFFECTS_PITCH"]
        );
    }
}

#[test]
fn literal_colors_are_not_translated_as_prose() {
    let info = inspect("dire [#f00]", "fr", "ar", json!({"texts":{"#f00":"red"}}));
    assert_eq!(inputs(&nodes(&info)[0])[0]["shape"], "color");
    assert_eq!(inputs(&nodes(&info)[0])[0]["value"], "#f00");
}

#[test]
fn attached_units_do_not_eat_a_numbered_argument() {
    let code = specs("en")["ev3.beepNote"]
        .replace("%1", "(60)")
        .replace("%2", "(2)");
    let info = inspect(&code, "en", "pt", json!({}));
    let parts = inputs(&nodes(&info)[0]);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0]["value"], "60");
    assert_eq!(parts[1]["value"], "2");
    assert!(nodes(&info)[0]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["kind"] == "label" && p["value"] == "s"));
}

#[test]
fn strict_menu_rendering_does_not_hide_incomplete_translation() {
    let payload = json!({"code":"arrêter [custom v]", "language":"fr", "target_language":"ar", "strict_menus":true});
    let error = render_request_json(&payload.to_string()).unwrap_err();
    assert!(error.contains("CONTROL_STOP:1"));
    let override_payload = json!({"code":"arrêter [custom v]", "language":"fr", "target_language":"ar", "strict_menus":true, "dictionary":{"menus":{"CONTROL_STOP:1":{"custom":"مخصص"}}}});
    assert!(render_request_json(&override_payload.to_string()).is_ok());
}
