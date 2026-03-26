//! Inline directive tags in model text (`[[tag]]` / `[[tag: arg]]`): parse, strip for display,
//! and expose delivery hints for Discord (thread reply, screenshot attach, paragraph splits).

use regex::Regex;
use std::sync::OnceLock;

/// Appended to agent-router execution system prompts (both fast-path and normal execution).
pub const EXECUTION_SYSTEM_SECTION: &str = r#"

## Reply delivery tags (optional)

You may include these **inline** markers anywhere in your **final** assistant text. The app removes them before the user sees the message; they only control delivery.

- `[[thread_reply]]` — (Discord) reply to the user’s message in-line (message reference).
- `[[attach_screenshot]]` — attach the most recent browser screenshot from this run, if one exists.
- `[[split_long]]` — (Discord) prefer sending the reply as multiple messages split on paragraph boundaries (blank lines), when that yields more than one chunk.

Use sparingly; omit all of them for a normal reply."#;

/// CPU / non-agent chat: short reminder so models know tags exist (full list matches execution).
pub const NON_AGENT_DIRECTIVE_APPEND: &str = r#"

**Optional delivery tags** (stripped before the user sees text): `[[thread_reply]]`, `[[attach_screenshot]]`, `[[split_long]]` — same meaning as in the agent router when those surfaces apply."#;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirectiveDeliveryFlags {
    pub thread_reply: bool,
    pub attach_screenshot: bool,
    pub split_long: bool,
}

fn directive_tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\[\[([a-zA-Z0-9_]+)(?::[^\]]*)?\]\]")
            .expect("directive tag regex")
    })
}

/// Remove all `[[...]]` directive spans and collect known delivery flags (case-insensitive names).
pub fn parse_and_strip_directive_tags(text: &str) -> (String, DirectiveDeliveryFlags) {
    let re = directive_tag_regex();
    let mut flags = DirectiveDeliveryFlags::default();
    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;
    for cap in re.captures_iter(text) {
        let m = match cap.get(0) {
            Some(x) => x,
            None => continue,
        };
        out.push_str(&text[last..m.start()]);
        if let Some(name_cap) = cap.get(1) {
            match name_cap.as_str().to_ascii_lowercase().as_str() {
                "attach_screenshot" => flags.attach_screenshot = true,
                "thread_reply" => flags.thread_reply = true,
                "split_long" => flags.split_long = true,
                _ => {}
            }
        }
        last = m.end();
    }
    out.push_str(&text[last..]);
    (out, flags)
}

#[inline]
pub fn strip_inline_directive_tags_for_display(text: &str) -> String {
    parse_and_strip_directive_tags(text).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_known_tags_and_sets_flags() {
        let (s, f) = parse_and_strip_directive_tags(
            "Hello [[thread_reply]] world [[attach_screenshot]] x [[split_long]]!",
        );
        assert_eq!(s, "Hello  world  x !");
        assert!(f.thread_reply && f.attach_screenshot && f.split_long);
    }

    #[test]
    fn tag_with_arg_still_recognized() {
        let (s, f) = parse_and_strip_directive_tags("Done. [[attach_screenshot: current]]");
        assert_eq!(s, "Done. ");
        assert!(f.attach_screenshot);
    }

    #[test]
    fn case_insensitive() {
        let (_, f) = parse_and_strip_directive_tags("[[THREAD_REPLY]]");
        assert!(f.thread_reply);
    }

    #[test]
    fn unknown_tag_stripped_only() {
        let (s, f) = parse_and_strip_directive_tags("a [[unknown_thing]] b");
        assert_eq!(s, "a  b");
        assert!(!f.thread_reply && !f.attach_screenshot && !f.split_long);
    }
}
