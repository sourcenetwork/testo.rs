extern crate proc_macro;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use syn::{Data, DataEnum, DeriveInput, Expr, Lit, Meta, parse_macro_input};

#[proc_macro_derive(Action, attributes(TState))]
pub fn derive_action(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let state = input
        .attrs
        .iter()
        .find_map(|a| match &a.meta {
            Meta::NameValue(n) => {
                if n.path.is_ident("TState") {
                    match &n.value {
                        Expr::Lit(l) => match &l.lit {
                            Lit::Str(s) => Some(s.value()),
                            _ => todo!(),
                        },
                        _ => todo!(),
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap();

    let expanded = match input.data {
        Data::Enum(data) => implement_action_enum(input.ident, state, data),
        _ => todo!(),
    };

    proc_macro::TokenStream::from(expanded)
}

fn implement_action_enum(ident: Ident, state: String, data: DataEnum) -> TokenStream {
    let action: Vec<TokenStream> = data
        .variants
        .iter()
        .map(|v| {
            let name = v.ident.clone();
            quote! {#name}
        })
        .collect();

    let state: TokenStream = state.parse().unwrap();

    quote! {
        impl Action for #ident {
            type TState = #state;

            fn execute(&self, state: &mut #state) {
                match self {
                    #(Self::#action(a) => a.execute(state),)*
                }
            }
        }

        #(
            impl From<#action> for #ident {
                fn from(value: #action) -> Self {
                    #ident::#action(value)
                }
            }
        )*
    }
}

#[proc_macro_derive(Multiplier, attributes(TActions, TActionAsync))]
pub fn derive_multipliers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let actions_async = input
        .attrs
        .iter()
        .find_map(|a| match &a.meta {
            Meta::NameValue(n) => {
                if n.path.is_ident("TActionAsync") {
                    match &n.value {
                        Expr::Lit(l) => match &l.lit {
                            Lit::Str(s) => Some(s.value()),
                            _ => todo!(),
                        },
                        _ => todo!(),
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap();

    let actions = input
        .attrs
        .iter()
        .find_map(|a| match &a.meta {
            Meta::NameValue(n) => {
                if n.path.is_ident("TActions") {
                    match &n.value {
                        Expr::Lit(l) => match &l.lit {
                            Lit::Str(s) => Some(s.value()),
                            _ => todo!(),
                        },
                        _ => todo!(),
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap_or(actions_async.clone());

    let expanded = match input.data {
        Data::Enum(data) => implement_multiplier_enum(input.ident, actions, actions_async, data),
        _ => todo!(),
    };

    proc_macro::TokenStream::from(expanded)
}

fn implement_multiplier_enum(ident: Ident, actions: String, actions_async: String, data: DataEnum) -> TokenStream {
    let multiplier: Vec<TokenStream> = data
        .variants
        .iter()
        .map(|v| {
            let name = v.ident.clone();
            quote! {#name}
        })
        .collect();

    let actions: TokenStream = actions.parse().unwrap();
    let actions_async: TokenStream = actions_async.parse().unwrap();

    quote! {
        impl Multiplier<#actions, #actions_async> for #ident {
            const NAME: &str = "#ident"; // todo - this is ugly, reconsider

            fn name(&self) -> &str {
                match self {
                    #(Multipliers::#multiplier(m) => m.name(),)*
                }
            }

            fn apply(
                &self,
                actions: &[ActionInstance<#actions_async, #actions>],
            ) -> Vec<ActionInstance<#actions_async, #actions>> {
                match self {
                    #(Multipliers::#multiplier(m) => m.apply(actions),)*
                }
            }
        }
    }
}
