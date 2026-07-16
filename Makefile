MAKEFLAGS += -rR --no-print-directory

root ?= http://localhost:8080/
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

# This rule Gemini's AI slop generated with few changes
.PHONY: watch_host
watch: $(web_binary)
	@echo "[ WATCH ] Watching"
	@SERVER_PID=""; \
	trap 'if [ -n "$$SERVER_PID" ]; then kill $$SERVER_PID 2>/dev/null; fi; exit' INT TERM; \
	$(MAKE) host & SERVER_PID=$$!; \
	while true; do \
		if ! $(MAKE) -q $(addprefix $(web_dir)/,$(files)); then \
			echo "[ WATCH ] There changes, rebuilding"; \
			$(MAKE) all; \
			if [ -n "$$SERVER_PID" ]; then \
				kill $$SERVER_PID 2> /dev/null || true; \
			fi; \
			wait "$$SERVER_PID" 2> /dev/null; \
			$(MAKE) host & SERVER_PID=$$!; \
		fi; \
		sleep 0.5; \
	done

$(web_dumped_cookie): $(web_binary)
	$(web_binary) dump "$(root)" "$(web_dir)"
	@touch $(web_dumped_cookie)

.PHONY: host
host: $(web_binary)
	$(web_binary) serve 127.0.0.1 8080

.PHONY: clean
clean:
	cargo clean
	rm -rf $(output_dir)

$(web_binary):
	cargo build --release --package web

# Also include Rust's makefile deps
-include $(web_binary_dep)

