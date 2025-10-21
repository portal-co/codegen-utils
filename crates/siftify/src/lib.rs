use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::token::Pub;
use syn::Field;
use syn::{punctuated::Punctuated, Fields, FieldsUnnamed, TypeParam};
pub fn patch(e: &mut syn::ItemEnum) -> TokenStream{
    for v in e.variants.iter(){
        let p = format_ident!("__Pattern_{}",v.ident);
        e.generics.params.push(syn::GenericParam::Type(TypeParam{attrs: vec![], ident: p, colon_token: None, bounds: Default::default(), eq_token: Some(Default::default()), default: Some(syn::parse_str("()").unwrap()) }));
    }
    for v in e.variants.iter_mut(){
        let p = format_ident!("__Pattern_{}",v.ident);
        let ty = syn::parse2(quote!{#p}).unwrap();
        let mut f = Field{ attrs: vec![], vis: syn::Visibility::Public(Pub::default()), mutability: syn::FieldMutability::None, ident: None, colon_token: None, ty };
        match &mut v.fields{
            syn::Fields::Named(n) => {
                f.ident = Some(format_ident!("__field_{}",v.ident));
                f.colon_token = Some(Default::default());
                n.named.push(f);
            },
            syn::Fields::Unnamed(u) => {
                u.unnamed.push(f);
            },
            syn::Fields::Unit => v.fields = Fields::Unnamed(FieldsUnnamed{paren_token: Default::default(), unnamed: Punctuated::from_iter(vec![f])}),
        }
    }
    quote!{
    }
}
