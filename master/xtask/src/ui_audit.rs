//! `cargo xtask studio-audit` UI-consistency scanner (workstream A2, design-consistency effort).
//!
//! Scans every `.svelte` file under `app-tauri/src` and reports `file:line rule message`.
//! Report-only by default; `--strict <glob-list-file>` fails on any violation in the listed
//! ("swept") files; `--write-inventory` regenerates `docs/UI_CONSISTENCY_INVENTORY.md`.
//! Pure functions only (no I/O) except `scan_tree`, so every rule is unit-tested on fixture strings.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    RawColour,
    RawPx,
    LocalKey,
    SharedComponent,
    IconOnlyLabel,
    TitleCase,
    Overclaim,
}

impl Rule {
    pub const ALL: [Rule; 7] = [
        Rule::RawColour,
        Rule::RawPx,
        Rule::LocalKey,
        Rule::SharedComponent,
        Rule::IconOnlyLabel,
        Rule::TitleCase,
        Rule::Overclaim,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Rule::RawColour => "raw-colour",
            Rule::RawPx => "raw-px",
            Rule::LocalKey => "local-key",
            Rule::SharedComponent => "shared-component",
            Rule::IconOnlyLabel => "icon-only-label",
            Rule::TitleCase => "title-case",
            Rule::Overclaim => "overclaim",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub file: String,
    pub line: usize,
    pub rule: Rule,
    /// The offending token (for grouping in the inventory), e.g. `#ff3b30` or `padding: 12px`.
    pub detail: String,
    pub message: String,
}

impl Violation {
    pub fn render(&self) -> String {
        format!(
            "{}:{} {} {}",
            self.file,
            self.line,
            self.rule.id(),
            self.message
        )
    }
}

// ---------------------------------------------------------------------------------------------
// Documented lists (edit here, not in the checks)
// ---------------------------------------------------------------------------------------------

/// Shared components, per raw tag. A raw `<tag>` is a violation only when one of these exists as
/// `lib/components/**/<Name>.svelte` (so the rule bites the moment the component lands, and never
/// asks for something that does not exist). The component's own file may use the raw tag.
pub const SHARED_COMPONENTS: &[(&str, &[&str])] = &[
    (
        "button",
        &["Button", "IconButton", "Toggle", "SidebarSection"],
    ),
    (
        "input",
        &[
            "Input",
            "TextField",
            "TextInput",
            "Checkbox",
            "Toggle",
            "Slider",
            "NumberField",
        ],
    ),
    ("select", &["Select", "Dropdown"]),
    ("dialog", &["Modal", "Dialog"]),
];

/// px values that are allowed in spacing / font-size properties: zero and 1px hairlines.
pub const PX_EXCEPTIONS: &[f64] = &[0.0, 1.0];

/// CSS properties whose px values must come from spacing / type tokens.
fn is_spacing_prop(p: &str) -> bool {
    matches!(
        p,
        "gap" | "row-gap" | "column-gap" | "grid-gap" | "font-size" | "line-height" | "font"
    ) || p.starts_with("padding")
        || p.starts_with("margin")
}

/// Words that overclaim. `100%` is handled separately (see `overclaim_hits`).
const OVERCLAIM_WORDS: &[&str] = &["instantly", "perfect", "flawless"];

/// Proper nouns that legitimately start with a capital inside a sentence-case button label.
const PROPER_NOUNS: &[&str] = &[
    "Finder",
    "Explorer",
    "Word",
    "Excel",
    "PowerPoint",
    "Markdown",
    "Windows",
    "Linux",
    "macOS",
    "Studio",
    "Labs",
    "Tauri",
];

// ---------------------------------------------------------------------------------------------
// Source preparation: regions + comment blanking
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Region {
    Markup,
    Script,
    Style,
}

struct Line {
    no: usize,
    text: String,
    region: Region,
}

fn blank_block_comments(line: &str, open: &str, close: &str, in_c: &mut bool) -> String {
    let mut out = String::new();
    let mut rest = line;
    loop {
        if *in_c {
            match rest.find(close) {
                Some(p) => {
                    *in_c = false;
                    rest = &rest[p + close.len()..];
                }
                None => return out,
            }
        } else {
            match rest.find(open) {
                Some(p) => {
                    out.push_str(&rest[..p]);
                    *in_c = true;
                    rest = &rest[p + open.len()..];
                }
                None => {
                    out.push_str(rest);
                    return out;
                }
            }
        }
    }
}

fn strip_line_comment(s: &str) -> String {
    let b = s.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b'/'
            && b[i + 1] == b'/'
            && (i == 0 || b[i - 1].is_ascii_whitespace() || b[i - 1] == b';')
        {
            return s[..i].to_string();
        }
        i += 1;
    }
    s.to_string()
}

fn prepare(src: &str) -> Vec<Line> {
    let mut region = Region::Markup;
    let (mut in_html, mut in_js, mut in_css) = (false, false, false);
    let mut out = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let mut line_region = region;
        let mut next = region;
        match region {
            Region::Markup if !in_html => {
                if let Some(p) = raw.find("<script") {
                    line_region = Region::Script;
                    next = if raw[p..].contains("</script>") {
                        Region::Markup
                    } else {
                        Region::Script
                    };
                } else if let Some(p) = raw.find("<style") {
                    line_region = Region::Style;
                    next = if raw[p..].contains("</style>") {
                        Region::Markup
                    } else {
                        Region::Style
                    };
                }
            }
            Region::Script if raw.contains("</script>") => next = Region::Markup,
            Region::Style if raw.contains("</style>") => next = Region::Markup,
            _ => {}
        }
        let text = match line_region {
            Region::Markup => blank_block_comments(raw, "<!--", "-->", &mut in_html),
            Region::Script => {
                strip_line_comment(&blank_block_comments(raw, "/*", "*/", &mut in_js))
            }
            Region::Style => blank_block_comments(raw, "/*", "*/", &mut in_css),
        };
        out.push(Line {
            no: i + 1,
            text,
            region: line_region,
        });
        region = next;
    }
    out
}

/// Markup-only text, line-aligned with the source (other lines are blank) so that the number of
/// newlines before an offset is the zero-based source line.
fn markup_text(lines: &[Line]) -> String {
    let mut s = String::new();
    for l in lines {
        if l.region == Region::Markup {
            s.push_str(&l.text);
        }
        s.push('\n');
    }
    s
}

