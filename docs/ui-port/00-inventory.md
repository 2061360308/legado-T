# Android UI 全量复刻清单

> 生成目录：app/src/main/res
> 排序规则：layout → layout-land → menu → xml，组内按文件名升序。
> 状态：待复刻 / 进行中 / 已完成 / 视觉禁用 / 不适用。

- layout: 183
- layout-land: 1
- menu: 90
- xml: 12

| # | XML | 目标 Vue/组件 | 状态 | 备注 |
|---:|---|---|---|---|
| 1 | `layout/activity_about.xml` | views/config/AboutView.vue | 已完成 |  |
| 2 | `layout/activity_all_bookmark.xml` | views/config/BookmarksView.vue | 已完成 |  |
| 3 | `layout/activity_arrange_book.xml` | views/bookshelf/ArrangeBooksView.vue | 已完成 | SelectActionBar 展开为 view_select_action_bar 结构；TitleBar contentLayout 展开搜索框 |
| 4 | `layout/activity_audio_play.xml` | views/audio/AudioPlayView.vue | 视觉禁用 |  |
| 5 | `layout/activity_book_info_edit.xml` | views/bookshelf/BookInfoEditView.vue | 已完成 |  |
| 6 | `layout/activity_book_info.xml` | views/bookshelf/BookInfoView.vue | 已完成 |  |
| 7 | `layout/activity_book_read.xml` | views/reader/ReaderView.vue | 进行中 |  |
| 8 | `layout/activity_book_search.xml` | views/search/BookSearchView.vue | 已完成 | TitleBar contentLayout 展开搜索框 |
| 9 | `layout/activity_book_source_edit.xml` | views/source/BookSourceEditView.vue | 已完成 |  |
| 10 | `layout/activity_book_source.xml` | views/source/BookSourceView.vue | 已完成 | SelectActionBar 展开为 view_select_action_bar 结构；TitleBar contentLayout 展开搜索框 |
| 11 | `layout/activity_cache_book.xml` | views/source/CacheBookView.vue | 已完成 |  |
| 12 | `layout/activity_chapter_list.xml` | views/toc/TocView.vue | 已完成 | TitleBar contentLayout 展开 Tab 栏 |
| 13 | `layout/activity_config.xml` | views/config/ConfigView.vue | 进行中 |  |
| 14 | `layout/activity_dict_rule.xml` | views/dict/DictRulesView.vue | 进行中 | SelectActionBar 展开为 view_select_action_bar 结构 |
| 15 | `layout/activity_donate.xml` | design/generated/layout/activity_donate.vue | 壳子已生成 |  |
| 16 | `layout/activity_explore_show.xml` | design/generated/layout/activity_explore_show.vue | 壳子已生成 |  |
| 17 | `layout/activity_file_manage.xml` | design/generated/layout/activity_file_manage.vue | 壳子已生成 | TitleBar contentLayout 展开搜索框 |
| 18 | `layout/activity_import_book.xml` | views/import/ImportView.vue | 进行中 | SelectActionBar 展开为 view_select_action_bar 结构；TitleBar contentLayout 展开搜索框 |
| 19 | `layout/activity_main.xml` | App.vue + BottomNav | 进行中 |  |
| 20 | `layout/activity_manga.xml` | design/generated/layout/activity_manga.vue | 壳子已生成 |  |
| 21 | `layout/activity_qrcode_capture.xml` | design/generated/layout/activity_qrcode_capture.vue | 壳子已生成 |  |
| 22 | `layout/activity_read_record.xml` | design/generated/layout/activity_read_record.vue | 壳子已生成 | TitleBar contentLayout 展开搜索框 |
| 23 | `layout/activity_replace_edit.xml` | design/generated/layout/activity_replace_edit.vue | 壳子已生成 |  |
| 24 | `layout/activity_replace_rule.xml` | views/replace/ReplaceView.vue | 进行中 | SelectActionBar 展开为 view_select_action_bar 结构；TitleBar contentLayout 展开搜索框 |
| 25 | `layout/activity_rss_artivles.xml` | design/generated/layout/activity_rss_artivles.vue | 壳子已生成 |  |
| 26 | `layout/activity_rss_favorites.xml` | design/generated/layout/activity_rss_favorites.vue | 壳子已生成 |  |
| 27 | `layout/activity_rss_read.xml` | design/generated/layout/activity_rss_read.vue | 壳子已生成 |  |
| 28 | `layout/activity_rss_source_edit.xml` | design/generated/layout/activity_rss_source_edit.vue | 壳子已生成 |  |
| 29 | `layout/activity_rss_source.xml` | design/generated/layout/activity_rss_source.vue | 壳子已生成 | SelectActionBar 展开为 view_select_action_bar 结构；TitleBar contentLayout 展开搜索框 |
| 30 | `layout/activity_rule_sub.xml` | design/generated/layout/activity_rule_sub.vue | 壳子已生成 |  |
| 31 | `layout/activity_search_content.xml` | design/generated/layout/activity_search_content.vue | 壳子已生成 | TitleBar contentLayout 展开搜索框 |
| 32 | `layout/activity_source_debug.xml` | design/generated/layout/activity_source_debug.vue | 壳子已生成 | TitleBar contentLayout 展开搜索框 |
| 33 | `layout/activity_source_login.xml` | design/generated/layout/activity_source_login.vue | 壳子已生成 |  |
| 34 | `layout/activity_translucence.xml` | design/generated/layout/activity_translucence.vue | 壳子已生成 |  |
| 35 | `layout/activity_txt_toc_rule.xml` | views/toc/TocRulesView.vue | 进行中 | SelectActionBar 展开为 view_select_action_bar 结构 |
| 36 | `layout/activity_web_view.xml` | design/generated/layout/activity_web_view.vue | 壳子已生成 |  |
| 37 | `layout/activity_welcome.xml` | design/generated/layout/activity_welcome.vue | 壳子已生成 |  |
| 38 | `layout/dialog_add_to_bookshelf.xml` | design/generated/layout/dialog_add_to_bookshelf.vue | 壳子已生成 |  |
| 39 | `layout/dialog_auto_read.xml` | design/generated/layout/dialog_auto_read.vue | 壳子已生成 |  |
| 40 | `layout/dialog_book_change_source.xml` | design/generated/layout/dialog_book_change_source.vue | 壳子已生成 |  |
| 41 | `layout/dialog_book_group_edit.xml` | design/generated/layout/dialog_book_group_edit.vue | 壳子已生成 |  |
| 42 | `layout/dialog_book_group_picker.xml` | design/generated/layout/dialog_book_group_picker.vue | 壳子已生成 |  |
| 43 | `layout/dialog_bookmark.xml` | design/generated/layout/dialog_bookmark.vue | 壳子已生成 |  |
| 44 | `layout/dialog_bookshelf_config.xml` | design/generated/layout/dialog_bookshelf_config.vue | 壳子已生成 |  |
| 45 | `layout/dialog_change_cover.xml` | design/generated/layout/dialog_change_cover.vue | 壳子已生成 |  |
| 46 | `layout/dialog_chapter_change_source.xml` | design/generated/layout/dialog_chapter_change_source.vue | 壳子已生成 |  |
| 47 | `layout/dialog_check_source_config.xml` | design/generated/layout/dialog_check_source_config.vue | 壳子已生成 |  |
| 48 | `layout/dialog_click_action_config.xml` | design/generated/layout/dialog_click_action_config.vue | 壳子已生成 |  |
| 49 | `layout/dialog_code_view.xml` | design/generated/layout/dialog_code_view.vue | 壳子已生成 |  |
| 50 | `layout/dialog_content_edit.xml` | design/generated/layout/dialog_content_edit.vue | 壳子已生成 |  |
| 51 | `layout/dialog_cover_rule_config.xml` | design/generated/layout/dialog_cover_rule_config.vue | 壳子已生成 |  |
| 52 | `layout/dialog_custom_group.xml` | design/generated/layout/dialog_custom_group.vue | 壳子已生成 |  |
| 53 | `layout/dialog_dict_rule_edit.xml` | design/generated/layout/dialog_dict_rule_edit.vue | 壳子已生成 |  |
| 54 | `layout/dialog_dict.xml` | design/generated/layout/dialog_dict.vue | 壳子已生成 |  |
| 55 | `layout/dialog_direct_link_upload_config.xml` | design/generated/layout/dialog_direct_link_upload_config.vue | 壳子已生成 |  |
| 56 | `layout/dialog_download_choice.xml` | design/generated/layout/dialog_download_choice.vue | 壳子已生成 |  |
| 57 | `layout/dialog_edit_text.xml` | design/generated/layout/dialog_edit_text.vue | 壳子已生成 |  |
| 58 | `layout/dialog_file_chooser.xml` | design/generated/layout/dialog_file_chooser.vue | 壳子已生成 |  |
| 59 | `layout/dialog_font_select.xml` | design/generated/layout/dialog_font_select.vue | 壳子已生成 |  |
| 60 | `layout/dialog_http_tts_edit.xml` | design/generated/layout/dialog_http_tts_edit.vue | 壳子已生成 |  |
| 61 | `layout/dialog_image_blurring.xml` | design/generated/layout/dialog_image_blurring.vue | 壳子已生成 |  |
| 62 | `layout/dialog_login.xml` | design/generated/layout/dialog_login.vue | 壳子已生成 |  |
| 63 | `layout/dialog_manga_color_filter.xml` | design/generated/layout/dialog_manga_color_filter.vue | 壳子已生成 |  |
| 64 | `layout/dialog_manga_epaper.xml` | design/generated/layout/dialog_manga_epaper.vue | 壳子已生成 |  |
| 65 | `layout/dialog_manga_footer_setting.xml` | design/generated/layout/dialog_manga_footer_setting.vue | 壳子已生成 |  |
| 66 | `layout/dialog_multiple_edit_text.xml` | design/generated/layout/dialog_multiple_edit_text.vue | 壳子已生成 |  |
| 67 | `layout/dialog_number_picker.xml` | design/generated/layout/dialog_number_picker.vue | 壳子已生成 |  |
| 68 | `layout/dialog_open_url_confirm.xml` | design/generated/layout/dialog_open_url_confirm.vue | 壳子已生成 |  |
| 69 | `layout/dialog_page_key.xml` | design/generated/layout/dialog_page_key.vue | 壳子已生成 |  |
| 70 | `layout/dialog_photo_view.xml` | design/generated/layout/dialog_photo_view.vue | 壳子已生成 |  |
| 71 | `layout/dialog_progressbar_view.xml` | design/generated/layout/dialog_progressbar_view.vue | 壳子已生成 |  |
| 72 | `layout/dialog_read_aloud.xml` | design/generated/layout/dialog_read_aloud.vue | 壳子已生成 |  |
| 73 | `layout/dialog_read_bg_text.xml` | design/generated/layout/dialog_read_bg_text.vue | 壳子已生成 |  |
| 74 | `layout/dialog_read_book_style.xml` | design/generated/layout/dialog_read_book_style.vue | 壳子已生成 |  |
| 75 | `layout/dialog_read_padding.xml` | design/generated/layout/dialog_read_padding.vue | 壳子已生成 |  |
| 76 | `layout/dialog_recycler_view.xml` | design/generated/layout/dialog_recycler_view.vue | 壳子已生成 |  |
| 77 | `layout/dialog_rss_favorite_config.xml` | design/generated/layout/dialog_rss_favorite_config.vue | 壳子已生成 |  |
| 78 | `layout/dialog_rule_sub_edit.xml` | design/generated/layout/dialog_rule_sub_edit.vue | 壳子已生成 |  |
| 79 | `layout/dialog_search_scope.xml` | design/generated/layout/dialog_search_scope.vue | 壳子已生成 |  |
| 80 | `layout/dialog_select_section_export.xml` | design/generated/layout/dialog_select_section_export.vue | 壳子已生成 |  |
| 81 | `layout/dialog_simulated_reading.xml` | design/generated/layout/dialog_simulated_reading.vue | 壳子已生成 |  |
| 82 | `layout/dialog_source_picker.xml` | design/generated/layout/dialog_source_picker.vue | 壳子已生成 | TitleBar contentLayout 展开搜索框 |
| 83 | `layout/dialog_text_view.xml` | design/generated/layout/dialog_text_view.vue | 壳子已生成 |  |
| 84 | `layout/dialog_tip_config.xml` | design/generated/layout/dialog_tip_config.vue | 壳子已生成 |  |
| 85 | `layout/dialog_toc_regex_edit.xml` | design/generated/layout/dialog_toc_regex_edit.vue | 壳子已生成 |  |
| 86 | `layout/dialog_toc_regex.xml` | design/generated/layout/dialog_toc_regex.vue | 壳子已生成 |  |
| 87 | `layout/dialog_update.xml` | design/generated/layout/dialog_update.vue | 壳子已生成 |  |
| 88 | `layout/dialog_url_option_edit.xml` | design/generated/layout/dialog_url_option_edit.vue | 壳子已生成 |  |
| 89 | `layout/dialog_variable.xml` | design/generated/layout/dialog_variable.vue | 壳子已生成 |  |
| 90 | `layout/dialog_verification_code_view.xml` | design/generated/layout/dialog_verification_code_view.vue | 壳子已生成 |  |
| 91 | `layout/dialog_wait.xml` | design/generated/layout/dialog_wait.vue | 壳子已生成 |  |
| 92 | `layout/dialog_webdav_server.xml` | design/generated/layout/dialog_webdav_server.vue | 壳子已生成 |  |
| 93 | `layout/fragment_bookmark.xml` | design/generated/layout/fragment_bookmark.vue | 壳子已生成 |  |
| 94 | `layout/fragment_books.xml` | design/generated/layout/fragment_books.vue | 壳子已生成 |  |
| 95 | `layout/fragment_bookshelf1.xml` | design/generated/layout/fragment_bookshelf1.vue | 壳子已生成 | TitleBar contentLayout 展开 Tab 栏 |
| 96 | `layout/fragment_bookshelf2.xml` | design/generated/layout/fragment_bookshelf2.vue | 壳子已生成 |  |
| 97 | `layout/fragment_chapter_list.xml` | design/generated/layout/fragment_chapter_list.vue | 壳子已生成 |  |
| 98 | `layout/fragment_explore.xml` | design/generated/layout/fragment_explore.vue | 壳子已生成 | TitleBar contentLayout 展开搜索框 |
| 99 | `layout/fragment_my_config.xml` | design/generated/layout/fragment_my_config.vue | 壳子已生成 |  |
| 100 | `layout/fragment_rss_articles.xml` | design/generated/layout/fragment_rss_articles.vue | 壳子已生成 |  |
| 101 | `layout/fragment_rss.xml` | design/generated/layout/fragment_rss.vue | 壳子已生成 | TitleBar contentLayout 展开搜索框 |
| 102 | `layout/fragment_web_view_login.xml` | design/generated/layout/fragment_web_view_login.vue | 壳子已生成 |  |
| 103 | `layout/item_1line_text_and_del.xml` | design/generated/layout/item_1line_text_and_del.vue | 壳子已生成 |  |
| 104 | `layout/item_1line_text.xml` | design/generated/layout/item_1line_text.vue | 壳子已生成 |  |
| 105 | `layout/item_app_log.xml` | design/generated/layout/item_app_log.vue | 壳子已生成 |  |
| 106 | `layout/item_arrange_book.xml` | views/bookshelf/ArrangeBooksView.vue | 已完成 |  |
| 107 | `layout/item_bg_image.xml` | design/generated/layout/item_bg_image.vue | 壳子已生成 |  |
| 108 | `layout/item_book_file_import.xml` | design/generated/layout/item_book_file_import.vue | 壳子已生成 |  |
| 109 | `layout/item_book_group_manage.xml` | design/generated/layout/item_book_group_manage.vue | 壳子已生成 |  |
| 110 | `layout/item_book_manga_edge.xml` | design/generated/layout/item_book_manga_edge.vue | 壳子已生成 |  |
| 111 | `layout/item_book_manga_page.xml` | design/generated/layout/item_book_manga_page.vue | 壳子已生成 |  |
| 112 | `layout/item_book_source.xml` | design/generated/layout/item_book_source.vue | 壳子已生成 |  |
| 113 | `layout/item_bookmark.xml` | views/config/BookmarksView.vue | 已完成 |  |
| 114 | `layout/item_bookshelf_grid_group.xml` | design/generated/layout/item_bookshelf_grid_group.vue | 壳子已生成 |  |
| 115 | `layout/item_bookshelf_grid.xml` | design/generated/layout/item_bookshelf_grid.vue | 壳子已生成 |  |
| 116 | `layout/item_bookshelf_list_group.xml` | design/generated/layout/item_bookshelf_list_group.vue | 壳子已生成 |  |
| 117 | `layout/item_bookshelf_list.xml` | design/generated/layout/item_bookshelf_list.vue | 壳子已生成 |  |
| 118 | `layout/item_change_source.xml` | design/generated/layout/item_change_source.vue | 壳子已生成 |  |
| 119 | `layout/item_chapter_list.xml` | design/generated/layout/item_chapter_list.vue | 壳子已生成 |  |
| 120 | `layout/item_check_box.xml` | design/generated/layout/item_check_box.vue | 壳子已生成 |  |
| 121 | `layout/item_cover.xml` | design/generated/layout/item_cover.vue | 壳子已生成 |  |
| 122 | `layout/item_dict_rule.xml` | components/android/RuleRow.vue | 进行中 |  |
| 123 | `layout/item_download.xml` | design/generated/layout/item_download.vue | 壳子已生成 |  |
| 124 | `layout/item_file_picker.xml` | design/generated/layout/item_file_picker.vue | 壳子已生成 |  |
| 125 | `layout/item_file.xml` | design/generated/layout/item_file.vue | 壳子已生成 |  |
| 126 | `layout/item_fillet_text.xml` | design/generated/layout/item_fillet_text.vue | 壳子已生成 |  |
| 127 | `layout/item_find_book.xml` | design/generated/layout/item_find_book.vue | 壳子已生成 |  |
| 128 | `layout/item_font.xml` | design/generated/layout/item_font.vue | 壳子已生成 |  |
| 129 | `layout/item_group_manage.xml` | design/generated/layout/item_group_manage.vue | 壳子已生成 |  |
| 130 | `layout/item_group_select.xml` | design/generated/layout/item_group_select.vue | 壳子已生成 |  |
| 131 | `layout/item_http_tts.xml` | design/generated/layout/item_http_tts.vue | 壳子已生成 |  |
| 132 | `layout/item_icon_preference.xml` | design/generated/layout/item_icon_preference.vue | 壳子已生成 |  |
| 133 | `layout/item_import_book.xml` | design/generated/layout/item_import_book.vue | 壳子已生成 |  |
| 134 | `layout/item_log.xml` | design/generated/layout/item_log.vue | 壳子已生成 |  |
| 135 | `layout/item_path_picker.xml` | design/generated/layout/item_path_picker.vue | 壳子已生成 |  |
| 136 | `layout/item_radio_button.xml` | design/generated/layout/item_radio_button.vue | 壳子已生成 |  |
| 137 | `layout/item_read_record.xml` | design/generated/layout/item_read_record.vue | 壳子已生成 |  |
| 138 | `layout/item_read_style.xml` | design/generated/layout/item_read_style.vue | 壳子已生成 |  |
| 139 | `layout/item_replace_rule.xml` | components/android/RuleRow.vue | 进行中 |  |
| 140 | `layout/item_rss_article_1.xml` | design/generated/layout/item_rss_article_1.vue | 壳子已生成 |  |
| 141 | `layout/item_rss_article_2.xml` | design/generated/layout/item_rss_article_2.vue | 壳子已生成 |  |
| 142 | `layout/item_rss_article.xml` | design/generated/layout/item_rss_article.vue | 壳子已生成 |  |
| 143 | `layout/item_rss_read_record.xml` | design/generated/layout/item_rss_read_record.vue | 壳子已生成 |  |
| 144 | `layout/item_rss_source.xml` | design/generated/layout/item_rss_source.vue | 壳子已生成 |  |
| 145 | `layout/item_rss.xml` | design/generated/layout/item_rss.vue | 壳子已生成 |  |
| 146 | `layout/item_rule_sub.xml` | design/generated/layout/item_rule_sub.vue | 壳子已生成 |  |
| 147 | `layout/item_search_list.xml` | design/generated/layout/item_search_list.vue | 壳子已生成 |  |
| 148 | `layout/item_search.xml` | design/generated/layout/item_search.vue | 壳子已生成 |  |
| 149 | `layout/item_server_select.xml` | design/generated/layout/item_server_select.vue | 壳子已生成 |  |
| 150 | `layout/item_source_edit_check_box.xml` | design/generated/layout/item_source_edit_check_box.vue | 壳子已生成 |  |
| 151 | `layout/item_source_edit.xml` | design/generated/layout/item_source_edit.vue | 壳子已生成 |  |
| 152 | `layout/item_source_import.xml` | design/generated/layout/item_source_import.vue | 壳子已生成 |  |
| 153 | `layout/item_text.xml` | design/generated/layout/item_text.vue | 壳子已生成 |  |
| 154 | `layout/item_theme_config.xml` | design/generated/layout/item_theme_config.vue | 壳子已生成 |  |
| 155 | `layout/item_toc_regex.xml` | design/generated/layout/item_toc_regex.vue | 壳子已生成 |  |
| 156 | `layout/item_txt_toc_rule.xml` | components/android/RuleRow.vue | 进行中 |  |
| 157 | `layout/popup_action_menu.xml` | components/android/PopupMenu.vue | 已完成 |  |
| 158 | `layout/popup_action.xml` | design/generated/layout/popup_action.vue | 壳子已生成 |  |
| 159 | `layout/popup_keyboard_tool.xml` | design/generated/layout/popup_keyboard_tool.vue | 壳子已生成 |  |
| 160 | `layout/popup_seek_bar.xml` | design/generated/layout/popup_seek_bar.vue | 壳子已生成 |  |
| 161 | `layout/view_action_button.xml` | design/generated/layout/view_action_button.vue | 壳子已生成 |  |
| 162 | `layout/view_book_page.xml` | design/generated/layout/view_book_page.vue | 壳子已生成 |  |
| 163 | `layout/view_detail_seek_bar.xml` | design/generated/layout/view_detail_seek_bar.vue | 壳子已生成 |  |
| 164 | `layout/view_dynamic.xml` | design/generated/layout/view_dynamic.vue | 壳子已生成 |  |
| 165 | `layout/view_error.xml` | design/generated/layout/view_error.vue | 壳子已生成 |  |
| 166 | `layout/view_fastscroller.xml` | design/generated/layout/view_fastscroller.vue | 壳子已生成 |  |
| 167 | `layout/view_icon.xml` | design/generated/layout/view_icon.vue | 壳子已生成 |  |
| 168 | `layout/view_load_more.xml` | design/generated/layout/view_load_more.vue | 壳子已生成 |  |
| 169 | `layout/view_loading.xml` | design/generated/layout/view_loading.vue | 壳子已生成 |  |
| 170 | `layout/view_manga_menu.xml` | design/generated/layout/view_manga_menu.vue | 壳子已生成 |  |
| 171 | `layout/view_navigation_badge.xml` | design/generated/layout/view_navigation_badge.vue | 壳子已生成 |  |
| 172 | `layout/view_preference_category.xml` | components/android/PreferenceCategory.vue | 已完成 |  |
| 173 | `layout/view_preference.xml` | components/android/PreferenceRow.vue | 已完成 |  |
| 174 | `layout/view_read_menu.xml` | views/reader/ReaderView.vue | 进行中 |  |
| 175 | `layout/view_refresh_recycler.xml` | design/generated/layout/view_refresh_recycler.vue | 壳子已生成 |  |
| 176 | `layout/view_search_menu.xml` | design/generated/layout/view_search_menu.vue | 壳子已生成 |  |
| 177 | `layout/view_search.xml` | design/generated/layout/view_search.vue | 壳子已生成 |  |
| 178 | `layout/view_select_action_bar.xml` | design/generated/layout/view_select_action_bar.vue | 壳子已生成 |  |
| 179 | `layout/view_tab_layout_min.xml` | design/generated/layout/view_tab_layout_min.vue | 壳子已生成 |  |
| 180 | `layout/view_tab_layout.xml` | design/generated/layout/view_tab_layout.vue | 壳子已生成 |  |
| 181 | `layout/view_title_bar_dark.xml` | design/generated/layout/view_title_bar_dark.vue | 壳子已生成 |  |
| 182 | `layout/view_title_bar.xml` | components/android/TitleBar.vue | 已完成 |  |
| 183 | `layout/view_toast.xml` | design/generated/layout/view_toast.vue | 壳子已生成 |  |
| 184 | `layout-land/activity_book_info.xml` | design/generated/layout-land/activity_book_info.vue | 壳子已生成 |  |
| 185 | `menu/about.xml` | design/generated/menu/about.vue | 壳子已生成 |  |
| 186 | `menu/app_log.xml` | design/generated/menu/app_log.vue | 壳子已生成 |  |
| 187 | `menu/app_update.xml` | design/generated/menu/app_update.vue | 壳子已生成 |  |
| 188 | `menu/audio_play.xml` | design/generated/menu/audio_play.vue | 壳子已生成 |  |
| 189 | `menu/backup_restore.xml` | design/generated/menu/backup_restore.vue | 壳子已生成 |  |
| 190 | `menu/book_cache_download.xml` | design/generated/menu/book_cache_download.vue | 壳子已生成 |  |
| 191 | `menu/book_cache.xml` | design/generated/menu/book_cache.vue | 壳子已生成 |  |
| 192 | `menu/book_group_manage.xml` | design/generated/menu/book_group_manage.vue | 壳子已生成 |  |
| 193 | `menu/book_info_edit.xml` | design/generated/menu/book_info_edit.vue | 壳子已生成 |  |
| 194 | `menu/book_info.xml` | design/generated/menu/book_info.vue | 壳子已生成 |  |
| 195 | `menu/book_manga.xml` | design/generated/menu/book_manga.vue | 壳子已生成 |  |
| 196 | `menu/book_read_change_source.xml` | design/generated/menu/book_read_change_source.vue | 壳子已生成 |  |
| 197 | `menu/book_read_record.xml` | design/generated/menu/book_read_record.vue | 壳子已生成 |  |
| 198 | `menu/book_read_refresh.xml` | design/generated/menu/book_read_refresh.vue | 壳子已生成 |  |
| 199 | `menu/book_read_source.xml` | design/generated/menu/book_read_source.vue | 壳子已生成 |  |
| 200 | `menu/book_read.xml` | design/generated/menu/book_read.vue | 壳子已生成 |  |
| 201 | `menu/book_remote.xml` | design/generated/menu/book_remote.vue | 壳子已生成 |  |
| 202 | `menu/book_search_scope.xml` | design/generated/menu/book_search_scope.vue | 壳子已生成 |  |
| 203 | `menu/book_search.xml` | design/generated/menu/book_search.vue | 壳子已生成 |  |
| 204 | `menu/book_source_debug.xml` | design/generated/menu/book_source_debug.vue | 壳子已生成 |  |
| 205 | `menu/book_source_item.xml` | design/generated/menu/book_source_item.vue | 壳子已生成 |  |
| 206 | `menu/book_source_sel.xml` | design/generated/menu/book_source_sel.vue | 壳子已生成 |  |
| 207 | `menu/book_source.xml` | design/generated/menu/book_source.vue | 壳子已生成 |  |
| 208 | `menu/book_toc.xml` | design/generated/menu/book_toc.vue | 壳子已生成 |  |
| 209 | `menu/bookmark.xml` | design/generated/menu/bookmark.vue | 壳子已生成 |  |
| 210 | `menu/bookshelf_manage.xml` | design/generated/menu/bookshelf_manage.vue | 壳子已生成 |  |
| 211 | `menu/bookshelf_menage_sel.xml` | design/generated/menu/bookshelf_menage_sel.vue | 壳子已生成 |  |
| 212 | `menu/change_cover.xml` | design/generated/menu/change_cover.vue | 壳子已生成 |  |
| 213 | `menu/change_source_item.xml` | design/generated/menu/change_source_item.vue | 壳子已生成 |  |
| 214 | `menu/change_source.xml` | design/generated/menu/change_source.vue | 壳子已生成 |  |
| 215 | `menu/code_edit.xml` | design/generated/menu/code_edit.vue | 壳子已生成 |  |
| 216 | `menu/content_edit.xml` | design/generated/menu/content_edit.vue | 壳子已生成 |  |
| 217 | `menu/content_search.xml` | design/generated/menu/content_search.vue | 壳子已生成 |  |
| 218 | `menu/content_select_action.xml` | design/generated/menu/content_select_action.vue | 壳子已生成 |  |
| 219 | `menu/crash_log.xml` | design/generated/menu/crash_log.vue | 壳子已生成 |  |
| 220 | `menu/dialog_text.xml` | design/generated/menu/dialog_text.vue | 壳子已生成 |  |
| 221 | `menu/dict_rule_edit.xml` | design/generated/menu/dict_rule_edit.vue | 壳子已生成 |  |
| 222 | `menu/dict_rule_sel.xml` | design/generated/menu/dict_rule_sel.vue | 壳子已生成 |  |
| 223 | `menu/dict_rule.xml` | design/generated/menu/dict_rule.vue | 壳子已生成 |  |
| 224 | `menu/direct_link_upload_config.xml` | design/generated/menu/direct_link_upload_config.vue | 壳子已生成 |  |
| 225 | `menu/explore_item.xml` | design/generated/menu/explore_item.vue | 壳子已生成 |  |
| 226 | `menu/file_chooser.xml` | design/generated/menu/file_chooser.vue | 壳子已生成 |  |
| 227 | `menu/file_long_click.xml` | design/generated/menu/file_long_click.vue | 壳子已生成 |  |
| 228 | `menu/font_select.xml` | design/generated/menu/font_select.vue | 壳子已生成 |  |
| 229 | `menu/group_manage.xml` | design/generated/menu/group_manage.vue | 壳子已生成 |  |
| 230 | `menu/import_book_sel.xml` | design/generated/menu/import_book_sel.vue | 壳子已生成 |  |
| 231 | `menu/import_book.xml` | design/generated/menu/import_book.vue | 壳子已生成 |  |
| 232 | `menu/import_replace.xml` | design/generated/menu/import_replace.vue | 壳子已生成 |  |
| 233 | `menu/import_source.xml` | design/generated/menu/import_source.vue | 壳子已生成 |  |
| 234 | `menu/keyboard_assists_config.xml` | design/generated/menu/keyboard_assists_config.vue | 壳子已生成 |  |
| 235 | `menu/main_bnv.xml` | design/generated/menu/main_bnv.vue | 壳子已生成 |  |
| 236 | `menu/main_bookshelf.xml` | design/generated/menu/main_bookshelf.vue | 壳子已生成 |  |
| 237 | `menu/main_explore.xml` | design/generated/menu/main_explore.vue | 壳子已生成 |  |
| 238 | `menu/main_my.xml` | design/generated/menu/main_my.vue | 壳子已生成 |  |
| 239 | `menu/main_rss.xml` | design/generated/menu/main_rss.vue | 壳子已生成 |  |
| 240 | `menu/open_url_confirm.xml` | design/generated/menu/open_url_confirm.vue | 壳子已生成 |  |
| 241 | `menu/qr_code_scan.xml` | design/generated/menu/qr_code_scan.vue | 壳子已生成 |  |
| 242 | `menu/replace_edit.xml` | design/generated/menu/replace_edit.vue | 壳子已生成 |  |
| 243 | `menu/replace_rule_item.xml` | design/generated/menu/replace_rule_item.vue | 壳子已生成 |  |
| 244 | `menu/replace_rule_sel.xml` | design/generated/menu/replace_rule_sel.vue | 壳子已生成 |  |
| 245 | `menu/replace_rule.xml` | design/generated/menu/replace_rule.vue | 壳子已生成 |  |
| 246 | `menu/rss_articles.xml` | design/generated/menu/rss_articles.vue | 壳子已生成 |  |
| 247 | `menu/rss_favorites.xml` | design/generated/menu/rss_favorites.vue | 壳子已生成 |  |
| 248 | `menu/rss_main_item.xml` | design/generated/menu/rss_main_item.vue | 壳子已生成 |  |
| 249 | `menu/rss_read_record.xml` | design/generated/menu/rss_read_record.vue | 壳子已生成 |  |
| 250 | `menu/rss_read.xml` | design/generated/menu/rss_read.vue | 壳子已生成 |  |
| 251 | `menu/rss_source_debug.xml` | design/generated/menu/rss_source_debug.vue | 壳子已生成 |  |
| 252 | `menu/rss_source_item.xml` | design/generated/menu/rss_source_item.vue | 壳子已生成 |  |
| 253 | `menu/rss_source_sel.xml` | design/generated/menu/rss_source_sel.vue | 壳子已生成 |  |
| 254 | `menu/rss_source.xml` | design/generated/menu/rss_source.vue | 壳子已生成 |  |
| 255 | `menu/save.xml` | design/generated/menu/save.vue | 壳子已生成 |  |
| 256 | `menu/search_view.xml` | design/generated/menu/search_view.vue | 壳子已生成 |  |
| 257 | `menu/server_config.xml` | design/generated/menu/server_config.vue | 壳子已生成 |  |
| 258 | `menu/servers.xml` | design/generated/menu/servers.vue | 壳子已生成 |  |
| 259 | `menu/source_edit.xml` | design/generated/menu/source_edit.vue | 壳子已生成 |  |
| 260 | `menu/source_login.xml` | design/generated/menu/source_login.vue | 壳子已生成 |  |
| 261 | `menu/source_picker.xml` | design/generated/menu/source_picker.vue | 壳子已生成 |  |
| 262 | `menu/source_sub_item.xml` | design/generated/menu/source_sub_item.vue | 壳子已生成 |  |
| 263 | `menu/source_subscription.xml` | design/generated/menu/source_subscription.vue | 壳子已生成 |  |
| 264 | `menu/source_webview_login.xml` | design/generated/menu/source_webview_login.vue | 壳子已生成 |  |
| 265 | `menu/speak_engine_edit.xml` | design/generated/menu/speak_engine_edit.vue | 壳子已生成 |  |
| 266 | `menu/speak_engine.xml` | design/generated/menu/speak_engine.vue | 壳子已生成 |  |
| 267 | `menu/theme_config.xml` | design/generated/menu/theme_config.vue | 壳子已生成 |  |
| 268 | `menu/theme_list.xml` | design/generated/menu/theme_list.vue | 壳子已生成 |  |
| 269 | `menu/txt_toc_rule_edit.xml` | design/generated/menu/txt_toc_rule_edit.vue | 壳子已生成 |  |
| 270 | `menu/txt_toc_rule_item.xml` | design/generated/menu/txt_toc_rule_item.vue | 壳子已生成 |  |
| 271 | `menu/txt_toc_rule_sel.xml` | design/generated/menu/txt_toc_rule_sel.vue | 壳子已生成 |  |
| 272 | `menu/txt_toc_rule.xml` | design/generated/menu/txt_toc_rule.vue | 壳子已生成 |  |
| 273 | `menu/verification_code.xml` | design/generated/menu/verification_code.vue | 壳子已生成 |  |
| 274 | `menu/web_view.xml` | design/generated/menu/web_view.vue | 壳子已生成 |  |
| 275 | `xml/about.xml` | views/config/AboutView.vue | 已完成 |  |
| 276 | `xml/file_paths.xml` |  | 不适用 |  |
| 277 | `xml/network_security_config.xml` |  | 不适用 |  |
| 278 | `xml/pref_config_aloud.xml` | design/generated/xml/pref_config_aloud.vue | 壳子已生成 |  |
| 279 | `xml/pref_config_backup.xml` | design/generated/xml/pref_config_backup.vue | 壳子已生成 |  |
| 280 | `xml/pref_config_cover.xml` | design/generated/xml/pref_config_cover.vue | 壳子已生成 |  |
| 281 | `xml/pref_config_other.xml` | design/generated/xml/pref_config_other.vue | 壳子已生成 |  |
| 282 | `xml/pref_config_read.xml` | design/generated/xml/pref_config_read.vue | 壳子已生成 |  |
| 283 | `xml/pref_config_theme.xml` | design/generated/xml/pref_config_theme.vue | 壳子已生成 |  |
| 284 | `xml/pref_config_welcome.xml` | design/generated/xml/pref_config_welcome.vue | 壳子已生成 |  |
| 285 | `xml/pref_main.xml` | design/generated/xml/pref_main.vue | 壳子已生成 |  |
| 286 | `xml/spen_remote_actions.xml` |  | 不适用 |  |
