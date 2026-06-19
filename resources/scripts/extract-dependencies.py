#!/bin/env python
import os
import subprocess
from pathlib import Path

ROOT_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
BIN_FILE = ROOT_DIR / "src-tauri" / "target" / "release" / "garmin-fit-rs"

dependencies = set()
print(os.path.basename(str(BIN_FILE)))
output = subprocess.check_output(["ldd", str(BIN_FILE)]).decode().splitlines()
for line in output:
    if " => " in line:
        so_file = line.split(" => ")[1].split(" (")[0]
        p_output = subprocess.check_output(
            ["pacman", "-Qo", so_file], env={"LANG": "C"}
        ).decode()
        dependencies.add(p_output.split(" ")[-2])
dependencies = sorted(list(dependencies))
for dep in dependencies:
    print(f"\t{dep}")
