all: server client

server:
	cargo build --manifest-path src/services/server/Cargo.toml --release

client:
	bun install && bun run build
