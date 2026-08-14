FROM rust:alpine3.22 AS builder

WORKDIR /usr/src/rs-cmdb

RUN apk add --no-cache musl-dev nodejs npm

RUN cargo install trunk

RUN rustup target add wasm32-unknown-unknown x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY client/Cargo.toml ./client/
COPY common/Cargo.toml ./common/
COPY front/Cargo.toml ./front/
RUN mkdir -p server/src client/src common/src front/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > client/src/main.rs && \
    echo "fn main() {}" > front/src/main.rs && \
    echo "// common lib" > common/src/lib.rs

RUN cargo build --locked --release --package server --target x86_64-unknown-linux-musl
RUN cargo build --locked --release --package client --target x86_64-unknown-linux-musl


COPY . .

RUN touch server/src/main.rs client/src/main.rs common/src/lib.rs front/src/main.rs
RUN cargo build --locked --release --package server --target x86_64-unknown-linux-musl
RUN cargo build --locked --release --package client --target x86_64-unknown-linux-musl

WORKDIR /usr/src/rs-cmdb/front
RUN npm ci
RUN trunk build --release

# Final stage
FROM alpine:3.22
WORKDIR /app

COPY --from=builder /usr/src/rs-cmdb/target/x86_64-unknown-linux-musl/release/rs-cmdb-server ./rs-cmdb-server
COPY --from=builder /usr/src/rs-cmdb/target/x86_64-unknown-linux-musl/release/rs-cmdb-client ./binaries/linux/x86_64/rs-cmdb-client
COPY --from=builder /usr/src/rs-cmdb/front/dist ./dist
COPY --from=builder /usr/src/rs-cmdb/config/default.toml ./config/default.toml

RUN chmod +x ./rs-cmdb-server ./binaries/linux/x86_64/rs-cmdb-client && \
    addgroup -S cmdb && adduser -S cmdb -G cmdb && \
    mkdir -p /app/data && chown cmdb:cmdb /app/data

EXPOSE 8080
USER cmdb
CMD ["./rs-cmdb-server"]
