MAKEFLAGS += -rR --no-print-directory

current_dir := $(shell pwd)
output_dir 	:= $(current_dir)/output
input_dir 	:= $(current_dir)/site
files 			:= $(shell find '$(input_dir)' -type f -not -name '*.inc.html' -printf '$(output_dir)/%P\n')

.DEFAULT_GOAL := all
.PHONY: all
all: $(files)
	@true

$(output_dir):
	mkdir -- '$@'

# For html files
$(output_dir)/%.html: $(input_dir)/%.html | $(output_dir)
	clang '-I$(input_dir)' -Wno-invalid-pp-token -E -P -xc - < '$<' > '$@'

# For non html files
$(output_dir)/%: $(input_dir)/% | $(output_dir)
	@mkdir -p -- '$(dir $@)'
	cp -- '$<' '$@'

.PHONY: clean
clean:
	rm -rf -- '$(output_dir)'

