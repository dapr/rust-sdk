use std::iter;

use proc_macro::TokenStream;
use quote::quote;

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