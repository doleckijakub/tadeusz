use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Type, parse_macro_input};

struct ToolArgs {
    name: String,
    description: String,
}

impl syn::parse::Parse for ToolArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name: Option<(String, proc_macro2::Span)> = None;
        let mut description: Option<(String, proc_macro2::Span)> = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let value: LitStr = input.parse()?;

            match key.to_string().as_str() {
                "name" => name = Some((value.value(), key.span())),
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

        let name = name
            .ok_or_else(|| input.error("missing `name` in #[tool(...)]"))?
            .0;
        let description = description
            .ok_or_else(|| input.error("missing `description` in #[tool(...)]"))?
            .0;

        Ok(ToolArgs { name, description })
    }
}

struct FieldInfo {
    ident: String,
    description: Option<String>,
    required: bool,
    json_type: &'static str,
}

fn rust_type_to_json_schema(ty: &Type) -> &'static str {
    let Type::Path(tp) = ty else { return "string" };
    let Some(seg) = tp.path.segments.last() else {
        return "string";
    };

    if seg.ident == "Option"
        && let syn::PathArguments::AngleBracketed(ab) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = ab.args.first()
    {
        return rust_type_to_json_schema(inner);
    }

    match seg.ident.to_string().as_str() {
        "String" | "str" => "string",
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "isize"
        | "usize" => "integer",
        "f32" | "f64" => "number",
        "bool" => "boolean",
        "Vec" => "array",
        _ => "object",
    }
}

fn derive_tool_inner(input: DeriveInput) -> syn::Result<TokenStream2> {
    let tool_attr = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("tool"))
        .ok_or_else(|| {
            Error::new_spanned(
                &input,
                "#[derive(Tool)] requires a #[tool(name = \"...\", description = \"...\")] attribute",
            )
        })?;

    let ToolArgs { name, description } = tool_attr.parse_args::<ToolArgs>()?;

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

        let json_type = rust_type_to_json_schema(&field.ty);
        let mut field_desc: Option<String> = None;
        let mut required = false;

        for attr in &field.attrs {
            if attr.path().is_ident("required") {
                attr.meta.require_path_only().map_err(|_| {
                    Error::new_spanned(attr, "#[required] does not take any arguments")
                })?;
                required = true;
            } else if attr.path().is_ident("description") {
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
            json_type,
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
            let json_type = f.json_type;
            match &f.description {
                Some(desc) => quote! {
                    properties.insert(
                        #field_name.to_string(),
                        ::serde_json::json!({ "type": #json_type, "description": #desc }),
                    );
                },
                None => quote! {
                    properties.insert(
                        #field_name.to_string(),
                        ::serde_json::json!({ "type": #json_type }),
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
            ::tool::registry::ToolRegistration(|| {
                ::std::boxed::Box::new(
                    ::tool::registry::TypedTool::<#struct_name>::default()
                )
            })
        }
    }
}

#[proc_macro_derive(Tool, attributes(tool, required, description))]
pub fn derive_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_tool_inner(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
