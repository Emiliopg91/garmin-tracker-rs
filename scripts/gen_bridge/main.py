#!/usr/bin/env python

from pathlib import Path
import os
from lib.method_definition import MethodDefinition
from lib.struct_definition import StructDefinition


BASE_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..","..")))

SRC_TAURI = BASE_DIR / "src-tauri" / "src"
MODELS_FLD = SRC_TAURI / "models"
LIB_RS = SRC_TAURI / "lib.rs"

SRC_TS = BASE_DIR / "src"
BRIDGE_FILE = SRC_TS / "utils/RustBridge.ts"


print("---- TS code generator for Tauri commands ----")
print("  Scanning for structs...")
custom_types = []
structs = set()
for file in MODELS_FLD.glob("**/*.rs"):
    for s in StructDefinition.get_definitions(file):
        structs.add(s)
        for t in s.get_custom_types():
            custom_types.append(t)

print("  Scanning for tauri commands...")
methods = MethodDefinition.get_definitions(LIB_RS)
for d in methods:
    for t in d.get_custom_types():
        custom_types.append(t)

for custom_type in custom_types:
    found = False
    for struct in structs:
        if struct.name==custom_type:
            found = True
            break
    
    if not found:
        """"""


print("  Generating bridge...")
result_lines = []
result_lines.append('//Auto generated file, do not edit manually')
result_lines.append('')
result_lines.append('import { invoke } from "@tauri-apps/api/core";')
result_lines.append("")

for struct in structs :
    result_lines.append(struct.to_typescript())
    result_lines.append("")


result_lines.append("export class RustBridge {")
for method in methods:
    result_lines.append("")
    result_lines.append(method.to_typescript())
    result_lines.append("")
result_lines.append("}")

if not BRIDGE_FILE.parent.exists():
    os.makedirs(BRIDGE_FILE.parent)

with open(BRIDGE_FILE, "w", encoding="utf-8") as f:
    f.write("\n".join(result_lines))

print("  Finished")
print("----------------------------------------------")