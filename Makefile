
run:
	@RUSTC_WRAPPER=sccache mold --run pnpm tauri dev -- -- -v

build:
	@mold --run pnpm tauri build

lint:
	@pnpm lint
	@cd src-tauri && cargo clippy

clean:
	@rm -Rf node_modules dist && cd src-tauri && cargo clean

release:
	@python resources/scripts/release.py 

setup-toolchain:
	@paru -S $(shell bash -c 'source resources/PKGBUILD && printf "%s " "$${makedepends[@]}" "$${depends[@]}"')

update:
	@python resources/scripts/update-dependencies.py

publish:  
	@RUSTC_WRAPPER=sccache python resources/scripts/publish.py