import os
import shutil
import subprocess
from pathlib import Path

PROJ_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
CARGO_TOML_PATH = PROJ_DIR / "Cargo.toml"
PKGBUILD_PATH = PROJ_DIR / "resources" / "PKGBUILD"
INSTALL_PATH = PROJ_DIR / "resources" / "git-flow-rs.sh"
DIST_DIR = PROJ_DIR / "dist"
PKGBUILD_DIST_PATH = PROJ_DIR / "dist" / "PKGBUILD"
INSTALL_DIST_PATH = PROJ_DIR / "dist" / "git-flow-rs.install"


def generate_srcinfo():
    print("Generating .SRCINFO...")
    os.chmod(DIST_DIR, 0o777)
    subprocess.run(
        [
            "docker",
            "run",
            "--rm",
            "-u",
            f"{os.getuid()}:{os.getgid()}",
            "-v",
            f"{DIST_DIR}:/pkg",
            "epulidogil/rog-perf-tuner-srcinfo:latest",
        ],
        check=True,
    )


def generate_pkgbuild():
    print("Generating PKGBUILD...")

    shutil.copy2(PKGBUILD_PATH, PKGBUILD_DIST_PATH)

    with open(CARGO_TOML_PATH, "r", encoding="utf-8") as f:
        toml = f.read()

    toml = toml.splitlines()
    version = ""
    for line in toml:
        if line.startswith("version"):
            version = line.split(" = ")[1].replace('"', "")
            break

    with open(PKGBUILD_DIST_PATH, "r", encoding="utf-8") as f:
        content = f.read()

    content = content.replace("pkgver=", f"pkgver={version}")

    with open(PKGBUILD_DIST_PATH, "w", encoding="utf-8") as f:
        f.write(content)


def copy_install():
    shutil.copy2(INSTALL_PATH, INSTALL_DIST_PATH)


def create_dist_dir():
    if os.path.exists(DIST_DIR):
        print("Cleaning old dist folder...")
        shutil.rmtree(DIST_DIR)
    print("Creating dist folder...")
    os.makedirs(DIST_DIR)


if __name__ == "__main__":
    create_dist_dir()
    generate_pkgbuild()
    copy_install()
    generate_srcinfo()
    print("Release finished")
