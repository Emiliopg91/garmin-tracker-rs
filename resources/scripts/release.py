import json
import os
import re
import shutil
import subprocess
from pathlib import Path

PROJ_DIR = Path(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..")))
PACKAGE_JSON_PATH = PROJ_DIR / "package.json"
PKGBUILD_PATH = PROJ_DIR / "resources" / "PKGBUILD"
DIST_DIR = PROJ_DIR / "dist"
PKGBUILD_DIST_PATH = PROJ_DIR / "dist" / "PKGBUILD"
CHANGELOG_MD_FILE = PROJ_DIR / "dist" /  "changelog.md"


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



def generate_changelog():
    print("Generating changelog...")
    TAG_REGEX = re.compile(r"^\d+\.\d+\.\d+(-\d)?+$")

    def run_git(cmd):
        return subprocess.check_output(cmd, text=True).strip()

    def get_previous_version_tag():
        tags = run_git(["git", "tag", "--sort=-creatordate"]).splitlines()

        matched_tags = [tag for tag in tags if TAG_REGEX.match(tag)]

        if len(matched_tags) > 0:
            return matched_tags[0]
        return None

    def get_commits_since_tag(tag):
        if not tag:
            cmd = ["git", "log", "--pretty=format:%H %s"]
        else:
            cmd = ["git", "log", f"{tag}..HEAD", "--pretty=format:%H----%s"]
        print(f"Obteniendo commits desde {tag}")
        return run_git(cmd).splitlines()

    tag = get_previous_version_tag()
    commits = get_commits_since_tag(tag)
    commits.reverse()

    entries = {"feature": [], "improve": [], "fix": []}

    for c in commits:
        commit_hash, msg = c.split("----")
        msg = msg.replace("[ci skip]", "").strip()

        for typeEntry, typeEntries in entries.items():
            if msg.startswith(f"[{typeEntry}]"):
                typeEntries.append(
                    f'<td><a href="https://github.com/Emiliopg91/garmin-tracker-rs/commit/{commit_hash}">{commit_hash[0:7]}</a></td><td>{msg.replace(f"[{typeEntry}]", "").strip().capitalize()}</td>'
                )

    lines: list[str] = ["# No changelog available"]

    if (
        len(entries.get("feature")) > 0
        or len(entries.get("improve")) > 0
        or len(entries.get("fix")) > 0
    ):
        lines: list[str] = [
            "# Changes for release",
            "<table>",
            "<tr><th>Category</th><th>Commit</th><th>Message</th></tr>",
        ]

        for category in [
            ("feature", "New Features"),
            ("improve", "Improvements"),
            ("fix", "Fixes"),
        ]:
            entry, title = category
            for i in range(len(entries.get(entry))):
                line = "<tr>"
                if i == 0:
                    line = f'{line}<td rowspan="{len(entries.get(entry))}" style="vertical-align: top;"><b>{title}</b></td>'
                line = f"{line}{entries.get(entry)[i]}</tr>"
                lines.append(line)

        lines.append("</table>")

    if os.path.exists(CHANGELOG_MD_FILE):
        os.unlink(CHANGELOG_MD_FILE)
    with open(CHANGELOG_MD_FILE, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))


if __name__ == "__main__":
    create_dist_dir()
    generate_pkgbuild()
    generate_srcinfo()
    generate_changelog()
    print("Release finished")
