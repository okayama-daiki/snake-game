all: server client

server:
	cargo build --manifest-path src/services/server/Cargo.toml --release

client:
	npm install && npm run build
