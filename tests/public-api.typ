#import "../lib.typ": blockstrad, blockstrad-info, blockstrad-languages, blockstrad-catalog
#let code = "demander [Quel nombre ?] et attendre\nmettre [réponse v] à (réponse)"
#let data = (texts: ("Quel nombre ?": "أي عدد؟"), variables: ("réponse": "النتيجة"))
#assert.eq(blockstrad-languages().len(), 26)
#assert.eq(blockstrad-info(code, "ar", dictionary: data), blockstrad-info(raw(code), "ar", dictionary: data))
#let info = blockstrad-info(code, "ar", dictionary: data)
#assert.eq(info.warnings, ())
#assert.eq(info.scripts.first().at(1).category, "variables")
#metadata(info) <info>
#set page(width: auto, height: auto, margin: 5mm)
#set text(font: "DejaVu Sans")
#grid(columns: 2, gutter: 5mm,
  blockstrad(raw(code), "ar", dictionary: data),
  blockstrad(code, "ar", dictionary: data, theme: "grayscale", line-numbers: true),
  blockstrad(code, "en", theme: "print"),
  blockstrad(code, "ja", theme: "high-contrast"),
)
