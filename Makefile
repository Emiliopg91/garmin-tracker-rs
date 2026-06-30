run:
	@mold --run pnpm tauri dev 

build:
	@mold --run pnpm tauri build

lint:
	@pnpm lint
	@cd src-tauri && cargo clippy

clean:
	@rm -Rf node_modules dist && cd src-tauri && cargo clean

version:
	@python resources/scripts/set_version.py $(ARGS) && make clean lint && git add package.json src-tauri/Cargo.lock src-tauri/Cargo.toml src-tauri/tauri.conf.json"

release:
	@python resources/scripts/release.py 