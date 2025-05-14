use blueprint_sdk::extract::Context;
use blueprint_sdk::tangle::extract::TangleResult;
use blueprint_sdk::tangle::extract::{ServiceId, TangleArg};

use crate::MyContext;
use crate::error::Error;

/// Stop the configured MCP server
pub async fn mcp_stop(
    Context(ctx): Context<MyContext>,
    ServiceId(service_id): ServiceId,
    TangleArg(_): TangleArg<()>,
) -> Result<TangleResult<bool>, Error> {
    let mut manager = ctx.mcp_server_manager.lock().await;
    let stopped = manager.stop_server(service_id)?;
    Ok(TangleResult(stopped))
}
