# Generate post from drawings.lua
$(intermediate_dir0)/gallery/%.html.unpreprocessed: $(input_dir)/gallery/drawings.lua
	$(make_dirs)
	@echo '[ GEN  ] Generating $(@:$(output_dir)%=%)'
	@lua5.4 -- '$(input_dir)/gallery/post_generator.lua' '$<' '$@'

# Preprocess the result of preprocessed
$(intermediate_dir1)/gallery/%.html.strings_unmerged: $(intermediate_dir0)/gallery/%.html.unpreprocessed
	$(preprocess)

# Merge the strings
$(intermediate_dir2)/gallery/%.html: $(intermediate_dir1)/gallery/%.html.strings_unmerged
	$(merge_string)

