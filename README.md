# Single Node Orchestrator

A Rust and Svelte starter for a local web UI that manages a machine hosting Kubernetes.

## Project Layout

- `backend/` - Rust API server built with Axum.
- `frontend/` - Svelte/Vite UI that calls the backend with axios.

## Getting Started

Use the project Node version:

```powershell
$nodeVersion = Get-Content .nvmrc
nvm install $nodeVersion
nvm use $nodeVersion
```

Run the backend:

```sh
cargo run -p single-node-orchestrator-backend
```

Run the frontend:

```sh
cd frontend
npm install
npm run dev
```

The frontend dev server proxies `/api` requests to `http://127.0.0.1:3000`.

## Starter API

- `GET /api/health` returns a basic health response.
- `GET /api/system/summary` returns placeholder host and Kubernetes summary data.
