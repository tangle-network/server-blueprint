use blueprint_sdk::extract::Context;
use blueprint_sdk::tangle::extract::TangleResult;
use blueprint_sdk::tangle::extract::{ServiceId, TangleArg};

use crate::MyContext;
use crate::error::Error;

/// Stop the configured server
pub async fn server_stop(
    Context(ctx): Context<MyContext>,
    ServiceId(service_id): ServiceId,
    TangleArg(_): TangleArg<()>,
) -> Result<TangleResult<bool>, Error> {
    let mut manager = ctx.server_manager.lock().await;
    let stopped = manager.stop_server(service_id).await?;
    let bridge = ctx.env.bridge().await?;
    bridge
        .unregister_blueprint_service_proxy(service_id)
        .await?;
    Ok(TangleResult(stopped))
}
