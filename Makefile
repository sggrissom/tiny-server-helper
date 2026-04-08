.PHONY: build deploy

APP_NAME    := metrics-server
DEPLOY_HOST := vps
BINARY      := target/release/metrics-server

build:
	cargo build --release -p metrics-server

deploy: build
	./bin/deploy $(APP_NAME) $(DEPLOY_HOST) $(BINARY) internal
