<p align="center">
  <img src="apps/web/public/logo-transparent.png" alt="TNotes logo" width="112" />
</p>

<h1 align="center">TNotes</h1>

<p align="center">A personal, self-hosted Markdown notebook with multi-device sync.</p>

> TNotes is in early development. Expect unfinished features, changing APIs, and rough edges.

## What is here

- **Web** — React, TanStack Router, and Tailwind CSS, served by the Rust server.
- **Server** — Axum and SQLite, with authentication, device pairing, HTTP sync, and WebSocket notifications.
- **Mobile** — Expo/React Native with local SQLite storage and QR-code pairing foundations.
- **Desktop** — A GPUI-based Rust client in active development.

## Quick start

### Web and server

Build the web frontend first, then start the server from the repository root:

```bash
cd apps/web
pnpm install
pnpm run build
cd ../..
cargo run -p tnotes-server
```

Open [http://localhost:8787](http://localhost:8787). The first visit walks through server setup.

For a containerized server, use:

```bash
docker compose up --build
```

The server stores its SQLite data in `./data` locally or in the `tnotes-data` Docker volume. Configuration can be adjusted with `TNOTES_HOST`, `TNOTES_PORT`, `TNOTES_DATA_DIR`, and `TNOTES_SERVER_URL`.

### Mobile

```bash
cd apps/mobile
pnpm install
pnpm start
```

Use the Expo CLI to open the app on a simulator or device. The mobile client is not yet distributed as a finished release.

## Development commands

```bash
cargo test --workspace
cd apps/web && pnpm run typecheck && pnpm run lint
```

The release build script builds the frontend and server together:

```bash
./scripts/build.sh
```

## Project layout

```text
apps/
  mobile/              Expo client
  server/              Axum API, WebSocket sync, and embedded web app
  web/                 Browser client
  desktop/             GPUI desktop client
crates/tnotes-core/    Shared models, database, auth, and sync logic
docs/                  Design and deployment notes
```

## Documentation

- [Web app development](apps/web/README.md)
- [Self-hosting](docs/SELF_HOSTING.md)
- [Sync protocol](docs/SYNC_PROTOCOL.md)
- [Theming](docs/THEMING.md)

## License

TNotes is available under the [MIT License](LICENSE).
