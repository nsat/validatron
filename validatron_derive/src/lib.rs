#![recursion_limit = "512"]

extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_derive(Validate, attributes(validatron))]
pub fn validatron_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_validatron(&ast).into()
}

fn build_named(name: &str, error: TokenStream) -> TokenStream {
    quote! {
        eb.at_named(#name, #error);
    }
}

fn lit_to_path(lit: &syn::Lit) -> syn::Path {
    match lit {
        syn::Lit::Str(s) => syn::parse_str(&s.value()).unwrap(),
        _ => panic!("invalid literal"),
    }
}

fn gen_type_check(mvn: &syn::MetaNameValue) -> TokenStream {
    let name = mvn.path.get_ident().unwrap().to_string();

    let lit = &mvn.lit;

    let func = match name.as_str() {
        "function" => {
            let custom_func = lit_to_path(&lit);
            build_named(
                &quote! {#lit}.to_string(),
                quote! {
                    #custom_func(&self)
                },
            )
        }
        _ => panic!("Unknown validator '{}'", name),
    };

    func
}

fn get_field_validator(meta: &syn::Meta, field: &TokenStream) -> TokenStream {
    match meta {
        syn::Meta::Path(path) => {
            let name = path.get_ident().unwrap().to_string();

            match name.as_str() {
                "required" => quote! {
                    ::validatron::validators::is_required(&self.#field)
                },
                _ => panic!("Unknown validator '{}'", name),
            }
        }
        syn::Meta::List(_) => panic!("not currently supported"),
        syn::Meta::NameValue(mnv) => {
            let name = mnv.path.get_ident().unwrap().to_string();

            let lit = &mnv.lit;

            match name.as_str() {
                "function" => {
                    let custom_func = lit_to_path(&lit);
                    quote! {
                        #custom_func(&self.#field)
                    }
                }
                "min" => quote! {
                    ::validatron::validators::min(&self.#field, #lit)
                },
                "max" => quote! {
                    ::validatron::validators::max(&self.#field, #lit)
                },
                "equal" => quote! {
                    ::validatron::validators::is_equal(&self.#field, #lit)
                },
                "min_len" => quote! {
                    ::validatron::validators::is_min_length(&self.#field, #lit)
                },
                "max_len" => quote! {
                    ::validatron::validators::is_max_length(&self.#field, #lit)
                },
                _ => panic!("Unknown validator '{}'", name),
            }
        }
    }
}

// such as #[validatron(function="validate_my_struct")]
fn build_type_validator(ast: &syn::DeriveInput) -> Vec<TokenStream> {
    let mut type_validators = vec![];
    for attr in ast.attrs.iter().filter(|x| x.path.is_ident("validatron")) {
        let meta = attr.parse_meta().unwrap();

        if let syn::Meta::List(list) = meta {
            for item in list.nested.iter() {
                if let syn::NestedMeta::Meta(meta) = item {
                    if let syn::Meta::NameValue(mnv) = meta {
                        type_validators.push(gen_type_check(&mnv));
                    }
                }
            }
        }
    }

    type_validators
}

fn build_field_validators(fields: &syn::Fields) -> Vec<TokenStream> {
    // we split these out so we that we only recurse after we have completed all other
    // validation tasks for a given struct
    let mut nested_field_validators = vec![];
    let mut custom_field_validators = vec![];

    for (i, field) in fields.iter().enumerate() {
        // check for and iterate over #[validatron] directives
        for attr in field.attrs.iter().filter(|x| x.path.is_ident("validatron")) {
            let meta = attr.parse_meta().unwrap();

            let ident = field
                .ident
                .as_ref()
                .map(|name| quote! {#name})
                .unwrap_or_else(|| {
                    let i = syn::Index::from(i);
                    quote! {#i}
                });

            let push = |func: TokenStream| {
                if let Some(name) = &field.ident {
                    let name = name.to_string();
                    quote! {
                        eb.at_named(#name, #func);
                    }
                } else {
                    quote! {
                        eb.at_index(#i, #func);
                    }
                }
            };

            match meta {
                // #[validatron]
                syn::Meta::Path(_) => {
                    let f = quote! { self.#ident.validate() };
                    nested_field_validators.push(push(f))
                }
                // #[validatron(...)]
                syn::Meta::List(list) => {
                    for item in list.nested.iter() {
                        if let syn::NestedMeta::Meta(meta) = item {
                            let validator = get_field_validator(&meta, &ident);

                            custom_field_validators.push(push(validator))
                        }
                    }
                }
                _ => panic!("argument not supported"),
            }
        }
    }

    custom_field_validators.extend(nested_field_validators.into_iter());

    custom_field_validators
}

fn iflet_members(fields: &syn::Fields) -> Vec<TokenStream> {
    let mut tokens = Vec::new();

    match fields {
        syn::Fields::Named(fields) => {
            for field in &fields.named {
                let fi = field.ident.as_ref().unwrap();
                tokens.push(quote! {
                    #fi
                });
            }
        }
        syn::Fields::Unnamed(fields) => {
            for (i, _field) in fields.unnamed.iter().enumerate() {
                let arg_name = syn::Ident::new(&format!("_{}", i), proc_macro2::Span::call_site());
                tokens.push(quote! {
                    #arg_name
                });
            }
        }
        syn::Fields::Unit => unreachable!(),
    }

    tokens
}

fn escape_iflet_members(fields: &syn::Fields, tokens: &[TokenStream]) -> TokenStream {
    match fields {
        syn::Fields::Named(_) => quote! {
            {#(#tokens),*}
        },
        syn::Fields::Unnamed(_) => quote! {
            (#(#tokens),*)
        },
        syn::Fields::Unit => unreachable!(),
    }
}

fn validate_variant_members(tokens: &[TokenStream]) -> Vec<TokenStream> {
    tokens
        .iter()
        .map(|x| {
            let variant_loc = x.to_string();
            quote! { eb.at_named(#variant_loc, #x.validate()); }
        })
        .collect()
}

fn build_enum_variant_validator(de: &syn::DataEnum) -> Vec<TokenStream> {
    let mut tokens = Vec::new();

    for var in &de.variants {
        for attr in var.attrs.iter().filter(|x| x.path.is_ident("validatron")) {
            let meta = attr.parse_meta().unwrap();

            let variant = var.ident.clone();

            let fields = iflet_members(&var.fields);
            let escaped = escape_iflet_members(&var.fields, &fields);

            match meta {
                // #[validatron]
                syn::Meta::Path(_) => {
                    let validators = validate_variant_members(&fields);
                    tokens.push(quote! {
                        if let Self::#variant#escaped = &self {
                            #(#validators)*
                        }
                    });
                }
                // #[validatron(...)]
                // currently only "function" is supported
                syn::Meta::List(list) => {
                    for item in list.nested {
                        if let syn::NestedMeta::Meta(meta) = item {
                            // such as #[validatron(min=1)]
                            if let syn::Meta::NameValue(mnv) = meta {
                                if mnv.path.get_ident().unwrap() == "function" {
                                    let path = lit_to_path(&mnv.lit);

                                    let var_name = variant.to_string();

                                    tokens.push(quote! {
                                        if let Self::#variant#escaped = &self {
                                            eb.at_named(
                                                #var_name,
                                                #path(#(&#fields),*)
                                            );
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    tokens
}

fn impl_validatron(ast: &syn::DeriveInput) -> TokenStream {
    let type_validators = build_type_validator(&ast);

    let validators = match &ast.data {
        syn::Data::Struct(ds) => build_field_validators(&ds.fields),
        syn::Data::Enum(de) => build_enum_variant_validator(&de),
        syn::Data::Union(_) => panic!("Union types are not supported"),
    };

    let derive_target = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        const _: () = {
            extern crate validatron;

            impl #impl_generics ::validatron::Validate for #derive_target #ty_generics #where_clause {
                fn validate(&self) -> ::validatron::Result<()> {
                    use ::validatron::{Location, Error, ErrorBuilder};
                    let mut eb = ErrorBuilder::new();

                    #(#validators)*

                    #(#type_validators)*

                    eb.build()
                }
            }
        };
    };

    expanded
}
