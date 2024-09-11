# Cool HTMX chat application demo for JDP talk

### yep

## Overview

This is a really, really basic chat application that uses HTMX to handle message sending and loading in a very simple manner.
Its goal is to use as little JavaScript as possible to handle fetching, and let HTMX do everything for it.

## Stack
Chat server: Rust + Axum
Database: A very sad file database
Frontend: HTMX

## Running
Want to run this yourself? You're in luck! With either `cargo` or `docker` installed, run:

```
cargo run
```

or

```
docker build -t jdp-chat:1.0.0 . && docker run -p 8000:8000 jdp-chat:1.0.0
```

## Development
If you are cool and have Nix installed, you can install all required dependencies into a shell with `nix develop`.
