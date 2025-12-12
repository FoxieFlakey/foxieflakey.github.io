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

# Intermediate directories containing half complete data
# 0 mean its the content is closer to the input
# while larger is closer to the output
intermediate_dir0 := $(output_dir)/intermediate/0
intermediate_dir1 := $(output_dir)/intermediate/1
intermediate_dir2 := $(output_dir)/intermediate/2
intermediate_dir3 := $(output_dir)/intermediate/3
intermediate_dir4 := $(output_dir)/intermediate/4

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
	\
	gallery/2025/12/muted_wish.html \
	gallery/2025/12/muted_wish.png \
	\
	gallery/2025/12/index.html \
	\
	gallery/2025/11/door_mat_foxie.html \
	gallery/2025/11/door_mat_foxie.png \
	\
	gallery/2025/11/triangle_hungry_blank.html \
	gallery/2025/11/triangle_hungry_blank.png \
	\
	gallery/2025/11/new_side_profile.html \
	gallery/2025/11/new_side_profile.png \
	\
	gallery/2025/11/index.html \
	\
	gallery/2025/10/micro_foxie.html \
	gallery/2025/10/micro_foxie.png \
	\
	gallery/2025/10/wink_foxie.html \
	gallery/2025/10/wink_foxie.png \
	\
	gallery/2025/10/sofa_foxie.html \
	gallery/2025/10/sofa_foxie.png \
	\
	gallery/2025/10/animation_slurrrp.html \
	gallery/2025/10/animation_slurrrp.gif \
	\
	gallery/2025/10/sometime_idk.html \
	gallery/2025/10/sometime_idk.png \
	\
	gallery/2025/10/a_lil_website.html \
	gallery/2025/10/a_lil_website.png \
	\
	gallery/2025/10/animation_hiiii.html \
	gallery/2025/10/animation_hiiii.gif \
	\
	gallery/2025/10/index.html \
	\
	gallery/2025/index.html \
	\
	gallery/index.html \
	\
	favicon.ico \
	favicon_for_opengraph.png \
	img/profile.gif \
	img/Gallery_Icon.png \
	img/Home_Icon.png \
	img/parasitic_popup/left_arrow.png \
	img/parasitic_popup/right_arrow.png \
	js/error.js \
	js/auto-fit-iframe.js \
	css/index.css \
	css/navbar.css \
	css/pages/home.css \
	404.html \
	css/pages/gallery_common.css \
	css/pages/gallery_post.css \
	css/parasitic_popup.css \
	 \
	creations/index.html \
	 \
	creations/CSS_animation/index.html \
	creations/CSS_animation/popup.css \
	creations/CSS_animation/img/left_arrow.png \
	creations/CSS_animation/img/right_arrow.png \
	 \
	creations/InteraksiManusiaKomputer_ServerDashboard/index.html \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/favicon.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/green_button.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/red_button.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/save_icon.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/plus_icon.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/checkmark_inactive.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/checkmark_active.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/foxie_icon.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/red_button.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/x_icon.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/question_icon.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerCluster_Failed.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerCluster_Warning.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerRack_Failed.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerRack_Warning.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerRack.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerCluster.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServiceStatus_Warning.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/ServiceStatus_Failed.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/CPU_Chart.svg \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/Network_Chart.svg \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/Restart.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/Shutdown.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/AdminPfp.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/Edit.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/img/Placeholder.png \
	creations/InteraksiManusiaKomputer_ServerDashboard/css/global.css \
	creations/InteraksiManusiaKomputer_ServerDashboard/css/font.ttf \
	\
	college_stuff/img/favicon.png \
	college_stuff/img/foxie_icon.png \
	college_stuff/img/cookie_window.png \
	college_stuff/img/question_icon.png \
	college_stuff/img/x_icon.png \
	college_stuff/img/checkmark_active.png \
	college_stuff/img/save_icon.png \
	college_stuff/img/plus_icon.png \
	college_stuff/img/green_button.png \
	college_stuff/img/red_button.png \
	college_stuff/img/checkmark_inactive.png

