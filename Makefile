run:
	@mold --run pnpm tauri dev 

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
	@python resources/scripts/publish.py

delete:
	@python resources/scripts/set_version.py $(ARGS) && \
	make clean lint && \
	git add package.json src-tauri/Cargo.lock src-tauri/Cargo.toml src-tauri/tauri.conf.json && \

	