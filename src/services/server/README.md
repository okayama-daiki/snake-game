# Snake Game Server

Backend server for the online PvP Snake Game.

## Configuration

- `HOST`: Bind address. Defaults to `0.0.0.0`.
- `PORT`: HTTP and WebSocket port. Defaults to `5173`.
- `BOT_COUNT`: Number of reinforcement-learning bots. Defaults to `6` and is capped at `32`. Set to `0` to disable bots.
- `PRIVATE_KEY_FILE` / `CERTIFICATE_CHAIN_FILE`: Enable TLS when both are set.

The in-memory leaderboard contains only currently active snakes and ranks them by their
current length.

## Train the bot

The checked-in Q-table was trained against the production `GameEngine`. To generate a new table:

```bash
cargo run --release \
  --manifest-path src/services/game/Cargo.toml \
  --bin train_bot -- \
  src/services/server/assets/bot_policy.json
```

Run the server tests after replacing the policy to validate its dimensions and integration.
