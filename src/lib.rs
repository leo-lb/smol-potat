#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![recursion_limit = "512"]

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Enables an async main function.
///
/// # Examples
///
/// ```ignore
/// #[smol::main]
/// async fn main() -> std::io::Result<()> {
///     Ok(())
/// }
/// ```
#[cfg(not(test))] // NOTE: exporting main breaks tests, we should file an issue.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if name != "main" {
        return TokenStream::from(quote_spanned! { name.span() =>
            compile_error!("only the main function can be tagged with #[smol::main]"),
        });
    }

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        fn main() #ret {
            #(#attrs)*
            async fn main(#inputs) #ret {
                #body
            }

            smol::run(async {
                main().await
            })
        }

    };

    result.into()
}

/// Enables an async test function.
///
/// # Examples
///
/// ```ignore
/// #[smol::test]
/// async fn my_test() -> std::io::Result<()> {
///     assert_eq!(2 * 2, 4);
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        #[test]
        #(#attrs)*
        fn #name() #ret {
            smol::run(async { #body })
        }
    };

    result.into()
}

/// Enables an async benchmark function.
///
/// # Examples
///
/// ```ignore
/// #![feature(test)]
/// extern crate test;
///
/// #[smol::bench]
/// async fn bench() {
///     println!("hello world");
/// }
/// ```
#[proc_macro_attribute]
pub fn bench(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let args = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    if !args.is_empty() {
        return TokenStream::from(quote_spanned! { args.span() =>
            compile_error!("async benchmarks don't take any arguments"),
        });
    }

    let result = quote! {
        #[bench]
        #(#attrs)*
        fn #name(b: &mut test::Bencher) #ret {
            let _ = b.iter(|| {
                smol::block_on(async {
                    #body
                })
            });
        }
    };

    result.into()
}