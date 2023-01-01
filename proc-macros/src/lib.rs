extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenTree};

use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr};

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

/// Example of user-defined [procedural macro attribute][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
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
            println!("{}", type_ident);
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
        impl AppCallback for #struct_name_ident {
            async fn on_invoke(
                &self,
                _request: Request<InvokeRequest>,
            ) -> Result<Response<InvokeResponse>, Status> {
                Ok(Response::new(InvokeResponse::default()))
            }

            async fn list_topic_subscriptions(
                &self,
                _request: Request<()>,
            ) -> Result<Response<ListTopicSubscriptionsResponse>, Status> {
                let topic = #topic.to_string();
                let pubsub_name = #pub_sub_name.to_string();

                let list_subscriptions = ListTopicSubscriptionsResponse::topic(pubsub_name, topic);

                Ok(Response::new(list_subscriptions))
            }

            async fn on_topic_event(
                &self,
                request: Request<TopicEventRequest>,
            ) -> Result<Response<TopicEventResponse>, Status> {
                let r = request.into_inner();
                let data = &r.data;
                let data_content_type = &r.data_content_type;

                let message = String::from_utf8_lossy(&data);

                #parse_statement

                #name_ident(message).await;

                Ok(Response::new(TopicEventResponse::default()))
            }

            async fn list_input_bindings(
                &self,
                _request: Request<()>,
            ) -> Result<Response<ListInputBindingsResponse>, Status> {
                Ok(Response::new(ListInputBindingsResponse::default()))
            }

            async fn on_binding_event(
                &self,
                _request: Request<BindingEventRequest>,
            ) -> Result<Response<BindingEventResponse>, Status> {
                Ok(Response::new(BindingEventResponse::default()))
            }
        }
    };

    tokens.into()
}
