use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(SoA, attributes(soa))]
pub fn derive_soa(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let vis = input.vis.clone();
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(named) => named.named,
            _ => {
                return syn::Error::new_spanned(
                    s.fields,
                    "SoA derive requires a braced struct with named fields",
                )
                .to_compile_error()
                .into()
            }
        },
        _ => {
            return syn::Error::new_spanned(&ident, "SoA derive works only on structs")
                .to_compile_error()
                .into()
        }
    };

    if fields.is_empty() {
        return syn::Error::new_spanned(&ident, "SoA derive requires at least one field")
            .to_compile_error()
            .into();
    }

    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| f.ty.clone()).collect();

    let soa_ident = format_ident!("{}SoA", ident);
    let view_ident = format_ident!("{}View", ident);
    let view_mut_ident = format_ident!("{}Mut", ident);

    let columns = field_idents.iter().zip(field_types.iter()).map(|(id, ty)| {
        quote! { #id: ::std::vec::Vec<#ty> }
    });

    let push_moves = field_idents.iter().map(|id| {
        quote! { self.#id.push(v.#id); }
    });

    let view_fields = field_idents.iter().zip(field_types.iter()).map(|(id, ty)| {
        quote! { pub #id: &'a #ty }
    });

    let view_mut_fields = field_idents.iter().zip(field_types.iter()).map(|(id, ty)| {
        quote! { pub #id: &'a mut #ty }
    });

    let view_ctor_bind = field_idents.iter().map(|id| {
        quote! { #id: &self.#id[i] }
    });
    let view_mut_ctor_bind = field_idents.iter().map(|id| {
        quote! { #id: &mut self.#id[i] }
    });

    let first_field = &field_idents[0];
    let equal_len_asserts = field_idents.iter().map(|id| {
        quote! { debug_assert_eq!(self.#first_field.len(), self.#id.len(), "SoA columns length mismatch"); }
    });

    let raw_array_methods = field_idents.iter().zip(field_types.iter()).map(|(id, ty)| {
        let method_name = format_ident!("{}_raw_array", id);
        quote! {
            #vis fn #method_name(&self) -> &[#ty] {
                &self.#id
            }
        }
    });

    let expanded = quote! {
        #[derive(Clone)]
        #vis struct #soa_ident {
            #( #columns, )*
        }

        impl #soa_ident {
            #vis fn new() -> Self {
                Self { #( #field_idents: ::std::vec::Vec::new(), )* }
            }
            #vis fn with_capacity(cap: usize) -> Self {
                Self { #( #field_idents: ::std::vec::Vec::with_capacity(cap), )* }
            }
            #vis fn len(&self) -> usize {
                #( #equal_len_asserts )*
                self.#first_field.len()
            }
            #vis fn is_empty(&self) -> bool { self.len() == 0 }
            #vis fn push(&mut self, v: #ident #generics) -> usize {
                #( #push_moves )*
                self.len() - 1
            }
            #vis fn view(&self, i: usize) -> #view_ident<'_> {
                #view_ident { #( #view_ctor_bind, )* }
            }
            #vis fn view_mut(&mut self, i: usize) -> #view_mut_ident<'_> {
                #view_mut_ident { #( #view_mut_ctor_bind, )* }
            }
            #vis fn iter(&self) -> impl ::std::iter::Iterator<Item = #view_ident<'_>> + '_ {
                (0..self.len()).map(|i| self.view(i))
            }
        }

        // Raw array accessor methods for performance optimizations
        impl #soa_ident {
            #( #raw_array_methods )*
        }

        #vis struct #view_ident<'a> { #( #view_fields, )* }
        #vis struct #view_mut_ident<'a> { #( #view_mut_fields, )* }

        impl soa_runtime::SoaModel for #ident #generics {
            type Soa = #soa_ident;
            type View<'a> = #view_ident<'a> where Self: 'a;
            type ViewMut<'a> = #view_mut_ident<'a> where Self: 'a;

            fn push_into(soa: &mut Self::Soa, v: Self) {
                soa.push(v);
            }
            fn view(soa: &Self::Soa, i: usize) -> Self::View<'_> {
                soa.view(i)
            }
            fn view_mut(soa: &mut Self::Soa, i: usize) -> Self::ViewMut<'_> {
                soa.view_mut(i)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(SoAStore, attributes(soa_store))]
