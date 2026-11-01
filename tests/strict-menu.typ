#import "../lib.typ": blockstrad, blockstrad-info
#let code = "arrêter [custom v]"
#assert.eq(blockstrad-info(code, "ar").warnings.len(), 1)
#blockstrad(code, "ar", strict-menus: sys.inputs.at("mode", default: "strict") != "relaxed")
