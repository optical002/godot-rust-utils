use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input, parse_quote};

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

#[proc_macro_attribute]
pub fn init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    let Data::Struct(ref mut data_struct) = input.data else {
        return syn::Error::new_spanned(&input, "init can only be used on structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(ref mut fields) = data_struct.fields else {
        return syn::Error::new_spanned(&input, "init requires named fields")
            .to_compile_error()
            .into();
    };

    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    if !struct_name_str.ends_with("Init") {
        return syn::Error::new_spanned(
            &input,
            format!(
                "struct name '{}' must end with 'Init' suffix",
                struct_name_str
            ),
        )
        .to_compile_error()
        .into();
    }

    let snake_case_name = to_snake_case(&struct_name_str);
    let assertion_name = quote::format_ident!("_assert_{}_implements_traits", snake_case_name);

    fields.named.insert(
        0,
        parse_quote! {
            init_id: init_system::data::init_id::InitId
        },
    );
    fields.named.insert(
        1,
        parse_quote! {
            ctx: init_system::core::InitContext
        },
    );

    let expanded = quote! {
        #input

        impl init_system::interfaces::GetInitId for #struct_name {
            fn get_id(&self) -> init_system::data::init_id::InitId {
                self.init_id
            }
        }

        impl init_system::interfaces::GetInitContext for #struct_name {
            fn get_ctx(&self) -> init_system::core::InitContext {
                self.ctx.clone()
            }
        }

        const _: () = {
            fn #assertion_name()
            where
                #struct_name: init_system::interfaces::Init,
            {}
        };
    };

    TokenStream::from(expanded)
}
