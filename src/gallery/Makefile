# Generator for gallery posts
# TODO: Generate the .post file from single /gallery/drawings.lua
$(intermediate_dir1)/gallery/%.html.unpreprocessed: $(intermediate_dir0)/gallery/%.post
	$(make_dirs)
	@echo '[ GEN  ] Generating $(@:$(output_dir)%=%)'
	@lua5.4 -- '$(input_dir)/gallery/post_generator.lua' '$<' '$@'

# Preprocess some files that is specifically ended with .unpreprocessed
$(intermediate_dir2)/%.html.strings_unmerged: $(intermediate_dir1)/%.html.unpreprocessed
	$(preprocess)

# Merge the strings
$(intermediate_dir3)/%.html: $(intermediate_dir2)/%.html.strings_unmerged
	$(merge_string)

