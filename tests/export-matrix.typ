// Test the shipped binary through Typst's actual plugin host.
#let engine = plugin("../libs/blockstrad.wasm")
#let programs = json("fixtures/all-languages.json")
#let catalog = json(engine.locale_catalog(bytes("")))
#let pairs = ()
#for (from, code) in programs.pairs() {
  for to in programs.keys() {
    let payload = (code: code, language: from, target_language: to)
    pairs.push(json(engine.translate_info(bytes(json.encode(payload)))))
  }
}
#let rendered = ()
#for theme in ("normal", "grayscale", "print", "high-contrast") {
  for to in programs.keys() {
    rendered.push((
      to: to, theme: theme,
      svg: str(engine.render_code_json(bytes(json.encode((code: programs.fr, language: "fr", target_language: to, theme: theme))))),
    ))
  }
}
#let user-code = "demander [Quel nombre ?] et attendre\nmettre [réponse v] à (réponse)\ndire (n)"
#let custom = json(engine.translate_info(bytes(json.encode((
  code: user-code, language: "fr", target_language: "ar",
  dictionary: (texts: ("Quel nombre ?": "أي عدد؟"), variables: ("réponse": "النتيجة", "n": "الإجابة")),
)))))
#metadata((catalog: catalog, pairs: pairs, rendered: rendered, custom: custom)) <matrix>
