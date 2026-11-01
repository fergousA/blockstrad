#import "../lib.typ": blockstrad, blockstrad-languages
#let code = "quand @greenFlag est cliqué\nrépéter (2) fois\navancer de (10) pas\ndire [Scratch]\nfin"
#set document(title: "blockstrad — galerie des 26 langues")
#set page(paper: "a4", margin: 13mm, footer: context align(right, text(size: 8pt, fill: rgb("777777"))[#counter(page).display()]))
#set text(font: "DejaVu Sans", size: 10pt, fill: rgb("252525"))
#let names = (
 ar: "Arabe", ca: "Catalan", cs: "Tchèque", cy: "Gallois", de: "Allemand", el: "Grec",
 en: "Anglais", es: "Espagnol", fa: "Persan", fr: "Français", gd: "Gaélique écossais",
 he: "Hébreu", hi: "Hindi", hr: "Croate", hu: "Hongrois", id: "Indonésien", it: "Italien",
 ja: "Japonais", nb: "Norvégien bokmål", nl: "Néerlandais", pl: "Polonais", pt: "Portugais",
 ro: "Roumain", ru: "Russe", sl: "Slovène", tr: "Turc",
)
#let langs = blockstrad-languages()
#for page-index in range(calc.ceil(langs.len() / 6)) {
  if page-index > 0 { pagebreak() }
  text(size: 22pt, weight: "bold")[26 langues, un seul code]
  v(2mm)
  text(size: 9pt, fill: rgb("686868"))[blockstrad 0.1.0 · Source française · Données inchangées, sans dictionnaire]
  v(6mm)
  grid(
    columns: (1fr, 1fr), column-gutter: 9mm, row-gutter: 8mm,
    ..langs.slice(page-index * 6, calc.min((page-index + 1) * 6, langs.len())).map(lang => block[
      #text(size: 10pt, weight: "bold", names.at(lang)) #h(1fr) #text(size: 8pt, fill: rgb("686868"), lang)
      #v(3mm)
      #blockstrad(code, lang, scale: 66%)
    ]),
  )
}
