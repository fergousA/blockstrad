# blockstrad

**Un programme source. Des blocs Scratch dans une autre langue.**

Prototype local **0.1.0**, non publié sur Typst Universe. Fonctionne hors ligne
avec Typst **0.15.0 ou plus récent**. Le WASM est déjà compilé : Rust n’est pas
nécessaire pour utiliser le package.

## Démarrage

Place le dossier `blockstrad/` à côté de ton document :

```typst
#import "blockstrad/lib.typ": blockstrad

#let programme-fr = `
quand @greenFlag est cliqué
demander [Quel est le nombre de départ ?] et attendre
mettre [n v] à (réponse)
répéter (2) fois
  mettre [n v] à ((n) * (3))
  mettre [n v] à ((n) + (4))
fin
dire (n) pendant (2) secondes
`

#blockstrad(programme-fr, "ar")
```

La langue source est `fr` par défaut. `from:` permet d’en choisir une autre :

```typst
#blockstrad("move (10) steps", "ja", from: "en")
```

Les libellés des blocs sont traduits ; les questions, messages et noms saisis
par l’utilisateur restent inchangés tant qu’aucun dictionnaire ne les remplace.

### Avec un dictionnaire facultatif

```typst
#let dico-ar = (
  texts: (
    "Quel est le nombre de départ ?": "ما هو عدد البداية؟",
  ),
  variables: ("n": "ن"),
)

#blockstrad(programme-fr, "ar", dictionary: dico-ar)
#blockstrad(programme-fr, "ar", dictionary: dico-ar, theme: "grayscale")
```

Il n’y a **ni IA de traduction à l’exécution, ni appel réseau caché**. Les
correspondances du dictionnaire sont exactes ; les valeurs non mentionnées
restent inchangées. Les nombres et les couleurs littérales ne sont pas
traduits comme du texte.

## Pourquoi cela évite le défaut français/arabe

```text
Texte source → parseur blockst → identifiants et arguments
                             → modèles de la langue cible
                             → rendu direct de l’arbre traduit
```

Le bloc `DATA_SETVARIABLETO` conserve son identifiant, sa catégorie et sa forme.
Le modèle arabe canonique est sélectionné directement dans la locale blockst.
Le texte traduit **n’est pas réanalysé** pour tenter de retrouver une catégorie.

Les arguments sont associés aux numéros `%1`, `%2`, `%3`, pas à leur position
visuelle : l’arabe emploie par exemple `%3`, puis `%1`, puis `%2` pour remplacer
un élément de liste. Les textes cibles sont mesurés avec la police réelle avant
rendu, y compris les expressions imbriquées et les libellés `sinon`.

## Langues et couverture

Les **26 locales** embarquées sont disponibles comme source et comme cible :

```text
ar ca cs cy de el en es fa fr gd he hi hr hu id it ja nb nl pl pt ro ru sl tr
```

L’arabe, le persan et l’hébreu déclenchent automatiquement la disposition RTL
des blocs. L’alignement de l’image dans la page reste celui de ton document ;
utilise `align(right, ...)` si tu souhaites l’aligner à droite.

La couverture n’est pas artificiellement annoncée à 100 % :

| Catalogue de blocs | Modèles présents / 226 définitions |
| --- | --- |
| Anglais | 224 |
| Français, allemand | 215 chacun |
| Les 23 autres locales, dont l’arabe | 214 chacune |

Les absences concernent surtout la détection des visages, `SENSING_ONLINE`
et quelques pseudo-blocs. Un modèle présent n’est pas une garantie de parfaite
localisation linguistique de chaque extension : les tables blockst peuvent
conserver certains mots anglais. `blockstrad-catalog()` donne les absences
précises par langue.

Les menus utilisent **53 groupes contextuels** et **179 entrées officielles**
supplémentaires : touches, effets, rotation, arrêt, dates, listes, stylo,
instruments et certaines extensions. Ce n’est pas un dictionnaire universel
de tous les menus de toutes les extensions.

