import os
import re
import json
import sys
import subprocess
from pathlib import Path

PROJ_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
PACKAGE_JSON_FILE = PROJ_DIR / "package.json"
CARGO_TOML_FILE = PROJ_DIR / "src-tauri" / "Cargo.toml"
TAURI_CONF_FILE = PROJ_DIR / "src-tauri" / "tauri.conf.json"

def get_current_version():
    with open(PACKAGE_JSON_FILE, "r",encoding="utf-8") as f:
        pkg_json_content = json.load(f)
    return pkg_json_content["version"]

def replace_version(file, regex, version):
    with open(file, "r", encoding="utf-8") as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        if regex.match(line):
            lines[i] = regex.sub(rf"\g<1>{version}\2", line)
            break

    with open(file, "w", encoding="utf-8") as f:
        f.writelines(lines)

def check_version_tag(version):
    tags = subprocess.check_output(["git","tag"],text=True).splitlines()
    return not version in tags

if __name__ == "__main__":
    current_version = get_current_version()
    print(f"Current version: {current_version}")

    while True:
        if len(sys.argv)>1:
            new_version = sys.argv[1]
        else:
            new_version = input("Enter new version: ")

        parts = new_version.split(".")
        if len(parts) != 3 or not all(p.isdigit() for p in parts):
            print(f"Invalid format: {new_version}")
            if len(sys.argv)>1:
                sys.exit(1)
            else:
                continue

        if not check_version_tag(new_version):
            print(f"Tag for version {new_version} already exists")
            if len(sys.argv)>1:
                sys.exit(1)
            else:
                continue

        break


    print(f"Setting version {new_version}")

    version_re_json = re.compile(r'^(\s*"version"\s*:\s*")[^"]*(".*)$')
    version_re_toml = re.compile(r'^(\s*version\s*=\s*")[^"]*(".*)$')

    replace_version(PACKAGE_JSON_FILE, version_re_json, new_version)
    replace_version(CARGO_TOML_FILE, version_re_toml, new_version)
    replace_version(TAURI_CONF_FILE, version_re_json, new_version)
