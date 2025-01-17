use syn::{parse::Parse, Attribute};

pub fn parse_attr(attrs: &[Attribute]) -> Option<(String, String)> {
    for attr in attrs {
        if attr.path().is_ident("serde") {
            struct SerdeAttr {
                tag: Option<String>,
                content: Option<String>,
            }

            impl Parse for SerdeAttr {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    let tag_ident: syn::Ident = input.parse()?;
                    if tag_ident != "tag" {
                        return Err(syn::Error::new(tag_ident.span(), "expected `tag`"));
                    }
                    input.parse::<syn::Token![=]>()?;
                    let tag = input.parse::<syn::LitStr>()?.value();
                    input.parse::<syn::Token![,]>()?;
                    let content_ident: syn::Ident = input.parse()?;
                    if content_ident != "content" {
                        return Err(syn::Error::new(content_ident.span(), "expected `content`"));
                    }
                    input.parse::<syn::Token![=]>()?;
                    let content = input.parse::<syn::LitStr>()?.value();
                    Ok(SerdeAttr {
                        tag: Some(tag),
                        content: Some(content),
                    })
                }
            }

            // Try parsing #[serde(tag = "type", content = "data")]
            if let Ok(args) = attr.parse_args::<SerdeAttr>() {
                return Some((args.tag.unwrap(), args.content.unwrap()));
            }
        }
    }
    None
}
