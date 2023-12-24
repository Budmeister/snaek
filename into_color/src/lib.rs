// In your procedural macro crate

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr, parse::Parse, parse::ParseStream, Token, bracketed, LitInt, LitFloat};
use quote::quote;

fn as_color_fn(color_str: &str) -> Result<(u8, u8, u8), &'static str> {
    // Ensure the string is a valid color code
    if !color_str.starts_with('#') || (color_str.len() != 7 && color_str.len() != 9) {
        return Err("Invalid color format");
    }

    // Parse color components
    let r = match u8::from_str_radix(&color_str[1..3], 16) { Ok(r) => r, Err(_) => return Err("Unable to parse hex") };
    let g = match u8::from_str_radix(&color_str[3..5], 16) { Ok(g) => g, Err(_) => return Err("Unable to parse hex") };
    let b = match u8::from_str_radix(&color_str[5..7], 16) { Ok(b) => b, Err(_) => return Err("Unable to parse hex") };
    let _a = if color_str.len() == 9 {
        u8::from_str_radix(&color_str[7..9], 16).unwrap()
    } else {
        u8::MAX
    };

    Ok((r, g, b))
}

#[proc_macro]
pub fn as_color(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let color_str = input.value();

    let (r, g, b) = match as_color_fn(&color_str) {
        Ok(rgb) => rgb,
        Err(err) => return quote! { compile_error!(#err); }.into(),
    };

    // Output the generated const
    quote! { (#r, #g, #b) }.into()
}


struct ColorSpaceInput {
    colors: Vec<ColorPointExpr>,
    steps: LitInt,
}

struct ColorPointExpr {
    color: LitStr,
    position: LitFloat,
}

impl Parse for ColorSpaceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let colors: syn::punctuated::Punctuated<ColorPointExpr, Token![,]> = content.parse_terminated(ColorPointExpr::parse, Token![,])?;
        input.parse::<Token![,]>()?;
        let steps: LitInt = input.parse()?;
        Ok(ColorSpaceInput { colors: colors.into_iter().collect(), steps })
    }
}

impl Parse for ColorPointExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let color: LitStr = content.parse()?;
        let _: Token![,] = content.parse()?;
        let position: LitFloat = content.parse()?;
        Ok(ColorPointExpr { color, position })
    }
}

#[proc_macro]
pub fn color_space(input: TokenStream) -> TokenStream {
    let ColorSpaceInput { colors, steps } = parse_macro_input!(input as ColorSpaceInput);

    let colors = colors.into_iter()
            .map(|cpe| {

                let color = cpe.color.value();
                let color = as_color_fn(&color)?;
                let position = match cpe.position.base10_parse() { Ok(lit_float) => lit_float, Err(_) => return Err("Invalid color position") };

                Ok(ColorPoint {
                    color,
                    position,
                })
            })
            .collect::<Result<_, &'static str>>();
    let colors: Vec<ColorPoint> = match colors {
        Ok(colors) => colors,
        Err(err) => return quote! { compile_error!(#err); }.into(),
    };

    let steps = match steps.base10_parse() {
        Ok(steps) => steps,
        Err(_) => return quote! { compile_error!("Invalid steps") }.into(),
    };

    let interp_colors: Vec<_> = color_space_fn(colors, steps)
            .into_iter()
            .map(|(r, g, b)| quote! { (#r, #g, #b) })
            .collect();

    quote! {
        [
            #( #interp_colors ),*
        ]
    }.into()
}

struct ColorPoint {
    color: (u8, u8, u8),
    position: f64,
}

fn color_space_fn(colors: Vec<ColorPoint>, steps: usize) -> Vec<(u8, u8, u8)> {
    let mut result = Vec::new();

    for i in 0..(steps - 1) {
        let progress = i as f64 / (steps - 1) as f64;
        let mut prev_color = &colors[0];
        
        for j in 1..colors.len() {
            let current_color = &colors[j];
            if progress <= current_color.position {
                let t = (progress - prev_color.position) / (current_color.position - prev_color.position);
                let r = lerp(prev_color.color .0, current_color.color .0, t);
                let g = lerp(prev_color.color .1, current_color.color .1, t);
                let b = lerp(prev_color.color .2, current_color.color .2, t);
                result.push((r, g, b));
                break;
            }
            prev_color = current_color;
        }
    }

    result.push(colors.last().unwrap().color); // Ensure the last color is added
    result
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).round() as u8
}


