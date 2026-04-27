use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let struct_name = &ast.ident;
    let struct_name_lower = struct_name.to_string().to_lowercase();
    let fn_name = quote::format_ident!("__deserialize_{}", struct_name_lower);

    let generated = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #ast

        impl Component for #struct_name {
            const NAME: &'static str = stringify!(#struct_name);
        }

        fn #fn_name(data: &[u8]) -> Box<dyn AnyStorage> {
            Box::new(bincode::deserialize::<SparseSet<#struct_name>>(data).unwrap())
        }

        #[::ctor::ctor]
        fn register() {
            ::phantom_core::ecs::component_info::register_component(
                #struct_name::NAME,
                #fn_name
            );
        }


    };

    generated.into()
}
