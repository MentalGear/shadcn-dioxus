use dioxus::prelude::*;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);
    // Deliberately NOT `cargo:rerun-if-changed=src/components`. That
    // blanket, recursive directory watch made cargo rerun this script on
    // ANY change under any component folder -- including `component.rs`/
    // `style.css`/`variants/**`, none of which this script reads -- and
    // `walk_markdown_dir` below unconditionally rewrites every component's
    // `description.txt`/`docs.html` in `OUT_DIR` on every run regardless of
    // which file triggered it. Since `components/mod.rs` `include_str!`s
    // those two per component, their bumped mtimes made cargo treat
    // `preview` itself as dirty too, compounding the separate compile-time
    // CSS embed that `dev-docs/dev-loop.md`'s CSS section root-causes as
    // the primary reason a component `style.css` edit forced a full
    // rebuild instead of hot-reloading. This script only ever reads `*.md`
    // and `component.json` files; each one already registers its own
    // precise `cargo:rerun-if-changed` below (`process_markdown_to_html`/
    // `read_component_description`), which is the complete, correct watch
    // list for what this script actually does -- narrowing to just that
    // removes the compounding, on top of the primary fix in `main.rs`/
    // `components/mod.rs`.
    //
    // Known, accepted trade-off: a brand new `.md`/`component.json` file
    // (one this script has never seen) isn't watched until something else
    // causes it to rerun -- cargo has no "watch this directory for new
    // entries only" primitive, only the recursive watch just removed.
    // NOT self-healing, despite what this note used to claim: editing
    // `components/mod.rs` makes cargo rebuild `preview`, but it does NOT
    // rerun this build script (cargo tracks `mod.rs` as a source file of
    // the crate, not as an input of this script), so adding a component
    // leaves `OUT_DIR` without that component's `description.txt`/
    // `docs.html`. What the original note got right is that the failure is
    // LOUD rather than silent: `cargo clippy`/`test`/`doc` all fail with
    // `couldn't read .../out/<name>/description.txt`, and `dx serve`'s own
    // wasm build fails the same way separately, since it has its own
    // target dir. Fix: `touch preview/build.rs` (or `cargo clean -p
    // preview`), once per target dir. Observed for real on 2026-09-20 when
    // Carousel landed -- see `dev-docs/backlog.md` row 92. A lane building
    // in a fresh target dir never sees it, because there this script runs
    // with the new component already present.
    // Process all markdown files in each component folder.
    for folder in std::fs::read_dir("src/components").unwrap().flatten() {
        if !folder.file_type().unwrap().is_dir() {
            continue;
        }
        let folder_path = folder.path();
        walk_markdown_dir(&folder_path, &out_dir).unwrap();
    }
    write_variant_notes_index(&out_dir);
}

fn walk_markdown_dir(dir: &std::path::Path, out_dir: &std::path::Path) -> std::io::Result<()> {
    let folder_name = dir.file_name().unwrap();
    let folder_name = folder_name.to_string_lossy();
    let out_folder = out_dir.join(&*folder_name);
    std::fs::create_dir_all(&out_folder).unwrap();
    for file in std::fs::read_dir(dir).unwrap().flatten() {
        if file.file_type().unwrap().is_dir() {
            walk_markdown_dir(&file.path(), &out_folder)?;
            continue;
        }
        if file.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if file.path().extension() == Some(std::ffi::OsStr::new("md")) {
            let markdown = process_markdown_to_html(&file.path());
            let out_file_path = out_folder.join(file.file_name()).with_extension("html");
            std::fs::write(out_file_path, markdown).unwrap();
            continue;
        }
        if file.file_name() == "component.json" {
            let description = read_component_description(&file.path());
            let out_file_path = out_folder.join("description.txt");
            std::fs::write(out_file_path, description).unwrap();
        }
    }
    Ok(())
}

