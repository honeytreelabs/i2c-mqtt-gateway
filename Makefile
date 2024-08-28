PROFILE ?= release

all: target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-gateway

.PHONY: target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-gateway  # dependencies checked by cargo
target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-gateway:
	export PATH="$${HOME}/x-tools/armv7-rpi2-linux-musleabihf/bin:$${PATH}"; \
	if [ "$(PROFILE)" = "debug" ]; then \
		cargo build --target armv7-unknown-linux-musleabihf; \
	else \
		cargo build --$(PROFILE) --target armv7-unknown-linux-musleabihf; \
	fi

.PHONY: target/$(PROFILE)/i2c-mqtt-gateway  # dependencies checked by cargo
target/$(PROFILE)/i2c-mqtt-gateway:
	if [ "$(PROFILE)" = "debug" ]; then \
		cargo build; \
	else \
		cargo build --$(PROFILE); \
	fi

.PHONY: run
run:
	cargo run config/config.yaml

.PHONY: test
test:
	cargo test -- --nocapture

.PHONY: lint
lint:
	cargo clippy

deploy: target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-gateway
	scp -O $< raspberry-o.lan:/tmp

.PHONY: clean
clean:
	-rm -f target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-gateway
	-rm -f target/debug/i2c-mqtt-gateway target/release/i2c-mqtt-gateway

.PHONY: distclean
distclean:
	-rm -rf target/armv7-unknown-linux-musleabihf/
	-rm -rf target/debug target/release
