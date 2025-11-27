# Generate the main index.html for gallery
$(intermediate_dir0)/gallery/index.html.unpreprocessed: $(input_dir)/gallery/index.html $(input_dir)/gallery/drawings.lua $(input_dir)/gallery/main_list_generator.lua
	$(make_dirs)
	@echo '[ GEN  ] Generating $(@:$(output_dir)%=%)'
	@lua5.4 -- '$(input_dir)/gallery/main_list_generator.lua' '$<' '$@' '$(input_dir)/gallery/drawings.lua'

# Generate monthly listing from drawings.lua
$(intermediate_dir0)/gallery/%/index.html.unpreprocessed: $(input_dir)/gallery/drawings.lua $(input_dir)/gallery/list_generator.lua
	$(make_dirs)
	@echo '[ GEN  ] Generating $(@:$(output_dir)%=%)'
	@lua5.4 -- '$(input_dir)/gallery/list_generator.lua' '$<' '$@'

# Generate post from drawings.lua
$(intermediate_dir0)/gallery/%.html.unpreprocessed: $(input_dir)/gallery/drawings.lua $(input_dir)/gallery/post_generator.lua
	$(make_dirs)
	@echo '[ GEN  ] Generating $(@:$(output_dir)%=%)'
	@lua5.4 -- '$(input_dir)/gallery/post_generator.lua' '$<' '$@'

# Preprocess the result of preprocessed
$(intermediate_dir1)/gallery/%.html.strings_unmerged: $(intermediate_dir0)/gallery/%.html.unpreprocessed
	$(preprocess)

# Merge the strings
$(intermediate_dir2)/gallery/%.html: $(intermediate_dir1)/gallery/%.html.strings_unmerged
	$(merge_string)

