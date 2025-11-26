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
deps_dir		:= $(output_dir)/file_deps
web_dir			:= $(output_dir)/web

export deps_dir
export web_dir
export current_dir
export input_dir
export output_dir
export site_root
export site_host_root
export giscus_category_name
export giscus_category_id

files 			:= \
	index.html \
	gallery/index.html \
	gallery/2025/index.html \
	gallery/2025/10/index.html \
	gallery/2025/10/micro_foxie.html \
	gallery/2025/10/micro_foxie.png \
	gallery/2025/10/wink_foxie.html \
	gallery/2025/10/wink_foxie.png \
	gallery/2025/10/sofa_foxie.html \
	gallery/2025/10/sofa_foxie.png \
	gallery/2025/10/animation_slurrrp.html \
	gallery/2025/10/animation_slurrrp.gif \
	gallery/2025/10/sometime_idk.html \
	gallery/2025/10/sometime_idk.png \
	gallery/2025/10/a_lil_website.html \
	gallery/2025/10/a_lil_website.png \
	gallery/2025/10/animation_hiiii.html \
	gallery/2025/10/animation_hiiii.gif \
	gallery/2025/11/index.html \
	gallery/2025/11/new_side_profile.html \
	gallery/2025/11/new_side_profile.png \
	gallery/2025/11/triangle_hungry_blank.html \
	gallery/2025/11/triangle_hungry_blank.png \
	gallery/2025/11/door_mat_foxie.html \
	gallery/2025/11/door_mat_foxie.png \
	favicon.ico \
	favicon_for_opengraph.png \
	img/profile.gif \
	img/Gallery_Icon.png \
	img/Home_Icon.png \
	js/error.js \
	js/auto-fit-iframe.js \
	css/index.css \
	css/navbar.css \
	css/pages/home.css \
	404.html \
	css/pages/gallery_common.css \
	css/pages/gallery_post.css

.DEFAULT_GOAL := all
.PHONY: all
all: create_dirs .WAIT $(addprefix $(web_dir)/,$(files))
	@true

.PHONY: create_dirs
create_dirs:
	@mkdir -p -- "$(web_dir)"
	@mkdir -p -- "$(deps_dir)"

define make_dirs
	@mkdir -p -- '$(dir $@)'
	@mkdir -p -- '$(dir $(@:$(web_dir)%=$(deps_dir)%))'
endef

define preprocess
	$(make_dirs)
	@echo "[ CC   ] Preprocess $(@:$(web_dir)=)"
	@clang '-I$(input_dir)' '-I$(input_dir)/include' '-DSITE_HOST_ROOT="$(site_host_root)"' '-DGISCUS_CATEGORY_ID="$(giscus_category_id)"' '-DGISCUS_CATEGORY_NAME="$(giscus_category_name)"' '-DSITE_ROOT="$(site_root)"' -include "include/preinclude.html" -Wno-invalid-pp-token -E -P -CC -MMD -MP -MF '$(@:$(web_dir)%=$(deps_dir)%).d' -MT '$@' $(preprocess_flags) -xc '$<' -o '$@.tmp'
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
endef

# For some files
$(web_dir)/%.js: $(input_dir)/%.js
	$(preprocess)
$(web_dir)/%.html: $(input_dir)/%.html
	$(preprocess)

# For files that don't need to be preprocessed
$(web_dir)/%: $(input_dir)/%
	$(make_dirs)
	@cp -- '$<' '$@'
	@echo "[ COPY ] Updating $(@:$(web_dir)=)"

.PHONY: clean
clean:
	@rm -rf -- '$(output_dir)'

.PHONY: host
host: all
	@echo "[HOST  ] Locally hosting at localhost:8080 with php's builtin webserver"
	@php -S localhost:8080 -t '$(web_dir)/' router.php

# See https://stackoverflow.com/questions/2483182/recursive-wildcards-in-gnu-make
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
include $(call rwildcard,$(deps_dir),*.d)

