use super::super::component::*;
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Topping {
    Pepperoni,
    Mushroom,
    Onion,
    Olive,
    Pineapple,
}

impl Topping {
    const ALL: [Topping; 5] = [Topping::Pepperoni, Topping::Mushroom, Topping::Onion, Topping::Olive, Topping::Pineapple];
}

impl std::fmt::Display for Topping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Topping {
    const fn emoji(&self) -> &'static str {
        match self {
            Topping::Pepperoni => "🍕",
            Topping::Mushroom => "🍄",
            Topping::Onion => "🧅",
            Topping::Olive => "🫒",
            Topping::Pineapple => "🍍",
        }
    }
}

#[component]
pub fn Demo() -> Element {
    let toppings = Topping::ALL.into_iter().enumerate().map(|(i, t)| {
        rsx! {
            SelectOption::<Topping> { index: i, value: t, text_value: "{t}",
                "{t.emoji()} {t}"
            }
        }
    });

    rsx! {
        SelectMulti::<Topping> {
            default_values: vec![Topping::Pepperoni, Topping::Mushroom],
            width: "16rem",
            SelectGroup {
                SelectGroupLabel { "Toppings" }
                {toppings}
            }
        }
    }
}
