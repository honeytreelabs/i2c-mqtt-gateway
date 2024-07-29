PROFILE ?= release

all: target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-bridge 

.PHONY: target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-bridge  # dependencies checked by cargo
target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-bridge:
	export PATH="$${HOME}/x-tools/armv7-rpi2-linux-musleabihf/bin:$${PATH}"; \
	if [ "$(PROFILE)" = "debug" ]; then \
		cargo build --target armv7-unknown-linux-musleabihf; \
	else \
		cargo build --$(PROFILE) --target armv7-unknown-linux-musleabihf; \
	fi

.PHONY: clean
clean:
	-rm -rf target/armv7-unknown-linux-musleabihf/$(PROFILE)/i2c-mqtt-bridge

.PHONY: dist-clean
dist-clean:
	-rm -rf target/armv7-unknown-linux-musleabihf/
