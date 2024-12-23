// Let's simplify this.
// 1. I am writing this only for myself.
//    - this means one-crate support is fine, we don't have to do anything special
//      about different crates (for them to implement feature gated schema export)
//    - if we decide to publish `rawr`, consumers then must feature gate schema
//      export, since that is the only correct way, basically:
//     `#[cfg_attr(feature = "schema", rawr::export)`

// 2. Upgrade to distributed_slice.
//    - it doesn't run code, so no startup delay
//    - sadly, it seems like the static doesn't get optimized out by the compiler
//      even, if it's not used anywhere. still, it's whatever, since we'd need
//      to use feature gates for that either way.

// 3. For dynamically allocating the registry, instead of the structures themselves
// we'd simply pass in the callbacks which return them.

#[cfg(feature = "export")]
const _: () = {
    #[rawr::distributed_slice(rawr::REGISTRY)]
    static ___: &'static str = "foo";
};

#[cfg(feature = "export")]
const _: () = {
    #[rawr::distributed_slice(rawr::REGISTRY)]
    static ___: &'static str = "bar";
};

pub fn export() -> Vec<&'static str> {
    rawr::REGISTRY.iter().cloned().collect()
}

//
// Will callback get optimized out?
//
// Nope! But that's fine. We just need to add feature gate "export".

#[rawr::distributed_slice]
pub static CALLBACK_REGISTRY: [fn() -> &'static str];

const _: () = {
    #[rawr::distributed_slice(CALLBACK_REGISTRY)]
    static ___: fn() -> &'static str = callback;

    fn callback() -> &'static str {
        "alcazar"
    }
};
