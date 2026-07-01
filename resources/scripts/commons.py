"""Project-wide path and environment constants."""

import os
from pathlib import Path

# --- Root ---
PROJ_DIR = Path(__file__).resolve().parents[2]

# --- Tauri / Rust ---
SRC_DIR = PROJ_DIR / "src"
SRC_TAURI_DIR = PROJ_DIR / "src-tauri"
SRC_TAURI_SRC_DIR = PROJ_DIR / "src-tauri/src"
CARGO_TOML_FILE = SRC_TAURI_DIR / "Cargo.toml"
CARGO_LOCK_FILE = SRC_TAURI_DIR / "Cargo.lock"
TAURI_CONF_FILE = SRC_TAURI_DIR / "tauri.conf.json"
BIN_FILE = SRC_TAURI_DIR / "target" / "release" / "garmin-tracker-rs"

# --- Node / pnpm ---
PACKAGE_JSON_PATH = PROJ_DIR / "package.json"
PNPM_LOCK_FILE = PROJ_DIR / "pnpm-lock.yaml"

# --- Resources ---
RESOURCES_DIR = PROJ_DIR / "resources"
PKGBUILD_PATH = RESOURCES_DIR / "PKGBUILD"

SCRIPTS_DIR = RESOURCES_DIR / "scripts"
SET_VERSION_SCRIPT = SCRIPTS_DIR / "set_version.py"

VERSIONS_DIR = RESOURCES_DIR / "versions"
CURRENT_VERSIONS_FILE = VERSIONS_DIR / "current.yaml"
PREVIOUS_VERSIONS_FILE = VERSIONS_DIR / "previous.yaml"

# --- Dist ---
DIST_DIR = PROJ_DIR / "dist"
PKGBUILD_DIST_PATH = DIST_DIR / "PKGBUILD"
CHANGELOG_MD_FILE = DIST_DIR / "changelog.md"

# --- Environment ---
ENV_C = {**os.environ, "LANG": "C"}
