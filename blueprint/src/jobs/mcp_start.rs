use blueprint_sdk::auth::models::ServiceOwnerModel;
use blueprint_sdk::auth::proxy::DEFAULT_AUTH_PROXY_PORT;
use blueprint_sdk::auth::types::KeyType;
use blueprint_sdk::tangle::extract::{BlockHash, List, ServiceId, TangleArg};
use blueprint_sdk::tangle::serde::from_field;
use blueprint_sdk::tangle_subxt::tangle_testnet_runtime::api;
use blueprint_sdk::{
    contexts::tangle::TangleClientContext, extract::Context, tangle::extract::TangleResult,
};
use futures::TryFutureExt;

use crate::MyContext;
use crate::error::Error;

/// Start the configured MCP server
pub async fn mcp_start(
    Context(ctx): Context<MyContext>,
    ServiceId(service_id): ServiceId,
    BlockHash(block_hash): BlockHash,
    TangleArg(List(ecdsa_owner)): TangleArg<List<u8>>,
) -> Result<TangleResult<String>, Error> {
    let client = ctx
        .env
        .tangle_client()
        .map_err(Into::into)
        .map_err(Error::Sdk)
        .await?;
    let current_instance_key = api::storage().services().instances(service_id);

    let maybe_current_instance = client
        .storage()
        .at(block_hash)
        .fetch(&current_instance_key)
        .map_err(Into::into)
        .map_err(Error::Sdk)
        .await?;

    let (owner, mut request_args) = match maybe_current_instance {
        Some(instance) => (instance.owner, instance.args),
        None => {
            return Err(Error::ServiceNotFound(service_id));
        }
    };

    if request_args.0.is_empty() {
        return Err(Error::MissingRequestParams);
    }

    let config = from_field::<crate::RequestParams>(request_args.0.pop().unwrap())
        .map(|p| p.config)
        .map_err(Error::InvalidRequestParams)?;

    blueprint_sdk::debug!(?config, %service_id, %owner, "Starting MCP server with config");

    let mut mcp_server_manager = ctx.mcp_server_manager.lock().await;
    let endpoint = mcp_server_manager
        .start_server(&ctx, service_id, owner.clone(), config)
        .await?;

    let bridge = ctx.env.bridge().await?;

    bridge
        .register_blueprint_service_proxy(
            service_id,
            Some("mcp_"),
            &endpoint,
            &[
                ServiceOwnerModel {
                    key_type: KeyType::Sr25519 as _,
                    key_bytes: owner.0.to_vec(),
                },
                ServiceOwnerModel {
                    key_type: KeyType::Ecdsa as _,
                    key_bytes: ecdsa_owner,
                },
            ],
        )
        .await?;

    let endpoint = format!("http://127.0.0.1:{DEFAULT_AUTH_PROXY_PORT}");

    Ok(TangleResult(endpoint))
}
