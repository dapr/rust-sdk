use proc_macro::TokenStream;
use std::iter;

use proc_macro2::TokenTree;
use quote::{format_ident, quote};
use syn::{Ident, LitStr, parse_macro_input};
use syn::parse::{Parse, ParseStream};

macro_rules! derive_parse {(
    @derive_only
    $( #[$attr:meta] )*
    $pub:vis
    struct $StructName:ident {
        $(
            $( #[$field_attr:meta] )*
            $field_pub:vis
            $field_name:ident : $FieldTy:ty
        ),* $(,)?
    }
) => (
    impl Parse for $StructName {
        fn parse (input: ParseStream)
          -> ::syn::Result<Self>
        {
            mod kw {
                $(
                    ::syn::custom_keyword!( $field_name );
                )*
            }
            use ::core::ops::Not as _;

            $(
                let mut $field_name = ::core::option::Option::None::< $FieldTy >;
            )*
            while input.is_empty().not() {
                let lookahead = input.lookahead1();
                match () {
                  $(
                    _case if lookahead.peek(kw::$field_name) => {
                        let span = input.parse::<kw::$field_name>().unwrap().span;
                        let _: ::syn::Token![ = ] = input.parse()?;
                        let prev = $field_name.replace(input.parse()?);
                        if prev.is_some() {
                            return ::syn::Result::Err(::syn::Error::new(span, "Duplicate key"));
                        }
                    },
                  )*
                    _default => return ::syn::Result::Err(lookahead.error()),
                }
                let _: ::core::option::Option<::syn::Token![ , ]> = input.parse()?;
            }
            Ok(Self {
                $(
                    $field_name: $field_name.ok_or_else(|| ::syn::Error::new(
                        ::proc_macro2::Span::call_site(),
                        ::core::concat!("Missing key `", ::core::stringify!($field_name), "`"),
                    ))?,
                )*
            })
        }
    }
); (
    $( #[$attr:meta] )* $pub:vis struct $($rest:tt)*
) => (
    $( #[$attr] )* $pub struct $($rest)*

    derive_parse! { @derive_only  $( #[$attr] )* $pub struct $($rest)* }
)}

derive_parse! {
    struct TopicArgs {
        pub_sub_name: LitStr,
        topic: LitStr
    }
}

#[proc_macro_attribute]
pub fn actor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let actor_struct_name = match syn::parse::<syn::ItemStruct>(item.clone()) {
        Ok(actor_struct) => actor_struct.ident.clone(),
        Err(_) => match syn::parse::<syn::ItemType>(item.clone()) {
            Ok(ty) => ty.ident.clone(),
            Err(e) => panic!("Error parsing actor struct: {}", e),
        },
    };

    let mut result = TokenStream::from(quote!(
        #[async_trait::async_trait]
        impl dapr::server::actor::axum::extract::FromRequestParts<dapr::server::actor::runtime::ActorState> for &#actor_struct_name {
            type Rejection = dapr::server::actor::ActorRejection;

            async fn from_request_parts(
                parts: &mut dapr::server::actor::axum::http::request::Parts,
                state: &dapr::server::actor::runtime::ActorState,
            ) -> Result<Self, Self::Rejection> {
                let path = match dapr::server::actor::axum::extract::Path::<dapr::server::actor::ActorPath>::from_request_parts(parts, state).await {
                    Ok(path) => path,
                    Err(e) => {
                        log::error!("Error getting path: {}", e);
                        return Err(dapr::server::actor::ActorRejection::Path(e));
                    }
                };
                let actor_type = state.actor_type.clone();
                let actor_id = path.actor_id.clone();
                log::info!(
                    "Request for actor_type: {}, actor_id: {}",
                    actor_type,
                    actor_id
                );
                let actor = match state
                    .runtime
                    .get_or_create_actor(&actor_type, &actor_id)
                    .await
                {
                    Ok(actor) => actor,
                    Err(e) => {
                        log::error!("Error getting actor: {}", e);
                        return Err(dapr::server::actor::ActorRejection::ActorError(e.to_string()));
                    }
                };
                let actor = actor.as_ref();
                let well_known_actor =
                    unsafe { &*(actor as *const dyn dapr::server::actor::Actor as *const #actor_struct_name) };
                Ok(well_known_actor)
            }
        }
    ));

    result.extend(iter::once(item));

    result
}

#[proc_macro_attribute]
pub fn topic(args: TokenStream, input: TokenStream) -> TokenStream {
    let new_input = proc_macro2::TokenStream::from(input);
    let mut iter = new_input.clone().into_iter().filter(|i| match i {
        TokenTree::Group(_) => true,
        TokenTree::Ident(_) => true,
        TokenTree::Punct(_) => false,
        TokenTree::Literal(_) => false,
    });

    let mut current = iter.next().unwrap();

    while current.to_string() != "fn" {
        current = iter.next().unwrap()
    }

    let name = iter.next().unwrap();

    let struct_name = name
        .to_string()
        .split('_')
        .into_iter()
        .map(|i| {
            let mut chars: Vec<char> = i.chars().collect();
            chars[0] = chars[0].to_ascii_uppercase();
            let new_string: String = chars.into_iter().collect();
            new_string
        })
        .collect::<Vec<String>>()
        .join("");

    let name_ident = Ident::new(name.to_string().as_str(), name.span());

    let struct_name_ident = Ident::new(struct_name.as_str(), name.span());

    let vars: Vec<String> = iter
        .next()
        .unwrap()
        .to_string()
        .replace(['(', ')'], "")
        .split(':')
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i % 2 != 0)
        .map(|(_, i)| i.trim().to_string())
        .collect();

    assert_eq!(vars.len(), 1, "Expected to only have one input variable");

    let parse_statement = match vars[0] == *"String" {
        true => {
            quote! {
                let message = message.to_string();
            }
        }
        false => {
            let type_ident = format_ident!("{}", vars[0]);
            quote! {
               let message: #type_ident = dapr::serde_json::from_str(message.to_string().as_str()).unwrap();
            }
        }
    };

    let args = parse_macro_input!(args as TopicArgs);

    let topic = args.topic.value();

    let pub_sub_name = args.pub_sub_name.value();

    let tokens = quote! {
        #new_input

        #[derive(Default)]
        struct #struct_name_ident;

        #[tonic::async_trait]
        impl dapr::appcallback::HandlerMethod for #struct_name_ident {
            async fn handler(
                &self,
                request: TopicEventRequest,
            ) -> Result<tonic::Response<TopicEventResponse>, tonic::Status> {
                let data = &request.data;
                let data_content_type = &request.data_content_type;

                let message = String::from_utf8_lossy(&data);

                #parse_statement

                #name_ident(message).await;

                Ok(tonic::Response::new(TopicEventResponse::default()))
            }
        }
        impl #struct_name_ident {
            pub fn get_handler(self) -> dapr::appcallback::Handler {
                dapr::appcallback::Handler {
                    pub_sub_name: #pub_sub_name.to_string(),
                    topic: #topic.to_string(),
                    handler: Box::new(self)
                }
            }
        }
    };

    tokens.into()
}