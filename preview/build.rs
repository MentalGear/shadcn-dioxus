use dioxus::prelude::*;

// The `#[derive(Props)]` scanner shared with the `scripts/check-props-docs.sh` gate (backlog row 165).
#[path = "props_scan.rs"]
mod props_scan;

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
    write_variant_notes_index(&out_dir);

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
            let mut markdown = process_markdown_to_html(&file.path());
            // A component's own `docs.md` (not a variant note) ends with its props reference.
            if file.file_name() == "docs.md"
                && dir.parent() == Some(std::path::Path::new("src/components"))
            {
                markdown.push_str(&render_props_section(&folder_name, &markdown));
            }
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

/// The text of the props disclosure's heading. The id is `docs-props-reference`, like every
/// `wrap_sections_in_disclosures` row, and a `docs.md` that declares the same `##` fails the build.
const PROPS_HEADING: &str = "Props reference";

/// Where a component's props structs live: its primitive (`primitives/src/<name>.rs` and/or the
/// `primitives/src/<name>/` module directory) and its styled wrapper (`src/components/<name>/component.rs`).
/// Each file is registered with `cargo:rerun-if-changed`, so editing a prop or its doc comment re-runs this
/// script (row 92's stale-`OUT_DIR` class); a brand-new primitive file is picked up by the next run, like a
/// new `.md`.
fn props_sources(name: &str) -> Vec<(&'static str, std::path::PathBuf)> {
    fn rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                rs_files(&path, out);
            } else if path.extension() == Some(std::ffi::OsStr::new("rs")) {
                out.push(path);
            }
        }
    }
    let mut sources = Vec::new();
    let wrapper = std::path::PathBuf::from(format!("src/components/{name}/component.rs"));
    if wrapper.is_file() {
        sources.push(("Styled component", wrapper));
    }
    let mut primitive = Vec::new();
    let file = std::path::PathBuf::from(format!("../primitives/src/{name}.rs"));
    if file.is_file() {
        primitive.push(file);
    }
    rs_files(
        &std::path::PathBuf::from(format!("../primitives/src/{name}")),
        &mut primitive,
    );
    primitive.sort();
    sources.extend(primitive.into_iter().map(|path| ("Primitive", path)));
    sources
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A doc comment as inline HTML: markdown, with rustdoc's intra-doc links (`[`Foo`]`) reduced to code.
fn doc_html(doc: &str) -> String {
    let doc = doc.replace("[`", "`").replace("`]", "`");
    let mut out = String::new();
    pulldown_cmark::html::push_html(&mut out, pulldown_cmark::Parser::new(&doc));
    out
}

/// The "Props reference" disclosure appended to a component's docs page: one table per
/// `#[derive(Props)]` struct of the component's primitive and styled wrapper (name, type, default,
/// description), built from the source by `props_scan`, so the page cannot disagree with the code.
/// `scripts/check-props-docs.sh` fails a prop with no doc comment, so no Description cell is empty.
///
/// Display rules: `ReadSignal<T>` and `Option<T>` show `T` (a plain `T` is accepted for a
/// `ReadSignal<T>`; an optional prop defaults to `None`); `#[props(default)]` shows the type's default;
/// a `#[props(extends = ..)]` spread is not a row but the sentence "plus the global attributes".
fn render_props_section(name: &str, docs_html: &str) -> String {
    let id = "docs-props-reference";
    assert!(
        !docs_html.contains(&format!("id=\"{id}\"")),
        "src/components/{name}/docs.md declares `## {PROPS_HEADING}`, which build.rs adds itself"
    );
    let mut body = String::new();
    for (label, path) in props_sources(name) {
        println!("cargo:rerun-if-changed={}", path.display());
        let source = std::fs::read_to_string(&path).unwrap();
        for item in props_scan::scan_props(&source) {
            let rows: Vec<&props_scan::PropField> =
                item.fields.iter().filter(|f| f.extends.is_none()).collect();
            let spreads: Vec<String> = item
                .fields
                .iter()
                .filter_map(|f| f.extends.as_deref())
                .map(|target| {
                    if target == "GlobalAttributes" {
                        "the global attributes".to_string()
                    } else {
                        format!(
                            "the <code>&lt;{}&gt;</code> element's attributes",
                            html_escape(target)
                        )
                    }
                })
                .collect();
            body.push_str(&format!(
                "<section class=\"dx-docs-props\" aria-label=\"{label} props: {n}\">\n<h3><code>{n}</code> <span class=\"dx-docs-props-kind\">{label}</span></h3>\n",
                n = html_escape(&item.name),
            ));
            if !item.doc.trim().is_empty() {
                body.push_str(&format!(
                    "<div class=\"dx-docs-props-intro\">{}</div>\n",
                    doc_html(&item.doc)
                ));
            }
            let plus = if spreads.is_empty() {
                String::new()
            } else {
                format!("Plus {}.", spreads.join(" and "))
            };
            if rows.is_empty() {
                body.push_str(&format!(
                    "<p class=\"dx-docs-muted\">No props of its own. {plus}</p>\n"
                ));
            } else {
                body.push_str(&format!(
                    "<div class=\"dx-docs-table-scroll\" role=\"region\" aria-label=\"{n} props\" tabindex=\"0\"><table class=\"dx-docs-props-table\"><thead><tr><th scope=\"col\">Prop</th><th scope=\"col\">Type</th><th scope=\"col\">Default</th><th scope=\"col\">Description</th></tr></thead><tbody>\n",
                    n = html_escape(&item.name)
                ));
                for field in rows {
                    let (ty, _) = props_scan::display_type(&field.ty);
                    let default = props_scan::display_default(field);
                    let default_cell = if default == "required" {
                        "<span class=\"dx-docs-props-required\">required</span>".to_string()
                    } else {
                        format!("<code>{}</code>", html_escape(&default))
                    };
                    body.push_str(&format!(
                        "<tr data-prop=\"{name}\"><td><code>{name}</code></td><td><code>{ty}</code></td><td>{default}</td><td>{doc}</td></tr>\n",
                        name = html_escape(&field.name),
                        ty = html_escape(&ty),
                        default = default_cell,
                        doc = if field.doc.trim().is_empty() { "&mdash;".to_string() } else { doc_html(&field.doc) },
                    ));
                }
                body.push_str("</tbody></table></div>\n");
                if !plus.is_empty() {
                    body.push_str(&format!("<p class=\"dx-docs-muted\">{plus}</p>\n"));
                }
            }
            body.push_str("</section>\n");
        }
    }
    if body.is_empty() {
        body.push_str(
            "<p class=\"dx-docs-muted\">This component declares no props struct of its own; see Usage notes.</p>\n",
        );
    }
    format!(
        "<details class=\"dx-docs-disclosure\"><summary><h2 id=\"{id}\">{PROPS_HEADING}</h2></summary><div class=\"dx-docs-disclosure-body\">\n{body}</div></details>\n"
    )
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
