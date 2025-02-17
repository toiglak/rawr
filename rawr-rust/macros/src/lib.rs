use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, GenericParam, TypeParam};

mod serde;

#[proc_macro_derive(Schema)]
pub fn derive_schema(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let generics = &ast.generics;

    let expanded = match &ast.data {
        Data::Struct(data_struct) => generate_struct_schema(name, data_struct, generics),
        Data::Enum(data_enum) => generate_enum_schema(name, data_enum, &ast.attrs, generics),
        Data::Union(_) => panic!("Unions are not supported"),
    };

    ::proc_macro::TokenStream::from(expanded)
}

fn generate_struct_schema(
    name: &syn::Ident,
    data: &syn::DataStruct,
    generics: &syn::Generics,
) -> proc_macro2::TokenStream {
    let generics = add_schema_bound(generics);
    let generic_field = generate_generic_field(&generics, name);

    match &data.fields {
        Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap().to_string();
                let ty = &f.ty;
                quote! {
                    ::rawr::FieldDef {
                        name: #name,
                        schema: ::rawr::SchemaPtr(<#ty as ::rawr::Schema>::schema),
                    }
                }
            });

            let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

            quote! {
                impl #impl_generics ::rawr::Schema for #name #type_generics #where_clause {
                    fn schema() -> ::rawr::SchemaDef {
                        ::rawr::SchemaDef::Struct(::rawr::StructDef {
                            name: stringify!(#name),
                            module_path: ::core::module_path!(),
                            shape: ::rawr::Shape::Map(&[
                                #( #fields ),*
                            ]),
                            generic: #generic_field,
                        })
                    }
                }
            }
        }
        Fields::Unnamed(fields_unnamed) => {
            let schemas = fields_unnamed.unnamed.iter().map(|f| {
                let ty = &f.ty;
                quote! { ::rawr::SchemaPtr(<#ty as ::rawr::Schema>::schema) }
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

            let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

            quote! {
                impl #impl_generics ::rawr::Schema for #name #type_generics #where_clause {
                    fn schema() -> ::rawr::SchemaDef {
                        ::rawr::SchemaDef::Struct(::rawr::StructDef {
                            name: stringify!(#name),
                            module_path: ::core::module_path!(),
                            shape: #shape,
                            generic: #generic_field,
                        })
                    }
                }
            }
        }
        Fields::Unit => {
            let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

            quote! {
                impl #impl_generics ::rawr::Schema for #name #type_generics #where_clause {
                    fn schema() -> ::rawr::SchemaDef {
                        ::rawr::SchemaDef::Struct(::rawr::StructDef {
                            name: stringify!(#name),
                            module_path: ::core::module_path!(),
                            shape: ::rawr::Shape::Unit,
                            generic: #generic_field,
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
    generics: &syn::Generics,
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
                            schema: ::rawr::SchemaPtr(<#ty as ::rawr::Schema>::schema),
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
                    quote! { ::rawr::SchemaPtr(<#ty as ::rawr::Schema>::schema) }
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

    let generics = add_schema_bound(generics);
    let generic_field = generate_generic_field(&generics, name);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::rawr::Schema for #name #type_generics #where_clause {
            fn schema() -> ::rawr::SchemaDef {
                ::rawr::SchemaDef::Enum(::rawr::EnumDef {
                    name: stringify!(#name),
                    module_path: ::core::module_path!(),
                    representation: #rep,
                    variants: &[
                        #( #variants_iter ),*
                    ],
                    generic: #generic_field,
                })
            }
        }
    }
}

fn generate_generic_field(generics: &syn::Generics, name: &syn::Ident) -> proc_macro2::TokenStream {
    if generics.params.is_empty() {
        return quote! { ::core::option::Option::None };
    }

    let params = generics.params.iter().filter_map(|param| {
        if let GenericParam::Type(TypeParam { ident, .. }) = param {
            Some(quote! {
                ::rawr::SchemaPtr(<#ident as ::rawr::Schema>::schema)
            })
        } else {
            None
        }
    });

    // Create synthetic identifiers for the generic parameters
    let param_idents: Vec<_> = generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(TypeParam { ident, .. }) = param {
                Some(ident)
            } else {
                None
            }
        })
        .collect();

    // Generate the generic schema definition
    quote! {
        Some(::rawr::GenericDef {
            params: &[#(#params),*],
            schema: ::rawr::SchemaPtr(|| {
                #(
                    struct #param_idents;
                    impl ::rawr::Schema for #param_idents {
                        fn schema() -> ::rawr::SchemaDef {
                            ::rawr::SchemaDef::GenericParameter(stringify!(#param_idents))
                        }
                    }
                )*
                <#name<#(#param_idents),*> as ::rawr::Schema>::schema()
            }),
        })
    }
}

/// Adds `Schema` bound to all generic parameters.
fn add_schema_bound(generics: &syn::Generics) -> syn::Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(::rawr::Schema));
        }
    }
    generics
}
