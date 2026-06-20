import json
import os
import shutil
import subprocess
from pathlib import Path

PROJ_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
PACKAGE_JSON_PATH = PROJ_DIR / "package.json"
PKGBUILD_PATH = PROJ_DIR / "resources" / "PKGBUILD"
DIST_DIR = PROJ_DIR / "dist"
PKGBUILD_DIST_PATH = PROJ_DIR / "dist" / "PKGBUILD"


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

    with open(PACKAGE_JSON_PATH, "r", encoding="utf-8") as f:
        package_json = json.load(f)

    with open(PKGBUILD_DIST_PATH, "r", encoding="utf-8") as f:
        content = f.read()

    content = content.replace("pkgver=", f"pkgver={package_json["version"]}")

    with open(PKGBUILD_DIST_PATH, "w", encoding="utf-8") as f:
        f.write(content)


def create_dist_dir():
    if os.path.exists(DIST_DIR):
        print("Cleaning old dist folder...")
        shutil.rmtree(DIST_DIR)
    print("Creating dist folder...")
    os.makedirs(DIST_DIR)


if __name__ == "__main__":
    create_dist_dir()
    generate_pkgbuild()
    generate_srcinfo()
    print("Release finished")
