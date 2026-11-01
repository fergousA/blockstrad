#!/usr/bin/env python3
"""Import official menu strings from the pinned BSD-3-Clause scratch-l10n snapshot.

Block templates are not imported here: they remain exactly blockst's TOML data.
This build-time script is never needed by a Typst document (no network at runtime).
"""
import concurrent.futures
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
SOURCE = json.loads((ROOT / "scripts/menu-source.json").read_text())
BASE = f"https://raw.githubusercontent.com/scratchfoundation/scratch-l10n/{SOURCE['commit']}/editor/"
LANGS = sorted(p.stem for p in (ROOT / "engine/data/locales").glob("*.toml"))

# Refuse a silent license change if the snapshot is updated later.
license_url = f"https://raw.githubusercontent.com/scratchfoundation/scratch-l10n/{SOURCE['commit']}/LICENSE"
with urllib.request.urlopen(license_url) as response:
    upstream_license = response.read().decode("utf-8").strip()
assert upstream_license == (ROOT / "licenses/scratch-l10n-BSD.txt").read_text().strip(), "License changed: review provenance before updating menus."


def load(lang):
    labels = {}
    for section in ("blocks", "extensions"):
        request = urllib.request.Request(BASE + section + "/" + lang + ".json", headers={"User-Agent": "blockstrad-data-builder"})
        with urllib.request.urlopen(request) as response:
            labels.update(json.load(response))
    return lang, labels


with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
    all_labels = dict(pool.map(load, LANGS))
en = all_labels["en"]
groups = {}


def group(name, *patterns):
    keys = set()
    for pattern in patterns:
        if pattern.endswith("*"):
            keys.update(k for k in en if k.startswith(pattern[:-1]))
        else:
            assert pattern in en, pattern
            keys.add(pattern)
    groups[name] = sorted(keys)


group("%m.stop", "CONTROL_STOP_ALL", "CONTROL_STOP_THIS", "CONTROL_STOP_OTHER")
group("%m.rotationStyle", "MOTION_SETROTATIONSTYLE_*")
group("%m.mathOp", "OPERATORS_MATHOP_*")
group("%m.key", "EVENT_WHENKEYPRESSED_*")
group("%m.effect", "LOOKS_EFFECT_*")
group("%m.timeAndDate", "SENSING_CURRENT_*")
group("%m.triggerSensor", "EVENT_WHENGREATERTHAN_*")
group("%m.attribute", "SENSING_OF_*")
groups["%m.attribute"].remove("SENSING_OF_STAGE")
group("%m.spriteOrStage", "SENSING_OF_STAGE")
group("%m.spriteOnly", "CONTROL_CREATECLONEOF_MYSELF")
group("%m.touching", "SENSING_TOUCHINGOBJECT_*")
group("%m.spriteOrMouse", "MOTION_POINTTOWARDS_POINTER")
group("%m.location", "MOTION_GOTO_POINTER", "MOTION_GOTO_RANDOM")
group("MOTION_POINTTOWARDS:1", "MOTION_POINTTOWARDS_POINTER", "MOTION_POINTTOWARDS_RANDOM")
group("MOTION_GLIDETO:2", "MOTION_GLIDETO_POINTER", "MOTION_GLIDETO_RANDOM")
group("%m.backdrop", "LOOKS_NEXTBACKDROP", "LOOKS_PREVIOUSBACKDROP", "LOOKS_RANDOMBACKDROP")
group("%d.listItem", "DATA_INDEX_LAST", "DATA_INDEX_RANDOM")
group("%d.listDeleteItem", "DATA_INDEX_ALL", "DATA_INDEX_LAST", "DATA_INDEX_RANDOM")
group("%d.instrument", "music.instrument*")
group("%d.drum", "music.drum*")
group("%m.color", "pen.colorMenu.*")
group("%m.motor", "wedo2.motorId.*")
groups["%m.motor2"] = groups["%m.motor"]
group("%m.motorDirection", "wedo2.motorDirection.*")
group("%m.xxx", "wedo2.tiltDirection.*")
group("%m.videoMotionType", "videoSensing.motion", "videoSensing.direction")
group("%m.videoState", "videoSensing.on", "videoSensing.off", "videoSensing.onFlipped")
group("%m.stageOrThis", "videoSensing.sprite", "videoSensing.stage")
for block in ("SOUND_CHANGEEFFECTBY", "SOUND_SETEFFECTO"):
    group(block + ":1", "SOUND_EFFECTS_*")
for block in ("LOOKS_COSTUMENUMBERNAME", "LOOKS_BACKDROPNUMBERNAME"):
    group(block + ":1", "LOOKS_NUMBERNAME_*")
group("LOOKS_GOTOFRONTBACK:1", "LOOKS_GOTOFRONTBACK_*")
group("LOOKS_GOFORWARDBACKWARDLAYERS:1", "LOOKS_GOFORWARDBACKWARDLAYERS_*")
group("SENSING_SETDRAGMODE:1", "SENSING_SETDRAGMODE_*")
group("text2speech.setVoiceBlock:1", "text2speech.alto", "text2speech.tenor", "text2speech.squeak", "text2speech.giant", "text2speech.kitten")
for block in ("microbit.whenButtonPressed", "microbit.isButtonPressed"):
    group(block + ":1", "microbit.buttonsMenu.*")
group("microbit.whenGesture:1", "microbit.gesturesMenu.*")
for block in ("microbit.whenTilted", "microbit.isTilted", "microbit.tiltAngle"):
    group(block + ":1", "microbit.tiltDirectionMenu.*")
for block in ("boost.whenColor", "boost.seeingColor"):
    group(block + ":1", "boost.color.*")
for block in ("boost.whenTilted", "boost.getTiltAngle"):
    group(block + ":1", "boost.tiltDirection.*")
group("boost.setMotorDirection:2", "boost.motorDirection.*")
group("wedo2.isTilted:1", "wedo2.tiltDirection.*")
group("gdxfor.whenForcePushedOrPulled:1", "gdxfor.pushed", "gdxfor.pulled")
group("gdxfor.whenGesture:1", "gdxfor.shaken", "gdxfor.startedFalling", "gdxfor.turnedFaceUp", "gdxfor.turnedFaceDown")
for block in ("gdxfor.whenTilted", "gdxfor.isTilted", "gdxfor.getTilt"):
    group(block + ":1", "gdxfor.tiltDirectionMenu.*")
keys = set(k for group_keys in groups.values() for k in group_keys)
# Do not bundle unrelated application UI translations.
labels = {lang: {k: text for k, text in values.items() if k in keys} for lang, values in all_labels.items()}
out = {"source": SOURCE, "groups": groups, "labels": labels}
(ROOT / "engine/data/menus.json").write_text(json.dumps(out, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")
print(f"{len(LANGS)} languages, {len(groups)} contextual menu groups, {len(keys)} official menu entries.")
