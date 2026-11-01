#!/usr/bin/env python3
"""Build the 26 canonical source programs from the pinned blockst templates."""
import json
from pathlib import Path
import re
import tomllib
ROOT = Path(__file__).resolve().parents[1]
programs = {}
def make(specs, identifier, *args):
    return re.sub(r"%(\d+)", lambda m: args[int(m[1])-1], specs[identifier])
for file in sorted((ROOT / "engine/data/locales").glob("*.toml")):
    d = tomllib.loads(file.read_text())["specs"]
    lines = [
        make(d,"EVENT_WHENFLAGCLICKED"),
        make(d,"SENSING_ASKANDWAIT","[Question test ?]"),
        make(d,"DATA_SETVARIABLETO","[n v]","("+d["SENSING_ANSWER"]+")"),
        make(d,"CONTROL_REPEAT","(2)"),
        make(d,"DATA_SETVARIABLETO","[n v]","("+make(d,"OPERATORS_MULTIPLY","(n)","(3)")+")"),
        make(d,"DATA_SETVARIABLETO","[n v]","("+make(d,"OPERATORS_ADD","(n)","(4)")+")"),
        d["scratchblocks:end"],
        make(d,"LOOKS_SAYFORSECS","(n)","(2)"),
        make(d,"DATA_INSERTATLIST","[A]","(1)","[items v]"),
        make(d,"DATA_REPLACEITEMOFLIST","(1)","[items v]","[B]"),
    ]
    programs[file.stem] = "\n".join(lines)
(ROOT / "tests/fixtures/all-languages.json").write_text(json.dumps(programs,ensure_ascii=False,indent=2)+"\n")
print(f"Generated {len(programs)} canonical fixtures. Review diffs before committing.")