pub fn derive_soa_store(input: TokenStream) -> TokenStream {
    use syn::{Data, DeriveInput, Fields, Ident, LitInt, LitStr};

    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let vis = input.vis.clone();

    // Validate struct with named fields
    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(named) => named.named,
            _ => {
                return syn::Error::new_spanned(
                    s.fields,
                    "SoAStore derive requires a braced struct with named fields",
                )
                .to_compile_error()
                .into()
            }
        },
        _ => {
            return syn::Error::new_spanned(&ident, "SoAStore derive works only on structs")
                .to_compile_error()
                .into()
        }
    };

    // Defaults
    let mut shard_key = Ident::new("id", ident.span());
    let mut shards_default: usize = 16;

    // Parse: #[soa_store(key = "id", shards = 16)]
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("soa_store"))
    {
        let res = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                let lit: LitStr = meta.value()?.parse()?;
                shard_key = Ident::new(&lit.value(), lit.span());
                Ok(())
            } else if meta.path.is_ident("shards") {
                let lit: LitInt = meta.value()?.parse()?;
                shards_default = lit.base10_parse::<usize>()?;
                Ok(())
            } else {
                Err(meta.error("unknown attribute for soa_store (expected `key` or `shards`)"))
            }
        });
        if let Err(e) = res {
            return e.to_compile_error().into();
        }
    }

    // Validate shard key exists
    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();
    if !field_idents.iter().any(|f| f == &shard_key) {
        return syn::Error::new(
            shard_key.span(),
            "soa_store key must be a field of the struct",
        )
        .to_compile_error()
        .into();
    }

    let soa_ident = format_ident!("{}SoA", ident);
    let store_ident = format_ident!("{}Store", ident);
    let sharded_ident = format_ident!("{}ShardedStore", ident);

    let expanded = quote! {
        #vis struct #store_ident {
            inner: ::std::sync::Arc<#soa_ident>,
        }

        impl ::std::clone::Clone for #store_ident {
            fn clone(&self) -> Self { Self { inner: self.inner.clone() } }
        }
        impl ::std::default::Default for #store_ident {
            fn default() -> Self { Self { inner: ::std::sync::Arc::new(#soa_ident::new()) } }
        }

        impl #store_ident {
            #vis fn new() -> Self { Self::default() }
            #vis fn add(&mut self, v: #ident) -> usize {
                let inner = ::std::sync::Arc::make_mut(&mut self.inner);
                inner.push(v)
            }
            #vis fn kernel(&self) -> &#soa_ident { &self.inner }
            #vis fn kernel_mut(&mut self) -> &mut #soa_ident { ::std::sync::Arc::make_mut(&mut self.inner) }
        }

        #vis struct #sharded_ident {
            shards: ::std::vec::Vec<soa_runtime::CachePadded<#soa_ident>>,
        }

        impl #sharded_ident {
            #vis const DEFAULT_SHARDS: usize = #shards_default;

            #vis fn with_shards(n: usize, cap_per: usize) -> Self {
                let mut shards = ::std::vec::Vec::with_capacity(n);
                for _ in 0..n {
                    shards.push(soa_runtime::CachePadded(#soa_ident::with_capacity(cap_per)));
                }
                Self { shards }
            }

            #[inline]
            fn shard_idx_from_key<K: ::std::hash::Hash>(key: &K, n: usize) -> usize {
                use ::std::hash::{Hash, Hasher};
                let mut h = ::std::collections::hash_map::DefaultHasher::new();
                key.hash(&mut h);
                (h.finish() as usize) % n
            }

            #vis fn add(&mut self, v: #ident) -> (usize, usize) {
                let n = self.shards.len();
                let si = {
                    let keyref = &v.#shard_key;
                    Self::shard_idx_from_key(keyref, n)
                };
                let row = self.shards[si].0.push(v);
                (si, row)
            }

            #vis fn shard_count(&self) -> usize { self.shards.len() }
            #vis fn shard(&self, i: usize) -> &#soa_ident { &self.shards[i].0 }
            #vis fn shard_mut(&mut self, i: usize) -> &mut #soa_ident { &mut self.shards[i].0 }
        }
    };

    TokenStream::from(expanded)
}
