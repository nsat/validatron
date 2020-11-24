#![recursion_limit = "512"]

extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_derive(Validate, attributes(validatron))]
pub fn validatron_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_validatron(&ast).into()
}

fn inner_expand(function: &syn::Path, arguments: &[TokenStream]) -> TokenStream {
    quote! {
        #function(#(#arguments),*)
    }
}

fn build_named(name: &str, error: TokenStream) -> TokenStream {
    quote! {
        eb.at_named(#name, #error);
    }
}

fn gen_custom_struct_validation(function: &syn::Path) -> TokenStream {
    let field_name = function.get_ident().unwrap().to_string();

    build_named(&field_name, inner_expand(function, &[quote!(&self)]))
}

fn gen_recurse(field: &syn::Ident) -> TokenStream {
    let field_name = field.to_string();

    build_named(&field_name, quote!(self.#field.validate()))
}

// something like #[validatron(required)]
fn gen_explicit_check(field: &syn::Ident, path: &syn::Path) -> TokenStream {
    let name = path.get_ident().unwrap().to_string();
    let func = match name.as_str() {
        "required" => quote! {
            ::validatron::validators::is_required(&self.#field)
        },
        _ => panic!("Unknown validator '{}'", name),
    };

    build_named(&field.to_string(), func)
}

fn lit_to_path(lit: &syn::Lit) -> syn::Path {
    match lit {
        syn::Lit::Str(s) => syn::parse_str(&s.value()).unwrap(),
        _ => panic!("invalid literal"),
    }
}

// something like #[validatron(min=1)]
fn gen_argument_check(field: &syn::Ident, mvn: &syn::MetaNameValue) -> TokenStream {
    let name = mvn.path.get_ident().unwrap().to_string();

    let lit = &mvn.lit;

    let func = match name.as_str() {
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
    };

    build_named(&field.to_string(), func)
}

fn build_type_validator(ast: &syn::DeriveInput) -> Vec<TokenStream> {
    let mut type_validators = vec![];
    for attr in ast.attrs.iter().filter(|x| x.path.is_ident("validatron")) {
        let meta = attr.parse_meta().unwrap();

        if let syn::Meta::List(list) = meta {
            for item in list.nested.iter() {
                if let syn::NestedMeta::Meta(meta) = item {
                    // such as #[validatron(function="validate_my_struct")] for a struct
                    if let syn::Meta::NameValue(mnv) = meta {
                        if mnv.path.is_ident("function") {
                            type_validators
                                .push(gen_custom_struct_validation(&lit_to_path(&mnv.lit)));
                        } else {
                            panic!("Unsupported param");
                        }
                    }
                }
            }
        }
    }

    type_validators
}

fn build_field_validators(ds: &syn::DataStruct) -> Vec<TokenStream> {
    // we split these out so we that we only recurse after we have completed all other
    // validation tasks for a given struct
    let mut nested_field_validators = vec![];
    let mut custom_field_validators = vec![];

    for field in &ds.fields {
        // check for and iterate over #[validatron] directives
        for attr in field.attrs.iter().filter(|x| x.path.is_ident("validatron")) {
            let meta = attr.parse_meta().unwrap();

            if let Some(field) = field.ident.as_ref() {
                match meta {
                    // #[validatron]
                    syn::Meta::Path(_) => {
                        nested_field_validators.push(gen_recurse(field));
                    }
                    // #[validatron(...)]
                    syn::Meta::List(list) => {
                        for item in list.nested.iter() {
                            if let syn::NestedMeta::Meta(meta) = item {
                                match meta {
                                    // such as #[validatron(required)]
                                    syn::Meta::Path(p) => {
                                        custom_field_validators
                                            .push(gen_explicit_check(&field, &p));
                                    }
                                    // such as #[validatron(min=1)]
                                    syn::Meta::NameValue(mnv) => {
                                        custom_field_validators
                                            .push(gen_argument_check(&field, &mnv));
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                    _ => (),
                }
            } else {
                println!("Doesn't current support anonymous fields")
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
        if let syn::Fields::Unit = var.fields {
            // Skipping unit type as there is nothing to validate

            continue;
        }

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

    let field_validators = if let syn::Data::Struct(ds) = &ast.data {
        build_field_validators(&ds)
    } else {
        vec![]
    };

    let variant_validators = if let syn::Data::Enum(de) = &ast.data {
        build_enum_variant_validator(&de)
    } else {
        vec![]
    };

    let derive_target = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::validatron::Validate for #derive_target #ty_generics #where_clause {
            fn validate(&self) -> ::validatron::Result<()> {
                use ::validatron::{Location, Error, ErrorBuilder};
                let mut eb = ErrorBuilder::new();

                #(#field_validators)*

                #(#variant_validators)*

                #(#type_validators)*

                eb.build()
            }
        }
    };

    expanded
}
