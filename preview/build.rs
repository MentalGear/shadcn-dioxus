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
    // (Row 219 update: this script now also registers `src/components/mod.rs` and every
    // component's `variants/` directory, so adding a component -- which edits `CATALOG` in
    // `mod.rs` -- or a variant re-runs it. The trade-off below still holds for a new `.md` file
    // alone.)
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

    // The component/variant registry (backlog row 219): `examples.rs` is the one `examples!(..)`
    // invocation `components/mod.rs` includes, derived from the directories instead of a list that
    // every demo-adding change edited. Editing `mod.rs` (a CATALOG row is how a new component starts)
    // re-runs this script, so a new component directory is picked up without a manual `touch`.
    println!("cargo:rerun-if-changed=src/components/mod.rs");
    let components = discover_components(std::path::Path::new("src/components"));
    check_block_mirror(&components);
    write_if_changed(&out_dir.join("examples.rs"), &render_examples(&components));
}

/// One component directory as the registry sees it.
struct Component {
    name: String,
    /// `variants/demo.css` exists: a Block component, rendered in an iframe page of its own.
    block: bool,
    /// The variants after the implicit `main`, in display order.
    variants: Vec<String>,
}

/// Every `src/components/<dir>` that holds a `component.json`, sorted by name, with its variants.
///
/// A variant is a `variants/<name>/` directory (`main` is always first and implicit). Display order
/// is `variants/order.txt` (one name per line, `#` comments) for the names it lists; a variant it
/// does not list follows, sorted by name. So a new demo is one new directory and no shared line,
/// and a curated order is a file inside the component's own directory. A name in `order.txt`
/// with no directory, or a directory with no `mod.rs`, fails the build instead of dropping a demo.
fn discover_components(root: &std::path::Path) -> Vec<Component> {
    let mut components = Vec::new();
    for entry in std::fs::read_dir(root).unwrap().flatten() {
        let dir = entry.path();
        if !dir.join("component.json").is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let variants_dir = dir.join("variants");
        // Cargo rescans this directory recursively: a new `variants/<x>/` or an `order.txt`
        // edit re-runs the script. (`write_if_changed` keeps an unrelated edit under it, e.g. a
        // block's `demo.css`, from bumping the mtime of the generated files.)
        println!("cargo:rerun-if-changed={}", variants_dir.display());
        let mut found = Vec::new();
        for variant in std::fs::read_dir(&variants_dir)
            .unwrap_or_else(|e| panic!("{} has no variants/ directory: {e}", dir.display()))
            .flatten()
        {
            if !variant.file_type().unwrap().is_dir() {
                continue;
            }
            let variant_name = variant.file_name().to_string_lossy().into_owned();
            assert!(
                variant.path().join("mod.rs").is_file(),
                "{}/variants/{variant_name} has no mod.rs (a variant is a directory with a `Demo` component in mod.rs)",
                dir.display()
            );
            assert!(
                variant_name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                    && !variant_name.starts_with(|c: char| c.is_ascii_digit()),
                "{}/variants/{variant_name}: a variant directory name must be a snake_case Rust module name",
                dir.display()
            );
            found.push(variant_name);
        }
        assert!(
            found.iter().any(|v| v == "main"),
            "{}/variants/main is missing: every component's first demo is `main`",
            dir.display()
        );
        found.retain(|v| v != "main");
        found.sort();

        let mut variants = Vec::new();
        if let Ok(order) = std::fs::read_to_string(variants_dir.join("order.txt")) {
            for line in order.lines() {
                let line = line.split('#').next().unwrap().trim();
                if line.is_empty() {
                    continue;
                }
                let at = found.iter().position(|v| v == line).unwrap_or_else(|| {
                    panic!(
                        "{}/variants/order.txt lists `{line}`, which is `main`, listed twice or has no variants/{line}/ directory",
                        dir.display()
                    )
                });
                variants.push(found.remove(at));
            }
        }
        variants.extend(found);

        components.push(Component {
            name,
            block: variants_dir.join("demo.css").is_file(),
            variants,
        });
    }
    components.sort_by(|a, b| a.name.cmp(&b.name));
    components
}

/// The generated file: one `with_registry!` macro that hands the whole component list to the macro
/// named by its caller (`examples!` in `components/mod.rs`), one `name(block)[a, b],` row per
/// component (`name[a, b],` or a bare `name,` when it has no variant beyond `main`).
///
/// A macro, not the invocation itself, because the `mod component;` declarations `examples!`
/// expands to are resolved relative to the file the expansion is rooted in: written out in an
/// `include!`d `OUT_DIR` file they would look for `OUT_DIR/accordion/component.rs`, while expanded
/// from a call in `mod.rs` they resolve against `src/components/`.
fn render_examples(components: &[Component]) -> String {
    let mut out = String::from(
        "// Generated by preview/build.rs (discover_components). Do not edit.\n\
         macro_rules! with_registry {\n    ($callback:ident) => {\n        $callback!(\n",
    );
    for component in components {
        out.push_str("            ");
        out.push_str(&block_row(component));
        out.push('\n');
    }
    out.push_str("        );\n    };\n}\n");
    out
}

fn block_row(component: &Component) -> String {
    let kind = if component.block { "(block)" } else { "" };
    let variants = if component.variants.is_empty() {
        String::new()
    } else {
        format!("[{}]", component.variants.join(", "))
    };
    format!("{}{kind}{variants},", component.name)
}

/// The Playwright specs (`all-routes`, `demo-forms-no-navigation`, `popup-style-isolation`) still
/// find the block demo pages by reading `name(block)[variant, ..]` rows after the text
/// `examples` + `!(` + newline in `components/mod.rs`. Those rows are kept there as a cfg'd-out mirror
/// of the derived block components, and this check fails the build when they drift, so the specs'
/// route list cannot go quietly stale. Delete the mirror together with the specs' text-scrape (a spec
/// that reads `variants/` would not need it).
fn check_block_mirror(components: &[Component]) {
    let source = std::fs::read_to_string("src/components/mod.rs").unwrap();
    let mirror = source
        .split("examples!(\n")
        .nth(1)
        .and_then(|rest| rest.split("\n);").next())
        .unwrap_or_default();
    let mirrored: Vec<&str> = mirror
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let derived: Vec<String> = components
        .iter()
        .filter(|c| c.block)
        .map(block_row)
        .collect();
    assert!(
        mirrored == derived.iter().map(String::as_str).collect::<Vec<_>>(),
        "the block-demo mirror in src/components/mod.rs is out of date.\n  mirror:  {mirrored:?}\n  derived: {derived:?}\n  \
         Edit the rows inside the cfg'd-out invocation near the end of mod.rs to match."
    );
}

/// Write only when the bytes differ: a re-run that changed nothing must not bump the mtime of a
/// file the crate `include_str!`s (that alone makes cargo rebuild `preview`).
fn write_if_changed(path: &std::path::Path, contents: &str) {
    if std::fs::read_to_string(path).is_ok_and(|existing| existing == contents) {
        return;
    }
    std::fs::write(path, contents).unwrap();
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
            write_if_changed(&out_file_path, &markdown);
            continue;
        }
        if file.file_name() == "component.json" {
            let description = read_component_description(&file.path());
            let out_file_path = out_folder.join("description.txt");
            write_if_changed(&out_file_path, &description);
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

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
    html_output
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
