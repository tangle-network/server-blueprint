use blueprint_sdk::tangle::extract::{BlockHash, ServiceId, TangleArg};
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
    TangleArg(_): TangleArg<()>,
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
        .start_server(service_id, owner, config)
        .await?;

    // TODO: register the endpoint, service id and owner in the auth proxy.

    Ok(TangleResult(endpoint))
}
