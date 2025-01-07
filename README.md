# Actix Svelte Template

Actix-web template that uses SvelteKit built and served as an embeded static files.

## Features

- [Actix web](https://actix.rs/) Backend server
- [SvelteKit](https://kit.svelte.dev/) Frontend built as SSG

## Setup

### Dev Requirements

- [ ] cargo: `curl https://sh.rustup.rs -sSf | sh`
- [ ] node: https://nodejs.org/en/download/current/

## Debug

### Frontend

```bash
cd client
npm run dev
```

### Backend

```bash
cargo run
```

### Build

You can build the project with cargo. The `build.rs` will automatically compile the frontend to static files in the ./client/build directory.

```bash
cargo build --release
```