fn process_markdown_to_html(markdown_path: &std::path::Path) -> String {
    println!("cargo:rerun-if-changed={}", markdown_path.display());
    use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
    let markdown_input =
        std::fs::read_to_string(markdown_path).expect("Failed to read markdown file");
    let mut options = Options::empty();
    options.insert(Options::ENABLE_GFM);
    // `ENABLE_GFM` alone does not turn on pipe tables (nor the other GFM
    // extensions), so they shipped as raw `|` text.
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&markdown_input, options);
    let mut events = Vec::new();
    let mut code_block: Option<(CodeBlockKind<'_>, String)> = None;

    for event in parser {
        match (&mut code_block, event) {
            (None, Event::Start(Tag::CodeBlock(kind))) => {
                code_block = Some((kind, String::new()));
            }
            (Some((_, source)), Event::Text(text)) => {
                source.push_str(&text);
            }
            (Some((kind, source)), Event::End(TagEnd::CodeBlock)) => {
                events.push(Event::Html(CowStr::Boxed(
                    render_code_block_html(kind, source).into_boxed_str(),
                )));
                code_block = None;
            }
            // Wrap every table in a scroll container so a wide table scrolls
            // inside its own box instead of widening the page.
            (None, Event::Start(tag @ Tag::Table(_))) => {
                events.push(Event::Html(CowStr::Borrowed(
                    "<div class=\"dx-docs-table-scroll\">",
                )));
                events.push(Event::Start(tag));
            }
            (None, Event::End(TagEnd::Table)) => {
                events.push(Event::End(TagEnd::Table));
                events.push(Event::Html(CowStr::Borrowed("</div>\n")));
            }
            (None, event) => events.push(event),
            (Some((_, source)), Event::Code(text)) => {
                source.push_str(&text);
            }
            (Some((_, source)), Event::Html(html) | Event::InlineHtml(html)) => {
                source.push_str(&html);
            }
            (Some(_), _) => {}
        }
    }

    // Only a component's `docs.md` is a page of sections; a variant's `notes.md` is one short note.
    let events = if markdown_path.file_name() == Some(std::ffi::OsStr::new("docs.md")) {
        wrap_sections_in_disclosures(events)
    } else {
        events
    };

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
    html_output
}

/// The docs.md -> HTML "disclosure pass" (backlog row 233).
///
/// Every `##` section of a `docs.md` becomes a closed native `<details class="dx-docs-disclosure">`
/// whose `<summary>` holds the `h2` (so the heading outline and its `id` anchor stay intact), and the
/// text before the first `##` becomes a closed "Usage notes" disclosure. A page is then a short list of
/// one-line rows instead of a wall of prose, and no `docs.md` has to be edited: a new section is
/// collapsed by construction. `<details>` needs no hydration, so the closed state is the server's
/// markup and nothing flashes. The site opens the disclosure that holds the URL fragment (`main.rs`,
/// `use_open_disclosure_for_hash`).
///
/// The heading text equals `PAGE_HEADING_USAGE` in `main.rs`, which `docs_md_has_no_template_h2`
/// reserves so a `docs.md` cannot declare the same heading.
const USAGE_NOTES_HEADING: &str = "Usage notes";

fn wrap_sections_in_disclosures(
    events: Vec<pulldown_cmark::Event<'_>>,
) -> Vec<pulldown_cmark::Event<'_>> {
    use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag, TagEnd};

    fn is_h2_start(event: &Event<'_>) -> bool {
        matches!(
            event,
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            })
        )
    }
    fn html(text: String) -> Event<'static> {
        Event::Html(CowStr::Boxed(text.into_boxed_str()))
    }
    fn open(id: &str) -> Event<'static> {
        html(format!(
            "<details class=\"dx-docs-disclosure\"><summary><h2 id=\"docs-{id}\">"
        ))
    }
    const BODY: &str = "</h2></summary><div class=\"dx-docs-disclosure-body\">\n";
    const CLOSE: &str = "</div></details>\n";

    let mut slugs: Vec<String> = Vec::new();
    let mut unique_slug = |title: &str| {
        let base = slugify(title);
        let mut slug = base.clone();
        let mut n = 2;
        while slugs.contains(&slug) {
            slug = format!("{base}-{n}");
            n += 1;
        }
        slugs.push(slug.clone());
        slug
    };

    let mut out = Vec::with_capacity(events.len() + 8);
    let mut iter = events.into_iter().peekable();

    // The text before the first `##`.
    let mut intro = Vec::new();
    while let Some(event) = iter.next_if(|event| !is_h2_start(event)) {
        intro.push(event);
    }
    if !intro.is_empty() {
        out.push(open(&unique_slug(USAGE_NOTES_HEADING)));
        out.push(Event::Text(CowStr::Borrowed(USAGE_NOTES_HEADING)));
        out.push(html(BODY.to_string()));
        out.extend(intro);
        out.push(html(CLOSE.to_string()));
    }

    while iter.peek().is_some() {
        // `iter` is at a `##` start: take its inline events up to the matching end.
        iter.next();
        let mut title_events = Vec::new();
        let mut title = String::new();
        for event in iter.by_ref() {
            match &event {
                Event::End(TagEnd::Heading(_)) => break,
                Event::Text(text) | Event::Code(text) => title.push_str(text),
                _ => {}
            }
            title_events.push(event);
        }
        out.push(open(&unique_slug(&title)));
        out.extend(title_events);
        out.push(html(BODY.to_string()));
        while let Some(event) = iter.next_if(|event| !is_h2_start(event)) {
            out.push(event);
        }
        out.push(html(CLOSE.to_string()));
    }
    out
}

