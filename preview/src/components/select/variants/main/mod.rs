use super::super::component::*;
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Fruit {
    Apple,
    Banana,
    Orange,
    Orangeade,
    Strawberry,
    Watermelon,
}

impl Fruit {
    const ALL: [Fruit; 6] = [Fruit::Apple, Fruit::Banana, Fruit::Orange, Fruit::Orangeade, Fruit::Strawberry, Fruit::Watermelon];
}

impl std::fmt::Display for Fruit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Fruit {
    const fn emoji(&self) -> &'static str {
        match self {
            Fruit::Apple => "🍎",
            Fruit::Banana => "🍌",
            Fruit::Orange => "🍊",
            Fruit::Orangeade => "🧃",
            Fruit::Strawberry => "🍓",
            Fruit::Watermelon => "🍉",
        }
    }
}

#[component]
pub fn Demo() -> Element {
    let fruits = Fruit::ALL.into_iter().enumerate().map(|(i, f)| {
        rsx! {
            SelectOption::<Option<Fruit>> { index: i, value: f, text_value: "{f}",
                disabled: matches!(f, Fruit::Orange),
                "{f.emoji()} {f}"
            }
        }
    });

    rsx! {
        Select::<Option<Fruit>> {
            // The trigger's text, read at render time (matches each option's `text_value`).
            item_to_label: |fruit: Option<Fruit>| match fruit {
                Some(fruit) => fruit.to_string(),
                None => "Other".to_string(),
            },
            width: "12rem",
            SelectGroup {
                SelectGroupLabel { "Fruits" }
                {fruits}
            }
            SelectGroup {
                SelectGroupLabel { "Other" }
                SelectOption::<Option<Fruit>> {
                    index: Fruit::ALL.len(),
                    value: None,
                    text_value: "Other",
                    "Other"
                }
            }
        }
    }
}
