MAKEFLAGS += -rR --no-print-directory

include base_config.mk
-include local_config.mk

current_dir := $(shell pwd)
output_dir := $(current_dir)/output
web_dir := $(output_dir)/web

web_dumped_cookie := $(output_dir)/.dumped
web_binary := $(current_dir)/target/release/web
web_binary_dep := $(current_dir)/target/release/web.d

.DEFAULT_GOAL := all
.PHONY: all
all: $(web_dumped_cookie)
	@true

.PHONY: watch_host
watch: $(web_binary)
	$(web_binary) serve --web-src-dir "$(current_dir)/web/data" 127.0.0.1 8080

$(web_dumped_cookie): $(web_binary)
	$(web_binary) dump "$(config_root)" "$(web_dir)"
	@touch $(web_dumped_cookie)

.PHONY: host
host: $(web_binary)
	$(web_binary) serve 127.0.0.1 8080

.PHONY: clean
clean:
	cargo clean
	rm -rf $(output_dir)

# Cargo know whats changed, so ehh lets make this PHONY
.PHONY: $(web_binary)
$(web_binary):
	cargo build --release --package web --features "$(config_features)"


