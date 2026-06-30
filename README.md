# Cab Logger — Dispatch Desk Portfolio Demo

Cab Logger is an internal cab booking and dispatch ticket management tool designed for employees and vendor communication. This repository contains the complete portfolio setup as requested.

## Technical Stack
- **Frontend:** Rust + [Leptos](https://github.com/leptos-rs/leptos) (WebAssembly Client-Side Rendered SPA)
- **Backend:** TypeScript + [Fastify](https://www.fastify.io/)
- **Database:** [DuckDB](https://duckdb.org/) (Fast, local relational engine storing dispatch tickets)
- **Deployment & Running Platform:** Docker + Docker Compose

---

## Architecture Overview

```
 ┌─────────────────────────┐         ┌────────────────────────┐         ┌───────────────────┐
 │   Leptos Rust Frontend  │ ──────> │  Fastify TS API Server │ ──────> │   DuckDB Engine   │
 │   (Served via Nginx)    │         │      (Node.js Core)    │         │ (cab_logger.db file)│
 └─────────────────────────┘         └────────────────────────┘         └───────────────────┘
```

1. **Leptos Frontend (Port 8090):** A highly performant Rust application compiled directly into WebAssembly. Utilizes a tailored tactile **"Dispatch Desk" design language** (Soot `#1F2421`, Manila `#F2EFE6`, and Taxi Vermilion `#C8553D` accents) with ticket-stub ledger views and a custom segmented mechanical Odometer reading.
2. **Fastify Backend (Port 8095):** High-speed API server proxying requests, performing validation, tracking state, and persisting logs to the database.
3. **DuckDB Database:** Serves as the main analytical/relational persistent storage. Schema tables (`bookings`, `sent_emails`) are created and pre-seeded automatically on start.

---

## Getting Started (Run with Docker)

### Prerequisites
Make sure you have [Docker](https://www.docker.com/) and [Docker Compose](https://docs.docker.com/compose/) installed on your machine.

### Where is the DuckDB Database?
DuckDB is an **embedded database** engine. Unlike heavy databases (such as PostgreSQL or MySQL), it runs in-process and stores all tables, indices, and seeded logs in a single local database file (`cab_logger.db`).

- When you run `docker-compose up`, the Fastify backend automatically creates and initializes this file in the `/app/data` workspace directory inside the container.
- Thanks to the mapped volume mount (`./backend/data:/app/data`), this database will automatically materialize on your local host system at `./backend/data/cab_logger.db`.
- The folder `backend/data/` is already set up and tracked in git via a placeholder `.gitkeep` file.
- You can query or inspect this database locally using any standard DuckDB CLI or visual database explorer (e.g. DBeaver) by pointing it to `./backend/data/cab_logger.db`.

### Quick Start Instruction
Navigate to the root directory of the project and run the following command to download/build the Rust toolchain, compile the WASM assets, and spin up the DuckDB backend:

```bash
docker-compose up --build
```

---

## Getting Started (Run Bare Metal / Local Host)

If you prefer to run both applications directly on your local system without Docker ("bare metal"), we have provided an automated startup script `run-local.sh` that checks for dependencies, configures critical environment flags, and launches both environments.

### Local Prerequisites
Ensure you have the following installed on your machine:
- **Node.js** (v18 or higher) and **npm**
- **Rust Toolchain** (`cargo`, `rustc`, `rustup`)
- **Trunk** (WebAssembly bundler for Rust; if missing, the script offers to install it via `cargo install trunk`)

### Automated Local Run
Simply navigate to the project root and execute:

```bash
# Make the script executable
chmod +x run-local.sh

# Run the bare-metal environment
./run-local.sh
```

### What does this script do?
1. **Toolchain Inspection:** Checks if `node`, `npm`, and `cargo` are installed.
2. **Target Setup:** Installs the `wasm32-unknown-unknown` Rust compilation target automatically if it's missing.
3. **M1/M2/M3 Apple Silicon Safety:** Checks if you are on an ARM64 Apple Silicon machine and automatically exports `RUSTFLAGS="-C target-feature=-reference-types"` to prevent known WASM bindgen generation issues (`failed to find intrinsics...`).
4. **Dependency Sync:** Installs backend npm dependencies if `node_modules` is missing.
5. **Concurrent Processes:** Launches the Fastify + DuckDB server in the background and the Leptos + Trunk server in the foreground, then handles cleanup (SIGINT) to shut both down cleanly on `Ctrl+C`.

### Port Mapping & UI Access
Once the script initializes:
- **Frontend Panel:** Open [http://localhost:8090](http://localhost:8090) to access the interactive Cab Logger Dispatch Board.
- **Backend API Server:** Access [http://localhost:8095/api/bookings](http://localhost:8095/api/bookings) to check the raw DuckDB database.

---

### Port Mapping & UI Access
Once the containers are fully initialized:
- **Frontend Panel:** Open [http://localhost:8090](http://localhost:8090) to access the interactive Cab Logger Dispatch Board.
- **Backend API Server:** Access [http://localhost:8095/api/bookings](http://localhost:8095/api/bookings) to check the direct raw DuckDB analytical output.

---

## Core Features Implemented

1. **The Dispatch Ticket Requisition Form:** Includes validation for passenger name, department selection, locations, and vendor routing with pre-populated email links.
2. **DuckDB Storage & Automated Seed:** Automatically populates `cab_logger.db` with 5 sample historical bookings on first launch, ensuring the board looks lively immediately.
3. **Mechanical Segmented Odometer:** A tabular segmented widget tracking today's booking count that live-updates as dispatches occur.
4. **Perforated Ticket Ledger:** A list rendering each cab request with dynamic "Pending" vs "Sent to Vendor" pulsing status lamps.
5. **Live Email Notification Monitor:** Inspect full email templates sent to the cab vendors right inside your dashboard.

---

## Intentionally Out of Scope
As this is a streamlined demo project:
- No real SMTP email connections are wired. Outbound vendor emails are logged in stdout and recorded inside DuckDB.
- Authentication/login flows are omitted.
- No automated regression suites or production deployment configuration files are present.
