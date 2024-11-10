// extern crate proc_macro;
// use proc_macro::TokenStream;

// #[proc_macro_attribute]
// pub fn derive_answer_fn(_args: TokenStream, input: TokenStream) -> TokenStream {
//     "fn answer() -> u32 { 42 }".parse().unwrap()
// }

pub mod core;
pub mod filters;
pub mod score;
pub mod tones;

pub mod generator;

// Should be last, maps inputs to functions of the previous mods
pub mod inputs;

#[cfg(feature = "plotting")]
pub mod plotting;

#[cfg(test)]
pub mod tests;

use filters::{FilterMetadata, Metadata};

// Todo: this function must be moved to a more correct place
pub fn filter_metadata() -> Vec<FilterMetadata> {
    vec![
        filters::GainFilter::get_metadata(),
        filters::DelayFilter::get_metadata(),
        filters::LowPassFilter::get_metadata(),
        filters::HighPassFilter::get_metadata(),
        filters::CombinatorFilter::get_metadata(),
        filters::DuplicateFilter::get_metadata(),
    ]
}
