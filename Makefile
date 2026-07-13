MAKEFLAGS += -rR --no-print-directory

files := \
	index.html

root ?= https://localhost:8080/
current_dir := $(shell pwd)
output_dir := $(current_dir)/output
web_dir := $(output_dir)/web
input_dir := $(current_dir)/web
deps_dir := $(output_dir)/deps

.DEFAULT_GOAL := all
.PHONY: all
all: create_dirs .WAIT $(addprefix $(web_dir)/,$(files))
	@true

$(output_dir)/%: $(input_dir)/%
	@echo "[ COPY ] $<"
	@cp -- "$<" "$@"

$(web_dir)/%.html: $(input_dir)/%.html $(current_dir)/target/release/html_preprocess
	@echo "[ RUST ] Preprocessing '$@'"
	@$(current_dir)/target/release/html_preprocess \
		"-Droot=$(root)" \
		-s "$(input_dir)" \
		--makefile-depedency ""$(patsubst $(web_dir)/%,$(deps_dir)/%.d,$@)"" \
		-m "$(patsubst $(web_dir)/%,%,$@)" "$@"

# The HTML preprocesser i made
$(current_dir)/target/release/html_preprocess:
	@echo "[ CARGO ] Building preprocessor"
	@cargo build --release

.PHONY: create_dirs
create_dirs:
	@mkdir -p -- "$(output_dir)"
	@mkdir -p -- "$(web_dir)"
	@mkdir -p -- "$(deps_dir)"

.PHONY: clean
clean:
	@echo "[ RM   ] Removing $(output_dir)"
	@rm -rf -- "$(output_dir)"

.PHONY: host
host: all
	@echo "[HOST  ] Locally hosting at localhost:8080 with php's builtin webserver"
	@php -S localhost:8080 -t '$(web_dir)/' router.php

# See https://stackoverflow.com/questions/2483182/recursive-wildcards-in-gnu-make
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
include $(call rwildcard,$(deps_dir),*.d)

# Also include Rust's makefile deps
-include $(current_dir)/target/release/html_preprocess.d

