//! The kind table: the one registration of every kind, and what is generated from it.

use crate::wire::{self, FromValue};

/// A kind as the generated adapter and the page see it: the wire keys of its three inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindSpec {
    /// The wire `kind` (kebab-case, the `kind` of the Rust `Behaviour`).
    pub name: &'static str,
    /// The keys of the behaviour's parameters the decision reads.
    pub config: &'static [&'static str],
    /// The attribute reads taken before the event (`Snapshot`).
    pub reads: &'static [&'static str],
    /// The event fields collected for the decision (`Event`).
    pub fields: &'static [&'static str],
}

/// Registers the kinds of the kernel: `"wire-name" => module`, where the module holds `Config`, `Snapshot`, `Event` and
/// `decide`. The position in this list is the kind's index on the wire, so the table ([`KINDS`]), the dispatch
/// ([`decide`]) and the page's generated table are one list.
macro_rules! kinds {
    ($($name:literal => $module:ident),+ $(,)?) => {
        /// The index of each kind in [`KINDS`] (the order of the registration).
        #[expect(non_camel_case_types, reason = "a variant is the kind's module name, so the dispatch cannot drift from the table")]
        enum Index { $($module),+ }

        /// Every kind of the kernel, in wire-index order.
        pub const KINDS: &[KindSpec] = &[
            $( KindSpec {
                name: $name,
                config: <crate::$module::Config as FromValue>::KEYS,
                reads: <crate::$module::Snapshot as FromValue>::KEYS,
                fields: <crate::$module::Event as FromValue>::KEYS,
            } ),+
        ];

        /// Decides for the kind at `index` over the encoded request; `None` for an unknown index or a request the kind
        /// cannot read.
        pub fn decide(index: usize, request: &[u8]) -> Option<Vec<u8>> {
            $(
                if index == Index::$module as usize {
                    return wire::run::<crate::$module::Config, crate::$module::Snapshot, crate::$module::Event>(
                        request,
                        crate::$module::decide,
                    );
                }
            )+
            None
        }
    };
}

kinds! {
    "fixture" => fixture,
}

/// The table as text: one line per kind, `name`, then its config keys, reads and fields, tab-separated, each list
/// comma-separated. It is what [`table_hash`] hashes and what the generated `behaviours.gen.ts` is derived from.
pub fn table_text() -> String {
    let mut text = String::new();
    for kind in KINDS {
        text.push_str(kind.name);
        for list in [kind.config, kind.reads, kind.fields] {
            text.push('\t');
            text.push_str(&list.join(","));
        }
        text.push('\n');
    }
    text
}

const FNV_BASIS: u32 = 0x811c_9dc5;

/// FNV-1a over `text`, continuing from `hash`.
const fn fnv(mut hash: u32, text: &str) -> u32 {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        hash = (hash ^ bytes[i] as u32).wrapping_mul(0x0100_0193);
        i += 1;
    }
    hash
}

/// The hash of [`table_text`] (32-bit FNV-1a), computed at compile time so the module carries a constant: it
/// exports it as `dx_kinds`, the generated file carries it, and the page refuses a module whose hash differs from its
/// own.
const fn hash_of_table() -> u32 {
    let mut hash = FNV_BASIS;
    let mut k = 0;
    while k < KINDS.len() {
        hash = fnv(hash, KINDS[k].name);
        let lists = [KINDS[k].config, KINDS[k].reads, KINDS[k].fields];
        let mut l = 0;
        while l < lists.len() {
            hash = fnv(hash, "\t");
            let mut i = 0;
            while i < lists[l].len() {
                if i > 0 {
                    hash = fnv(hash, ",");
                }
                hash = fnv(hash, lists[l][i]);
                i += 1;
            }
            l += 1;
        }
        hash = fnv(hash, "\n");
        k += 1;
    }
    hash
}

/// The hash of the kind table, [`table_text`] hashed by FNV-1a.
pub const TABLE_HASH: u32 = hash_of_table();

/// [`TABLE_HASH`].
pub fn table_hash() -> u32 {
    TABLE_HASH
}

/// The decision of a kind by its wire name over an encoded request (the native route of `cargo test`).
pub fn decide_named(name: &str, request: &[u8]) -> Option<Vec<u8>> {
    decide(KINDS.iter().position(|k| k.name == name)?, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_lists_every_kind_once_in_index_order() {
        let names: Vec<&str> = KINDS.iter().map(|k| k.name).collect();
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len(), "a kind is registered once");
        assert_eq!(names[0], "fixture");
    }

    #[test]
    fn the_hash_follows_the_table() {
        assert_eq!(table_text().lines().count(), KINDS.len());
        // the compile-time hash is FNV-1a of the text
        let runtime = table_text()
            .bytes()
            .fold(0x811c_9dc5_u32, |h, b| (h ^ u32::from(b)).wrapping_mul(0x0100_0193));
        assert_eq!(table_hash(), runtime);
        assert_ne!(table_hash(), 0x811c_9dc5);
    }

    #[test]
    fn an_unknown_index_has_no_answer() {
        assert!(decide(KINDS.len(), &[]).is_none());
        assert!(decide(usize::MAX, &[]).is_none());
        assert!(decide_named("nope", &[]).is_none());
    }
}
