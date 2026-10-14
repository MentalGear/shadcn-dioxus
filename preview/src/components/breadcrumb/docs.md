The Breadcrumb component displays the current page's location within a navigational hierarchy, as a trail of links.

## Component Structure

```rust
Breadcrumb {
    BreadcrumbList {
        BreadcrumbItem {
            BreadcrumbLink { href: "/", "Home" }
        }
        BreadcrumbSeparator {}
        BreadcrumbItem {
            BreadcrumbLink { href: "/components", "Components" }
        }
        BreadcrumbSeparator {}
        BreadcrumbItem {
            // The current page: not a link, marked `aria-current="page"`.
            BreadcrumbPage { "Breadcrumb" }
        }
    }
}
```

To collapse a run of crumbs, put `BreadcrumbEllipsis {}` inside its own `BreadcrumbItem`. It renders a `<span>` (not an `<li>`, so it nests validly in the item) and is hidden from assistive technology, as in shadcn:

```rust
BreadcrumbItem {
    BreadcrumbEllipsis {}
}
```

### In a menu trigger

Because it is a `span`, `BreadcrumbEllipsis` can be the content of a button, so the collapsed crumbs can open a menu (shadcn's `breadcrumb-demo`). The ellipsis and its `More` text are `aria-hidden`, so name the trigger yourself:

```rust
BreadcrumbItem {
    DropdownMenu {
        DropdownMenuTrigger { aria_label: "Toggle menu",
            as: move |attributes: Vec<Attribute>| rsx! {
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::IconSm, attributes,
                    BreadcrumbEllipsis {}
                }
            },
        }
        DropdownMenuContent { /* DropdownMenuItem { .. } */ }
    }
}
```
