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
        robots.txt \
        \
        googlece00914e0cd14977.html \
	\
	gallery/2026/index.html \
	\
	gallery/2026/1/index.html \
	\
	gallery/2026/1/new_year_2026.html \
	gallery/2026/1/new_year_2026.png \
	gallery/2026/1/flattened_foxie.png \
	gallery/2026/1/hammock_goober.html \
	gallery/2026/1/hammock_goober.png \
	gallery/2026/1/boots_foxie.html \
	gallery/2026/1/boots_foxie.png \
	gallery/2026/1/coned_foxie.html \
	gallery/2026/1/coned_foxie.png \
	gallery/2026/1/pancake_foxie.html \
	gallery/2026/1/pancake_foxie.png \
	gallery/2026/1/goober_new_foxie_skirt.html \
	gallery/2026/1/goober_new_foxie_skirt.png \
	gallery/2026/1/mail_goober_the_vulpix.html \
	gallery/2026/1/mail_goober_the_vulpix.png \
	gallery/2026/1/happy_new_year_2026_from_blankface_foxie.html \
	gallery/2026/1/happy_new_year_2026_from_blankface_foxie.png \
	gallery/2026/1/perma_grin_foxie.html \
	gallery/2026/1/perma_grin_foxie.png \
	gallery/2025/index.html \
	\
	gallery/2025/12/index.html \
	\
	gallery/2025/12/catto_angelico_and_foxie_flakey_introduction.html \
	gallery/2025/12/catto_angelico_and_foxie_flakey_introduction.png \
	gallery/2025/12/furball_foxie.html \
	gallery/2025/12/furball_foxie.png \
	gallery/2025/12/erased_face_foxie.html \
	gallery/2025/12/erased_face_foxie.png \
	gallery/2025/12/gift_box_foxie.html \
	gallery/2025/12/gift_box_foxie.png \
	gallery/2025/12/spooked_foxie.html \
	gallery/2025/12/spooked_foxie.png \
	gallery/2025/12/statue_foxie.html \
	gallery/2025/12/statue_foxie.png \
	gallery/2025/12/beachball_foxie.html \
	gallery/2025/12/beachball_foxie.png \
	gallery/2025/12/waffle_foxie.html \
	gallery/2025/12/waffle_foxie.png \
	gallery/2025/12/muted_wish.html \
	gallery/2025/12/muted_wish.png \
	gallery/2025/11/index.html \
	\
	gallery/2025/11/door_mat_foxie.html \
	gallery/2025/11/door_mat_foxie.png \
	gallery/2025/11/triangle_hungry_blank.html \
	gallery/2025/11/triangle_hungry_blank.png \
	gallery/2025/11/new_side_profile.html \
	gallery/2025/11/new_side_profile.png \
	gallery/2025/10/index.html \
	\
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
	gallery/2025/10/cursor_foxie_64x64px.html \
	gallery/2025/10/cursor_foxie_64x64px.png \
	gallery/2025/10/cursor_foxie_more.html \
	gallery/2025/10/cursor_foxie_more.png \
	gallery/2025/10/cursor_foxie.html \
	gallery/2025/10/cursor_foxie.png \
	gallery/2025/9/index.html \
	\
	gallery/2025/9/floor_tile_foxie_v2.html \
	gallery/2025/9/floor_tile_foxie_v2.png \
	gallery/2025/9/floor_tile.html \
	gallery/2025/9/floor_tile.png \
	gallery/2025/9/ate_a_bomb_v3.html \
	gallery/2025/9/ate_a_bomb_v3.png \
	gallery/2025/9/ate_a_bomb_v2.html \
	gallery/2025/9/ate_a_bomb_v2.png \
	gallery/2025/9/ate_a_bomb.html \
	gallery/2025/9/ate_a_bomb.png \
	gallery/2025/9/toaster_foxie.html \
	gallery/2025/9/toaster_foxie.png \
	gallery/2025/9/headless.html \
	gallery/2025/9/headless.png \
	gallery/2025/9/sleeping.html \
	gallery/2025/9/sleeping.png \
	gallery/2025/9/ice_cream_and_foxie.html \
	gallery/2025/9/ice_cream_and_foxie.png \
	gallery/2025/8/index.html \
	\
	gallery/2025/8/muted_foxie_v2.html \
	gallery/2025/8/muted_foxie_v2.png \
	gallery/2025/8/muted_foxie.html \
	gallery/2025/8/muted_foxie.png \
	gallery/2025/8/eating_ice_cream.html \
	gallery/2025/8/eating_ice_cream.png \
	gallery/2025/8/trumpet_head.html \
	gallery/2025/8/trumpet_head.png \
	gallery/2025/8/pool_toy_foxie.html \
	gallery/2025/8/pool_toy_foxie.png \
	gallery/2025/8/umbrella_foxie.html \
	gallery/2025/8/umbrella_foxie.png \
	gallery/2025/8/stabby.html \
	gallery/2025/8/stabby.png \
	gallery/2025/8/ice_cream_mavagk.html \
	gallery/2025/8/ice_cream_mavagk.png \
	gallery/2025/7/index.html \
	\
	gallery/2025/7/zipped_mouth.html \
	gallery/2025/7/zipped_mouth.png \
	gallery/2025/7/artfight_tanuki_huggy.html \
	gallery/2025/7/artfight_tanuki_huggy.png \
	gallery/2025/6/index.html \
	\
	gallery/2025/6/artfight_card.html \
	gallery/2025/6/artfight_card.png \
	gallery/2025/6/socks.html \
	gallery/2025/6/socks.png \
	gallery/2025/6/tall_foxie.html \
	gallery/2025/6/tall_foxie.png \
	gallery/2025/5/index.html \
	\
	gallery/2025/5/cursed_flower.html \
	gallery/2025/5/cursed_flower.png \
	gallery/2025/5/swapped_outfit.html \
	gallery/2025/5/swapped_outfit.png \
	gallery/2025/5/leaf_hand.html \
	gallery/2025/5/leaf_hand.png \
	gallery/2025/5/mice_stole_foxie_fries.html \
	gallery/2025/5/mice_stole_foxie_fries.png \
	gallery/2025/5/foxie_paws.html \
	gallery/2025/5/foxie_paws.png \
	gallery/2025/5/washing_machine_foxie.html \
	gallery/2025/5/washing_machine_foxie.png \
	gallery/2025/5/bunny_foxie.html \
	gallery/2025/5/bunny_foxie.png \
	gallery/2025/5/drawer_foxie.html \
	gallery/2025/5/drawer_foxie.png \
	gallery/2025/5/side_view.html \
	gallery/2025/5/side_view.png \
	gallery/2025/4/index.html \
	\
	gallery/2025/4/me.html \
	gallery/2025/4/me.png \
	gallery/2025/4/floofie_law.html \
	gallery/2025/4/floofie_law.png \
	gallery/2025/4/foxie_field.html \
	gallery/2025/4/foxie_field.png \
	gallery/2025/4/lineless_foxie.html \
	gallery/2025/4/lineless_foxie.png \
	gallery/2025/4/nerd_foxie.html \
	gallery/2025/4/nerd_foxie.png \
	gallery/2025/4/cabinet_foxie.html \
	gallery/2025/4/cabinet_foxie.png \
	gallery/2025/3/index.html \
	\
	gallery/2025/3/paw.html \
	gallery/2025/3/paw.png \
	gallery/2025/3/lots_of_collars.html \
	gallery/2025/3/lots_of_collars.png \
	gallery/2025/3/foxie_thick_whisker.html \
	gallery/2025/3/foxie_thick_whisker.png \
	gallery/2025/3/long_socks.html \
	gallery/2025/3/long_socks.png \
	gallery/2025/3/f_foxie.html \
	gallery/2025/3/f_foxie.png \
	gallery/2025/3/four_eyes.html \
	gallery/2025/3/four_eyes.png \
	gallery/2025/3/chest_foxie_icon.html \
	gallery/2025/3/chest_foxie_icon.png \
	gallery/2025/3/chest_foxie_icon_orig.html \
	gallery/2025/3/chest_foxie_icon_orig.png \
	gallery/2025/3/firework_foxie.html \
	gallery/2025/3/firework_foxie.png \
	gallery/2025/3/large_scale_terrain.html \
	gallery/2025/3/large_scale_terrain.png \
	gallery/2025/3/i_want_food.html \
	gallery/2025/3/i_want_food.png \
	gallery/2025/3/foxie_looking_for_fooood.html \
	gallery/2025/3/foxie_looking_for_fooood.png \
	gallery/2025/3/stomach_faced.html \
	gallery/2025/3/stomach_faced.png \
	gallery/2025/2/index.html \
	\
	gallery/2025/2/comfy_blanket.html \
	gallery/2025/2/comfy_blanket.png \
	gallery/2025/2/bone_from_foxie.html \
	gallery/2025/2/bone_from_foxie.png \
	gallery/2025/2/long_neck.html \
	gallery/2025/2/long_neck.png \
	gallery/2025/2/blep.html \
	gallery/2025/2/blep.png \
	gallery/2025/2/clothes_for_foxie.html \
	gallery/2025/2/clothes_for_foxie.png \
	gallery/2025/1/index.html \
	\
	gallery/2025/1/pawbs.html \
	gallery/2025/1/pawbs.png \
	gallery/2025/1/sleeping_under_tree_v2.html \
	gallery/2025/1/sleeping_under_tree_v2.png \
	gallery/2025/1/foxie_thinking_on_ground.html \
	gallery/2025/1/foxie_thinking_on_ground.png \
	gallery/2025/1/flatsie.html \
	gallery/2025/1/flatsie.png \
	gallery/2025/1/upside_down_foxie.html \
	gallery/2025/1/upside_down_foxie.png \
	gallery/2025/1/towel_tramp_and_foxie.html \
	gallery/2025/1/towel_tramp_and_foxie.png \
	gallery/2025/1/chosen_one_sword_foxie.html \
	gallery/2025/1/chosen_one_sword_foxie.png \
	gallery/2025/1/janitor_foxie_cleaning_window.html \
	gallery/2025/1/janitor_foxie_cleaning_window.png \
	gallery/2025/1/boxed_foxie.html \
	gallery/2025/1/boxed_foxie.png \
	gallery/2025/1/gift_from_foxie_to_tanuki.html \
	gallery/2025/1/gift_from_foxie_to_tanuki.png \
	gallery/2024/index.html \
	\
	gallery/2024/12/index.html \
	\
	gallery/2024/12/merry_christmas_2024.html \
	gallery/2024/12/merry_christmas_2024.png \
	gallery/2024/12/walking_park_v2.html \
	gallery/2024/12/walking_park_v2.png \
	gallery/2024/12/walking_park.html \
	gallery/2024/12/walking_park.png \
	gallery/2024/12/pipe_foxie.html \
	gallery/2024/12/pipe_foxie.png \
	gallery/2024/12/tongue_stuck.html \
	gallery/2024/12/tongue_stuck.png \
	gallery/2024/11/index.html \
	\
	gallery/2024/11/i_nom_lazy.html \
	gallery/2024/11/i_nom_lazy.png \
	gallery/2024/11/normal_day_as_ship_janitor.html \
	gallery/2024/11/normal_day_as_ship_janitor.png \
	gallery/2024/11/foxie_plush_no_steal.html \
	gallery/2024/11/foxie_plush_no_steal.png \
	gallery/2024/11/satellite_foxie.html \
	gallery/2024/11/satellite_foxie.png \
	gallery/2024/10/index.html \
	\
	gallery/2024/10/hiiiiii_blender.html \
	gallery/2024/10/hiiiiii_blender.png \
	gallery/2024/10/milk_foxie.html \
	gallery/2024/10/milk_foxie.png \
	gallery/2024/10/pat_foxie.html \
	gallery/2024/10/pat_foxie.png \
	gallery/2024/9/index.html \
	\
	gallery/2024/9/breakfast.html \
	gallery/2024/9/breakfast.png \
	gallery/2024/9/four_foxies.html \
	gallery/2024/9/four_foxies.png \
	gallery/2024/9/tissue_box_foxie.html \
	gallery/2024/9/tissue_box_foxie.png \
	gallery/2024/9/missing_limbs_foxie.html \
	gallery/2024/9/missing_limbs_foxie.png \
	gallery/2024/8/index.html \
	\
	gallery/2024/8/eating_alot_foxie.html \
	gallery/2024/8/eating_alot_foxie.png \
	gallery/2024/8/uh_oh.html \
	gallery/2024/8/uh_oh.png \
	gallery/2024/8/slime_foxie.html \
	gallery/2024/8/slime_foxie.png \
	gallery/2024/8/fridge_foxie_blender.html \
	gallery/2024/8/fridge_foxie_blender.png \
	gallery/2024/8/what_a_nice_view.html \
	gallery/2024/8/what_a_nice_view.png \
	gallery/2024/8/janitor_foxie.html \
	gallery/2024/8/janitor_foxie.png \
	gallery/2024/7/index.html \
	\
	gallery/2024/7/fridge_foxie.html \
	gallery/2024/7/fridge_foxie.png \
	gallery/2024/7/derped_foxie.html \
	gallery/2024/7/derped_foxie.png \
	gallery/2024/7/stretchy.html \
	gallery/2024/7/stretchy.png \
	gallery/2024/7/vscode_background.html \
	gallery/2024/7/vscode_background.png \
	gallery/2024/7/cat_themed_bowl.html \
	gallery/2024/7/cat_themed_bowl.png \
	gallery/2024/7/space_foxie.html \
	gallery/2024/7/space_foxie.png \
	gallery/2024/7/foxie_looking_at_snow_forest.html \
	gallery/2024/7/foxie_looking_at_snow_forest.png \
	gallery/2024/7/maid_suit_foxie.html \
	gallery/2024/7/maid_suit_foxie.png \
	gallery/2024/7/stretched_foxie.html \
	gallery/2024/7/stretched_foxie.png \
	gallery/2024/7/swapped_mana_and_foxie.html \
	gallery/2024/7/swapped_mana_and_foxie.png \
	gallery/2024/7/artfight_card.html \
	gallery/2024/7/artfight_card.png \
	gallery/2024/7/lying_foxie.html \
	gallery/2024/7/lying_foxie.png \
	gallery/2024/6/index.html \
	\
	gallery/2024/6/oven_foxie.html \
	gallery/2024/6/oven_foxie.png \
	gallery/2024/6/sleeping_foxie.html \
	gallery/2024/6/sleeping_foxie.png \
	gallery/2024/6/raytraced_foxie.html \
	gallery/2024/6/raytraced_foxie.png \
	gallery/2024/6/skateboard_foxie.html \
	gallery/2024/6/skateboard_foxie.png \
	gallery/2024/6/carpet_tanuki.html \
	gallery/2024/6/carpet_tanuki.png \
	gallery/2024/5/index.html \
	\
	gallery/2024/5/plush_foxie.html \
	gallery/2024/5/plush_foxie.png \
	gallery/2024/5/gifts.html \
	gallery/2024/5/gifts.png \
	gallery/2024/5/dress_foxie.html \
	gallery/2024/5/dress_foxie.png \
	gallery/2024/5/hoodie_foxie.html \
	gallery/2024/5/hoodie_foxie.png \
	gallery/2024/5/trash_can.html \
	gallery/2024/5/trash_can.png \
	gallery/2024/5/pweaseee.html \
	gallery/2024/5/pweaseee.png \
	gallery/2024/5/flower.html \
	gallery/2024/5/flower.png \
	gallery/2024/4/index.html \
	\
	gallery/2024/4/flattened.html \
	gallery/2024/4/flattened.png \
	gallery/2024/4/cuteness_doctor.html \
	gallery/2024/4/cuteness_doctor.png \
	gallery/2024/4/fwooomp.html \
	gallery/2024/4/fwooomp.png \
	gallery/2024/4/keychain_foxie.html \
	gallery/2024/4/keychain_foxie.png \
	gallery/2024/4/oh_nooo.html \
	gallery/2024/4/oh_nooo.png \
	gallery/2024/4/candy_bar.html \
	gallery/2024/4/candy_bar.png \
	gallery/2024/4/eeee.html \
	gallery/2024/4/eeee.png \
	gallery/2024/4/birb_sparky.html \
	gallery/2024/4/birb_sparky.png \
	gallery/2024/4/foxes_are_cute.html \
	gallery/2024/4/foxes_are_cute.png \
	gallery/2024/4/helllooo.html \
	gallery/2024/4/helllooo.png \
	gallery/2024/3/index.html \
	\
	gallery/2024/3/pov_drawer_foxie.html \
	gallery/2024/3/pov_drawer_foxie.png \
	gallery/2024/3/microwaving_foxie.html \
	gallery/2024/3/microwaving_foxie.png \
	gallery/2024/3/clock_sparky.html \
	gallery/2024/3/clock_sparky.png \
	gallery/2024/3/mug_foxie.html \
	gallery/2024/3/mug_foxie.png \
	gallery/2024/3/drawer_foxie.html \
	gallery/2024/3/drawer_foxie.png \
	gallery/2024/3/bed_tanuki.html \
	gallery/2024/3/bed_tanuki.png \
	gallery/2024/3/lots_of_forms.html \
	gallery/2024/3/lots_of_forms.png \
	gallery/2024/2/index.html \
	\
	gallery/2024/2/foxie_got_fish.html \
	gallery/2024/2/foxie_got_fish.png \
	gallery/2024/2/feral_foxie_by_yuki.html \
	gallery/2024/2/feral_foxie_by_yuki.png \
	gallery/2024/2/foxie_bluey.html \
	gallery/2024/2/foxie_bluey.png \
	gallery/2024/1/index.html \
	\
	gallery/2024/1/foxie_cake.html \
	gallery/2024/1/foxie_cake.png \
	gallery/2024/1/thank_you.html \
	gallery/2024/1/thank_you.png \
	gallery/2024/1/drawer_foxie_and_cake.html \
	gallery/2024/1/drawer_foxie_and_cake.png \
	gallery/2024/1/drawer_foxie.html \
	gallery/2024/1/drawer_foxie.png \
	gallery/2023/index.html \
	\
	gallery/2023/12/index.html \
	\
	gallery/2023/12/drawing_with_tanuki.html \
	gallery/2023/12/drawing_with_tanuki.png \
	gallery/2023/12/plushie_foxie.html \
	gallery/2023/12/plushie_foxie.png \
	gallery/2023/12/snow_angel.html \
	gallery/2023/12/snow_angel.png \
	gallery/2023/11/index.html \
	\
	gallery/2023/11/aweeee.html \
	gallery/2023/11/aweeee.png \
	gallery/2023/10/index.html \
	\
	gallery/2023/10/peeks.html \
	gallery/2023/10/peeks.png \
	gallery/2023/10/river.html \
	gallery/2023/10/river.png \
	gallery/2023/8/index.html \
	\
	gallery/2023/8/eat_math_test.html \
	gallery/2023/8/eat_math_test.png \
	gallery/2023/7/index.html \
	\
	gallery/2023/7/tiny_bot_foxie.html \
	gallery/2023/7/tiny_bot_foxie.png \
	gallery/2023/7/moon_cake.html \
	gallery/2023/7/moon_cake.png \
	gallery/2023/7/comfy_sofa.html \
	gallery/2023/7/comfy_sofa.png \
	gallery/2023/7/pillow_foxie.html \
	gallery/2023/7/pillow_foxie.png \
	gallery/2023/7/post_it_fox.html \
	gallery/2023/7/post_it_fox.png \
	gallery/2023/7/pizza_fox_box.html \
	gallery/2023/7/pizza_fox_box.png \
	gallery/2023/7/box_fox.html \
	gallery/2023/7/box_fox.png \
	gallery/2023/7/toasty_pg2.html \
	gallery/2023/7/toasty_pg2.png \
	gallery/2023/7/toasty_pg1.html \
	gallery/2023/7/toasty_pg1.png \
	gallery/2023/7/blanket_foxie.html \
	gallery/2023/7/blanket_foxie.png \
	gallery/2023/7/my_oc.html \
	gallery/2023/7/my_oc.png \
	\
	gallery/sitemap.xml \
	gallery/index.html \
	\
	favicon.ico \
	favicon_for_opengraph.png \
	img/profile.gif \
	img/Gallery_Icon.png \
	img/Home_Icon.png \
	img/parasitic_popup/left_arrow.png \
	img/parasitic_popup/right_arrow.png \
	css/index.css \
	css/navbar.css \
	css/gallery.css \
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
	creations/RPGGameCollege/index.html \
	creations/RPGGameCollege/rpg-college-0.1.0.jar \
	 \
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
	college_stuff/css/global.css \
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
	college_stuff/img/news.png \
	college_stuff/img/checkmark_inactive.png \
	.nojekyll

