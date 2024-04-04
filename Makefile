MAKEFLAGS += -rR --no-print-directory

output_dir := ./output
input_dir := ./site

.DEFAULT_GOAL := all
all: $(shell find '$(input_dir)' -type f -printf '$(output_dir)/%P\n')
	@true

$(output_dir):
	mkdir -- '$@'

# For html files
$(output_dir)/%.html: $(input_dir)/%.html | $(output_dir)
	clang '-I$(input_dir)' -Wno-invalid-pp-token -E -P -xc - < '$<' > '$@'

# For non html files
$(output_dir)/%: $(input_dir)/% | $(output_dir)
	@mkdir -p -- '$(dir "$@")'
	cp --reflink -- '$<' '$@'

clean:
	rm -rf -- '$(output_dir)'

