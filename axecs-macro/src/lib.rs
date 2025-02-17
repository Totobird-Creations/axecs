use proc_macro::TokenStream as TokenStream1;
use syn::{ parse_macro_input, DeriveInput, Data, DataStruct, DataEnum, DataUnion, Fields, FieldsNamed, FieldsUnnamed, Field, Index };
use syn::spanned::Spanned;
use quote::{ quote, quote_spanned };



#[proc_macro_derive(Component)]
pub fn derive_component(input : TokenStream1) -> TokenStream1 {
    let DeriveInput {
        ident,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!{
        impl #impl_generics axecs::component::Component for #ident #ty_generics #where_clause { }
    }.into()
}



#[proc_macro_derive(Bundle)]
pub fn derive_bundle(input : TokenStream1) -> TokenStream1 {
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match (data) {


        Data::Struct(DataStruct { fields, .. }) => {
            let mut type_infos  = Vec::with_capacity(fields.len());
            let mut field_names = Vec::with_capacity(fields.len());
            match (fields) {

                Fields::Named(FieldsNamed { named, .. }) => {
                    for Field { ident, ty, .. } in named {
                        let field_span = ty.span();
                        let ident      = ident.unwrap();
                        let ident_span = ident.span();
                        type_infos  .push(quote_spanned!{ field_span => #ty    });
                        field_names .push(quote_spanned!{ ident_span => #ident });
                    }
                },

                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                    for (i, Field { ty, .. }) in unnamed.iter().enumerate() {
                        let i          = Index::from(i);
                        let field_span = ty.span();
                        type_infos  .push(quote_spanned!{ field_span => #ty });
                        field_names .push(quote_spanned!{ field_span => #i  });
                    }
                },

                Fields::Unit => { }

            }
            quote!{
                unsafe impl #impl_generics axecs::component::bundle::ComponentBundle for #ident #ty_generics #where_clause {

                    fn type_info() -> Vec<axecs::component::ComponentTypeInfo> {
                        let mut ctis = Vec::new();
                        #( ctis.append(
                            &mut <(#type_infos) as axecs::component::bundle::ComponentBundle>
                                ::type_info()
                        ); )*
                        ctis
                    }

                    unsafe fn push_into(self, archetype : &mut axecs::component::archetype::Archetype) {
                        #( unsafe{
                            <(#type_infos) as axecs::component::bundle::ComponentBundle>
                                ::push_into((self.#field_names), archetype)
                        }; )*
                    }

                    unsafe fn write_into(self, archetype : &mut axecs::component::archetype::Archetype, row : usize) {
                        #( unsafe{
                            <(#type_infos) as axecs::component::bundle::ComponentBundle>
                                ::write_into((self.#field_names), archetype, row)
                        }; )*
                    }

                    fn validate() -> axecs::component::bundle::BundleValidator {
                        let mut bundle = axecs::component::bundle::BundleValidator::empty();
                        #( bundle = axecs::component::bundle::BundleValidator::join(bundle, unsafe{
                            <(#type_infos) as axecs::component::bundle::ComponentBundle>
                                ::validate()
                        }); )*
                        bundle
                    }

                }
            }
        },


        Data::Enum(DataEnum { enum_token, .. }) => {
            let span = enum_token.span;
            quote_spanned!{ span => compile_error!("`ComponentBundle` can not be derived for enums"); }
        }
        Data::Union(DataUnion { union_token, .. }) => {
            let span = union_token.span;
            quote_spanned!{ span => compile_error!("`ComponentBundle` can not be derived for unions"); }
        },

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
        impl #impl_generics axecs::resource::Resource for #ident #ty_generics #where_clause { }
    }.into()
}



#[proc_macro_derive(Label)]
pub fn derive_label(input : TokenStream1) -> TokenStream1 {
    let DeriveInput {
        ident,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote!{
        impl #impl_generics axecs::schedule::label::ScheduleLabel for #ident #ty_generics #where_clause { }
    }.into()
}
