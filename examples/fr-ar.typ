#import "../lib.typ": blockstrad
#let programme-fr = raw(read("programme-fr.txt"))
#let dico-ar = (
  texts: ("Quel est le nombre de départ ?": "ما هو عدد البداية؟"),
  variables: ("réponse": "النتيجة"),
)
#set document(title: "blockstrad — un programme, deux langues")
#set page(paper: "a4", margin: (x: 9mm, top: 9mm, bottom: 9mm))
#set text(font: "DejaVu Sans", lang: "fr", size: 9pt, fill: rgb("242424"))
#set par(spacing: 0pt)
#grid(columns: (1fr, auto), align: horizon,
  text(size: 21pt, weight: "bold")[blockstrad],
  text(size: 8pt, fill: rgb("686868"))[0.1.0 · Prototype local],
)
#v(2mm)
#text(size: 10pt)[Un seul programme en français. Un rendu Scratch en arabe.]
#v(2mm)
#line(length: 100%, stroke: 0.6pt + rgb("242424"))
#v(2mm)
#grid(columns: (1fr, 1fr),
  text(weight: "bold")[Français · source],
  align(right, text(font: "Noto Sans Arabic", dir: rtl, weight: "bold")[العربية · ترجمة]),
)
#let comparison(theme, label) = {
  v(3mm)
  text(size: 8pt, weight: "bold", label)
  v(1mm)
  grid(columns: (1fr, 1fr), column-gutter: 5mm, align: top,
    blockstrad(programme-fr, "fr", theme: theme, scale: 65%),
    align(right, blockstrad(programme-fr, "ar", dictionary: dico-ar, theme: theme, scale: 65%)),
  )
}
#comparison("normal", "01  Couleur")
#comparison("grayscale", "02  Niveaux de gris")
#comparison("print", "03  Impression")
#v(3mm)
#line(length: 100%, stroke: 0.4pt + rgb("dedede"))
#v(1mm)
#text(size: 7pt, fill: rgb("686868"))[
  Libellés officiels des blocs · Question et nom de variable traduits par un dictionnaire explicite.
]