## API

### `blockstrad(code, to, ...)`

`code` accepte une chaîne ou un bloc `raw`. Principales options :

| Option | Défaut | Rôle |
| --- | --- | --- |
| `from` | `"fr"` | Langue source explicite, sans détection automatique |
| `dictionary` | `(:)` | Traductions facultatives des données |
| `theme` | `"normal"` | `normal`, `grayscale`, `print`, `high-contrast` |
| `font` | `auto` | Famille choisie selon la langue cible, ou nom explicite |
| `scale` | `100%` | Échelle du dessin ; un nombre représente un facteur |
| `inset-scale` | `100%` | Espacement interne, sans changer la taille du texte |
| `line-numbers` | `false` | Afficher les numéros de blocs |
| `line-number-start` | `1` | Numéro initial |
| `line-number-gutter` | `24` | Gouttière en unités SVG |
| `missing` | `"error"` | Erreur si l’instruction ou le modèle cible manque |
| `strict-menus` | `true` | Erreur si un menu ne peut pas être localisé |

Les options sont propres à blockstrad ; elles n’héritent pas de
`set-blockst`. Pour des valeurs communes, utilise une fonction préconfigurée :

```typst
#let traduit = blockstrad.with(from: "fr", theme: "grayscale", scale: 75%)
#traduit(programme-fr, "ar", dictionary: dico-ar)
```

### Dictionnaires séparés par rôle

- `texts` : questions et chaînes littérales dans les entrées textuelles ;
- `variables`, `lists`, `broadcasts` : noms de variables, listes et messages diffusés ;
- `sprites`, `costumes`, `backdrops`, `sounds` : noms d’objets du projet ;
- `menus` : correspondances avancées par identifiant de bloc et numéro d’entrée.

Exemple de menu non encore couvert, à adapter à ton cas :

```typst
#let dico = (
  menus: (
    "text2speech.setLanguageBlock:1": ("arabe": "العربية"),
  ),
)
```

La clé `:1` désigne la première entrée **sémantique** du bloc, et non sa position
dans le rendu RTL. Les noms des procédures et de leurs paramètres ne sont pas
traduits automatiquement.

### `blockstrad-info(code, to, from: "fr", dictionary: (:), missing: "error")`

Renvoie un dictionnaire : `from`, `to`, `rtl`, `scripts` et `warnings`.
`scripts` contient l’arbre traduit avec identifiants, catégories, arguments,
corps de boucles, branches et labels de lignes. Cette fonction ne bloque pas
sur les avertissements de menus : elle permet précisément de les examiner.

```typst
#import "blockstrad/lib.typ": blockstrad-info
#let info = blockstrad-info(programme-fr, "ar", dictionary: dico-ar)
#repr(info.warnings)
```

Pour un rendu explicitement partiel, utilise `missing: "keep"` et/ou
`strict-menus: false`, puis consulte `warnings`. Un élément conservé dans la
langue source n’est pas présenté comme une traduction réussie.

### `blockstrad-languages()` et `blockstrad-catalog()`

La première donne la liste des codes. La seconde donne la couverture et les
identifiants manquants pour chaque locale, ainsi que le nombre de groupes de menus.

## Attention au sens du programme source

- `(réponse)` est ici le **reporter intégré de réponse à une question**.
  `[réponse v]` est un **nom de variable**. Leur texte peut être identique,
  mais ils n’ont pas le même rôle. Traduire ou renommer l’un ne doit pas
  transformer l’autre. Le prototype conserve cette distinction, même si une
  variable est renommée avec le même libellé qu’un reporter intégré.
- Si tu veux modifier une variable puis relire sa nouvelle valeur, un nom
  distinct comme `n` évite cette ambiguïté. L’exemple de démarrage ci-dessus
  l’utilise. Le fichier `examples/programme-fr.txt` conserve en revanche les
  références `(réponse)` du programme fourni : le traducteur ne corrige pas
  silencieusement l’algorithme.
