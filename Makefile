MAKEFLAGS += -rR --no-print-directory

current_dir := $(shell pwd)
output_dir 	:= $(current_dir)/output
# This is site's root NOT root in host filesystem
site_root		?= /./
preprocess_flags ?=
site_host_root ?= http://localhost:8080/
giscus_category_name ?= Giscus thing for localhost:8080/
giscus_category_id	?= DIC_kwDOLoNF_M4CxmCj

# At here we can include local config which updated
# to suit each needs
ifeq ($(wildcard local-config.mk),local-config.mk)
	include local-config.mk
endif

# Just current current dir either way people can see
# on fiinal result lol
input_dir 	:= $(current_dir)/src
files 			:= \
	index.html \
	gallery/index.html \
	gallery/2025/index.html \
	gallery/2025/10/index.html \
	gallery/2025/10/micro_foxie.html \
	gallery/2025/10/micro_foxie.png \
	gallery/2025/11/index.html \
	gallery/2025/11/new_side_profile.html \
	gallery/2025/11/new_side_profile.png \
	favicon.ico \
	img/profile.gif \
	img/Gallery_Icon.png \
	img/Home_Icon.png \
	js/error.js \
	css/index.css \
	css/navbar.css \
	css/pages/home.css \
	404.html \
	css/pages/gallery_common.css \
	css/pages/gallery_post.css
deps_dir		:= $(current_dir)/file_deps

.DEFAULT_GOAL := all
.PHONY: all
all: $(addprefix $(output_dir)/,$(files))
	@true

$(output_dir):
	@mkdir -- '$@'

$(deps_dir):
	@mkdir -- '$@'

# For some files
$(output_dir)/%.js: $(input_dir)/%.js | $(output_dir) $(deps_dir)
	@mkdir -p -- '$(dir $@)'
	@mkdir -p -- '$(dir $(@:$(output_dir)%=$(deps_dir)%).d)'
	@echo "[ CC   ] Preprocess $(@:$(output_dir)=)"
	@clang '-I$(input_dir)' '-I$(input_dir)/include' '-DSITE_HOST_ROOT="$(site_host_root)"' '-DGISCUS_CATEGORY_ID="$(giscus_category_id)"' '-DGISCUS_CATEGORY_NAME="$(giscus_category_name)"' '-DSITE_ROOT="$(site_root)"' -include "include/preinclude.html" -Wno-invalid-pp-token -E -P -CC -MMD -MP -MF '$(@:$(output_dir)%=$(deps_dir)%).d' -MT '$@' $(preprocess_flags) -xc '$<' -o '$@.tmp'
	@( \
		cat '$@.tmp' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' \
	) > '$@'
	@rm '$@.tmp'

$(output_dir)/%.html: $(input_dir)/%.html | $(output_dir) $(deps_dir)
	@mkdir -p -- '$(dir $@)'
	@mkdir -p -- '$(dir $(@:$(output_dir)%=$(deps_dir)%).d)'
	@echo "[ CC   ] Preprocess $(@:$(output_dir)=)"
	@clang '-I$(input_dir)' '-I$(input_dir)/include' '-DSITE_HOST_ROOT="$(site_host_root)"' '-DGISCUS_CATEGORY_ID="$(giscus_category_id)"' '-DGISCUS_CATEGORY_NAME="$(giscus_category_name)"' '-DSITE_ROOT="$(site_root)"' -include "include/preinclude.html" -Wno-invalid-pp-token -E -P -CC -MMD -MP -MF '$(@:$(output_dir)%=$(deps_dir)%).d' -MT '$@' $(preprocess_flags) -xc '$<' -o '$@.tmp'
	@( \
		cat '$@.tmp' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' | \
		sed -E 's/"([^"]*?)"[ \t\n]"([^"]*?)"/"\1\2"/g' \
	) > '$@'
	@rm '$@.tmp'

# For files that don't need to be preprocessed
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

# See https://stackoverflow.com/questions/2483182/recursive-wildcards-in-gnu-make
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
include $(call rwildcard,$(deps_dir),*.d)

