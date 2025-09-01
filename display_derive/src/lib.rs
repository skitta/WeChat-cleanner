use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Attribute, Meta};

/// Derive macro for implementing Display trait
/// 
/// Usage:
/// ```
/// #[derive(Display)]
/// pub struct MyStruct {
///     #[display(summary, name="自定义名称")]
///     pub field1: u64,
///     #[display(details)]
///     pub field2: String,
/// }
/// ```
#[proc_macro_derive(Display, attributes(display))]
pub fn derive_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Display can only be derived for structs with named fields"),
        },
        _ => panic!("Display can only be derived for structs"),
    };

    let summary_fields = generate_field_display(fields, DisplayMode::Summary);
    let details_fields = generate_field_display(fields, DisplayMode::Details);

    let expanded = quote! {
        impl #impl_generics display_core::Display for #name #ty_generics #where_clause {
            fn display_summary(&self) -> String {
                let mut result = Vec::new();
                #(#summary_fields)*
                if result.is_empty() {
                    format!("{}(no summary fields)", stringify!(#name))
                } else {
                    result.join("\n")
                }
            }

            fn display_details(&self) -> String {
                let mut result = Vec::new();
                #(#details_fields)*
                if result.is_empty() {
                    format!("{}(no detail fields)", stringify!(#name))
                } else {
                    result.join("\n")
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[derive(PartialEq)]
enum DisplayMode {
    Summary,
    Details,
}

fn generate_field_display(
    fields: &syn::punctuated::Punctuated<Field, syn::Token![,]>,
    mode: DisplayMode,
) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let field_type = &field.ty;
            
            let display_attr = parse_display_attribute(&field.attrs)?;
            
            // 检查字段是否应该在当前模式下显示
            let should_display = match mode {
                DisplayMode::Summary => {
                    display_attr.summary || (display_attr.name.is_some() && !display_attr.details_only)
                }
                DisplayMode::Details => {
                    display_attr.details || display_attr.summary || display_attr.name.is_some()
                }
            };
            
            if !should_display {
                return None;
            }
            
            let display_name = display_attr.name
                .unwrap_or_else(|| field_name.to_string());
                
            let format_method = match mode {
                DisplayMode::Summary => quote! { format_display_summary },
                DisplayMode::Details => quote! { format_display_details },
            };
            
            Some(quote! {
                result.push(format!("{}: {}", #display_name,
                    <#field_type as display_core::DisplayValue>::#format_method(&self.#field_name)));
            })
        })
        .collect()
}

#[derive(Default)]
struct DisplayAttribute {
    summary: bool,
    details: bool,
    details_only: bool,
    name: Option<String>,
}

fn parse_display_attribute(attrs: &[Attribute]) -> Option<DisplayAttribute> {
    let mut display_attr = DisplayAttribute::default();
    let mut found_display = false;
    
    for attr in attrs {
        if attr.path().is_ident("display") {
            found_display = true;
            
            match &attr.meta {
                Meta::List(meta_list) => {
                    let tokens = &meta_list.tokens;
                    let content = tokens.to_string();
                    
                    // 解析逗号分隔的参数
                    for part in content.split(',') {
                        let part = part.trim();
                        
                        if part == "summary" {
                            display_attr.summary = true;
                        } else if part == "details" {
                            display_attr.details = true;
                            display_attr.details_only = !display_attr.summary;
                        } else if part.starts_with("name=") || part.starts_with("name =") {
                            // 解析 name="value" 或 name = "value" 格式
                            let name_value = if part.starts_with("name =") {
                                part.strip_prefix("name =")
                            } else {
                                part.strip_prefix("name=")
                            };
                            
                            if let Some(name_value) = name_value {
                                let name_value = name_value.trim();
                                if name_value.starts_with('"') && name_value.ends_with('"') {
                                    let extracted_name = &name_value[1..name_value.len()-1];
                                    display_attr.name = Some(extracted_name.to_string());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    if found_display {
        Some(display_attr)
    } else {
        None
    }
}