#![recursion_limit = "512"]

extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[proc_macro_derive(Validate, attributes(validatron))]
pub fn validatron_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_validatron(&ast).into()
}

fn build_named(name: &str, error: TokenStream) -> TokenStream {
    quote! {
        eb.try_at_named(#name, #error);
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

fn get_field_validator(meta: &syn::Meta, target: &TokenStream) -> TokenStream {
    match meta {
        syn::Meta::Path(path) => {
            let name = path.get_ident().unwrap().to_string();

            match name.as_str() {
                "required" => quote! {
                    ::validatron::validators::is_required(#target)
                },
                _ => panic!("Unknown validator '{}'", name),
            }
        }
        syn::Meta::List(_) => panic!("not currently supported"),
        syn::Meta::NameValue(mnv) => {
            let name = mnv.path.get_ident().unwrap().to_string();

            // If a user provides a string literal we shall treat it as an expression
            // this makes our comparison operators much more flexible.
            let lit = if let syn::Lit::Str(lit) = &mnv.lit {
                let x = syn::parse_str::<syn::Expr>(&lit.value()).unwrap();

                x.to_token_stream()
            } else {
                mnv.lit.to_token_stream()
            };

            match name.as_str() {
                "function" => {
                    let custom_func = lit_to_path(&mnv.lit);
                    quote! {
                        #custom_func(#target)
                    }
                }
                "predicate" => {
                    let lit = &mnv.lit;
                    let custom_func = lit_to_path(&lit);
                    let err_msg = format!("Predicate {} failed", quote!(#lit));
                    quote! {
                        if #custom_func(#target) {
                            Ok(())
                        } else {
                            Err(::validatron::Error::new(#err_msg))
                        }
                    }
                }
                "min" => quote! {
                    ::validatron::validators::min(#target, #lit)
                },
                "option_min" => quote! {
                    ::validatron::validators::option_min(#target, #lit)
                },
                "max" => quote! {
                    ::validatron::validators::max(#target, #lit)
                },
                "option_max" => quote! {
                    ::validatron::validators::option_max(#target, #lit)
                },
                "equal" => quote! {
                    ::validatron::validators::is_equal(#target, #lit)
                },
                "min_len" => quote! {
                    ::validatron::validators::is_min_length(#target, #lit)
                },
                "max_len" => quote! {
                    ::validatron::validators::is_max_length(#target, #lit)
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

        use syn::{Meta, NestedMeta};

        if let Meta::List(list) = meta {
            for item in list.nested.iter() {
                if let NestedMeta::Meta(Meta::NameValue(mnv)) = item {
                    type_validators.push(gen_type_check(&mnv));
                }
            }
        }
    }

    type_validators
}

fn build_field_validators(
    fields: &syn::Fields,
    target_prefix: Option<TokenStream>,
    borrow_fields: bool,
) -> Vec<TokenStream> {
    // we split these out so we that we only recurse after we have completed all other
    // validation tasks for a given struct
    let mut nested_field_validators = vec![];
    let mut custom_field_validators = vec![];

    for (i, field) in fields.iter().enumerate() {
        // check for and iterate over #[validatron] directives
        for attr in field.attrs.iter().filter(|x| x.path.is_ident("validatron")) {
            let meta = attr.parse_meta().unwrap();

            let target = field
                .ident
                .as_ref()
                .map(|name| {
                    quote! {
                        #target_prefix#name
                    }
                })
                .unwrap_or_else(|| {
                    if target_prefix.is_some() {
                        let i = syn::Index::from(i);
                        quote! {#target_prefix#i}
                    } else {
                        let arg_name = syn::Ident::new(
                            &format!("_field{}", i),
                            proc_macro2::Span::call_site(),
                        );
                        quote!(#arg_name)
                    }
                });

            let push = |func: TokenStream| {
                if let Some(name) = &field.ident {
                    let name = name.to_string();
                    quote! {
                        eb.try_at_named(#name, #func);
                    }
                } else {
                    quote! {
                        eb.try_at_index(#i, #func);
                    }
                }
            };

            match meta {
                // #[validatron]
                syn::Meta::Path(_) => {
                    let f = quote! { #target.validate() };
                    nested_field_validators.push(push(f))
                }
                // #[validatron(...)]
                syn::Meta::List(list) => {
                    for item in list.nested.iter() {
                        if let syn::NestedMeta::Meta(meta) = item {
                            let validator = if borrow_fields {
                                get_field_validator(&meta, &quote!(&#target))
                            } else {
                                get_field_validator(&meta, &target)
                            };

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

fn destructure_variant_bindings(fields: &syn::Fields) -> TokenStream {
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
                let arg_name =
                    syn::Ident::new(&format!("_field{}", i), proc_macro2::Span::call_site());
                tokens.push(quote! {
                    #arg_name
                });
            }
        }
        syn::Fields::Unit => {}
    }

    match fields {
        syn::Fields::Named(_) => quote! {
            {#(ref #tokens),*}
        },
        syn::Fields::Unnamed(_) => quote! {
            (#(ref #tokens),*)
        },
        syn::Fields::Unit => quote! {},
    }
}

fn build_enum_variant_validator(de: &syn::DataEnum) -> TokenStream {
    let mut tokens = Vec::new();

    for var in &de.variants {
        let ident = &var.ident;

        let escaped = destructure_variant_bindings(&var.fields);

        let field_tokens = build_field_validators(&var.fields, None, false);

        tokens.push(quote! {
            Self::#ident #escaped => {
                #(#field_tokens)*
            },
        });
    }

    quote! {
        match self {
            #(#tokens)*
            _ => {}
        };
    }
}

fn impl_validatron(ast: &syn::DeriveInput) -> TokenStream {
    let type_validators = build_type_validator(&ast);

    let validators = match &ast.data {
        syn::Data::Struct(ds) => build_field_validators(&ds.fields, Some(quote!(self.)), true),
        syn::Data::Enum(de) => vec![build_enum_variant_validator(&de)],
        syn::Data::Union(_) => panic!("Union types are not supported"),
    };

    let derive_target = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        const _: () = {
            extern crate validatron;

            impl #impl_generics ::validatron::Validate for #derive_target #ty_generics #where_clause {
                fn validate(&self) -> ::validatron::Result<()> {
                    let mut eb = ::validatron::Error::build();

                    #(#validators)*

                    #(#type_validators)*

                    eb.build()
                }
            }
        };
    };

    expanded
}