// ---------------------------------------------------------------------------------------------
// Tag scanning (quote- and brace-aware)
// ---------------------------------------------------------------------------------------------

/// Index of the `>` that closes the tag whose attributes start at `from` (or `s.len()`).
fn tag_end(s: &str, from: usize) -> usize {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for (i, c) in s[from..].char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if depth > 0 {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            '{' => depth += 1,
            '>' => return from + i,
            _ => {}
        }
    }
    s.len()
}

/// Index of the `}` matching the `{` at `open` (or `s.len()`).
fn brace_end(s: &str, open: usize) -> usize {
    let mut depth = 0i32;
    for (i, c) in s[open..].char_indices() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return open + i;
            }
        }
    }
    s.len()
}

struct Tag {
    line: usize,
    attrs: String,
    open_end: usize,
    self_closing: bool,
}

fn line_of(markup: &str, offset: usize) -> usize {
    markup.as_bytes()[..offset.min(markup.len())]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

fn find_tags(markup: &str, name: &str) -> Vec<Tag> {
    let pat = format!("<{name}");
    let b = markup.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    while from < markup.len() {
        let Some(p) = markup[from..].find(&pat) else {
            break;
        };
        let s = from + p;
        let after = s + pat.len();
        from = after;
        match b.get(after) {
            Some(c) if c.is_ascii_whitespace() || *c == b'>' || *c == b'/' => {}
            _ => continue,
        }
        let end = tag_end(markup, after);
        let attrs = markup[after..end].to_string();
        let self_closing = attrs.trim_end().ends_with('/');
        out.push(Tag {
            line: line_of(markup, s),
            attrs,
            open_end: (end + 1).min(markup.len()),
            self_closing,
        });
        from = end.max(after);
    }
    out
}

#[derive(Default)]
struct Inner {
    has_text: bool,
    dynamic: bool,
    blocks: bool,
    unknown: bool,
    text: String,
}

fn analyze_inner(inner: &str) -> Inner {
    let mut r = Inner::default();
    let mut idx = 0;
    while idx < inner.len() {
        let Some(c) = inner[idx..].chars().next() else {
            break;
        };
        match c {
            '<' => {
                idx = (tag_end(inner, idx + 1) + 1).min(inner.len());
                continue;
            }
            '{' => {
                let close = brace_end(inner, idx);
                let body = inner[idx + 1..close.min(inner.len())].trim_start();
                match body.chars().next() {
                    Some('#') | Some(':') | Some('/') => r.blocks = true,
                    Some('@') => r.unknown = true,
                    _ => {
                        r.dynamic = true;
                        r.has_text = true;
                    }
                }
                idx = (close + 1).min(inner.len());
                continue;
            }
            '&' => {
                // HTML entity (&times; &#215;) is a glyph, not a label.
                let rest = &inner[idx + 1..];
                let n = rest
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '#')
                    .count();
                if n > 0 && rest[n..].starts_with(';') {
                    idx += 1 + n + 1;
                    continue;
                }
            }
            _ => {}
        }
        if c.is_alphanumeric() {
            r.has_text = true;
        }
        r.text.push(c);
        idx += c.len_utf8();
    }
    r
}

