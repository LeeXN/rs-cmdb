# Project variables
APP_NAME := rs-cmdb
DOCKER_IMAGE := $(APP_NAME)
MUSL_TARGET := x86_64-unknown-linux-musl

.PHONY: all build build-server build-client build-front \
        build-musl build-server-musl build-client-musl \
        test lint docker clean help

# Default target
all: test build

# Build all components (glibc)
build: build-server build-client build-front

# Build Server (glibc)
build-server:
	@echo "Building Server..."
	cargo build --release --package server

# Build Client (glibc)
build-client:
	@echo "Building Client..."
	cargo build --release --package client

# Build all components (musl static)
build-musl: build-server-musl build-client-musl

# Build Server (musl static)
build-server-musl:
	@echo "Building Server (static musl)..."
	cargo build --release --package server --target $(MUSL_TARGET)

# Build Client (musl static)
build-client-musl:
	@echo "Building Client (static musl)..."
	cargo build --release --package client --target $(MUSL_TARGET)

# Build Frontend
build-front:
	@echo "Building Frontend..."
	cd front && npm install && trunk build --release

# Run Unit Tests
test:
	@echo "Running Unit Tests..."
	cargo test --workspace

# Run clippy (matching CI: --all-targets --all-features -- -D warnings)
lint:
	@echo "Running Clippy Linting (matching CI)..."
	cargo clippy --all-targets --all-features -- -D warnings

# Build Docker Image
docker:
	@echo "Building Docker Image..."
	docker build -t $(DOCKER_IMAGE):latest .

# Clean build artifacts
clean:
	@echo "Cleaning artifacts..."
	cargo clean
	rm -rf dist
	cd front && rm -rf dist pkg node_modules

# Help command
help:
	@echo "Available commands:"
	@echo "  make all            - Run tests and build everything (glibc)"
	@echo "  make build          - Build server, client and frontend (glibc)"
	@echo "  make build-server   - Build only the server (glibc)"
	@echo "  make build-client   - Build only the client (glibc)"
	@echo "  make build-musl     - Build server and client (static musl)"
	@echo "  make build-server-musl - Build only the server (static musl)"
	@echo "  make build-client-musl - Build only the client (static musl)"
	@echo "  make build-front    - Build only the frontend"
	@echo "  make test           - Run unit tests"
	@echo "  make docker         - Build Docker image"
	@echo "  make clean          - Clean build artifacts"
