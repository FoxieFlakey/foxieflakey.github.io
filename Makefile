MAKEFLAGS += -rR --no-print-directory

files := \
	index.html \
	css/global.css \
	css/pages/home.css \
	img/profile.gif

root ?= http://localhost:8080/
current_dir := $(shell pwd)
output_dir := $(current_dir)/output
web_dir := $(output_dir)/web
input_dir := $(current_dir)/web
deps_dir := $(output_dir)/deps

.DEFAULT_GOAL := all
.PHONY: all
all: create_dirs .WAIT $(addprefix $(web_dir)/,$(files))
	@true

$(web_dir)/%: $(input_dir)/%
	@echo "[ COPY ] $<"
	@mkdir -p -- "$(dir $(@))"
	@cp -- "$<" "$@"

$(web_dir)/%.html: $(input_dir)/%.html $(current_dir)/target/release/html_preprocess
	@echo "[ RUST ] Preprocessing '$@'"
	@mkdir -p -- "$(dir $(@))"
	@$(current_dir)/target/release/html_preprocess \
		"-Droot=$(root)" \
		-s "$(input_dir)" \
		--makefile-depedency ""$(patsubst $(web_dir)/%,$(deps_dir)/%.d,$@)"" \
		-m "$(patsubst $(web_dir)/%,%,$@)" "$@"

# The HTML preprocesser i made
$(current_dir)/target/release/html_preprocess:
	@echo "[ CARGO ] Building preprocessor"
	@cargo build --release

.PHONY: watch_host
watch_host: all
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

.PHONY: create_dirs
create_dirs:
	@mkdir -p -- "$(output_dir)"
	@mkdir -p -- "$(web_dir)"
	@mkdir -p -- "$(deps_dir)"

.PHONY: clean
clean:
	@echo "[ RM   ] Removing $(output_dir)"
	@rm -rf -- "$(output_dir)"
	@echo "[ CARGO ] Cargo cleaning"
	@cargo clean

.PHONY: host
host: all
	@echo "[HOST  ] Locally hosting at localhost:8080 with php's builtin webserver"
	@php -S localhost:8080 -t '$(web_dir)/' router.php

# See https://stackoverflow.com/questions/2483182/recursive-wildcards-in-gnu-make
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
include $(call rwildcard,$(deps_dir),*.d)

# Also include Rust's makefile deps
-include $(current_dir)/target/release/html_preprocess.d

