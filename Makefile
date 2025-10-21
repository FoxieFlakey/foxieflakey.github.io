MAKEFLAGS += -rR --no-print-directory

current_dir := $(shell pwd)
output_dir 	:= $(current_dir)/output

# Just current current dir either way people can see
# on fiinal result lol
input_dir 	:= $(current_dir)/
files 			:= \
	index.html \
	gallery.html \
	favicon.ico \
	img/profile.gif \
	img/Gallery_Icon.png \
	js/error.js \
	css/index.css \
	css/navbar.css \
	css/pages/home.css
deps_dir		:= $(current_dir)/file_deps

.DEFAULT_GOAL := all
.PHONY: all
all: $(addprefix $(output_dir)/,$(files))
	@true

$(output_dir):
	@mkdir -- '$@'

$(deps_dir):
	@mkdir -- '$@'

# For html files
$(output_dir)/%.html: $(input_dir)/%.html | $(output_dir) $(deps_dir)
	@mkdir -p -- '$(dir $@)'
	@mkdir -p -- '$(dir $(@:$(output_dir)%=$(deps_dir)%).d)'
	@clang '-I$(input_dir)' '-I$(input_dir)/include' -Wno-invalid-pp-token -E -P -CC -MMD -MP -MF '$(@:$(output_dir)%=$(deps_dir)%).d' -MT '$@' -xc - < '$<' > '$@'
	@echo "[ CC   ] Preprocess $(@:$(output_dir)=)"

# For non html files
$(output_dir)/%: $(input_dir)/% | $(output_dir)
	@mkdir -p -- '$(dir $@)'
	@cp -- '$<' '$@'
	@echo "[ COPY ] Updating $(@:$(output_dir)=)"

.PHONY: clean
clean:
	@rm -rf -- '$(output_dir)' '$(deps_dir)'

.PHONY: host
host: all
	@echo "[HOST  ] Locally hosting at localhost:8080 with php's builtin webserver"
	@php -S localhost:8080 -t '$(output_dir)/' router.php

include $(wildcard $(deps_dir)/*.d)

