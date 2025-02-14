use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields};

mod serde;

#[proc_macro_derive(Schema)]
pub fn derive_schema(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = match &ast.data {
        Data::Struct(data_struct) => generate_struct_schema(name, data_struct),
        Data::Enum(data_enum) => generate_enum_schema(name, data_enum, &ast.attrs),
        Data::Union(_) => panic!("Unions are not supported"),
    };

    ::proc_macro::TokenStream::from(expanded)
}

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
                            shape: ::rawr::Shape::Map(&[
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

            let shape = match fields_unnamed.unnamed.len() {
                1 => {
                    quote! {
                        ::rawr::Shape::Newtype(
                            {
                                #( #schemas )*
                            }
                        )
                    }
                }
                _ => {
                    quote! {
                        ::rawr::Shape::Tuple(&[
                            #( #schemas ),*
                        ])
                    }
                }
            };

            quote! {
                impl ::rawr::Schema for #name {
                    fn schema() -> ::rawr::SchemaDef {
                        ::rawr::SchemaDef::Struct(::rawr::StructDef {
                            name: stringify!(#name),
                            module_path: ::core::module_path!(),
                            shape: #shape,
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
                            shape: ::rawr::Shape::Unit,
                        })
                    }
                }
            }
        }
    }
}

fn generate_enum_schema(
    name: &syn::Ident,
    data: &syn::DataEnum,
    attrs: &[Attribute],
) -> proc_macro2::TokenStream {
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
                    ::rawr::VariantDef {
                        name: #variant_str,
                        shape: ::rawr::Shape::Map(&[
                            #( #fields_iter ),*
                        ]),
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let fields_iter = unnamed.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote! { <#ty as ::rawr::Schema>::schema }
                });

                let shape = match unnamed.unnamed.len() {
                    1 => {
                        quote! {
                            ::rawr::Shape::Newtype(
                                {
                                    #( #fields_iter )*
                                }
                            )
                        }
                    }
                    _ => {
                        quote! {
                            ::rawr::Shape::Tuple(&[
                                #( #fields_iter ),*
                            ])
                        }
                    }
                };

                quote! {
                    ::rawr::VariantDef {
                        name: #variant_str,
                        shape: #shape,
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    ::rawr::VariantDef {
                        name: #variant_str,
                        shape: ::rawr::Shape::Unit,
                    }
                }
            }
        }
    });

    // NOTE: I was thinking about including all attributes as string literals and
    // letting codegen to decide how to interpret them. This would allow downstream
    // code generators to be maximally flexible, as they would not depend on rawr
    // on supporting all libraries they're interested in.
    //
    // We could expose it as `Attribute { meta: &str, value: &str }` or similar. We
    // could even provide default implementations for common libraries like serde.

    // If no serde attributes are found, it's an externally tagged enum
    let rep = match serde::parse_attr(attrs) {
        Some((tag, content)) => {
            quote! {
                ::rawr::EnumRepr::Adjacent {
                    tag: #tag,
                    content: #content,
                }
            }
        }
        None => {
            quote! {
                ::rawr::EnumRepr::External
            }
        }
    };

    quote! {
        impl ::rawr::Schema for #name {
            fn schema() -> ::rawr::SchemaDef {
                ::rawr::SchemaDef::Enum(::rawr::EnumDef {
                    name: stringify!(#name),
                    module_path: ::core::module_path!(),
                    representation: #rep,
                    variants: &[
                        #( #variants_iter ),*
                    ],
                })
            }
        }
    }
}
