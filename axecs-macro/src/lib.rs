use proc_macro::TokenStream as TokenStream1;
use syn::{ parse_macro_input, DeriveInput };
use quote::quote;


#[proc_macro_derive(Component)]
pub fn derive_component(input : TokenStream1) -> TokenStream1 {
    let DeriveInput {
        ident,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!{
        impl #impl_generics ::axecs::component::Component for #ident #ty_generics #where_clause { }
    }.into()
}


#[proc_macro_derive(Resource)]
pub fn derive_resource(input : TokenStream1) -> TokenStream1 {
    let DeriveInput {
        ident,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!{
        impl #impl_generics ::axecs::resource::Resource for #ident #ty_generics #where_clause { }
    }.into()
}
