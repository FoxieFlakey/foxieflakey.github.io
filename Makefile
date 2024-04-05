MAKEFLAGS += -rR --no-print-directory

current_dir := $(shell pwd)
output_dir 	:= $(current_dir)/output
input_dir 	:= $(current_dir)/site
files 			:= $(shell find '$(input_dir)' -type f -not -name '*.inc.html' -printf '$(output_dir)/%P\n')
deps_dir		:= $(current_dir)/file_deps

.DEFAULT_GOAL := all
.PHONY: all
all: $(files)
	@true

$(output_dir):
	@mkdir -- '$@'

$(deps_dir):
	@mkdir -- '$@'

# For html files
$(output_dir)/%.html: $(input_dir)/%.html | $(output_dir) $(deps_dir)
	@mkdir -p -- '$(dir $@)'
	@mkdir -p -- '$(dir $(@:$(output_dir)%=$(deps_dir)%).d)'
	@clang '-I$(input_dir)' -Wno-invalid-pp-token -E -P -CC -MMD -MP -MF '$(@:$(output_dir)%=$(deps_dir)%).d' -MT '$@' -xc - < '$<' > '$@'
	@echo "[ CC   ] Preprocess $(@:$(output_dir)=)"

# For non html files
$(output_dir)/%: $(input_dir)/% | $(output_dir)
	@mkdir -p -- '$(dir $@)'
	@cp -- '$<' '$@'
	@echo "[ COPY ] Updating $(@:$(output_dir)=)"

.PHONY: clean
clean:
	@rm -rf -- '$(output_dir)' '$(deps_dir)'

include $(wildcard $(deps_dir)/*)