// ---------------------------------------------------------------------------------------------
// Rules
// ---------------------------------------------------------------------------------------------

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn colour_hits(text: &str, region: Region) -> Vec<String> {
    let mut hits = Vec::new();
    let b = text.as_bytes();
    // Hex literals.
    for (i, _) in text.match_indices('#') {
        let prev = if i == 0 { ' ' } else { b[i - 1] as char };
        let before = &text[..i];
        let allowed = match region {
            Region::Style => before.contains(':') && !is_word_char(prev),
            _ => {
                matches!(prev, '\'' | '"' | '`' | '(' | ':')
                    || (prev.is_whitespace()
                        && ["style", "fill", "stroke", "color", "background"]
                            .iter()
                            .any(|k| text.contains(k)))
            }
        };
        if !allowed
            || before.ends_with("url(")
            || (before.contains("href=") && matches!(prev, '"' | '\''))
        {
            continue;
        }
        let run = text[i + 1..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .count();
        let next = text[i + 1 + run..].chars().next();
        if matches!(run, 3 | 4 | 6 | 8) && !next.is_some_and(|c| is_word_char(c) || c == '-') {
            hits.push(text[i..i + 1 + run].to_string());
        }
    }
    // Functional notations.
    for f in [
        "rgb(", "rgba(", "hsl(", "hsla(", "hwb(", "oklch(", "oklab(", "lch(", "lab(",
    ] {
        for (i, _) in text.match_indices(f) {
            if text[..i]
                .chars()
                .next_back()
                .is_some_and(|c| is_word_char(c) || c == '-')
            {
                continue;
            }
            let close = text[i..].find(')').map_or(text.len(), |p| i + p + 1);
            let call = &text[i..close];
            if call.contains("var(") {
                continue; // built from theme tokens
            }
            hits.push(call.to_string());
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

fn px_literals(value: &str) -> Vec<String> {
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].1.is_ascii_digit() && (i == 0 || !is_word_char(chars[i - 1].1)) {
            let start = chars[i].0;
            let mut j = i;
            while j < chars.len() && (chars[j].1.is_ascii_digit() || chars[j].1 == '.') {
                j += 1;
            }
            let end = chars.get(j).map_or(value.len(), |c| c.0);
            let num = &value[start..end];
            let tail = &value[end..];
            // px (0 and 1px allowed) and rem / em (only 0 allowed) are all raw sizes.
            for unit in ["px", "rem", "em"] {
                if tail.starts_with(unit)
                    && !tail[unit.len()..].chars().next().is_some_and(is_word_char)
                {
                    let v: f64 = num.parse().unwrap_or(f64::NAN);
                    let allowed = if unit == "px" {
                        PX_EXCEPTIONS.contains(&v)
                    } else {
                        v == 0.0
                    };
                    if !allowed {
                        out.push(format!("{num}{unit}"));
                    }
                    break;
                }
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
}

/// px hits in one `prop: value` declaration.
fn decl_px(decl: &str) -> Vec<String> {
    let tail = decl.rsplit(['{', '}']).next().unwrap_or(decl);
    let Some((prop, value)) = tail.split_once(':') else {
        return Vec::new();
    };
    let prop = prop.trim().to_ascii_lowercase();
    if !is_spacing_prop(&prop) {
        return Vec::new();
    }
    px_literals(value)
        .into_iter()
        .map(|p| format!("{prop}: {p}"))
        .collect()
}

fn markup_px(text: &str) -> Vec<String> {
    let mut hits = Vec::new();
    // style="a: 1px; b: 2px"
    for q in ['"', '\''] {
        let pat = format!("style={q}");
        for (i, _) in text.match_indices(&pat) {
            let from = i + pat.len();
            let to = text[from..].find(q).map_or(text.len(), |p| from + p);
            for decl in text[from..to].split(';') {
                hits.extend(decl_px(decl));
            }
        }
    }
    // style:prop="12px" / style:prop={'12px'}
    for (i, _) in text.match_indices("style:") {
        let rest = &text[i + 6..];
        let Some(eq) = rest.find('=') else { continue };
        let prop = rest[..eq].split('|').next().unwrap_or("").trim();
        if prop.is_empty()
            || prop.contains(char::is_whitespace)
            || !is_spacing_prop(&prop.to_ascii_lowercase())
        {
            continue;
        }
        let v = &rest[eq + 1..];
        let value = match v.chars().next() {
            Some(q @ ('"' | '\'')) => v[1..].split(q).next().unwrap_or(""),
            Some('{') => &v[..brace_end(v, 0).min(v.len())],
            _ => "",
        };
        hits.extend(
            px_literals(value)
                .into_iter()
                .map(|p| format!("{}: {p}", prop.to_ascii_lowercase())),
        );
    }
    hits
}

fn key_hits(text: &str) -> Vec<String> {
    let mut hits = Vec::new();
    for ev in ["keydown", "keyup", "keypress"] {
        for pat in [format!("on{ev}"), format!("on:{ev}")] {
            for (i, _) in text.match_indices(&pat) {
                if !text[..i].chars().next_back().is_some_and(is_word_char) {
                    hits.push(ev.to_string());
                }
            }
        }
        for (i, _) in text.match_indices("addEventListener(") {
            let rest = text[i + 17..].trim_start();
            if let Some(r) = rest.strip_prefix(['\'', '"', '`']) {
                if r.starts_with(ev) {
                    hits.push(ev.to_string());
                }
            }
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

pub fn is_title_case(label: &str) -> bool {
    // Leading emoji / glyph tokens (no letters or digits) are decoration, not words.
    let words: Vec<&str> = label
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .collect();
    if words.len() < 2 {
        return false;
    }
    let mut caps = 0;
    for w in &words[1..] {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric());
        let Some(first) = w.chars().next() else {
            continue;
        };
        if w.chars().count() == 1 {
            continue; // "Flip V": a single letter is a label, not a capitalised word
        }
        let acronym =
            w.chars().count() >= 2 && w.chars().all(|c| c.is_uppercase() || c.is_ascii_digit());
        if acronym || w.chars().any(|c| c.is_ascii_digit()) || PROPER_NOUNS.contains(&w) {
            continue;
        }
        if first.is_lowercase() {
            let minor = [
                "a", "an", "the", "of", "to", "in", "on", "and", "or", "for", "with", "as", "at",
                "by", "from", "vs", "into",
            ];
            if minor.contains(&w) {
                continue;
            }
            return false; // lowercase content word: sentence case
        }
        if first.is_uppercase() {
            caps += 1;
        }
    }
    caps >= 1
}

fn overclaim_hits(lower: &str, allow_hundred_percent: bool) -> Vec<String> {
    let mut hits = Vec::new();
    for w in OVERCLAIM_WORDS {
        for (i, _) in lower.match_indices(w) {
            if lower[..i].chars().next_back().is_some_and(is_word_char) {
                continue;
            }
            let rest = &lower[i + w.len()..];
            let rest = if *w == "perfect" {
                rest.strip_prefix("ly").unwrap_or(rest)
            } else {
                rest
            };
            if !rest.chars().next().is_some_and(is_word_char) {
                hits.push((*w).to_string());
            }
        }
    }
    // "100% <word>" is a claim everywhere except the About component (ADR-005 statement); a bare
    // "100%" (zoom, CSS) is not a claim.
    for (i, _) in lower.match_indices("100%") {
        if lower[..i]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_digit() || c == '.')
        {
            continue;
        }
        let next = lower[i + 4..].trim_start().chars().next();
        if next.is_some_and(char::is_alphabetic) && !allow_hundred_percent {
            hits.push("100%".to_string());
        }
    }
    hits
}

/// The About component is the only place "100% offline" may appear.
fn is_about_file(file: &str) -> bool {
    file.rsplit('/')
        .next()
        .is_some_and(|n| n.starts_with("About"))
}

fn has_label_attr(attrs: &str) -> bool {
    attrs.contains("aria-label")
        || attrs.contains("{title}")
        || attrs.contains("{...")
        || attrs.match_indices("title=").any(|(i, _)| {
            attrs[..i]
                .chars()
                .next_back()
                .is_none_or(char::is_whitespace)
        })
}

fn is_shared_impl(file: &str, tag: &str) -> bool {
    let stem = file
        .rsplit('/')
        .next()
        .unwrap_or(file)
        .trim_end_matches(".svelte");
    SHARED_COMPONENTS
        .iter()
        .any(|(t, names)| *t == tag && names.contains(&stem))
}

/// Audit one `.svelte` source. `shared_tags` lists the raw tags (`button`, `input`, `select`,
/// `dialog`) for which a shared component currently exists in `lib/components`.
pub fn audit_source(file: &str, src: &str, shared_tags: &[&str]) -> Vec<Violation> {
    let lines = prepare(src);
    let mut out: Vec<Violation> = Vec::new();
    let mut push = |line: usize, rule: Rule, detail: String, message: String| {
        out.push(Violation {
            file: file.to_string(),
            line,
            rule,
            detail,
            message,
        });
    };

    for l in &lines {
        // (a) raw colours, any region.
        for h in colour_hits(&l.text, l.region) {
            push(
                l.no,
                Rule::RawColour,
                h.clone(),
                format!("raw colour `{h}`: use a theme token (var(--...) / themeContract)"),
            );
        }
        // (b) raw px: style blocks, then attributes.
        let px = if l.region == Region::Style {
            l.text.split(';').flat_map(decl_px).collect::<Vec<_>>()
        } else if l.region == Region::Markup {
            markup_px(&l.text)
        } else {
            Vec::new()
        };
        for h in px {
            push(
                l.no,
                Rule::RawPx,
                h.clone(),
                format!("raw px `{h}`: use a spacing / type token"),
            );
        }
        if l.region != Region::Style {
            // (c) local key listeners.
            for ev in key_hits(&l.text) {
                push(
                    l.no,
                    Rule::LocalKey,
                    ev.clone(),
                    format!(
                        "local {ev} listener: route shortcuts through core/commandEngine.svelte.ts"
                    ),
                );
            }
            // (f2) overclaiming words (visible strings live in markup and script).
            for w in overclaim_hits(&l.text.to_lowercase(), is_about_file(file)) {
                push(
                    l.no,
                    Rule::Overclaim,
                    w.clone(),
                    format!("overclaiming wording `{w}`: say what the feature does, not how well"),
                );
            }
        }
    }

    let markup = markup_text(&lines);

    // (d) one-off controls where a shared component exists.
    let mut seen: Vec<(usize, &str)> = Vec::new();
    for tag in ["button", "input", "select", "dialog"] {
        if !shared_tags.contains(&tag) || is_shared_impl(file, tag) {
            continue;
        }
        for t in find_tags(&markup, tag) {
            seen.push((t.line, tag));
        }
    }
    if shared_tags.contains(&"dialog") && !is_shared_impl(file, "dialog") {
        for (i, ml) in markup.lines().enumerate() {
            if [
                "role=\"dialog\"",
                "role=\"alertdialog\"",
                "role='dialog'",
                "role='alertdialog'",
            ]
            .iter()
            .any(|r| ml.contains(r))
            {
                seen.push((i + 1, "dialog"));
            }
        }
    }
    seen.sort();
    seen.dedup();
    for (line, tag) in seen {
        push(
            line,
            Rule::SharedComponent,
            format!("<{tag}>"),
            format!("one-off <{tag}>: use the shared component from lib/components"),
        );
    }

    // (e) icon-only buttons and (f1) Title Case labels.
    for t in find_tags(&markup, "button") {
        if t.self_closing {
            continue;
        }
        let Some(close) = markup[t.open_end..].find("</button") else {
            continue;
        };
        let inner = analyze_inner(&markup[t.open_end..t.open_end + close]);
        if inner.unknown {
            continue;
        }
        if !inner.has_text && !has_label_attr(&t.attrs) {
            push(
                t.line,
                Rule::IconOnlyLabel,
                "<button>".to_string(),
                "icon-only button has no aria-label or title".to_string(),
            );
        }
        if inner.has_text && !inner.dynamic && !inner.blocks {
            let label = inner.text.split_whitespace().collect::<Vec<_>>().join(" ");
            if is_title_case(&label) {
                push(
                    t.line,
                    Rule::TitleCase,
                    label.clone(),
                    format!("button label `{label}` is Title Case: use sentence case"),
                );
            }
        }
    }

    out.sort_by(|a, b| (a.line, a.rule, &a.detail).cmp(&(b.line, b.rule, &b.detail)));
    out.dedup();
    out
}

// ---------------------------------------------------------------------------------------------
// Strict-mode globs
// ---------------------------------------------------------------------------------------------

fn gm(p: &[u8], s: &[u8]) -> bool {
    if p.is_empty() {
        return s.is_empty();
    }
    if p.starts_with(b"**/") {
        if gm(&p[3..], s) {
            return true;
        }
        return (0..s.len()).any(|i| s[i] == b'/' && gm(&p[3..], &s[i + 1..]));
    }
    if p.starts_with(b"**") {
        return (0..=s.len()).any(|i| gm(&p[2..], &s[i..]));
    }
    match p[0] {
        b'*' => {
            for i in 0..=s.len() {
                if gm(&p[1..], &s[i..]) {
                    return true;
                }
                if i < s.len() && s[i] == b'/' {
                    break;
                }
            }
            false
        }
        b'?' => !s.is_empty() && s[0] != b'/' && gm(&p[1..], &s[1..]),
        c => !s.is_empty() && s[0] == c && gm(&p[1..], &s[1..]),
    }
}

/// `*` and `?` stay inside a path segment, `**` crosses `/`.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    gm(pattern.as_bytes(), path.as_bytes())
}

/// A file is swept when any pattern matches it as `app-tauri/src/...`, `lib/...` (relative to
/// `app-tauri/src`) or `master/app-tauri/src/...`.
pub fn is_swept(patterns: &[String], file: &str) -> bool {
    let rel = file.strip_prefix("app-tauri/src/").unwrap_or(file);
    let forms = [file.to_string(), rel.to_string(), format!("master/{file}")];
    patterns
        .iter()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty() && !p.starts_with('#'))
        .any(|p| forms.iter().any(|f| glob_match(p, f)))
}

pub fn parse_patterns(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Tree scan + inventory
// ---------------------------------------------------------------------------------------------

// ---------------------------------------------------------------------------------------------
// px allowlist: canvas / geometry code is exempt from the raw-px rule (tokens are UI chrome only)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum PxScope {
    Whole,
    Lines(usize, usize),
    /// Substring of the offending source line (selector, function or variable name).
    Symbol(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PxAllow {
    pub path: String,
    pub scope: PxScope,
    pub reason: String,
}

/// Parse `path[:line-range|:symbol] \u{2014} reason` lines. `#` comments and blanks are skipped.
/// An entry with no reason is an error.
pub fn parse_px_allowlist(content: &str) -> Result<Vec<PxAllow>, String> {
    let mut out = Vec::new();
    for (i, raw) in content.lines().enumerate() {
        let l = raw.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let (head, reason) = l
            .split_once(" \u{2014} ")
            .or_else(|| l.split_once(" \u{2014}"))
            .ok_or_else(|| format!("line {}: allowlist entry has no reason: `{l}`", i + 1))?;
        let reason = reason.trim();
        let head = head.trim();
        if reason.is_empty() || head.is_empty() {
            return Err(format!(
                "line {}: allowlist entry has no reason: `{l}`",
                i + 1
            ));
        }
        let (path, scope) = match head.split_once(':') {
            None => (head, PxScope::Whole),
            Some((p, rest)) => {
                let rest = rest.trim();
                let range = rest.split_once('-').and_then(|(a, b)| {
                    Some((
                        a.trim().parse::<usize>().ok()?,
                        b.trim().parse::<usize>().ok()?,
                    ))
                });
                let scope = match (range, rest.parse::<usize>()) {
                    (Some((a, b)), _) => PxScope::Lines(a, b),
                    (None, Ok(n)) => PxScope::Lines(n, n),
                    _ if rest.is_empty() => {
                        return Err(format!("line {}: empty scope after `:`", i + 1))
                    }
                    _ if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) => {
                        return Err(format!("line {}: bad line range `{rest}`", i + 1))
                    }
                    _ => PxScope::Symbol(rest.to_string()),
                };
                (p.trim(), scope)
            }
        };
        out.push(PxAllow {
            path: path.to_string(),
            scope,
            reason: reason.to_string(),
        });
    }
    Ok(out)
}

/// Is this violation exempted by the allowlist? Only `raw-px` and `local-key` can be exempted.
pub fn px_allowed(entries: &[PxAllow], v: &Violation, source_line: &str) -> bool {
    matches!(v.rule, Rule::RawPx | Rule::LocalKey)
        && entries.iter().any(|e| {
            (v.file == e.path || v.file.ends_with(&format!("/{}", e.path)))
                && match &e.scope {
                    PxScope::Whole => true,
                    PxScope::Lines(a, b) => (*a..=*b).contains(&v.line),
                    PxScope::Symbol(s) => source_line.contains(s.as_str()),
                }
        })
}

pub struct Scan {
    pub files: Vec<String>,
    pub violations: Vec<Violation>,
    pub shared_tags: Vec<&'static str>,
    pub census: Vec<(&'static str, usize)>,
    /// raw-px findings exempted by `ui-px-allowlist.txt`.
    pub px_exempt: usize,
}

/// Scan every `.svelte` under `<workspace_root>/app-tauri/src`.
pub fn scan_tree(workspace_root: &std::path::Path) -> Scan {
    let src_root = workspace_root.join("app-tauri/src");
    let mut paths: Vec<std::path::PathBuf> = walkdir::WalkDir::new(&src_root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.into_path())
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("svelte"))
        .collect();
    paths.sort();

    let allow_path = workspace_root.join("xtask/ui-px-allowlist.txt");
    let allow = match std::fs::read_to_string(&allow_path) {
        Ok(c) => parse_px_allowlist(&c).unwrap_or_else(|e| {
            eprintln!("studio-audit: ui-px-allowlist.txt: {e}");
            std::process::exit(2)
        }),
        Err(_) => Vec::new(),
    };
    let mut px_exempt = 0usize;

    let comp_root = src_root.join("lib/components");
    let stems: Vec<String> = paths
        .iter()
        .filter(|p| p.starts_with(&comp_root))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    let shared_tags: Vec<&'static str> = SHARED_COMPONENTS
        .iter()
        .filter(|(_, names)| names.iter().any(|n| stems.iter().any(|s| s == n)))
        .map(|(t, _)| *t)
        .collect();

    let mut census: Vec<(&'static str, usize)> =
        vec![("button", 0), ("input", 0), ("select", 0), ("dialog", 0)];
    let mut files = Vec::new();
    let mut violations = Vec::new();
    for p in &paths {
        let rel = p
            .strip_prefix(workspace_root)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        let content = std::fs::read_to_string(p).unwrap_or_default();
        let src_lines: Vec<&str> = content.lines().collect();
        for v in audit_source(&rel, &content, &shared_tags) {
            let sl = src_lines.get(v.line.wrapping_sub(1)).copied().unwrap_or("");
            if px_allowed(&allow, &v, sl) {
                px_exempt += 1;
            } else {
                violations.push(v);
            }
        }
        let markup = markup_text(&prepare(&content));
        for (tag, n) in census.iter_mut() {
            *n += find_tags(&markup, tag).len();
            if *tag == "dialog" {
                *n += markup.matches("role=\"dialog\"").count()
                    + markup.matches("role=\"alertdialog\"").count();
            }
        }
        files.push(rel);
    }
    Scan {
        files,
        violations,
        shared_tags,
        census,
        px_exempt,
    }
}

fn rule_blurb(r: Rule) -> &'static str {
    match r {
        Rule::RawColour => "hex / rgb() / hsl() colour literals (theme tokens are fine)",
        Rule::RawPx => {
            "raw px in padding, margin, gap, font-size, line-height (0px and 1px allowed)"
        }
        Rule::LocalKey => "keydown / keyup / keypress listeners outside commandEngine",
        Rule::SharedComponent => {
            "raw button / input / select / dialog where a shared component exists"
        }
        Rule::IconOnlyLabel => "icon-only buttons without aria-label or title",
        Rule::TitleCase => "Title Case button labels (sentence case wanted)",
        Rule::Overclaim => "overclaiming words: instantly, perfect, flawless, 100%",
    }
}

fn short_rule(r: Rule) -> &'static str {
    match r {
        Rule::RawColour => "colour",
        Rule::RawPx => "px",
        Rule::LocalKey => "key",
        Rule::SharedComponent => "shared",
        Rule::IconOnlyLabel => "icon",
        Rule::TitleCase => "case",
        Rule::Overclaim => "claim",
    }
}

fn md_cell(s: &str) -> String {
    let t: String = s.chars().take(40).collect();
    t.replace('|', "\\|").replace('`', "'")
}

/// Per-file counts and the top 3 `(rule, detail)` groups.
fn file_summary(vs: &[&Violation]) -> (Vec<usize>, String) {
    let counts: Vec<usize> = Rule::ALL
        .iter()
        .map(|r| vs.iter().filter(|v| v.rule == *r).count())
        .collect();
    let mut groups: Vec<((Rule, &str), usize)> = Vec::new();
    for v in vs {
        match groups
            .iter_mut()
            .find(|(k, _)| *k == (v.rule, v.detail.as_str()))
        {
            Some((_, n)) => *n += 1,
            None => groups.push(((v.rule, v.detail.as_str()), 1)),
        }
    }
    groups.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top = groups
        .iter()
        .take(3)
        .map(|((r, d), n)| format!("{} `{}` x{}", r.id(), md_cell(d), n))
        .collect::<Vec<_>>()
        .join("; ");
    (counts, top)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn render_inventory(violations: &[Violation], files: &[String]) -> String {
    render_inventory_with(violations, files, "")
}

pub fn render_inventory_with(violations: &[Violation], files: &[String], notes: &str) -> String {
    let mut by_file: Vec<(&str, Vec<&Violation>)> = Vec::new();
    for f in files {
        let vs: Vec<&Violation> = violations.iter().filter(|v| &v.file == f).collect();
        if !vs.is_empty() {
            by_file.push((f.as_str(), vs));
        }
    }
    by_file.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));

    let mut md = String::new();
    md.push_str("# UI consistency inventory\n\n");
    md.push_str("<!-- Generated by `cargo xtask studio-audit --write-inventory`. Do not edit by hand. -->\n\n");
    md.push_str(&format!(
        "{} `.svelte` files scanned under `app-tauri/src`; {} with violations, {} violations in total.\n\n",
        files.len(),
        by_file.len(),
        violations.len()
    ));
    md.push_str(
        "## Totals by rule\n\n| Rule | Violations | Files | Checks |\n|---|---:|---:|---|\n",
    );
    for r in Rule::ALL {
        let n = violations.iter().filter(|v| v.rule == r).count();
        let nf = by_file
            .iter()
            .filter(|(_, vs)| vs.iter().any(|v| v.rule == r))
            .count();
        md.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            r.id(),
            n,
            nf,
            rule_blurb(r)
        ));
    }
    md.push_str("\n## Per screen / file\n\nColumns: ");
    md.push_str(
        &Rule::ALL
            .iter()
            .map(|r| format!("{} = `{}`", short_rule(*r), r.id()))
            .collect::<Vec<_>>()
            .join(", "),
    );
    md.push_str(".\n\n| File ");
    for r in Rule::ALL {
        md.push_str(&format!("| {} ", short_rule(r)));
    }
    md.push_str("| Total | Top 3 issues |\n|---");
    for _ in Rule::ALL {
        md.push_str("|---:");
    }
    md.push_str("|---:|---|\n");
    for (f, vs) in &by_file {
        let (counts, top) = file_summary(vs);
        md.push_str(&format!("| `{f}` "));
        for c in counts {
            md.push_str(&format!(
                "| {} ",
                if c == 0 {
                    "-".to_string()
                } else {
                    c.to_string()
                }
            ));
        }
        md.push_str(&format!("| {} | {} |\n", vs.len(), top));
    }
    if !notes.is_empty() {
        md.push('\n');
        md.push_str(notes);
    }
    md
}

pub fn inventory_notes(scan: &Scan) -> String {
    let mut s = String::from("## Shared components and raw controls\n\n");
    s.push_str(&format!(
        "Findings exempted by `xtask/ui-px-allowlist.txt` (canvas geometry, deferred listeners): {}.\n\n",
        scan.px_exempt
    ));
    s.push_str("Rule `shared-component` only fires for a tag whose shared component exists in `lib/components` (`SHARED_COMPONENTS` in `xtask/src/ui_audit.rs`).\n\n");
    s.push_str(
        "| Raw tag | Raw uses | Shared component present | Candidate names |\n|---|---:|---|---|\n",
    );
    for (tag, n) in &scan.census {
        let names = SHARED_COMPONENTS
            .iter()
            .find(|(t, _)| t == tag)
            .map(|(_, n)| n.join(", "))
            .unwrap_or_default();
        let present = if scan.shared_tags.contains(tag) {
            "yes"
        } else {
            "no"
        };
        s.push_str(&format!("| `<{tag}>` | {n} | {present} | {names} |\n"));
    }
    s.push_str("\n## Documented exceptions\n\n");
    s.push_str("- px: `0` and `1px` (hairlines) are allowed in spacing / font properties; only padding*, margin*, gap, row-gap, column-gap, font-size, line-height and font are checked, in `<style>` blocks and `style=` / `style:` attributes.\n");
    s.push_str("- colour: `var(--token)` and colour functions built from `var(...)` are allowed; `href=\"#anchor\"` and `url(#id)` are not colours.\n");
    s.push_str("- wording: `100%` followed by a word is flagged unless the line says `offline` (the About statement); a bare `100%` (zoom, CSS) is fine. Proper nouns (Finder, Word, Markdown, ...) are allowed capitals in button labels.\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str, shared: &[&str]) -> Vec<Violation> {
        audit_source("lib/X.svelte", src, shared)
    }
    fn of(src: &str, rule: Rule) -> Vec<Violation> {
        run(src, &["button", "input", "select", "dialog"])
            .into_iter()
            .filter(|v| v.rule == rule)
            .collect()
    }
    fn count(src: &str, rule: Rule) -> usize {
        of(src, rule).len()
    }

    // (a) raw colours
    #[test]
    fn colour_positive() {
        assert_eq!(
            count(
                "<style>\n.a {\n  color: #ff3b30;\n}\n</style>",
                Rule::RawColour
            ),
            1
        );
        assert_eq!(
            count(
                "<div style=\"background: rgba(0,0,0,0.5)\"></div>",
                Rule::RawColour
            ),
            1
        );
        assert_eq!(
            count("<script>\n const c = '#abc';\n</script>", Rule::RawColour),
            1
        );
        assert_eq!(
            count(
                "<style>\n.a { border-color: hsl(10 20% 30%); }\n</style>",
                Rule::RawColour
            ),
            1
        );
        let v = of(
            "<style>\n.a {\n  color: #ff3b30;\n}\n</style>",
            Rule::RawColour,
        );
        assert_eq!(v[0].line, 3);
    }
    #[test]
    fn colour_negative() {
        assert_eq!(
            count(
                "<style>\n.a { color: var(--accent); }\n</style>",
                Rule::RawColour
            ),
            0
        );
        assert_eq!(count("<a href=\"#top\">Top</a>", Rule::RawColour), 0);
        assert_eq!(
            count(
                "<style>\n/* old: #fff */\n.a { color: currentColor; }\n</style>",
                Rule::RawColour
            ),
            0
        );
        assert_eq!(count("<p>&#123;x&#125; issue #123</p>", Rule::RawColour), 0);
        assert_eq!(
            count(
                "<script>\n const t = tokens.token('--accent');\n</script>",
                Rule::RawColour
            ),
            0
        );
    }

    // (b) raw px spacing / font sizes
    #[test]
    fn px_positive() {
        assert_eq!(
            count("<style>\n.a { padding: 12px 8px; }\n</style>", Rule::RawPx),
            2
        );
        assert_eq!(
            count("<style>\n.a { font-size: 13px; }\n</style>", Rule::RawPx),
            1
        );
        assert_eq!(
            count("<div style=\"margin-top: 6px\"></div>", Rule::RawPx),
            1
        );
        assert_eq!(
            count("<div style:font-size=\"14px\"></div>", Rule::RawPx),
            1
        );
        assert_eq!(
            count(
                "<style>\n.a { gap: calc(var(--s) + 4px); }\n</style>",
                Rule::RawPx
            ),
            1
        );
    }
    // rem / em literals in spacing and font properties are raw sizes too (sweep rule).
    #[test]
    fn rem_em_positive() {
        assert_eq!(
            count("<style>\n.a { font-size: 0.85rem; }\n</style>", Rule::RawPx),
            1
        );
        assert_eq!(
            count(
                "<style>\n.a { padding: 0.5rem 1rem; gap: 0.75em; }\n</style>",
                Rule::RawPx
            ),
            3
        );
        assert_eq!(
            count("<div style=\"margin-top: 1.25rem\"></div>", Rule::RawPx),
            1
        );
    }
    #[test]
    fn rem_em_negative() {
        assert_eq!(
            count(
                "<style>\n.a { padding: 0 0rem; font-size: var(--font-size-body); letter-spacing: -0.01em; width: 2rem; }\n</style>",
                Rule::RawPx
            ),
            0
        );
    }
    #[test]
    fn px_negative() {
        // 1px hairlines and zero are the documented exceptions.
        assert_eq!(
            count(
                "<style>\n.a { padding: 1px 0; margin: 0px; }\n</style>",
                Rule::RawPx
            ),
            0
        );
        assert_eq!(
            count(
                "<style>\n.a { padding: var(--space-2); }\n</style>",
                Rule::RawPx
            ),
            0
        );
        // Not a spacing / font property.
        assert_eq!(
            count(
                "<style>\n.a { width: 24px; border: 2px solid var(--x); }\n</style>",
                Rule::RawPx
            ),
            0
        );
        assert_eq!(
            count(
                "<style>\n@media (max-width: 600px) { .a { padding: var(--s); } }\n</style>",
                Rule::RawPx
            ),
            0
        );
        assert_eq!(
            count("<div style=\"padding: {n}px\"></div>", Rule::RawPx),
            0
        );
    }

    // (c) local key listeners
    #[test]
    fn key_positive() {
        assert_eq!(
            count(
                "<script>\n window.addEventListener('keydown', h);\n</script>",
                Rule::LocalKey
            ),
            1
        );
        assert_eq!(count("<div onkeydown={h}></div>", Rule::LocalKey), 1);
        assert_eq!(count("<div on:keyup={h}></div>", Rule::LocalKey), 1);
        assert_eq!(count("<svelte:window onkeyup={h} />", Rule::LocalKey), 1);
    }
    #[test]
    fn key_negative() {
        assert_eq!(
            count("<div onclick={h} onpointerdown={h}></div>", Rule::LocalKey),
            0
        );
        assert_eq!(
            count(
                "<script>\n // window.addEventListener('keydown', h);\n</script>",
                Rule::LocalKey
            ),
            0
        );
        assert_eq!(count("<!-- <div onkeydown={h}> -->", Rule::LocalKey), 0);
    }

    // (d) one-off controls where a shared component exists
    #[test]
    fn shared_positive() {
        assert_eq!(
            count("<button class=\"x\">Go</button>", Rule::SharedComponent),
            1
        );
        assert_eq!(count("<input type=\"text\" />", Rule::SharedComponent), 1);
        assert_eq!(
            count("<select bind:value={v}></select>", Rule::SharedComponent),
            1
        );
        assert_eq!(count("<dialog open></dialog>", Rule::SharedComponent), 1);
        assert_eq!(
            count(
                "<div role=\"dialog\" aria-modal=\"true\"></div>",
                Rule::SharedComponent
            ),
            1
        );
    }
    #[test]
    fn shared_negative() {
        // No shared component exists for the tag: nothing to prefer.
        assert_eq!(
            run("<button>Go</button>", &[])
                .iter()
                .filter(|v| v.rule == Rule::SharedComponent)
                .count(),
            0
        );
        assert_eq!(
            run("<input />", &["button"])
                .iter()
                .filter(|v| v.rule == Rule::SharedComponent)
                .count(),
            0
        );
        // Components and look-alike tags are not raw controls.
        assert_eq!(
            count(
                "<Button>Go</Button><buttonish></buttonish><div role=\"button\"></div>",
                Rule::SharedComponent
            ),
            0
        );
    }

    // (e) icon-only buttons need aria-label or title
    #[test]
    fn icon_only_positive() {
        assert_eq!(
            count(
                "<button onclick={x}><StudioIcon name=\"x\" /></button>",
                Rule::IconOnlyLabel
            ),
            1
        );
        assert_eq!(count("<button\n  class=\"c\"\n  onclick={() => a > b}\n>\n  <svg><path d=\"M0 0\"/></svg>\n</button>", Rule::IconOnlyLabel), 1);
        assert_eq!(
            count(
                "<button type=\"button\">\u{00d7}</button>",
                Rule::IconOnlyLabel
            ),
            1
        );
        let v = of(
            "<p>x</p>\n<button><span class=\"i\"></span></button>",
            Rule::IconOnlyLabel,
        );
        assert_eq!(v[0].line, 2);
    }
    #[test]
    fn icon_only_negative() {
        assert_eq!(
            count(
                "<button aria-label=\"Close\"><svg></svg></button>",
                Rule::IconOnlyLabel
            ),
            0
        );
        assert_eq!(
            count(
                "<button title=\"Close\">\u{00d7}</button>",
                Rule::IconOnlyLabel
            ),
            0
        );
        assert_eq!(
            count(
                "<button><StudioIcon name=\"x\" /> Save</button>",
                Rule::IconOnlyLabel
            ),
            0
        );
        assert_eq!(
            count("<button><svg></svg>{label}</button>", Rule::IconOnlyLabel),
            0
        );
        assert_eq!(
            count(
                "<button aria-label={t('close')}><svg></svg></button>",
                Rule::IconOnlyLabel
            ),
            0
        );
    }

    // (f1) Title Case button labels
    #[test]
    fn title_case_positive() {
        assert_eq!(count("<button>Save As</button>", Rule::TitleCase), 1);
        assert_eq!(
            count(
                "<button type=\"button\">\n  Open Recent File\n</button>",
                Rule::TitleCase
            ),
            1
        );
        assert_eq!(
            count(
                "<button><StudioIcon name=\"a\" /> Add New Page</button>",
                Rule::TitleCase
            ),
            1
        );
    }
    #[test]
    fn title_case_negative() {
        assert_eq!(count("<button>Save as PDF</button>", Rule::TitleCase), 0);
        assert_eq!(count("<button>Save</button>", Rule::TitleCase), 0);
        assert_eq!(count("<button>Open in Finder</button>", Rule::TitleCase), 0);
        assert_eq!(count("<button>Merge PDF</button>", Rule::TitleCase), 0);
        assert_eq!(
            count(
                "<button>{#if a}Pin Tool{:else}Unpin{/if}</button>",
                Rule::TitleCase
            ),
            0
        );
        assert_eq!(count("<button>{label} Now</button>", Rule::TitleCase), 0);
        // A leading emoji / glyph is not a word; single letters are not capitalised words.
        assert_eq!(
            count(
                "<button>\u{2699}\u{fe0f} Settings</button>",
                Rule::TitleCase
            ),
            0
        );
        assert_eq!(
            count(
                "<button>\u{1f4c1} Browse files...</button>",
                Rule::TitleCase
            ),
            0
        );
        assert_eq!(count("<button>Flip V</button>", Rule::TitleCase), 0);
        assert_eq!(
            count("<button>\u{1f4be} Save As...</button>", Rule::TitleCase),
            1
        );
    }

    // (f2) overclaiming words
    #[test]
    fn overclaim_positive() {
        assert_eq!(
            count("<p>Instantly converts every file.</p>", Rule::Overclaim),
            1
        );
        assert_eq!(count("<h2>Perfect results</h2>", Rule::Overclaim), 1);
        assert_eq!(count("<p>100% accurate OCR</p>", Rule::Overclaim), 1);
        assert_eq!(
            count(
                "<script>\n const m = 'Perfectly aligned';\n</script>",
                Rule::Overclaim
            ),
            1
        );
    }
    #[test]
    fn overclaim_negative() {
        // "100%" is allowed only in the About component (DESIGN_LANGUAGE section 5, ADR-005).
        assert_eq!(
            count("<style>\n.a { width: 100%; }\n</style>", Rule::Overclaim),
            0
        );
        assert_eq!(
            count(
                "<button>100%</button><div style=\"height: 100%\"></div>",
                Rule::Overclaim
            ),
            0
        );
        assert_eq!(
            count(
                "<p>An imperfect scan; perfectionism aside.</p>",
                Rule::Overclaim
            ),
            0
        );
        assert_eq!(
            count("<script>\n // instantly fast\n</script>", Rule::Overclaim),
            0
        );
    }

    #[test]
    fn hundred_percent_only_in_about() {
        for s in [
            "<p>100% local and offline</p>",
            "<p>Runs 100% offline.</p>",
            "<span>100% Offline & Private</span>",
        ] {
            assert_eq!(count(s, Rule::Overclaim), 1, "{s}");
        }
        for file in ["lib/views/AboutView.svelte", "lib/components/About.svelte"] {
            let v = audit_source(file, "<p>100% local and offline</p>", &[]);
            assert!(v.iter().all(|v| v.rule != Rule::Overclaim), "{file}");
        }
        // zoom / CSS percentages are not claims
        assert_eq!(count("<button>100%</button>", Rule::Overclaim), 0);
    }

    // strict-mode globs
    #[test]
    fn glob_basics() {
        assert!(glob_match(
            "lib/views/*.svelte",
            "lib/views/EditView.svelte"
        ));
        assert!(!glob_match(
            "lib/views/*.svelte",
            "lib/views/studio/SpatialCanvas.svelte"
        ));
        assert!(glob_match(
            "lib/views/**",
            "lib/views/studio/SpatialCanvas.svelte"
        ));
        assert!(glob_match(
            "**/Sidebar.svelte",
            "lib/components/Sidebar.svelte"
        ));
        assert!(!glob_match(
            "lib/views/EditView.svelte",
            "lib/views/EditView2.svelte"
        ));
    }
    #[test]
    fn swept_matches_any_prefix_form() {
        let pats = vec!["lib/components/Sidebar.svelte".to_string()];
        assert!(is_swept(
            &pats,
            "app-tauri/src/lib/components/Sidebar.svelte"
        ));
        assert!(!is_swept(
            &pats,
            "app-tauri/src/lib/components/StatusBar.svelte"
        ));
        assert!(!is_swept(
            &[],
            "app-tauri/src/lib/components/Sidebar.svelte"
        ));
    }

    // inventory
    #[test]
    fn inventory_lists_files_and_top_issues() {
        let src = "<style>\n.a { color: #fff; background: #fff; padding: 12px; }\n</style>";
        let v = audit_source("app-tauri/src/lib/Y.svelte", src, &[]);
        let md = render_inventory(&v, &["app-tauri/src/lib/Y.svelte".to_string()]);
        assert!(md.contains("| File "));
        assert!(md.contains("app-tauri/src/lib/Y.svelte"));
        assert!(md.contains("#fff"));
    }

    // px allowlist (canvas / geometry exemptions)
    #[test]
    fn allowlist_rejects_entry_without_reason() {
        assert!(parse_px_allowlist("a/b.svelte:10-20\n").is_err());
        assert!(parse_px_allowlist("a/b.svelte:10-20 \u{2014} \n").is_err());
        assert!(parse_px_allowlist("a/b.svelte:abc \u{2014}   \n").is_err());
        let ok = parse_px_allowlist(
            "# c\n\na/b.svelte:10-20 \u{2014} svg label\nc.svelte \u{2014} whole file\n",
        )
        .unwrap();
        assert_eq!(ok.len(), 2);
        assert_eq!(ok[0].reason, "svg label");
    }
    #[test]
    fn allowlist_exempts_by_range_and_symbol() {
        let es = parse_px_allowlist(
            "x/R.svelte:2-3 \u{2014} geometry\nx/S.svelte:.wedge \u{2014} geometry\n",
        )
        .unwrap();
        let mk = |file: &str, line| Violation {
            file: file.into(),
            line,
            rule: Rule::RawPx,
            detail: String::new(),
            message: String::new(),
        };
        assert!(px_allowed(&es, &mk("x/R.svelte", 2), ""));
        assert!(!px_allowed(&es, &mk("x/R.svelte", 4), ""));
        assert!(px_allowed(&es, &mk("x/S.svelte", 9), ".wedge-label {"));
        assert!(!px_allowed(&es, &mk("x/S.svelte", 9), ".other {"));
        // only raw-px / local-key are exempted
        let mut v = mk("x/R.svelte", 2);
        v.rule = Rule::LocalKey;
        assert!(px_allowed(&es, &v, ""));
        v.rule = Rule::RawColour;
        assert!(!px_allowed(&es, &v, ""));
    }
}
