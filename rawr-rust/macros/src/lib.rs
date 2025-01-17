use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

fn generate_struct_schema(name: &syn::Ident, data: &syn::DataStruct) -> proc_macro2::TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap().to_string();
                let ty = &f.ty;
                quote! {
                    ::rawr::FieldDef {
                        name: #name,
                        schema: <#ty as ::rawr::Schema>::schema,
                    }
                }
            });
            quote! {
                impl ::rawr::Schema for #name {
                    fn schema() -> ::rawr::SchemaDef {
                        ::rawr::SchemaDef::Struct(::rawr::StructDef {
                            name: stringify!(#name),
                            module_path: ::core::module_path!(),
                            fields: ::rawr::Fields::Named(&[
                                #( #fields ),*
                            ]),
                        })
                    }
                }
            }
        }
        Fields::Unnamed(fields_unnamed) => {
            let schemas = fields_unnamed.unnamed.iter().map(|f| {
                let ty = &f.ty;
                quote! { <#ty as ::rawr::Schema>::schema }
            });
            quote! {
                impl ::rawr::Schema for #name {
                    fn schema() -> ::rawr::SchemaDef {
                        ::rawr::SchemaDef::Struct(::rawr::StructDef {
                            name: stringify!(#name),
                            module_path: ::core::module_path!(),
                            fields: ::rawr::Fields::Unnamed(&[
                                #( #schemas ),*
                            ]),
                        })
                    }
                }
            }
        }
        Fields::Unit => {
            quote! {
                impl ::rawr::Schema for #name {
                    fn schema() -> ::rawr::SchemaDef {
                        ::rawr::SchemaDef::Struct(::rawr::StructDef {
                            name: stringify!(#name),
                            module_path: ::core::module_path!(),
                            fields: ::rawr::Fields::Unit,
                        })
                    }
                }
            }
        }
    }
}

fn generate_enum_schema(name: &syn::Ident, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let variants_iter = data.variants.iter().map(|v| {
        let variant_name = &v.ident;
        let variant_str = variant_name.to_string();
        match &v.fields {
            Fields::Named(named) => {
                let fields_iter = named.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap().to_string();
                    let ty = &field.ty;
                    quote! {
                        ::rawr::FieldDef {
                            name: #name,
                            schema: <#ty as ::rawr::Schema>::schema,
                        }
                    }
                });
                quote! {
                    ::rawr::EnumVariant::Struct {
                        name: #variant_str,
                        fields: &[
                            #( #fields_iter ),*
                        ],
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let fields_iter = unnamed.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote! { <#ty as ::rawr::Schema>::schema }
                });
                quote! {
                    ::rawr::EnumVariant::Tuple {
                        name: #variant_str,
                        fields: &[
                            #( #fields_iter ),*
                        ],
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    ::rawr::EnumVariant::Unit {
                        name: #variant_str,
                    }
                }
            }
        }
    });
    quote! {
        impl ::rawr::Schema for #name {
            fn schema() -> ::rawr::SchemaDef {
                ::rawr::SchemaDef::Enum(::rawr::EnumDef {
                    name: stringify!(#name),
                    module_path: ::core::module_path!(),
                    representation: ::rawr::EnumRepresentation::Adjacent {
                        tag: "type",
                        content: "value",
                    },
                    variants: &[
                        #( #variants_iter ),*
                    ],
                })
            }
        }
    }
}

#[proc_macro_derive(Schema)]
pub fn derive_schema(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = match &ast.data {
        Data::Struct(data_struct) => generate_struct_schema(name, data_struct),
        Data::Enum(data_enum) => generate_enum_schema(name, data_enum),
        Data::Union(_) => panic!("Unions are not supported"),
    };

    ::proc_macro::TokenStream::from(expanded)
}
