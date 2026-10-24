// chatkit lane: new components code_block, trace, checkpoint (retrigger)
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
    // `description.txt`/`docs_sections.rs` in `OUT_DIR` on every run regardless of
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
    // (Row 219: this script also registers `src/components/mod.rs` and every
    // component's `variants/` directory, so adding a component -- which edits `CATALOG` in
    // `mod.rs` -- or a variant re-runs it. The trade-off below still holds for a new `.md` file
    // alone.)
    //
    // Known, accepted trade-off: a brand new `.md` file inside an EXISTING component (one this
    // script has never seen) isn't watched until something else causes it to rerun -- cargo has no
    // "watch this directory for new entries only" primitive, only the recursive watch just removed.
    // A new COMPONENT or variant needs no manual `touch` (backlog rows 92 and 219): adding a
    // component edits `CATALOG` in `components/mod.rs`, and the
    // `rerun-if-changed` on that file and on every `variants/` directory below re-runs this script,
    // so `OUT_DIR` gets the component's `description.txt`/`docs.html` without help. (Without
    // that, `cargo clippy`/`test`/`doc` fail with `couldn't read
    // .../out/<name>/description.txt` until `touch preview/build.rs`.)
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
        // RTL demos always come last, whatever `order.txt` says (row 351): a stable partition, so
        // the listed order is kept inside each group. A curator cannot put one in the middle.
        variants.sort_by_key(|v| is_rtl_variant(v));

        components.push(Component {
            name,
            block: variants_dir.join("demo.css").is_file(),
            variants,
        });
    }
    components.sort_by(|a, b| a.name.cmp(&b.name));
    components
}

/// A variant named `rtl`, or ending in `_rtl` (`drag_free_rtl`): the right-to-left twin of a demo.
fn is_rtl_variant(name: &str) -> bool {
    name == "rtl" || name.ends_with("_rtl")
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
            // A component's own `docs.md` (not a variant note) is a page of sections: structured
            // data the page renders as its own `Accordion` (backlog row 233).
            if file.file_name() == "docs.md"
                && dir.parent() == Some(std::path::Path::new("src/components"))
            {
                let mut sections = process_docs_md_to_sections(&file.path());
                sections.push(render_props_section(&folder_name, &sections));
                write_if_changed(
                    &out_folder.join("docs_sections.rs"),
                    &render_docs_sections(&sections),
                );
                continue;
            }
            let html = process_markdown_to_html(&file.path());
            let out_file_path = out_folder.join(file.file_name()).with_extension("html");
            write_if_changed(&out_file_path, &html);
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

/// The text of the props section's heading. The id is `docs-props-reference`, like every
/// `split_docs_sections` row, and a `docs.md` that declares the same `##` fails the build.
const PROPS_HEADING: &str = "Props reference";

/// One section of a component page's docs: a docs.md `##` section, the "Usage notes" intro or the
/// generated "Props reference". `main.rs` renders each as one `AccordionItem` (its `DocsSection`
/// mirrors this), so the page is built from the library's own Accordion, not a native disclosure.
struct DocsSection {
    /// The `h2` id, the anchor target (`docs-<slug>`).
    id: String,
    /// The heading's inline HTML (a `##` may hold `code`).
    title_html: String,
    /// The section body.
    html: String,
}

/// The generated `docs_sections.rs` of one component: a `&[DocsSection]` expression that
/// `components/mod.rs` `include!`s as `ComponentDemoData::docs`.
fn render_docs_sections(sections: &[DocsSection]) -> String {
    let mut out =
        String::from("// Generated by preview/build.rs (render_docs_sections). Do not edit.\n&[\n");
    for section in sections {
        out.push_str(&format!(
            "    crate::DocsSection {{ id: {:?}, title_html: {:?}, html: {:?} }},\n",
            section.id, section.title_html, section.html
        ));
    }
    out.push_str("]\n");
    out
}

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
    // A doc comment is content under the page's own `<h3>` (struct) or inside a table cell: its `# Example` /
    // `## Styling` headings must never join the page outline (they rendered an `<h1>` and `<h2>`s in the docs
    // prose), so every heading level is demoted to `<h4>`.
    use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
    let events = pulldown_cmark::Parser::new(&doc).map(|event| match event {
        Event::Start(Tag::Heading {
            id, classes, attrs, ..
        }) => Event::Start(Tag::Heading {
            level: HeadingLevel::H4,
            id,
            classes,
            attrs,
        }),
        Event::End(TagEnd::Heading(_)) => Event::End(TagEnd::Heading(HeadingLevel::H4)),
        other => other,
    });
    pulldown_cmark::html::push_html(&mut out, events);
    out
}

