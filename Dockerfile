# Two images from one file:
#   target "api": the Rust API, listening on :3000, SQLite in /data
#   target "web": Caddy serving the built SPA and proxying /api to the API, with automatic HTTPS

# --- API build ---------------------------------------------------------------
# cargo-chef splits the build in two layers: the dependencies (slow, rarely change) and our
# own crates (fast). With a layer cache, a code change recompiles only our crates, and a
# change outside the files copied here (the web app, the docs) recompiles nothing.
FROM rust:1.99.1-slim-trixie AS chef
# Use the image's toolchain instead of letting rust-toolchain.toml download components.
ENV RUSTUP_TOOLCHAIN=1.99.1 SQLX_OFFLINE=true
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /src

FROM chef AS api-plan
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS api-build
COPY --from=api-plan /src/recipe.json recipe.json
RUN cargo chef cook --release -p nullbox-api --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY .sqlx .sqlx
COPY migrations migrations
COPY crates crates
RUN cargo build --release -p nullbox-api \
    && cp target/release/nullbox-api /nullbox-api

# --- Web build ---------------------------------------------------------------
FROM node:22-slim AS web-build
ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0
RUN corepack enable
WORKDIR /src
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/
RUN pnpm install --frozen-lockfile
COPY apps/web apps/web
# Same origin in production: the bundle calls /api on whatever host serves it.
RUN echo 'VITE_API_BASE_URL=' > apps/web/.env.production.local \
    && pnpm build \
    && ! grep -rq "localhost:3000" apps/web/dist

# --- API runtime -------------------------------------------------------------
FROM debian:trixie-slim AS api
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 nullbox \
    && mkdir /data && chown nullbox /data
COPY --from=api-build /nullbox-api /usr/local/bin/nullbox-api
USER nullbox
ENV BIND_ADDR=0.0.0.0:3000 DATABASE_URL=sqlite:///data/nullbox.db RUST_LOG=info
VOLUME /data
EXPOSE 3000
CMD ["nullbox-api"]

# --- Web runtime -------------------------------------------------------------
FROM caddy:2 AS web
COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=web-build /src/apps/web/dist /srv
