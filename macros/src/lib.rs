use std::iter;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn actor(_attr: TokenStream, item: TokenStream) -> TokenStream {

    let actor_struct = item.clone();
    let actor_struct = parse_macro_input!(actor_struct as syn::ItemStruct);
    let actor_struct_name = actor_struct.ident.clone();
    
    let mut result = TokenStream::from(quote!(    
        #[async_trait::async_trait]
        impl axum::extract::FromRequestParts<dapr::server::actor::runtime::ActorState> for &#actor_struct_name {
            type Rejection = dapr::server::actor::ActorRejection;

            async fn from_request_parts(
                parts: &mut axum::http::request::Parts,
                state: &dapr::server::actor::runtime::ActorState,
            ) -> Result<Self, Self::Rejection> {
                let path = match axum::extract::Path::<dapr::server::actor::ActorPath>::from_request_parts(parts, state).await {
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