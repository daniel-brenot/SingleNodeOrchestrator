# Single Node Orchestrator

A project in the works that will allow for managing a single node kubernetes instance easily.

This code is bundled into a single executable in the releases, and should automatically be provided to the updater service when it is complete.

This project is meant to run on a nixos iso that is set up to work for this. As such, downloading ISO files that can run should be done from that page...when it is available.

## Project Layout

- `backend/` - Rust API server built with Axum.
- `frontend/` - Sveltekit/Vite UI that calls the backend with axios.

## Getting Started

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

