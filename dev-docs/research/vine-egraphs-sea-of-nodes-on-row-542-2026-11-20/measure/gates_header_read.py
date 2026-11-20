"""Print the header of the first 28 text-pattern gates (gates_rust.log order) with a hand class.

R = the header says the gate decides a property of Rust items, expressions, rsx or attributes.
O = it reads .rs mainly for CSS classes or page-script (JavaScript) text.
The class column was assigned by reading each header below; the script only prints and counts.
Usage: python3 -I gates_header_read.py REPO
"""
import os
import re
import sys

CLASSES = [
    ("R", "check-preview-composition"), ("R", "check-cfg-axis"), ("R", "check-constant-signal"),
    ("R", "check-id-interpolation"), ("R", "check-component-call"), ("R", "check-test-harness-controlled"),
    ("R", "check-direction-keys"), ("R", "check-state-attributes"), ("R", "check-render-keys"),
    ("R", "check-render-clock"), ("R", "check-collection-derive"), ("R", "check-raw-text-interpolation"),
    ("R", "check-dynamic-style-shorthand"), ("R", "check-as-child"), ("R", "check-wrapper-props"),
    ("R", "check-wrapper-spread"), ("O", "check-uncleared-intervals"), ("O", "check-blocking-scroll-listeners"),
    ("R", "check-native-slots"), ("R", "check-hot-events"), ("R", "check-motion-gating"), ("R", "check-open-focus"),
    ("R", "check-unawaited-focus"), ("O", "check-theme-mode-writer"), ("O", "check-dx-class-prefix"),
    ("O", "check-cross-component-overrides"), ("O", "check-css-vars-defined"), ("O", "check-anchor-marker"),
]
repo = sys.argv[1]
for cls, name in CLASSES:
    lines = open(os.path.join(repo, "scripts", name + ".sh"), encoding="utf-8").read().split("\n")[2:40]
    head = " ".join(re.sub(r"^# ?", "", l) for l in lines if l.startswith("#"))
    print(f"== {cls} {name}: {head[:400]}")
r = sum(1 for c, _ in CLASSES if c == "R")
print(f"TOTAL R={r} O={len(CLASSES) - r} of {len(CLASSES)}")
