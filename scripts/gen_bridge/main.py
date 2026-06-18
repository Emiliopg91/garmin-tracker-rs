#!/usr/bin/env python

from pathlib import Path
import os
from lib.method_definition import MethodDefinition
from lib.struct_definition import StructDefinition
import time

BASE_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..","..")))

SRC_TAURI = BASE_DIR / "src-tauri" / "src"

SRC_TS = BASE_DIR / "src"
BRIDGE_FILE = SRC_TS / "utils/RustBridge.ts"


t0 = time.time()
print("---- TS code generator for Tauri commands ----")


print("  Scanning for tauri commands...")
methods = []
for file in SRC_TAURI.glob("**/*.rs"):
    for m in MethodDefinition.get_definitions(BASE_DIR, file):
        methods.append(m)

m_custom_types = []
for d in methods:
    for t in d.get_custom_types():
        m_custom_types.append(t)
print(f"! {m_custom_types}")

print("  Scanning for structs...")
s_custom_types = []

structs = set()
added = []
cont = True
while cont:
    cont=False
    for file in SRC_TAURI.glob("**/*.rs"):
        for s in StructDefinition.get_definitions(BASE_DIR, file):
            if s.name not in added and s.name in m_custom_types:
                structs.add(s)
                added.append(s.name)
                for ct in s.get_custom_types():
                    cont = True
                    m_custom_types.append(ct)


print("  Generating bridge...")
result_lines = []
result_lines.append('/* eslint-disable */')
result_lines.append('')
result_lines.append('//Auto generated file, do not edit manually')
result_lines.append('')
result_lines.append('import { invoke, InvokeArgs } from "@tauri-apps/api/core";')
result_lines.append("")

for struct in sorted(structs, key=lambda x: x.name):
    result_lines.append(struct.to_typescript())
    result_lines.append("")

result_lines.append("export class RustBridge {")
result_lines.append("")
for method in sorted(methods, key=lambda x: x.name):
    result_lines.append(method.to_typescript())
    result_lines.append("")
result_lines.append(
    "\tprivate static inner_invoke<R>(method: string, payload?: InvokeArgs): Promise<R> {"
)
result_lines.append('\t\treturn invoke(method, payload);')
result_lines.append("\t}")
result_lines.append("}")

if not BRIDGE_FILE.parent.exists():
    os.makedirs(BRIDGE_FILE.parent)

with open(BRIDGE_FILE, "w", encoding="utf-8") as f:
    f.write("\n".join(result_lines))

print(f"  Finished after {time.time()-t0}")
print("----------------------------------------------")
