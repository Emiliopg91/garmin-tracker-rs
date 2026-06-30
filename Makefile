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
	@python resources/scripts/set_version.py $(ARGS) && make clean lint && git add package.json src-tauri/Cargo.lock src-tauri/Cargo.toml src-tauri/tauri.conf.json

release:
	@python resources/scripts/release.py 

setup-toolchain:
	@paru -S $(shell bash -c 'source resources/PKGBUILD && printf "%s " "$${makedepends[@]}" "$${depends[@]}"')

update
	@pnpm update && cd src-tauri && cargo update