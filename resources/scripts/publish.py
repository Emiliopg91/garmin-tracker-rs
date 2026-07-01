import json
import subprocess
import sys

from commons import SET_VERSION_SCRIPT, PACKAGE_JSON_PATH, CARGO_TOML_FILE, PROJ_DIR, SRC_DIR, SRC_TAURI_SRC_DIR

if __name__ == "__main__":
    if subprocess.check_output(["git", "branch", "--show-current"], text=True).strip() != "main":
        print("ERROR: Releases are only allowed in main branch")
        sys.exit(1)

    if len(subprocess.check_output(["git", "status", "--porcelain"], text=True).strip().splitlines()) > 0:
        print("ERROR: The repository has uncommited changes")
        sys.exit(1)

    todos = {}
    for file in [*SRC_TAURI_SRC_DIR.rglob("*.rs"), *SRC_DIR.rglob("*.ts"), *SRC_DIR.rglob("*.tsx")]:
        with open(file, "r", encoding="utf-8") as f:
            local_todo = []
            for i, line in enumerate(f, start=1):
                if "TODO" in line:
                    local_todo.append((i, line.rstrip()))
            if len(local_todo) > 0:
                todos[str(file).replace(str(PROJ_DIR),"")[1:]]=local_todo

    if todos:
        print("TODOs not allowed in release, found:")
        for file,todos in todos.items():
            print(f"    {file}")
            for line_num, line_content in todos:
                print(f"        Line {str(line_num).ljust(4, " ")} -> {line_content}")
            print("")
        sys.exit(1)

    subprocess.check_call(["pnpm", "tsc"])
    subprocess.check_call(["cargo", "clippy", "--manifest-path", CARGO_TOML_FILE, "--", "-D", "warnings"])
    subprocess.check_call(["pnpm", "prettier", "--write", SRC_DIR])
    subprocess.check_call(["cargo", "fmt"], cwd=SRC_TAURI_SRC_DIR)

    subprocess.check_call(["make", "clean", "build"])

    if len(subprocess.check_output(["git", "status", "--porcelain"], text=True).strip().splitlines()) > 0:
        subprocess.check_call(["git", "commit", "-am", "[chore] Check and format before release"])

    subprocess.check_call(["python", SET_VERSION_SCRIPT])

    with open(PACKAGE_JSON_PATH, "r", encoding="utf-8") as f:
        version = json.load(f)["version"]

    subprocess.check_call(["git", "commit", "-am", f"[release] {version}"])
