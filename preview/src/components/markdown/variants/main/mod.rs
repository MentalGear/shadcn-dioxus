use super::super::component::*;
use dioxus::prelude::*;

const SOURCE: &str = r#"## Release notes

Markdown from a model, rendered **safely**: *emphasis*, `inline code`, ~~strikethrough~~ and a [link](https://example.com).

1. First step
2. Second step
   - a nested item
   - another one

- [x] Task lists render as disabled checkboxes
- [ ] and stay out of the tab order

> A quote reads quieter than the text around it.

| Component | Role |
|:----------|-----:|
| Markdown  | prose |
| Bubble    | frame |

```rust
fn main() {
    println!("hello");
}
```

Bare URLs such as https://example.com/docs become links too."#;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 36rem;",
            Markdown { source: SOURCE }
        }
    }
}
