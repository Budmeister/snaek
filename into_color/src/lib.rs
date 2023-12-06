// In your procedural macro crate

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};
use quote::quote;

#[proc_macro]
pub fn as_color(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let color_str = input.value();

    // Ensure the string is a valid color code
    if !color_str.starts_with('#') || (color_str.len() != 7 && color_str.len() != 9) {
        return quote! { compile_error!("Invalid color format"); }.into();
    }

    // Parse color components
    let r = u8::from_str_radix(&color_str[1..3], 16).expect("Unable to parse hex");
    let g = u8::from_str_radix(&color_str[3..5], 16).expect("Unable to parse hex");
    let b = u8::from_str_radix(&color_str[5..7], 16).expect("Unable to parse hex");
    let a = if color_str.len() == 9 {
        u8::from_str_radix(&color_str[7..9], 16).unwrap()
    } else {
        u8::MAX
    };

    // Convert to f32 and normalize
    let (r, g, b, a) = (
        r as f32 / u8::MAX as f32,
        g as f32 / u8::MAX as f32,
        b as f32 / u8::MAX as f32,
        a as f32 / u8::MAX as f32,
    );

    // Output the generated const
    quote! { [#r, #g, #b, #a] }.into()
}
