use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Type, parse_macro_input};

struct ToolArgs {
    description: String,
}

impl syn::parse::Parse for ToolArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut description: Option<(String, proc_macro2::Span)> = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let value: LitStr = input.parse()?;

            match key.to_string().as_str() {
                "description" => description = Some((value.value(), key.span())),
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!("unknown key `{other}` in #[tool(...)]"),
                    ));
                }
            }

            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        let description = description
            .ok_or_else(|| input.error("missing `description` in #[tool(...)]"))?
            .0;

        Ok(ToolArgs { description })
    }
}

struct FieldInfo {
    ident: String,
    description: Option<String>,
    required: bool,
    field_type: TokenStream2,
}

fn is_option_type(ty: &Type) -> bool {
    let Type::Path(tp) = ty else { return false };
    let Some(seg) = tp.path.segments.last() else {
        return false;
    };
    seg.ident == "Option"
}

fn derive_tool_inner(input: DeriveInput) -> syn::Result<TokenStream2> {
    let tool_attr = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("tool"))
        .ok_or_else(|| {
            Error::new_spanned(
                &input,
                "#[derive(Tool)] requires a #[tool(description = \"...\")] attribute",
            )
        })?;

    let ToolArgs { description } = tool_attr.parse_args::<ToolArgs>()?;
    let name = input.ident.to_string();

    let named_fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(nf) => &nf.named,
            Fields::Unit => return Ok(emit_impl(&input.ident, &name, &description, &[])),
            Fields::Unnamed(_) => {
                return Err(Error::new_spanned(
                    &input,
                    "#[derive(Tool)] does not support tuple structs",
                ));
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            return Err(Error::new_spanned(
                &input,
                "#[derive(Tool)] only supports structs",
            ));
        }
    };

    let mut fields: Vec<FieldInfo> = Vec::new();

    for field in named_fields {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(field, "expected a named field"))?
            .to_string();

        let required = !is_option_type(&field.ty);
        let ty = &field.ty;
        let field_type = quote! { #ty };
        let mut field_desc: Option<String> = None;

        for attr in &field.attrs {
            if attr.path().is_ident("description") {
                let lit: LitStr = attr.parse_args().map_err(|e| {
                    Error::new_spanned(
                        attr,
                        format!("#[description(...)] expects a string literal — {e}"),
                    )
                })?;
                field_desc = Some(lit.value());
            }
        }

        fields.push(FieldInfo {
            ident,
            description: field_desc,
            required,
            field_type,
        });
    }

    Ok(emit_impl(&input.ident, &name, &description, &fields))
}

fn emit_impl(
    struct_name: &syn::Ident,
    name: &str,
    description: &str,
    fields: &[FieldInfo],
) -> TokenStream2 {
    let property_inserts: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let field_name = &f.ident;
            let field_type = &f.field_type;
            match &f.description {
                Some(desc) => quote! {
                    {
                        let mut schema = <#field_type as ::tool::ToolFieldSchema>::field_schema();
                        if let Some(obj) = schema.as_object_mut() {
                            obj.insert("description".to_string(), ::serde_json::json!(#desc));
                        }
                        properties.insert(#field_name.to_string(), schema);
                    }
                },
                None => quote! {
                    properties.insert(
                        #field_name.to_string(),
                        <#field_type as ::tool::ToolFieldSchema>::field_schema(),
                    );
                },
            }
        })
        .collect();

    let required_names: Vec<&str> = fields
        .iter()
        .filter(|f| f.required)
        .map(|f| f.ident.as_str())
        .collect();

    quote! {
        impl ::tool::ToolSchema for #struct_name {
            fn name() -> &'static str {
                #name
            }

            fn description() -> &'static str {
                #description
            }

            fn parameters() -> ::serde_json::Value {
                let mut properties = ::serde_json::Map::new();
                #( #property_inserts )*
                ::serde_json::json!({
                    "type":       "object",
                    "properties": properties,
                    "required":   [ #( #required_names ),* ],
                })
            }
        }

        ::inventory::submit! {
            ::tool::registry::ToolRegistration(
                || {
                    ::std::boxed::Box::new(
                        ::tool::registry::TypedTool::<#struct_name>::default()
                    )
                },
                |args| {
                    ::serde_json::from_str::<#struct_name>(args)
                        .map_err(|e| format!("Serialization error: {e}"))
                        .map(|tool| {
                            ::std::boxed::Box::new(::tool::registry::PreparedTypedTool(tool))
                                as ::std::boxed::Box<dyn ::tool::registry::PreparedAnyTool>
                        })
                },
            )
        }
    }
}

#[proc_macro_derive(Tool, attributes(tool, description))]
pub fn derive_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_tool_inner(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn apply_rename_all(name: &str, rule: &str) -> String {
    match rule {
        "lowercase" => name.to_lowercase(),
        "UPPERCASE" => name.to_uppercase(),
        "camelCase" => {
            let mut chars = name.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_lowercase().to_string() + chars.as_str(),
            }
        }
        "snake_case" => to_snake_case(name),
        "SCREAMING_SNAKE_CASE" => to_snake_case(name).to_uppercase(),
        "kebab-case" => to_snake_case(name).replace('_', "-"),
        _ => name.to_string(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

fn derive_field_schema_inner(input: DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            &input,
            "#[derive(ToolFieldSchema)] only supports enums",
        ));
    };

    let mut rename_all: Option<String> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("serde") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    rename_all = Some(lit.value());
                }
                Ok(())
            });
        }
    }

    let variant_names: Vec<String> = data
        .variants
        .iter()
        .map(|variant| {
            let mut individual_rename: Option<String> = None;
            for attr in &variant.attrs {
                if attr.path().is_ident("serde") {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("rename") {
                            let value = meta.value()?;
                            let lit: LitStr = value.parse()?;
                            individual_rename = Some(lit.value());
                        }
                        Ok(())
                    });
                }
            }
            if let Some(renamed) = individual_rename {
                renamed
            } else {
                let raw = variant.ident.to_string();
                match rename_all.as_deref() {
                    Some(rule) => apply_rename_all(&raw, rule),
                    None => raw,
                }
            }
        })
        .collect();

    let enum_name = &input.ident;

    Ok(quote! {
        impl ::tool::ToolFieldSchema for #enum_name {
            fn field_schema() -> ::serde_json::Value {
                let mut m = ::serde_json::Map::new();
                m.insert("type".to_owned(), ::serde_json::Value::String("string".to_owned()));
                m.insert("enum".to_owned(), ::serde_json::Value::Array(vec![
                    #( ::serde_json::Value::String(#variant_names.to_owned()) ),*
                ]));
                ::serde_json::Value::Object(m)
            }
        }
    })
}

#[proc_macro_derive(ToolFieldSchema, attributes(serde))]
pub fn derive_tool_field_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_field_schema_inner(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
