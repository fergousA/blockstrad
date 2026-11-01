// blockstrad 0.1.0 — local prototype, not a published @preview release.
// Parser/renderer derived from blockst; see NOTICE.md and licenses/.
#let _engine = plugin("libs/blockstrad.wasm")
#let _catalog = json(_engine.locale_catalog(bytes("")))

/// List the language codes embedded in this version of the engine.
#let blockstrad-languages() = _catalog.languages.map(lang => lang.code)

/// Coverage of blockst templates, including missing opcode lists.
#let blockstrad-catalog() = _catalog

#let _source(value) = {
  if type(value) == str { return value }
  if type(value) == content and value.func() == raw { return value.text }
  panic("blockstrad: code must be a string or a raw code block")
}

#let dictionary-type = type((:))

#let _base(code, to, from, dictionary, missing) = {
  assert(blockstrad-languages().contains(from), message: "blockstrad: unknown source language " + from)
  assert(blockstrad-languages().contains(to), message: "blockstrad: unknown target language " + to)
  assert(type(dictionary) == dictionary-type, message: "blockstrad: dictionary must be a dictionary")
  (
    code: _source(code), language: from, target_language: to,
    dictionary: dictionary, missing: missing,
  )
}

#let _number(value, name) = {
  if type(value) == ratio { return value / 100% }
  assert(type(value) == int or type(value) == float, message: name + " must be a ratio or number")
  value
}

#let _font(language) = {
  if ("ar", "fa").contains(language) { "Noto Sans Arabic" }
  else if language == "he" { "Noto Sans Hebrew" }
  else if language == "hi" { "Noto Sans Devanagari" }
  else if language == "ja" { "Noto Sans CJK JP" }
  else { "DejaVu Sans" }
}

/// Translate and inspect without layout. User data without a dictionary entry
/// is unchanged. Warnings list menus or explicit fallbacks left untranslated.
#let blockstrad-info(code, to, from: "fr", dictionary: (:), missing: "error") = {
  json(_engine.translate_info(bytes(json.encode(_base(code, to, from, dictionary, missing)))))
}

/// Render Scratch in the target language. Source language defaults to French.
/// The translated AST goes straight to the renderer: no second text parse.
#let blockstrad(
  code, to,
  from: "fr",
  dictionary: (:),
  theme: "normal",
  font: auto,
  scale: 100%,
  inset-scale: 100%,
  line-numbers: false,
  line-number-start: 1,
  line-number-gutter: 24,
  missing: "error",
  strict-menus: true,
) = context {
  assert(("normal", "grayscale", "print", "high-contrast").contains(theme), message: "blockstrad: unknown theme " + theme)
  let family = if font == auto { _font(to) } else { font }
  assert(type(family) == str, message: "blockstrad: font must be one family name")
  let k = _number(scale, "scale")
  let inset = _number(inset-scale, "inset-scale")
  assert(k > 0 and inset > 0, message: "blockstrad: scales must be positive")
  let payload = _base(code, to, from, dictionary, missing)
  payload.insert("theme", theme)
  payload.insert("strict_menus", strict-menus)
  payload.insert("font", family)
  payload.insert("scale", k)
  payload.insert("inset_scale", inset)
  payload.insert("line_numbers", line-numbers)
  payload.insert("line_number_start", line-number-start)
  payload.insert("line_number_gutter", line-number-gutter)

  // Measure the TARGET strings, including translated else-labels and user
  // dictionary values. Measuring the source text clips longer translations.
  let tokens = json(_engine.extract_texts(bytes(json.encode(payload))))
  let widths = (:)
  for token in tokens {
    let dimensions = measure(text(font: family, size: 12pt, weight: 500, token))
    widths.insert(token, dimensions.width / 1pt * 96 / 72)
  }
  payload.insert("widths", widths)
  set text(font: family)
  image(
    _engine.render_code_json(bytes(json.encode(payload))),
    format: "svg",
    alt: "Scratch — " + from + " → " + to,
  )
}
