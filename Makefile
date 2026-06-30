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
	@python resources/scripts/set_version.py $(ARGS) && make clean

release:
	@python resources/scripts/release.py 