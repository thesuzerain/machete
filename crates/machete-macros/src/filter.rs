use quote::quote;
use syn::{parenthesized, parse_macro_input, Data, DeriveInput, Ident};

pub fn derive_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    }: DeriveInput = parse_macro_input!(input as DeriveInput);

    // Only generate code for struct.
    if let Data::Struct(data) = data {

        let mut default = None;

        // String fields (comparable using contains, etc
        let mut string_names = Vec::new();
        // Number fields (comparable using greater than, less than, etc)
        let mut number_names = Vec::new();

        // Enum fields (comparable using variants)
        let mut variant_field_name = vec![];
        let mut variant_enum_name = vec![];

        data.fields.iter().for_each(|field| {
            field.attrs.iter().for_each(|attr| {
                if attr.path().is_ident("filter") {
                    attr.parse_nested_meta(|meta| {
                        // #[filter(default)]
                        if meta.path.is_ident("default") {
                            default = Some(field.ident.as_ref().expect("Field must have a name").clone());
                            return Ok(());
                        }

                        // #[filter(string)]
                        if meta.path.is_ident("string") {
                            string_names.push(field.ident.as_ref().expect("Field must have a name").to_string());
                            return Ok(());
                        }

                        // #[filter(number)]
                        if meta.path.is_ident("number") {
                            number_names.push(field.ident.as_ref().expect("Field must have a name").to_string());
                            return Ok(());
                        }

                        // #[filter(variant(Enum)]
                        if meta.path.is_ident("iter") {
                            let content;
                            parenthesized!(content in meta.input);
                            let enum_name: Ident = content.parse()?;
                            variant_field_name.push(field.ident.as_ref().expect("Field must have a name").to_string());
                            variant_enum_name.push(enum_name);
                            return Ok(());
                        }
                        Err(meta.error("Identifier not recognized. Allowed filter identifiers are `string`, `number`, and `variants = Enum`"))
                        // TOOD: better error
                    }).expect("Error parsing filter attribute");
                }
            });

        });

        let default_field_name = default.clone().expect("No default field found");
        let mut default_field_type = None;
        if string_names.iter().any(|x| x == &default_field_name.to_string()) {
            default_field_type = Some(quote! {
                machete_core::filters::FilterType::Contains("".to_string())
            })
        }
        if number_names.iter().any(|x| x == &default_field_name.to_string()) {
            default_field_type = Some(quote! {
                machete_core::filters::FilterType::EqualToNumber(0.0)
            })
        }
        if variant_field_name.iter().any(|x| x == &default_field_name.to_string()) {
            default_field_type = Some(quote! {
                machete_core::filters::FilterType::EqualToChoice("".to_string())
            })
        }
        let default_field_type = default_field_type.expect("Default field needs a type");
        let where_clause = &generics.where_clause;

        quote! {
            impl #generics machete_core::filters::FilterableStruct for #ident #generics #where_clause {

                fn create_default_filter() -> machete_core::filters::Filter<Self> {
                    machete_core::filters::Filter {
                        id: machete_core::ids::InternalId::new(),
                        field: stringify!(#default).to_string(),
                        // TODO: not just name contains
                        filter_type: #default_field_type,
                        _phantom: std::marker::PhantomData,
                    }
                }

                fn iter_filter_variants_for_field(field: &str) -> Option<Vec<String>> {
                    match field {
                        #(
                            #variant_field_name => Some(#variant_enum_name::iter().map(|r| r.to_string()).collect()),
                        )*
                        _ => None,
                    }
                }

                fn iter_fields() -> Vec<&'static str> {
                    vec![
                        #(
                            #string_names,
                        )*
                        #(
                            #number_names,
                        )*
                        #(
                            #variant_field_name,
                        )*
                    ]
                }

                // todo: should these be returning result instead?
                fn iter_filter_types_for_field(field: &str) -> Option<Vec<machete_core::filters::FilterType>> {
                    use machete_core::filters::FilterType;


                    let mut filter_types = Vec::new();

                    // Add string filters
                    match field {
                        #(
                            #string_names => {
                                filter_types.push(machete_core::filters::FilterType::Contains("".to_string()));
                            }
                        )*
                        _ => {}
                    }

                    // Add number filters
                    match field {
                        #(
                            #number_names => {
                                filter_types.push(machete_core::filters::FilterType::GreaterThan(0.0));
                                filter_types.push(machete_core::filters::FilterType::LessThan(0.0));
                                filter_types.push(machete_core::filters::FilterType::EqualToNumber(0.0));
                            }
                        )*
                        _ => {}
                    }

                    // Add enum filters
                    match field {
                        #(
                            #variant_field_name => {
                                let s = Self::iter_filter_variants_for_field(field).unwrap().first().unwrap().to_string();
                                filter_types.push(machete_core::filters::FilterType::EqualToChoice(s));
                            }
                        )*
                        _ => {}
                    }

                    Some(filter_types)
                }

            }
        }

    } else {
      quote! {}
    }
    .into()
}
