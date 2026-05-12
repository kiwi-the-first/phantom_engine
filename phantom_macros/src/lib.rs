use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let struct_name = &ast.ident;
    let struct_name_lower = struct_name.to_string().to_lowercase();
    let deserialize_fn = quote::format_ident!("__phantom_deserialize_{}", struct_name_lower);
    let register_fn = quote::format_ident!("__phantom_register_{}", struct_name_lower);

    let generated = quote! {
        #[derive(::phantom_core::serde::Serialize, ::phantom_core::serde::Deserialize)]
        #[serde(crate = "::phantom_core::serde")]
        #ast

        impl ::phantom_core::ecs::component::Component for #struct_name {
            const NAME: &'static str = stringify!(#struct_name);
        }

        fn #deserialize_fn(data: &[u8]) -> ::std::boxed::Box<dyn ::phantom_core::ecs::AnyStorage> {
            ::std::boxed::Box::new(::phantom_core::bincode::deserialize::<::phantom_core::ecs::SparseSet<#struct_name>>(data).unwrap())
        }

        #[::phantom_core::ctor::ctor]
        fn #register_fn() {
            ::phantom_core::ecs::component_registry::register_component(<#struct_name as ::phantom_core::ecs::component::Component>::NAME, #deserialize_fn);
        }
    };

    generated.into()
}
