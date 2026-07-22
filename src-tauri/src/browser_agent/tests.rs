use super::*;
    use std::sync::{Mutex, OnceLock};

    /// `scale_click_coords_*` and related globals are process-wide; serialize those tests.
    static LLM_COORD_STATE_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    fn llm_coord_state_lock() -> std::sync::MutexGuard<'static, ()> {
        LLM_COORD_STATE_TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("llm coord state test mutex poisoned")
    }

    #[test]
    fn test_extract_phones() {
        let t = "Kontakt: +49 30 12345678 oder 0049 30 87654321";
        let p = extract_telephone_numbers(t);
        assert!(!p.is_empty());
    }

    #[test]
    fn search_page_text_from_plain_text_returns_no_matches_cleanly() {
        let result = search_page_text_from_plain_text("videos", "About us and services").unwrap();
        assert_eq!(result, "No matches found for \"videos\" on page.");
    }

    #[test]
    fn search_page_text_from_plain_text_returns_context_matches() {
        let result = search_page_text_from_plain_text(
            "videos",
            "About. Amvara's videos are featured on the about page.",
        )
        .unwrap();
        assert!(result.contains("Found 1 match"));
        assert!(result.contains("Amvara's videos"));
    }

    #[test]
    fn navigate_failed_detail_strips_headless_chrome_prefix() {
        assert_eq!(
            navigate_failed_detail_from_display("Navigate failed: net::ERR_NAME_NOT_RESOLVED"),
            "net::ERR_NAME_NOT_RESOLVED"
        );
    }

    #[test]
    fn chrome_error_url_detected_case_insensitive() {
        assert!(is_chrome_internal_error_document_url(
            "chrome-error://chromewebdata/"
        ));
    }

    #[test]
    fn post_navigate_ssrf_allows_chrome_error_and_about_blank() {
        assert!(assert_final_document_url_ssrf_post_check(
            "chrome-error://chromewebdata/",
            Some("https://example.com")
        )
        .is_ok());
        assert!(assert_final_document_url_ssrf_post_check("about:blank", None).is_ok());
        assert!(assert_final_document_url_ssrf_post_check("about:srcdoc", None).is_ok());
    }

    #[test]
    fn post_navigate_ssrf_rejects_loopback_http() {
        let r = assert_final_document_url_ssrf_post_check(
            "http://127.0.0.1/",
            Some("https://example.com"),
        );
        assert!(r.is_err(), "{:?}", r);
    }

    #[test]
    fn post_navigate_ssrf_rejects_file_and_javascript() {
        assert!(assert_final_document_url_ssrf_post_check("file:///etc/passwd", None).is_err());
        assert!(assert_final_document_url_ssrf_post_check("javascript:alert(1)", None).is_err());
    }

    #[test]
    fn post_navigate_ssrf_rejects_disallowed_about() {
        assert!(assert_final_document_url_ssrf_post_check("about:config", None).is_err());
    }

    #[test]
    fn redirect_chain_first_hop_matches_http_https_equivalent() {
        assert!(cdp_redirect_chain_first_hop_matches_request(
            "https://example.com/start",
            "http://example.com/start"
        ));
        assert!(!cdp_redirect_chain_first_hop_matches_request(
            "https://example.com/other",
            "http://example.com/start"
        ));
    }

    #[test]
    fn redirect_chain_extract_groups_same_loader_id() {
        let buf = vec![
            ("L1".to_string(), "https://a.example/start".to_string()),
            ("L1".to_string(), "https://b.example/next".to_string()),
            ("L2".to_string(), "https://c.example/".to_string()),
        ];
        let chain =
            cdp_extract_document_redirect_chain_from_rws_buffer(&buf, "http://a.example/start");
        assert_eq!(chain.as_ref().map(|c| c.len()), Some(2));
        assert!(chain.unwrap()[1].contains("b.example"));
    }

    #[test]
    fn redirect_chain_validate_blocks_private_hop_after_public_first() {
        let buf = Mutex::new(VecDeque::new());
        {
            let mut q = buf.lock().unwrap();
            q.push_back(("L9".to_string(), "https://example.com/start".to_string()));
            q.push_back(("L9".to_string(), "http://127.0.0.1/secret".to_string()));
        }
        let r = cdp_validate_redirect_chain_from_rws_buffer(&buf, "https://example.com/start");
        let err = r.expect_err("loopback hop must fail SSRF");
        assert!(
            err.contains("redirect hop") && err.contains("SSRF"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn redirect_chain_uncorrelated_buffer_skips_hop_validation() {
        let buf = Mutex::new(VecDeque::new());
        {
            let mut q = buf.lock().unwrap();
            q.push_back(("L1".to_string(), "https://other.example/".to_string()));
        }
        assert!(cdp_validate_redirect_chain_from_rws_buffer(&buf, "https://nope.example/").is_ok());
    }

    #[test]
    fn new_tab_page_url_matches_browser_use_set() {
        assert!(is_new_tab_page_url("about:blank"));
        assert!(is_new_tab_page_url("chrome://new-tab-page"));
        assert!(is_new_tab_page_url("chrome://new-tab-page/"));
        assert!(is_new_tab_page_url("chrome://newtab"));
        assert!(is_new_tab_page_url("chrome://newtab/"));
        assert!(is_new_tab_page_url("Chrome://newtab"));
        assert!(is_new_tab_page_url("  chrome://new-tab-page/  "));
        assert!(!is_new_tab_page_url("https://example.com"));
        assert!(!is_new_tab_page_url("chrome://settings"));
        assert!(!is_new_tab_page_url(""));
        assert!(!is_new_tab_page_url("About:blank"));
    }

    #[test]
    fn cdp_retry_skipped_when_health_error_also_looks_like_connection_error() {
        let msg = "Browser unresponsive (JavaScript engine not responding): Unable to make method calls because underlying connection is closed";
        assert!(
            !should_retry_cdp_after_clearing_session(msg),
            "health-check failures must not trigger same-call CDP reconnect even with connection substrings"
        );
        assert!(
            is_connection_error(msg),
            "fixture should still be a connection-shaped error for regression coverage"
        );
    }

    #[test]
    fn cdp_retry_allowed_for_plain_connection_error_without_health_prefix() {
        assert!(should_retry_cdp_after_clearing_session(
            "Unable to make method calls because underlying connection is closed"
        ));
    }

    #[test]
    fn format_browser_state_prepends_new_tab_hint() {
        let state = BrowserState {
            current_url: "chrome://new-tab-page/".to_string(),
            page_title: Some("New Tab".to_string()),
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: None,
        };
        let out = format_browser_state_for_llm(&state);
        assert!(out.starts_with("Note: tab is on a browser new-tab"));
        assert!(out.contains("Current page: chrome://new-tab-page/"));
    }

    #[test]
    fn format_browser_state_prepends_chrome_error_hint() {
        let state = BrowserState {
            current_url: "chrome-error://chromewebdata/".to_string(),
            page_title: None,
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: None,
        };
        let out = format_browser_state_for_llm(&state);
        assert!(out.starts_with("Warning: Chrome error or TLS interstitial"));
        assert!(out.contains("Current page: chrome-error://"));
    }

    #[test]
    fn scale_click_coords_scales_from_llm_image_to_viewport() {
        let _g = llm_coord_state_lock();
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(None);
        record_viewport_css_for_llm_coord_scaling(1920, 1080);
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(Some((1400, 850)));
        let (vx, vy) = scale_click_coords_from_llm_screenshot_space(700.0, 425.0);
        assert!((vx - 960.0).abs() < 0.01, "vx={vx}");
        assert!((vy - 540.0).abs() < 0.01, "vy={vy}");
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(None);
        record_viewport_css_for_llm_coord_scaling(0, 0);
    }

    #[test]
    fn scale_click_coords_pass_through_when_no_llm_resize_dims() {
        let _g = llm_coord_state_lock();
        set_last_llm_screenshot_pixel_dims_for_coord_scaling(None);
        record_viewport_css_for_llm_coord_scaling(1920, 1080);
        let (x, y) = scale_click_coords_from_llm_screenshot_space(123.0, 456.0);
        assert_eq!((x, y), (123.0, 456.0));
    }

    #[test]
    fn format_browser_state_prepends_viewport_document_scroll_before_current_page() {
        let state = BrowserState {
            current_url: "https://example.com/long".to_string(),
            page_title: None,
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: Some(BrowserLayoutMetrics {
                scroll_x: 0,
                scroll_y: 120,
                viewport_width: 1280,
                viewport_height: 720,
                document_width: 1280,
                document_height: 2400,
            }),
        };
        let out = format_browser_state_for_llm(&state);
        let idx_vp = out.find("Viewport: 1280x720\n").expect("viewport line");
        let idx_doc = out.find("Document: 1280x2400\n").expect("document line");
        let idx_scroll = out.find("Scroll: (0, 120)\n").expect("scroll line");
        let idx_cur = out.find("Current page:").expect("current page");
        assert!(idx_vp < idx_doc && idx_doc < idx_scroll && idx_scroll < idx_cur);
    }

    #[test]
    fn format_browser_state_includes_recent_js_dialogs_section() {
        clear_cdp_js_dialog_history();
        let state = BrowserState {
            current_url: "https://example.com".to_string(),
            page_title: None,
            interactables: vec![],
            resource_timing_entry_count: None,
            layout_metrics: None,
        };
        let out_empty = format_browser_state_for_llm(&state);
        assert!(out_empty.contains("Recent JS dialogs:\nNone\n"));

        record_cdp_js_dialog_dismissed(&DialogType::Alert, "token-xyz");
        let out = format_browser_state_for_llm(&state);
        assert!(out.contains("Recent JS dialogs:\n"));
        assert!(out.contains("[alert] token-xyz"));
        clear_cdp_js_dialog_history();
    }

    #[test]
    fn post_navigate_detects_chrome_error_document() {
        let m = post_navigate_load_failure_message(
            "https://example.com",
            "chrome-error://chromewebdata/",
            None,
        );
        assert!(m.is_some());
        assert!(m.unwrap().contains("Navigation failed"));
    }

    #[test]
    fn tls_cert_error_text_maps_to_tls_tool_message() {
        let detail = "net::ERR_CERT_AUTHORITY_INVALID";
        assert_eq!(
            tls_certificate_error_class_from_chrome_detail(detail),
            Some("cert_authority")
        );
        let msg = navigation_tool_result_for_failed_navigate("https://bad.example/", detail);
        assert!(msg.contains("TLS/certificate"));
        assert!(msg.contains("FETCH_URL"));
    }

    #[test]
    fn sanitize_navigation_error_redacts_user_path() {
        let s = sanitize_navigation_error_for_llm("net::ERR_FAILED /Users/alice/secret/file");
        assert!(!s.contains("/Users/alice"));
        assert!(s.contains("[path]"));
    }

    #[test]
    fn parse_browser_keys_accepts_escape_and_meta_f() {
        let (m, k) = parse_browser_keys_chord("Escape").unwrap();
        assert!(m.is_empty());
        assert_eq!(k, "Escape");
        let (m, k) = parse_browser_keys_chord("Meta+f").unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(k, "f");
        let (m, k) = parse_browser_keys_chord("Ctrl+Shift+Enter").unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(k, "Enter");
    }

    #[test]
    fn parse_browser_keys_rejects_plain_letter_and_unknown() {
        assert!(parse_browser_keys_chord("f").is_err());
        assert!(parse_browser_keys_chord("Meta+F12").is_err());
        assert!(parse_browser_keys_chord("Win+f").is_err());
    }

    #[test]
    fn truncate_markdown_at_blocks_unchanged_when_under_limit() {
        let s = "one\n\ntwo";
        assert_eq!(truncate_markdown_at_blocks(s, 100), s);
    }

    #[test]
    fn truncate_markdown_at_blocks_drops_tail_blocks() {
        let s = "a\n\nbbbb\n\ncccccccccc";
        let out = truncate_markdown_at_blocks(s, 8);
        assert!(out.starts_with("a\n\nbbbb"));
        assert!(out.contains("[Truncated:"));
        assert!(!out.contains("cccccccccc"));
    }

    fn sample_interactable(
        index: u32,
        tag: &str,
        accessible_name: Option<&str>,
        text: &str,
    ) -> Interactable {
        Interactable {
            index,
            tag: tag.to_string(),
            text: text.to_string(),
            href: None,
            placeholder: None,
            input_type: None,
            contenteditable: false,
            datepicker_like: false,
            accessible_name: accessible_name.map(String::from),
            ax_role: None,
            backend_dom_node_id: None,
            dom_name: None,
            aria_label: None,
            bounds_css: None,
            annot_bounds_css: None,
            from_subframe: false,
            covered: false,
        }
    }

    #[test]
    fn interactable_identity_remap_after_order_swap() {
        let stale_target = sample_interactable(2, "button", Some("Keep"), "Keep");
        let fresh = vec![
            sample_interactable(1, "button", Some("Keep"), "Keep"),
            sample_interactable(2, "button", Some("Transient"), "Transient"),
        ];
        assert_eq!(
            super::find_unique_identity_match(&stale_target, &fresh).unwrap(),
            1
        );
    }

    #[test]
    fn interactable_identity_ambiguous_returns_error() {
        let stale = sample_interactable(1, "button", Some("Go"), "Go");
        let fresh = vec![
            sample_interactable(1, "button", Some("Go"), "Go"),
            sample_interactable(2, "button", Some("Go"), "Go"),
        ];
        assert!(super::find_unique_identity_match(&stale, &fresh).is_err());
    }

    #[test]
    fn managed_tab_cap_focus_index_closes_tab_before_focus() {
        assert_eq!(new_focus_index_after_close(3, 1, 5), 2);
    }

    #[test]
    fn managed_tab_cap_focus_index_closes_tab_after_focus() {
        assert_eq!(new_focus_index_after_close(2, 4, 4), 2);
    }

    #[test]
    fn managed_tab_cap_focus_index_closes_focused_nonzero() {
        assert_eq!(new_focus_index_after_close(2, 2, 4), 1);
    }

    #[test]
    fn managed_tab_cap_focus_index_closes_focused_zero() {
        assert_eq!(new_focus_index_after_close(0, 0, 3), 0);
    }

    #[test]
    fn managed_tab_cap_focus_index_empty_tabs() {
        assert_eq!(new_focus_index_after_close(2, 0, 0), 0);
    }

    #[test]
    fn managed_tab_cap_focus_index_clamps_high() {
        assert_eq!(new_focus_index_after_close(10, 0, 3), 2);
    }
