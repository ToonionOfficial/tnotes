<p align="center">
  <img src="public/logo-transparent.png" alt="TNotes logo" width="96" />
</p>

<h1 align="center">TNotes Web</h1>

<p align="center">The browser client for TNotes.</p>

> This client is in early development and is currently intended for local development alongside the Rust server.

## Requirements

- Node.js 22 or newer
- Corepack-enabled `pnpm`
- A running TNotes server for authenticated API and sync flows

## Run locally

From this directory:

```bash
pnpm install
pnpm dev
```

Open [http://localhost:3000](http://localhost:3000). During development, Vite proxies `/api` and `/ws` to `http://localhost:8787`.

Start the server in a second terminal from the repository root:

```bash
cargo run -p tnotes-server
```

To build the production frontend:

```bash
pnpm run build
```

## Useful commands

| Command | Purpose |
| --- | --- |
| `pnpm dev` | Start the Vite development server on port 3000 |
| `pnpm run build` | Create the production bundle in `dist/` |
| `pnpm run typecheck` | Run TypeScript checks without emitting files |
| `pnpm run lint` | Check source files with Biome |
| `pnpm run format` | Format source files with Biome |
| `pnpm run generate-routes` | Regenerate the TanStack Router route tree |

## Project conventions

- Routes live in [`src/routes`](src/routes) and use TanStack Router’s file-based routing.
- Reusable UI components live in [`src/components`](src/components).
- API calls and shared client types live in [`src/lib`](src/lib).
- Static assets, including the transparent TNotes logo, live in [`public`](public).

When adding a route, create a file in `src/routes`; the TanStack Router plugin updates `src/routeTree.gen.ts` during development and builds.

## Related documentation

See the [root README](../README.md) for the full project overview, server setup, mobile client, and repository layout.
