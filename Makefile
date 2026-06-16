run:
	@pnpm tauri dev

build:
	@pnpm tauri build

lint:
	@pnpm lint
	@cd src-tauri && cargo clippy