# Ignore some directories
files += \
	img/sitemap.xml \
	css/sitemap.xml \
	college_stuff/sitemap.xml

export input_dir
export output_dir
export site_root

.DEFAULT_GOAL := all
.PHONY: all
all: create_dirs .WAIT $(addprefix $(web_dir)/,$(files))
	@echo "[ SITEMAP ] Generating sitemaps"
	@lua5.4 scripts/generate-sitemap.lua "$(web_dir)" "$(site_host_root)"
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

# Raw html, don't preprocess or do any processing
$(intermediate_dir4)/%.html: $(intermediate_dir3)/%.html-raw
	$(make_dirs)
	@ln -f '$<' '$@'
	@echo "[ COPY ] Copy raw HTML $(@:$(intermediate_dir4)=)"

# For files that don't need to be preprocessed
$(intermediate_dir0)/%: $(input_dir)/%
	$(make_dirs)
	@cp -- '$<' '$@'
	@echo "[ COPY ] Updating $(@:$(intermediate_dir0)=)"

# Append sitemap to robots.txt
$(intermediate_dir0)/robots.txt: $(input_dir)/robots.txt
	@echo "[ UPDATE ] Updating robots.txt"
	@cp -- '$<' '$@'
	@echo "Sitemap: $(site_host_root)/sitemap.xml" >> '$@'

.PHONY: clean
clean:
	@rm -rf -- '$(output_dir)'

.PHONY: host
host: all
	@echo "[HOST  ] Locally hosting at localhost:8080 with php's builtin webserver"
	@php -S localhost:8080 -t '$(web_dir)/' router.php

.PHONY: host-python
host-python: all
	@echo "[HOST  ] Locally hosting at localhost:8080 with Python's HTTP module"
	@cd '$(web_dir)/'; python -m http.server 8080

# See https://stackoverflow.com/questions/2483182/recursive-wildcards-in-gnu-make
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))
include $(call rwildcard,$(deps_dir),*.d)

