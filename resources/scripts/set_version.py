import os
import re
from pathlib import Path

PROJ_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
PACKAGE_JSON_FILE = PROJ_DIR / "package.json"
CARGO_TOML_FILE = PROJ_DIR / "src-tauri" / "Cargo.toml"
TAURI_CONF_FILE = PROJ_DIR / "src-tauri" / "tauri.conf.json"

def replace_version(file, regex):
    with open(file, "r", encoding="utf-8") as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        if regex.match(line):
            lines[i] = regex.sub(rf"\g<1>{value}\2", line)
            break

    with open(file, "w", encoding="utf-8") as f:
        f.writelines(lines)


if __name__ == "__main__":
    while True:
        value = input("Enter new version (x.y.z): ")

        parts = value.split(".")

        if len(parts) == 3 and all(p.isdigit() for p in parts):
            break

        print("Invalid format (ej: 1.2.3)")

    version_re_json = re.compile(r'^(\s*"version"\s*:\s*")[^"]*(".*)$')
    version_re_toml = re.compile(r'^(\s*version\s*=\s*")[^"]*(".*)$')

    replace_version(PACKAGE_JSON_FILE, version_re_json)
    replace_version(CARGO_TOML_FILE, version_re_toml)
    replace_version(TAURI_CONF_FILE, version_re_json)