export input_dir
export output_dir
export site_root

.DEFAULT_GOAL := all
.PHONY: all
all: create_dirs .WAIT $(addprefix $(web_dir)/,$(files))
	@true

.PHONY: create_dirs
create_dirs:
	@mkdir -p -- "$(web_dir)"
	@mkdir -p -- "$(deps_dir)"
	@mkdir -p -- "$(intermediate_dir0)"
	@mkdir -p -- "$(intermediate_dir1)"
	@mkdir -p -- "$(intermediate_dir2)"
	@mkdir -p -- "$(intermediate_dir3)"
	@mkdir -p -- "$(intermediate_dir4)"

define make_dirs
	@mkdir -p -- '$(dir $@)'
	@mkdir -p -- '$(dir $(@:$(output_dir)%=$(deps_dir)%))'
	@mkdir -p -- '$(dir $(@:$(web_dir)%=$(intermediate_dir0)%))'
	@mkdir -p -- '$(dir $(@:$(web_dir)%=$(intermediate_dir1)%))'
	@mkdir -p -- '$(dir $(@:$(web_dir)%=$(intermediate_dir2)%))'
	@mkdir -p -- '$(dir $(@:$(web_dir)%=$(intermediate_dir4)%))'
endef

define preprocess
	$(make_dirs)
	@echo "[ CC   ] Preprocess $(@:$(output_dir)%=%)"
	@clang '-I$(input_dir)' \
		'-I$(input_dir)/include' \
		'-DSITE_HOST_ROOT="$(site_host_root)"' \
		'-DGISCUS_CATEGORY_ID="$(giscus_category_id)"' \
		'-DGISCUS_CATEGORY_NAME="$(giscus_category_name)"' \
		'-DSITE_ROOT="$(site_root)"' \
		-include "include/preinclude.html" \
		-Wno-invalid-pp-token \
		-E \
		-P \
		-CC \
		-MMD \
		-MP \
		-MF '$(@:$(output_dir)%=$(deps_dir)%).d' \
		-MT \
		'$@' \
		$(preprocess_flags) \
		-xc '$<' -o '$@'
endef

define merge_string
	$(make_dirs)
	@( \
		cat '$<' | \
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
endef

# Move along the data for intermediate directory
.NOTINTERMEDIATE:
$(intermediate_dir0)/%: $(input_dir)/%
	$(make_dirs)
	@ln -f '$<' '$@'
$(intermediate_dir1)/%: $(intermediate_dir0)/%
	$(make_dirs)
	@ln -f '$<' '$@'
$(intermediate_dir2)/%: $(intermediate_dir1)/%
	$(make_dirs)
	@ln -f '$<' '$@'
$(intermediate_dir3)/%: $(intermediate_dir2)/%
	$(make_dirs)
	@ln -f '$<' '$@'
$(intermediate_dir4)/%: $(intermediate_dir3)/%
	$(make_dirs)
	@ln -f '$<' '$@'

# Finally on output do copy to ensure symlink resolved
$(web_dir)/%: $(intermediate_dir4)/%
	$(make_dirs)
	@cp --dereference -- '$<' '$@'

# Merge the strings
$(intermediate_dir4)/%.js: $(intermediate_dir3)/%.js
	$(merge_string)
$(intermediate_dir4)/%.html: $(intermediate_dir3)/%.html
	$(merge_string)

# Preprocess the JS and HTML
$(intermediate_dir0)/%.js: $(input_dir)/%.js
	$(preprocess)
$(intermediate_dir0)/%.html: $(input_dir)/%.html
	$(preprocess)

include $(input_dir)/gallery/Makefile

# For files that don't need to be preprocessed
$(intermediate_dir0)/%: $(input_dir)/%
	$(make_dirs)
	@cp -- '$<' '$@'
	@echo "[ COPY ] Updating $(@:$(intermediate_dir0)=)"

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