/// `Component Structure` -> `component-structure`; every run of non-alphanumerics is one dash.
fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

/// Per-variant notes (backlog row 233): `variants/<variant>/notes.md` is the prose about one demo. It is
/// rendered to `OUT_DIR/<component>/variants/<variant>/notes.html` by `walk_markdown_dir` like every
/// other `.md`; this writes `OUT_DIR/variant_notes.rs`, a `variant_notes(component, variant)` lookup
/// that `main.rs` `include!`s, so the page shows the note in a `<details>` under that demo. It is a
/// generated lookup rather than a field of the `examples!` macro because the macro cannot test whether
/// a file exists. `variants/` is excluded from the installed component (`component.json`), so a note
/// never reaches a user's app.
fn write_variant_notes_index(out_dir: &std::path::Path) {
    let mut arms = Vec::new();
    for component in std::fs::read_dir("src/components").unwrap().flatten() {
        let variants_dir = component.path().join("variants");
        let Ok(variants) = std::fs::read_dir(&variants_dir) else {
            continue;
        };
        for variant in variants.flatten() {
            let notes = variant.path().join("notes.md");
            if !notes.is_file() {
                continue;
            }
            let component = component.file_name().to_string_lossy().into_owned();
            let variant = variant.file_name().to_string_lossy().into_owned();
            arms.push(format!(
                "        ({component:?}, {variant:?}) => Some(include_str!(concat!(env!(\"OUT_DIR\"), \"/{component}/variants/{variant}/notes.html\"))),\n"
            ));
        }
    }
    arms.sort();
    let source = format!(
        "/// Generated by `build.rs` (`write_variant_notes_index`): the rendered `variants/<variant>/notes.md`, if any.\nfn variant_notes(component: &str, variant: &str) -> Option<&'static str> {{\n    match (component, variant) {{\n{}        _ => None,\n    }}\n}}\n",
        arms.concat()
    );
    std::fs::write(out_dir.join("variant_notes.rs"), source).unwrap();
}

fn read_component_description(component_json_path: &std::path::Path) -> String {
    println!("cargo:rerun-if-changed={}", component_json_path.display());
    let input =
        std::fs::read_to_string(component_json_path).expect("Failed to read component metadata");
    let json: serde_json::Value =
        serde_json::from_str(&input).expect("Failed to parse component metadata");
    json.get("description")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("A Dioxus component.")
        .trim()
        .to_string()
}

fn render_code_block_html(kind: &pulldown_cmark::CodeBlockKind<'_>, source: &str) -> String {
    let language = match kind {
        pulldown_cmark::CodeBlockKind::Fenced(info) => {
            let slug = info.split_whitespace().next().unwrap_or_default();
            language_from_slug(slug)
        }
        pulldown_cmark::CodeBlockKind::Indented => None,
    };

    let Some(language) = language else {
        return render_plain_code_block(source);
    };

    let source = source.trim_end_matches('\n');
    let highlighted: dioxus_code::advanced::HighlightedSource =
        dioxus_code::SourceCode::new(language, source).into();

    dioxus_ssr::render_element(rsx! {
        div {
            class: "dx-preview-code-theme",
            tabindex: "0",
            dioxus_code::Code {
                src: highlighted,
                theme: dioxus_code::CodeTheme::system(
                    dioxus_code::Theme::GITHUB_LIGHT,
                    dioxus_code::Theme::GITHUB_DARK,
                ),
            }
        }
    })
}

/// A fence with no highlighter (```html, ```text, an indented block). `.dx-docs-prose pre` scrolls
/// sideways, so it is a scrollable region and must be keyboard-reachable (axe
/// `scrollable-region-focusable`), exactly like the highlighted `div` above and `DocsPre` in main.rs.
fn render_plain_code_block(source: &str) -> String {
    let source = source.trim_end_matches('\n');

    dioxus_ssr::render_element(rsx! {
        pre { tabindex: "0",
            code { "{source}" }
        }
    })
}

fn language_from_slug(slug: &str) -> Option<dioxus_code::Language> {
    match slug {
        "" => Some(dioxus_code::Language::Rust),
        "rs" => Some(dioxus_code::Language::Rust),
        "rust" => Some(dioxus_code::Language::Rust),
        "css" => Some(dioxus_code::Language::Css),
        slug => dioxus_code::Language::from_slug(slug),
    }
}
