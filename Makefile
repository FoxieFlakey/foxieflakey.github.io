MAKEFLAGS += -rR --no-print-directory

files := \
	index.html

root ?= https://localhost:8080/
current_dir := $(shell pwd)
output_dir := $(current_dir)/output
web_dir := $(output_dir)/web
input_dir := $(current_dir)/web

.DEFAULT_GOAL := all
.PHONY: all
all: create_dirs .WAIT $(addprefix $(web_dir)/,$(files))
	@true

$(output_dir)/%: $(input_dir)/%
	@echo "[ COPY ] $<"
	@cp -- "$<" "$@"

$(web_dir)/%.html: $(input_dir)/%.html $(current_dir)/target/release/html_preprocess
	@echo "[ RUST ] Preprocessing '$@'"
	@$(current_dir)/target/release/html_preprocess "-Droot=$(root)" -s "$(input_dir)" -m "$(patsubst $(web_dir)/%,%,$@)" "$@"

# The HTML preprocesser i made
$(current_dir)/target/release/html_preprocess:
	cargo build --release

.PHONY: create_dirs
create_dirs:
	@mkdir -p -- "$(output_dir)"
	@mkdir -p -- "$(web_dir)"

.PHONY: clean
clean:
	@echo "[ RM   ] Removing $(output_dir)"
	@rm -rf -- "$(output_dir)"

.PHONY: host
host: all
	@echo "[HOST  ] Locally hosting at localhost:8080 with php's builtin webserver"
	@php -S localhost:8080 -t '$(web_dir)/' router.php

