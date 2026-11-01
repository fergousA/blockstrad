#!/usr/bin/env python3
"""Validate 26x26 translations and 26x4 SVGs from the shipped WASM. Python 3.11+."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
NS = "{http://www.w3.org/2000/svg}"
HEX = re.compile(r"#[0-9A-Fa-f]{6}(?![0-9A-Fa-f])|#[0-9A-Fa-f]{3}(?![0-9A-Fa-f])")


def semantics(node, specs):
    inputs = [p for p in node["parts"] if p["kind"] == "input"]
    slots = [int(n) for n in re.findall(r"%(\d+)", specs.get(node["id"], ""))]
    args = {slot: semantics(p["nested"], specs) if "nested" in p else {"value": p["value"], "shape": p["shape"]}
            for slot, p in zip(slots, inputs)}
    return {"id": node["id"], "category": node["category"], "shape": node["shape"], "args": args,
            "name": " ".join(p["value"] for p in node["parts"] if p["kind"] == "label") if node["id"] == "unrecognized" else "",
            "body": [semantics(n, specs) for n in node["body"]],
            "else": [semantics(n, specs) for n in node["else-body"]]}


def canonical(color):
    color = color.lower()
    if re.fullmatch(r"#[0-9a-f]{3}", color):
        color = "#" + "".join(c * 2 for c in color[1:])
    return color


def paints(svg):
    root = ET.fromstring(svg)
    values = Counter()
    for group in root:
        if group.tag == NS + "defs":
            continue
        for el in group.iter():
            if el.tag in (NS + "path", NS + "rect"):
                values[(el.tag, canonical(el.get("fill", "")), canonical(el.get("stroke", "")))] += 1
    assert values
    return values


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--typst", default="/usr/local/bin/typst")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    run = subprocess.run([args.typst, "eval", "query(<matrix>).first().value", "--in", str(ROOT / "tests/export-matrix.typ"), "--root", str(ROOT)], text=True, capture_output=True)
    if run.returncode or run.stderr:
        print(run.stderr, file=sys.stderr)
        return run.returncode or 1
    data = json.loads(run.stdout)
    specs = {p.stem: tomllib.loads(p.read_text())["specs"] for p in (ROOT / "engine/data/locales").glob("*.toml")}
    reference = next(p for p in data["pairs"] if p["from"] == p["to"] == "en")
    expected = [[semantics(n, specs["en"]) for n in script] for script in reference["scripts"]]
    assert len(data["pairs"]) == 26 * 26
    for pair in data["pairs"]:
        name = pair["from"] + " -> " + pair["to"]
        assert not pair["warnings"], (name, pair["warnings"])
        assert [[semantics(n, specs[pair["to"]]) for n in script] for script in pair["scripts"]] == expected, name
        assert pair["rtl"] == (pair["to"] in ("ar", "fa", "he")), name
    svg_refs = {r["theme"]: paints(r["svg"]) for r in data["rendered"] if r["to"] == "en"}
    assert len(data["rendered"]) == 26 * 4
    for result in data["rendered"]:
        svg = result["svg"]
        name = result["to"] + " / " + result["theme"]
        assert paints(svg) == svg_refs[result["theme"]], name
        assert ("scale(-1 1)" in svg) == (result["to"] in ("ar", "fa", "he")), name
        if result["theme"] == "grayscale":
            colors = [canonical(c) for c in HEX.findall(svg)]
            assert colors and all(c[1:3] == c[3:5] == c[5:7] for c in colors), name
    custom = data["custom"]["scripts"][0]
    assignment = custom[1]
    fields = [p for p in assignment["parts"] if p["kind"] == "input"]
    assert fields[0]["value"] == "النتيجة"
    assert fields[1]["nested"]["id"] == "SENSING_ANSWER"
    renamed_variable = next(p["nested"] for p in custom[2]["parts"] if "nested" in p)
    assert renamed_variable["category"] == "variables" and renamed_variable["id"] == "unrecognized"
    assert renamed_variable["parts"][0]["value"] == "الإجابة"
    assert not data["custom"]["warnings"]
    report = {
        "typst": subprocess.check_output([args.typst, "--version"], text=True).strip(),
        "wasm_sha256": hashlib.sha256((ROOT / "libs/blockstrad.wasm").read_bytes()).hexdigest(),
        "language_pairs": len(data["pairs"]), "svg_renders": len(data["rendered"]),
        "status": "PASS", "catalog": data["catalog"],
        "scope": "676 pairs on a nested, multi-category fixture; 104 SVG paint comparisons; not every possible program or menu value",
    }
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(report["typst"])
    print("PASS: 676/676 language pairs preserve opcodes, categories, numbered arguments, names and control structure.")
    print("PASS: 104/104 SVGs preserve category paints and target RTL direction; grayscale paints are achromatic.")
    print("PASS: user dictionaries preserve the distinction between an answer reporter and a variable with the same name.")
    print("WASM SHA-256:", report["wasm_sha256"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
