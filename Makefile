# Project variables
APP_NAME := rs-cmdb
DOCKER_IMAGE := $(APP_NAME)

.PHONY: all build build-server build-client build-front test docker clean help

# Default target
all: test build

# Build all components
build: build-server build-client build-front

# Build Server
build-server:
	@echo "Building Server..."
	cargo build --release --package server

# Build Client
build-client:
	@echo "Building Client..."
	cargo build --release --package client

# Build Frontend
build-front:
	@echo "Building Frontend..."
	cd front && npm install && trunk build --release

# Run Unit Tests
test:
	@echo "Running Unit Tests..."
	cargo test --workspace

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
	@echo "  make all          - Run tests and build everything"
	@echo "  make build        - Build server, client and frontend"
	@echo "  make build-server - Build only the server"
	@echo "  make build-client - Build only the client"
	@echo "  make build-front  - Build only the frontend"
	@echo "  make test         - Run unit tests"
	@echo "  make docker       - Build Docker image"
	@echo "  make clean        - Clean build artifacts"