- Dans les exemples livrés, `fin` a été ajouté **avant `dire`** pour placer
  celui-ci hors de la boucle. Dans la syntaxe blockst, l’indentation seule ne
  ferme pas la boucle. Le package n’insère pas lui-même des `fin` manquants.
- La reconnaissance source reste celle du parseur blockst, enrichie de la
  désambiguïsation des effets apparence/son par les menus officiels. Les phrases
  arbitraires ne sont pas interprétées comme du langage naturel. Les ambiguïtés
  de syntaxe restantes nécessitent une formulation reconnue ou un suffixe de
  catégorie blockst quand celui-ci est approprié.
- Un reporter variable ordinaire comme `(n)` est valide même si blockst utilise
  l’identifiant interne `unrecognized` pour le représenter. Ce cas n’est pas
  confondu avec une instruction inconnue.
- L’import de projets SB3, les moniteurs et l’exécution du programme ne font
  pas partie de cette version. Le résultat demandé et produit est un rendu.

## Polices

Sélection automatique :

| Langue cible | Police |
| --- | --- |
| `ar`, `fa` | Noto Sans Arabic |
| `he` | Noto Sans Hebrew |
| `hi` | Noto Sans Devanagari |
| `ja` | Noto Sans CJK JP |
| autres | DejaVu Sans |

Ces polices doivent être disponibles dans ton environnement Typst, comme pour
tout rendu de ces écritures. Elles sont incorporées aux PDF fournis, mais pas
redistribuées comme fichiers de polices dans le package. `font:` permet de
choisir une autre famille couvrant les caractères nécessaires.

Sur Debian/Ubuntu : `fonts-dejavu-core`, `fonts-noto-core`, `fonts-noto-cjk`.
Tu peux aussi fournir tes polices avec `typst --font-path` lors de la compilation.

## Compiler les exemples

Depuis le dossier `blockstrad/` :

```sh
/usr/local/bin/typst compile --root . examples/fr-ar.typ
/usr/local/bin/typst compile --root . examples/gallery.typ
```

Les exemples PDF sont également fournis. Typst est installé hors du workspace,
dans `/usr/local/bin/typst`, conformément à la demande initiale.

## Développement et tests

```sh
cargo test --locked --manifest-path engine/Cargo.toml
python3 tests/check-matrix.py --typst /usr/local/bin/typst

# Après modification du moteur :
rustup target add wasm32-unknown-unknown
bash scripts/build-engine.sh
python3 tests/check-matrix.py --typst /usr/local/bin/typst
```

Le vérificateur Python utilise seulement la bibliothèque standard de Python
3.11+. Les sources Rust, Cargo.lock, scripts de construction, fixtures et tests
sont inclus. Le moteur livré a été construit avec Rust 1.98.1.

## Relation avec blockst et provenance des traductions

Ce prototype embarque une **copie versionnée du moteur blockst**, avec les
correctifs couleur précédents et la nouvelle couche de traduction d’arbre.
Il ne dépend pas d’un hypothétique `@preview/blockst:0.3.1`, et ne modifie pas
le package blockst installé ailleurs. L’API publique actuelle de blockst
n’expose pas toutes les tables ni le rendu direct d’un arbre traduit ; une
future contribution en amont pourrait permettre une dépendance plus légère.

Les modèles des blocs sont ceux de blockst. Les menus supplémentaires viennent
d’un instantané officiel Scratch figé, sous licence BSD-3-Clause, antérieur au
changement de licence de scratch-l10n. Il n’est pas présenté comme la dernière
version en ligne. Voir [NOTICE.md](NOTICE.md) et `licenses/` pour les versions,
licences et attributions.

**Pas de publication distante effectuée.** La version 0.1.0 désigne ce prototype
local ; sa mise en ligne éventuelle nécessite une revue et des métadonnées de
mainteneur définitives.
