use super::super::component::*;
use dioxus::prelude::*;

// Every line is a classic injection attempt. They render as literal text: raw HTML is a text node,
// and the link policy refuses the script URLs.
const SOURCE: &str = "\
<script>window.__pwned = true</script>\n\n\
<img src=x onerror=\"window.__pwned = true\">\n\n\
Inline <b onclick=\"window.__pwned = true\">markup</b> and <iframe src=\"javascript:alert(1)\"></iframe>\n\n\
[click me](javascript:window.__pwned=true) and [tab](java\tscript:alert(1)) and [entity](javascript&colon;alert(1))\n\n\
<a href=\"javascript:alert(1)\">raw anchor</a>";

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem;", "data-testid": "xss-demo",
            Markdown { source: SOURCE, allow_images: true }
        }
    }
}
