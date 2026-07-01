import json
import subprocess
import toml
import yaml

from commons import SRC_TAURI_DIR, PACKAGE_JSON_PATH, PNPM_LOCK_FILE, CARGO_TOML_FILE, CURRENT_VERSIONS_FILE, CARGO_LOCK_FILE

subprocess.run(["pnpm", "update"], check=True)
subprocess.run(["cargo", "update"], check=True, cwd=SRC_TAURI_DIR)

node_packages = {}
with open(PACKAGE_JSON_PATH, "r", encoding="utf-8") as f:
    package_json = json.load(f)
with open(PNPM_LOCK_FILE, "r", encoding="utf-8") as f:
    content = yaml.safe_load(f)
    for package in content["packages"]:
        sep=package.rfind("@")
        name = package[0:sep]
        version = package[sep+1:]
        if name in package_json["dependencies"] or name in package_json["devDependencies"]:
            node_packages[name] = version


rust_packages = {}
with open(CARGO_TOML_FILE, "r", encoding="utf-8") as f:
    cargo_toml = toml.load(f)
with open(CARGO_LOCK_FILE, "r", encoding="utf-8") as f:
    content = toml.load(f)
    for package in content["package"]:
        if package["name"] in cargo_toml["dependencies"] or package["name"] in cargo_toml["build-dependencies"]:
            rust_packages[package["name"]] = package["version"]


versions = {
    "node": node_packages,
    "rust": rust_packages
}
with open(CURRENT_VERSIONS_FILE, "w", encoding="utf-8") as f:
    yaml.safe_dump(versions,f)
