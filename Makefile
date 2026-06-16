run:
	@pnpm tauri dev

build:
	@pnpm tauri build

lint:
	@pnpm lint
	@cd src-tauri && cargo clippy

clean:
	@rm -Rf node_modules && cd src-tauri && cargo clean