/// The "Props reference" section appended to a component's docs page: one table per
/// `#[derive(Props)]` struct of the component's primitive and styled wrapper (name, type, default,
/// description), built from the source by `props_scan`, so the page cannot disagree with the code.
/// `scripts/check-props-docs.sh` fails a prop with no doc comment, so no Description cell is empty.
///
/// Display rules: `ReadSignal<T>` and `Option<T>` show `T` (a plain `T` is accepted for a
/// `ReadSignal<T>`; an optional prop defaults to `None`); `#[props(default)]` shows the type's default;
/// a `#[props(extends = ..)]` spread is not a row but the sentence "plus the global attributes".
fn render_props_section(name: &str, sections: &[DocsSection]) -> DocsSection {
    let id = "docs-props-reference";
    assert!(
        !sections.iter().any(|s| s.id == id),
        "src/components/{name}/docs.md declares `## {PROPS_HEADING}`, which build.rs adds itself"
    );
    let mut body = String::new();
    // A `#[cfg(web)]`/`#[cfg(not(web))]` pair declares the same component twice: the first one is the page's.
    let mut seen = std::collections::HashSet::new();
    for (label, path) in props_sources(name) {
        println!("cargo:rerun-if-changed={}", path.display());
        let source = std::fs::read_to_string(&path).unwrap();
        for item in props_scan::scan_props(&source) {
            if !seen.insert((label, item.name.clone())) {
                continue;
            }
            let rows: Vec<&props_scan::PropField> =
                item.fields.iter().filter(|f| f.extends.is_none()).collect();
            let spreads: Vec<String> = item
                .fields
                .iter()
                .filter_map(|f| f.extends.as_deref())
                .map(|target| {
                    if target == "GlobalAttributes" {
                        format!(
                            "<a href=\"{MDN_GLOBAL_ATTRIBUTES}\" target=\"_blank\" rel=\"noopener noreferrer\">the global attributes</a>"
                        )
                    } else {
                        format!(
                            "the <code>&lt;{}&gt;</code> element's attributes",
                            html_escape(target)
                        )
                    }
                })
                .collect();
            body.push_str(&format!(
                "<section class=\"dx-docs-props\">\n<h3 id=\"{id}-{kind}-{slug}\"><code>{n}</code> <span class=\"dx-docs-props-kind\">{label}</span></h3>\n",
                n = html_escape(&item.name),
                kind = slugify(label),
                slug = slugify(&item.name),
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
                    "<div class=\"dx-docs-table-scroll\" role=\"region\" aria-label=\"{label}: {n} props\" tabindex=\"0\"><table class=\"{TABLE} dx-docs-props-table\"><thead class=\"{THEAD}\"><tr class=\"{TR}\"><th class=\"{TH}\" scope=\"col\">Prop</th><th class=\"{TH}\" scope=\"col\">Type</th><th class=\"{TH}\" scope=\"col\">Default</th><th class=\"{TH}\" scope=\"col\">Description</th></tr></thead><tbody class=\"{TBODY}\">\n",
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
                        "<tr class=\"{TR}\" data-prop=\"{name}\"><td class=\"{TD}\"><code>{name}</code></td><td class=\"{TD}\"><code>{ty}</code></td><td class=\"{TD}\">{default}</td><td class=\"{TD}\">{doc}</td></tr>\n",
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
    DocsSection {
        id: id.to_string(),
        title_html: PROPS_HEADING.to_string(),
        html: body,
    }
}

/// The Table component's part classes (`src/components/table/component.rs`, `style.css`): every table
/// this script emits (a docs.md pipe table, a props reference) wears them, so docs tables look like the
/// library's Table instead of a second hand-written table style. `ComponentHighlight` loads the sheet.
/// Where the "Plus the global attributes." sentence of a props table points (row 352).
const MDN_GLOBAL_ATTRIBUTES: &str =
    "https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes";

const TABLE: &str = "dx-table";
const THEAD: &str = "dx-table-header";
const TBODY: &str = "dx-table-body";
const TR: &str = "dx-table-row";
const TH: &str = "dx-table-head";
const TD: &str = "dx-table-cell";

/// A `.md` file as pulldown-cmark events, with code fences highlighted and every table written as the
/// Table component's markup inside a horizontal scroll container.
fn markdown_events(markdown_input: &str) -> Vec<pulldown_cmark::Event<'_>> {
    use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_GFM);
    // `ENABLE_GFM` alone does not turn on pipe tables (nor the other GFM
    // extensions), so they shipped as raw `|` text.
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(markdown_input, options);
    let mut events = Vec::new();
    let mut code_block: Option<(CodeBlockKind<'_>, String)> = None;
    fn html(text: String) -> Event<'static> {
        Event::Html(CowStr::Boxed(text.into_boxed_str()))
    }
    // Table state: the column alignments, whether the cells are head cells, the cell index.
    let mut aligns: Vec<Alignment> = Vec::new();
    let mut in_head = false;
    let mut cell = 0;

    for event in parser {
        match (&mut code_block, event) {
            (None, Event::Start(Tag::CodeBlock(kind))) => {
                code_block = Some((kind, String::new()));
            }
            (Some((_, source)), Event::Text(text)) => {
                source.push_str(&text);
            }
            (Some((kind, source)), Event::End(TagEnd::CodeBlock)) => {
                events.push(html(render_code_block_html(kind, source)));
                code_block = None;
            }
            // Wrap every table in a scroll container so a wide table scrolls
            // inside its own box instead of widening the page.
            (None, Event::Start(Tag::Table(a))) => {
                aligns = a;
                events.push(html(format!(
                    "<div class=\"dx-docs-table-scroll\"><table class=\"{TABLE}\">"
                )));
            }
            (None, Event::Start(Tag::TableHead)) => {
                in_head = true;
                cell = 0;
                events.push(html(format!(
                    "<thead class=\"{THEAD}\"><tr class=\"{TR}\">"
                )));
            }
            (None, Event::End(TagEnd::TableHead)) => {
                in_head = false;
                events.push(html(format!("</tr></thead><tbody class=\"{TBODY}\">\n")));
            }
            (None, Event::Start(Tag::TableRow)) => {
                cell = 0;
                events.push(html(format!("<tr class=\"{TR}\">")));
            }
            (None, Event::End(TagEnd::TableRow)) => {
                events.push(html("</tr>\n".to_string()));
            }
            (None, Event::Start(Tag::TableCell)) => {
                let (tag, class) = if in_head {
                    ("th scope=\"col\"", TH)
                } else {
                    ("td", TD)
                };
                let align = match aligns.get(cell) {
                    Some(Alignment::Left) => " style=\"text-align: left\"",
                    Some(Alignment::Center) => " style=\"text-align: center\"",
                    Some(Alignment::Right) => " style=\"text-align: right\"",
                    _ => "",
                };
                events.push(html(format!("<{tag} class=\"{class}\"{align}>")));
            }
            (None, Event::End(TagEnd::TableCell)) => {
                events.push(html(if in_head { "</th>" } else { "</td>" }.to_string()));
                cell += 1;
            }
            (None, Event::End(TagEnd::Table)) => {
                events.push(html("</tbody></table></div>\n".to_string()));
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
    events
}

/// A variant's `notes.md` (one short note) as HTML.
fn process_markdown_to_html(markdown_path: &std::path::Path) -> String {
    println!("cargo:rerun-if-changed={}", markdown_path.display());
    let markdown_input =
        std::fs::read_to_string(markdown_path).expect("Failed to read markdown file");
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(
        &mut html_output,
        markdown_events(&markdown_input).into_iter(),
    );
    html_output
}

/// A component's `docs.md` as its page sections (`split_docs_sections`).
fn process_docs_md_to_sections(markdown_path: &std::path::Path) -> Vec<DocsSection> {
    println!("cargo:rerun-if-changed={}", markdown_path.display());
    let markdown_input =
        std::fs::read_to_string(markdown_path).expect("Failed to read markdown file");
    split_docs_sections(markdown_events(&markdown_input))
}

/// The docs.md -> sections pass (backlog row 233).
///
/// Every `##` section of a `docs.md` becomes one [`DocsSection`] (its `h2` id `docs-<slug>`, its heading
/// HTML and its body), and the text before the first `##` becomes a "Usage notes" section. `main.rs`
/// renders them as closed items of the library's own `Accordion` (multi-open, `hidden_until_found`, the
/// `h2` wrapping the trigger so the outline and the anchors stay), so no `docs.md` has to be edited: a new
/// section is collapsed by construction. The site opens the item that holds the URL fragment (`main.rs`,
/// `use_open_section_for_hash`).
///
/// The heading text equals `PAGE_HEADING_USAGE` in `main.rs`, which `docs_md_has_no_template_h2`
/// reserves so a `docs.md` cannot declare the same heading.
const USAGE_NOTES_HEADING: &str = "Usage notes";

fn split_docs_sections(events: Vec<pulldown_cmark::Event<'_>>) -> Vec<DocsSection> {
    use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

    fn is_h2_start(event: &Event<'_>) -> bool {
        matches!(
            event,
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            })
        )
    }
    fn render(events: Vec<Event<'_>>) -> String {
        let mut out = String::new();
        pulldown_cmark::html::push_html(&mut out, events.into_iter());
        out
    }
    /// Every `###` inside a section gets the id `<section id>-<slug>`, so a link can point at a
    /// subsection; the page opens the closed item around it (`use_open_section_for_hash`).
    fn anchor_subheadings<'a>(section_id: &str, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
        let mut used: Vec<String> = Vec::new();
        let mut out = Vec::with_capacity(events.len());
        let mut iter = events.into_iter();
        while let Some(event) = iter.next() {
            let Event::Start(Tag::Heading {
                level: HeadingLevel::H3,
                id: None,
                classes,
                attrs,
            }) = event
            else {
                out.push(event);
                continue;
            };
            let mut inner = Vec::new();
            let mut text = String::new();
            for e in iter.by_ref() {
                let end = matches!(e, Event::End(TagEnd::Heading(_)));
                if let Event::Text(t) | Event::Code(t) = &e {
                    text.push_str(t);
                }
                inner.push(e);
                if end {
                    break;
                }
            }
            let base = format!("{section_id}-{}", slugify(&text));
            let mut id = base.clone();
            let mut n = 2;
            while used.contains(&id) {
                id = format!("{base}-{n}");
                n += 1;
            }
            used.push(id.clone());
            out.push(Event::Start(Tag::Heading {
                level: HeadingLevel::H3,
                id: Some(pulldown_cmark::CowStr::Boxed(id.into_boxed_str())),
                classes,
                attrs,
            }));
            out.extend(inner);
        }
        out
    }

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

    let mut out = Vec::new();
    let mut iter = events.into_iter().peekable();

    // The text before the first `##`.
    let mut intro = Vec::new();
    while let Some(event) = iter.next_if(|event| !is_h2_start(event)) {
        intro.push(event);
    }
    if !intro.is_empty() {
        let id = format!("docs-{}", unique_slug(USAGE_NOTES_HEADING));
        out.push(DocsSection {
            html: render(anchor_subheadings(&id, intro)),
            id,
            title_html: USAGE_NOTES_HEADING.to_string(),
        });
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
        let mut body = Vec::new();
        while let Some(event) = iter.next_if(|event| !is_h2_start(event)) {
            body.push(event);
        }
        let id = format!("docs-{}", unique_slug(&title));
        out.push(DocsSection {
            html: render(anchor_subheadings(&id, body)),
            id,
            title_html: render(title_events),
        });
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
/// that `main.rs` `include!`s, so the page shows the note in a `Collapsible` under that demo. It is a
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
