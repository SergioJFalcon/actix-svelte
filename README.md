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

## Create SQlite Database

    Make sure that sqlx-cli was installed
    cargo install sqlx-cli

    Make sure database URL in environment variable
    DATABASE_URL="sqlite:database.db"

    Create the databae
        cargo sqlx database create

    Run sql migrations
        cargo sqlx migrate run --source database/migrations

    Add sql migration
        cargo sqlx migrate add <name>

### Windows Service

Create service

```bash
sc.exe create "actix_example" binPath= "C:\Users\sfalcon\code\actix-svelte\target\release\actix-svelte.exe" type= own
```

Start Server

```bash
sc.exe start "actix_example"
```

Pause Server

```bash
sc.exe pause "actix_example"
```

Continue Server

```bash
sc.exe continue "actix_example"
```

Stop Actix Server

```bash
sc.exe stop "actix_example"
```

Delete Service

```bash
sc.exe delete "actix_example"
```
