use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Fields, parse_macro_input};

// ── Shared helpers ────────────────────────────────────────────────────────────

fn extract_inspectable_fields(ast: &mut DeriveInput) -> Vec<(syn::Ident, syn::Type)> {
    let mut inspectable = Vec::new();
    if let syn::Data::Struct(ref mut data) = ast.data {
        if let Fields::Named(ref mut fields) = data.fields {
            for field in fields.named.iter_mut() {
                let has = field.attrs.iter().any(|a| a.path().is_ident("inspectable"));
                if has {
                    field.attrs.retain(|a| !a.path().is_ident("inspectable"));
                    inspectable.push((field.ident.clone().unwrap(), field.ty.clone()));
                }
            }
        }
    }
    inspectable
}

fn type_to_get_expr(name: &syn::Ident, ty: &syn::Type) -> TokenStream2 {
    let ts = quote!(#ty).to_string().replace(" ", "");
    match ts.as_str() {
        "f32" => {
            quote! { ::phantom_core::reflecton::fields::Field::F32(stringify!(#name), self.#name) }
        }
        "bool" => {
            quote! { ::phantom_core::reflecton::fields::Field::Bool(stringify!(#name), self.#name) }
        }
        "i32" => {
            quote! { ::phantom_core::reflecton::fields::Field::I32(stringify!(#name), self.#name) }
        }
        "u32" => {
            quote! { ::phantom_core::reflecton::fields::Field::U32(stringify!(#name), self.#name) }
        }
        "String" => {
            quote! { ::phantom_core::reflecton::fields::Field::String(stringify!(#name), self.#name.clone()) }
        }
        "Vec2" | "glam::Vec2" => {
            quote! { ::phantom_core::reflecton::fields::Field::Vec2(stringify!(#name), self.#name) }
        }
        "Vec3" | "glam::Vec3" => {
            quote! { ::phantom_core::reflecton::fields::Field::Vec3(stringify!(#name), self.#name) }
        }
        "UVec2" | "glam::UVec2" => {
            quote! { ::phantom_core::reflecton::fields::Field::UVec2(stringify!(#name), self.#name) }
        }
        "Quat" | "glam::Quat" => {
            quote! { ::phantom_core::reflecton::fields::Field::TransQuat(stringify!(#name), self.#name) }
        }
        other => panic!(
            "Unsupported #[inspectable] type `{}`. Add a Field variant and update the macro.",
            other
        ),
    }
}

fn type_to_set_expr(idx: usize, name: &syn::Ident, ty: &syn::Type) -> TokenStream2 {
    let ts = quote!(#ty).to_string().replace(" ", "");
    match ts.as_str() {
        "f32" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::F32(_, v)        = &fields[#idx] { self.#name = *v; } }
        }
        "bool" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::Bool(_, v)       = &fields[#idx] { self.#name = *v; } }
        }
        "i32" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::I32(_, v)        = &fields[#idx] { self.#name = *v; } }
        }
        "u32" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::U32(_, v)        = &fields[#idx] { self.#name = *v; } }
        }
        "String" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::String(_, v)   = &fields[#idx] { self.#name = v.clone(); } }
        }
        "Vec2" | "glam::Vec2" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::Vec2(_, v)  = &fields[#idx] { self.#name = *v; } }
        }
        "Vec3" | "glam::Vec3" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::Vec3(_, v)  = &fields[#idx] { self.#name = *v; } }
        }
        "UVec2" | "glam::UVec2" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::UVec2(_, v) = &fields[#idx] { self.#name = *v; } }
        }
        "Quat" | "glam::Quat" => {
            quote! { if let ::phantom_core::reflecton::fields::Field::TransQuat(_, v) = &fields[#idx] { self.#name = *v; } }
        }
        other => panic!(
            "Unsupported #[inspectable] type `{}`. Add a Field variant and update the macro.",
            other
        ),
    }
}

fn gen_reflection(struct_name: &syn::Ident, fields: &[(syn::Ident, syn::Type)]) -> TokenStream2 {
    let gets: Vec<_> = fields.iter().map(|(n, t)| type_to_get_expr(n, t)).collect();
    let sets: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, (n, t))| type_to_set_expr(i, n, t))
        .collect();
    quote! {
        impl ::phantom_core::reflecton::Reflection for #struct_name {
            fn get_fields(&self) -> ::std::vec::Vec<::phantom_core::reflecton::fields::Field> {
                vec![ #(#gets),* ]
            }
            fn set_feilds(&mut self, fields: ::std::vec::Vec<::phantom_core::reflecton::fields::Field>) {
                #(#sets)*
            }
        }
    }
}

fn gen_deserialize_fn(deserialize_fn: &syn::Ident, struct_name: &syn::Ident) -> TokenStream2 {
    quote! {
        fn #deserialize_fn(data: &[u8]) -> ::std::boxed::Box<dyn ::phantom_core::ecs::AnyStorage> {
            match ::phantom_core::serde_json::from_slice::<::phantom_core::ecs::SparseSet<#struct_name>>(data) {
                Ok(set) => ::std::boxed::Box::new(set),
                Err(e) => {
                    eprintln!("[phantom] Failed to deserialize {}, schema may have changed. Resetting to empty. Error: {}", stringify!(#struct_name), e);
                    ::std::boxed::Box::new(::phantom_core::ecs::SparseSet::<#struct_name>::new())
                }
            }
        }
    }
}

fn gen_add_default_fn(add_default_fn: &syn::Ident, struct_name: &syn::Ident) -> TokenStream2 {
    quote! {
        fn #add_default_fn(entity: ::phantom_core::ecs::Entity) -> ::std::boxed::Box<dyn ::std::ops::FnOnce(&mut ::phantom_core::ecs::World)> {
            ::std::boxed::Box::new(move |world: &mut ::phantom_core::ecs::World| {
                world.add_component(entity, #struct_name::default());
            })
        }
    }
}

fn gen_remove_fn(remove_fn: &syn::Ident, struct_name: &syn::Ident) -> TokenStream2 {
    quote! {
        fn #remove_fn(world: &mut ::phantom_core::ecs::World, entity: ::phantom_core::ecs::Entity) {
            world.remove_component::<#struct_name>(entity);
        }
    }
}

// ── #[component] ─────────────────────────────────────────────────────────────

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast: DeriveInput = syn::parse(item).unwrap();
    let struct_name = ast.ident.clone();
    let lower = struct_name.to_string().to_lowercase();

    let deserialize_fn = quote::format_ident!("__phantom_deserialize_{}", lower);
    let add_default_fn = quote::format_ident!("__phantom_add_default_{}", lower);
    let remove_fn = quote::format_ident!("__phantom_remove_{}", lower);
    let register_fn = quote::format_ident!("__phantom_register_{}", lower);

    let inspectable = extract_inspectable_fields(&mut ast);
    let reflection = gen_reflection(&struct_name, &inspectable);
    let deserialize = gen_deserialize_fn(&deserialize_fn, &struct_name);
    let add_default = gen_add_default_fn(&add_default_fn, &struct_name);
    let remove = gen_remove_fn(&remove_fn, &struct_name);

    quote! {
        #[derive(::phantom_core::serde::Serialize, ::phantom_core::serde::Deserialize)]
        #[serde(crate = "::phantom_core::serde")]
        #ast

        impl ::phantom_core::ecs::component::Component for #struct_name {
            const NAME: &'static str = stringify!(#struct_name);
        }

        #reflection
        #deserialize
        #add_default
        #remove

        pub fn #register_fn(
            comp_reg: &mut ::std::collections::HashMap<&'static str, ::phantom_core::ecs::component_registry::ComponentEntry>,
            _script_reg: &mut ::std::collections::HashMap<&'static str, (fn(&mut ::phantom_core::ecs::World, &::phantom_core::scripting::ScriptContext), fn(&mut ::phantom_core::ecs::World, &::phantom_core::scripting::ScriptContext))>,
        ) {
            eprintln!("[register_{}] Starting registration", stringify!(#struct_name));
            comp_reg.insert(
                <#struct_name as ::phantom_core::ecs::component::Component>::NAME,
                ::phantom_core::ecs::component_registry::ComponentEntry(#deserialize_fn, #add_default_fn, #remove_fn, false)
            );
            eprintln!("[register_{}] Registration completed", stringify!(#struct_name));
        }
    }.into()
}

// ── #[script] ────────────────────────────────────────────────────────────────

#[proc_macro_attribute]
pub fn script(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast: DeriveInput = syn::parse(item).unwrap();
    let struct_name = ast.ident.clone();
    let lower = struct_name.to_string().to_lowercase();

    let deserialize_fn = quote::format_ident!("__phantom_deserialize_{}", lower);
    let add_default_fn = quote::format_ident!("__phantom_add_default_{}", lower);
    let remove_fn = quote::format_ident!("__phantom_remove_{}", lower);
    let start_all_fn = quote::format_ident!("__phantom_start_all_{}", lower);
    let update_all_fn = quote::format_ident!("__phantom_update_all_{}", lower);
    let register_fn = quote::format_ident!("__phantom_register_{}", lower);

    let inspectable = extract_inspectable_fields(&mut ast);
    let reflection = gen_reflection(&struct_name, &inspectable);
    let deserialize = gen_deserialize_fn(&deserialize_fn, &struct_name);
    let add_default = gen_add_default_fn(&add_default_fn, &struct_name);
    let remove = gen_remove_fn(&remove_fn, &struct_name);

    quote! {
        #[derive(::phantom_core::serde::Serialize, ::phantom_core::serde::Deserialize)]
        #[serde(crate = "::phantom_core::serde")]
        #[serde(default)]
        #ast

        impl ::phantom_core::ecs::component::Component for #struct_name {
            const NAME: &'static str = stringify!(#struct_name);
        }

        #reflection
        #deserialize
        #add_default
        #remove

        fn #start_all_fn(world: &mut ::phantom_core::ecs::World, ctx: &::phantom_core::scripting::ScriptContext) {
            let entities = world.query_with::<#struct_name>();
            for entity in entities {
                if let Some(script) = world.get_component_mut::<#struct_name>(entity) {
                    let script = script as *mut #struct_name;
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
                        unsafe { (*script).start(entity, world, ctx); }
                    }));
                    if let Err(_) = result {
                        eprintln!("[Phantom] Script panic in {}", stringify!(#struct_name));
                    }
                }
            }
        }

        fn #update_all_fn(world: &mut ::phantom_core::ecs::World, ctx: &::phantom_core::scripting::ScriptContext)  {
            let entities = world.query_with::<#struct_name>();
            for entity in entities {
                if let Some(script) = world.get_component_mut::<#struct_name>(entity) {
                    let script = script as *mut #struct_name;
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
                        unsafe { (*script).update(entity, world, ctx); }
                    }));
                    if let Err(_) = result {
                        eprintln!("[Phantom] Script panic in {}", stringify!(#struct_name));
                    }
                }

            }
        }

        pub fn #register_fn(
            comp_reg: &mut ::std::collections::HashMap<&'static str, ::phantom_core::ecs::component_registry::ComponentEntry>,
            script_reg: &mut ::std::collections::HashMap<&'static str, (fn(&mut ::phantom_core::ecs::World, &::phantom_core::scripting::ScriptContext), fn(&mut ::phantom_core::ecs::World, &::phantom_core::scripting::ScriptContext))>,
        ) {
            eprintln!("[register_{}] Starting registration", stringify!(#struct_name));
            comp_reg.insert(
                <#struct_name as ::phantom_core::ecs::component::Component>::NAME,
                ::phantom_core::ecs::component_registry::ComponentEntry(#deserialize_fn, #add_default_fn, #remove_fn, false)
            );
            script_reg.insert(
                <#struct_name as ::phantom_core::ecs::component::Component>::NAME,
                (#start_all_fn, #update_all_fn)
            );
            eprintln!("[register_{}] Registration completed", stringify!(#struct_name));
        }
    }.into()
}

// ── phantom_register! ─────────────────────────────────────────────────────────

#[proc_macro]
pub fn phantom_register(input: TokenStream) -> TokenStream {
    let paths = parse_macro_input!(input with syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated);

    let register_calls = paths.iter().map(|path| {
        let struct_name = &path.segments.last().unwrap().ident;
        let register_fn = quote::format_ident!(
            "__phantom_register_{}",
            struct_name.to_string().to_lowercase()
        );
        let mut mod_path = path.clone();
        mod_path.segments.pop();
        if mod_path.segments.is_empty() {
            quote! { #register_fn(comp_reg_ref, script_reg_ref); }
        } else {
            quote! { #mod_path #register_fn(comp_reg_ref, script_reg_ref); }
        }
    });

    quote! {
        // The trait-object reference (`&dyn Log`) is a fat pointer, which trips the
        // improper_ctypes lint. It's sound here: host and dylib share the same `log`
        // crate version, so the layout matches, and the logger lives in the host which
        // never unloads.
        #[unsafe(no_mangle)]
        #[allow(improper_ctypes_definitions)]
        pub extern "C" fn phantom_init(
            comp_reg: *mut ::std::collections::HashMap<&'static str, ::phantom_core::ecs::component_registry::ComponentEntry>,
            script_reg: *mut ::std::collections::HashMap<&'static str, (fn(&mut ::phantom_core::ecs::World, &::phantom_core::scripting::ScriptContext), fn(&mut ::phantom_core::ecs::World, &::phantom_core::scripting::ScriptContext))>,
            logger: &'static dyn ::phantom_core::log::Log,
            max_level: ::phantom_core::log::LevelFilter,
        ) {
            // Install the host's logger into this dylib's own (separate) `log` global so
            // that `log::info!` etc. from game code route to the host sink. A fresh dylib
            // load has an uninitialized global, so this succeeds once per (re)load.
            let _ = ::phantom_core::log::set_logger(logger);
            ::phantom_core::log::set_max_level(max_level);

            eprintln!("[phantom_init] Called with pointers: comp_reg={:p} script_reg={:p}", comp_reg, script_reg);
            if comp_reg.is_null() || script_reg.is_null() {
                eprintln!("[phantom_init] ERROR: Null pointer received!");
                return;
            }
            eprintln!("[phantom_init] Pointers valid, dereferencing to mutable references...");
            unsafe {
                let comp_reg_ref = &mut *comp_reg;
                let script_reg_ref = &mut *script_reg;
                eprintln!("[phantom_init] References obtained, calling register functions...");
                #(#register_calls)*
                eprintln!("[phantom_init] All register functions completed");
            }
        }
    }.into()
}
