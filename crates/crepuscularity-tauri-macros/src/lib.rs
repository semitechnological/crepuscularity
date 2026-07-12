use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, FnArg, Ident, ItemFn, Pat,
    ReturnType, Token, Type,
};

#[proc_macro_attribute]
pub fn command(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let name = &function.sig.ident;
    let bridge = format_ident!("__crepus_command_{name}");
    let mut bindings = Vec::new();
    let mut arguments = Vec::new();
    for argument in &function.sig.inputs {
        let FnArg::Typed(argument) = argument else {
            return quote!(compile_error!("crepuscularity-tauri commands cannot use a receiver");)
                .into();
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return quote!(compile_error!("crepuscularity-tauri command parameters need names");)
                .into();
        };
        let binding = &pattern.ident;
        let key = binding.to_string();
        let ty = &argument.ty;
        match ty.as_ref() {
            Type::Reference(reference) if matches!(reference.elem.as_ref(), Type::Path(path) if path.path.is_ident("str")) =>
            {
                bindings.push(quote! {
                    let #binding: String = ::serde_json::from_value(payload.get(#key).cloned().unwrap_or(::serde_json::Value::Null))
                        .map_err(|error| format!("invalid command argument {}: {error}", #key))?;
                });
                arguments.push(quote!(&#binding));
            }
            _ => {
                bindings.push(quote! {
                    let #binding: #ty = ::serde_json::from_value(payload.get(#key).cloned().unwrap_or(::serde_json::Value::Null))
                        .map_err(|error| format!("invalid command argument {}: {error}", #key))?;
                });
                arguments.push(quote!(#binding));
            }
        }
    }
    let invoke = if function.sig.asyncness.is_some() {
        quote!(::crepuscularity_tauri::block_on(#name(#(#arguments),*)))
    } else {
        quote!(#name(#(#arguments),*))
    };
    let call = match &function.sig.output {
        ReturnType::Default => quote! { #invoke; ::serde_json::Value::Null },
        ReturnType::Type(_, output) if is_result(output) => {
            quote! { #invoke.map_err(|error| error.to_string())? }
        }
        ReturnType::Type(_, _) => quote! { #invoke },
    };
    quote! {
        #function
        #[doc(hidden)]
        pub fn #bridge(payload: ::serde_json::Value) -> Result<::serde_json::Value, String> {
            #(#bindings)*
            ::serde_json::to_value(#call).map_err(|error| error.to_string())
        }
    }
    .into()
}

fn is_result(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Result"))
}

#[proc_macro]
pub fn generate_handler(input: TokenStream) -> TokenStream {
    let names = match Punctuated::<Ident, Token![,]>::parse_terminated.parse(input) {
        Ok(names) => names,
        Err(error) => return error.into_compile_error().into(),
    };
    let commands = names.iter().map(|name| {
        let bridge = format_ident!("__crepus_command_{name}");
        quote!(::crepuscularity_tauri::Command::new(stringify!(#name), #bridge))
    });
    quote!(vec![#(#commands),*]).into()
}
