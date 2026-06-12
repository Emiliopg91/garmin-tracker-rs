run:
	@pnpm tauri dev

lint:
	@pnpm lint
	@cd src-tauri && cargo clippy