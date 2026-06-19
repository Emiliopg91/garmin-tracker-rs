#!/usr/bin/env python

from pathlib import Path
import os
from lib.method_definition import MethodDefinition
from lib.struct_definition import StructDefinition
import time

BASE_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..","..","..")))

SRC_TAURI = BASE_DIR / "src-tauri" / "src"

SRC_TS = BASE_DIR / "src"
MODELS_FILE = SRC_TS / "utils/backend/models.ts"
CLIENT_FILE = SRC_TS / "utils/backend/client.ts"


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
print(f"    Found {len(methods)} commands")

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
print(f"    Found {len(structs)} structs")

if not MODELS_FILE.parent.exists():
    os.makedirs(MODELS_FILE.parent)

print("  Generating models file...")
StructDefinition.generate_file(MODELS_FILE, structs)
print(f"    Generated {MODELS_FILE.relative_to(BASE_DIR)}")

print("  Generating client file...")
MethodDefinition.generate_file(CLIENT_FILE, methods)
print(f"    Generated {CLIENT_FILE.relative_to(BASE_DIR)}")

print(f"  Finished after {time.time()-t0}")
print("----------------------------------------------")
