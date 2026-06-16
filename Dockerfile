FROM rust:alpine3.22 AS builder

WORKDIR /usr/src/rs-cmdb

RUN apk add --no-cache musl-dev nodejs npm

RUN cargo install trunk

RUN rustup target add wasm32-unknown-unknown x86_64-unknown-linux-musl

COPY Cargo.toml ./
COPY server/Cargo.toml ./server/
COPY client/Cargo.toml ./client/
COPY common/Cargo.toml ./common/
COPY front/Cargo.toml ./front/
RUN mkdir -p server/src client/src common/src front/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > client/src/main.rs && \
    echo "fn main() {}" > front/src/main.rs && \
    echo "// common lib" > common/src/lib.rs

RUN cargo build --release --package server --target x86_64-unknown-linux-musl
RUN cargo build --release --package client --target x86_64-unknown-linux-musl


COPY . .

RUN touch server/src/main.rs client/src/main.rs common/src/lib.rs front/src/main.rs
RUN cargo build --release --package server --target x86_64-unknown-linux-musl
RUN cargo build --release --package client --target x86_64-unknown-linux-musl

WORKDIR /usr/src/rs-cmdb/front
RUN npm install
RUN trunk build --release

# Final stage
FROM alpine:3.22
WORKDIR /app

COPY --from=builder /usr/src/rs-cmdb/target/x86_64-unknown-linux-musl/release/server ./rs-cmdb-server
COPY --from=builder /usr/src/rs-cmdb/target/x86_64-unknown-linux-musl/release/client ./binary/rs-cmdb-client
COPY --from=builder /usr/src/rs-cmdb/front/dist ./dist
COPY --from=builder /usr/src/rs-cmdb/config/default.toml ./config/default.toml

EXPOSE 8080
CMD ["./rs-cmdb-server"]