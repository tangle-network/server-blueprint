//! MCP Servers Manager
//!
//! it is responsible for managing the MCP servers based on the configuration
//! provided in the request parameters.
//!
//! The MCP servers can be run using different runtimes:
//! 1. Python (using uvx)
//! 2. Javascript (using bunx)
//! 3. Docker (using docker)
//!
//! The MCP servers can be run in the background and the endpoint will be returned
//! to the caller.

use std::collections::BTreeMap;

use blueprint_sdk::tangle_subxt::subxt::utils::AccountId32;
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::{McpRuntime, SupportedTransportAdapter};

/// TBD
pub mod docker;
/// Uses bunx to run the mcp server
pub mod js;
/// Uses uvx to run the mcp server
pub mod python;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct McpServerManager {
    /// Service Id to the McpServer mapping
    pub servers: BTreeMap<u64, McpServer>,
    /// Mapping of service id to the owner
    pub owners: BTreeMap<u64, AccountId32>,
    /// Mapping of service id to the endpoint
    pub endpoints: BTreeMap<u64, String>,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct McpServer {
    /// Runtime of the mcp server
    pub runtime: McpRuntime,
    /// The package to use for the mcp server or the docker image
    pub package: String,
    /// A list of arguments to pass to the mcp server
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables to pass to the mcp server
    #[serde(default)]
    pub env_vars: BTreeMap<String, String>,

    /// The cancellation token for the mcp server
    #[serde(skip)]
    pub cancellation_token: Option<CancellationToken>,
}

pub trait McpRunner {
    /// Start the mcp server
    /// Returns (CancellationToken, endpoint)
    async fn start(
        &self,
        ctx: &crate::MyContext,
        service_id: u64,
        package: String,
        args: Vec<String>,
        env_vars: BTreeMap<String, String>,
        transport_adapter: SupportedTransportAdapter,
    ) -> Result<CancellationToken, Error>;

    /// Check if the runtime is installed and available
    async fn check(&self, ctx: &crate::MyContext) -> Result<bool, Error>;

    /// Install the runtime if not present
    async fn install(&self, ctx: &crate::MyContext) -> Result<(), Error>;
}

impl McpServerManager {
    #[tracing::instrument(skip(self, ctx, config), fields(service_id, %owner))]
    pub async fn start_server(
        &mut self,
        ctx: &crate::MyContext,
        service_id: u64,
        owner: AccountId32,
        config: crate::McpServerConfig,
    ) -> Result<String, Error> {
        use crate::manager::docker::DockerRunner;
        use crate::manager::js::JsRunner;
        use crate::manager::python::PythonRunner;

        let args = config.args.0.unwrap_or_default().0.clone();

        let allocated_port = ctx.next_available_port().await?;

        let env_vars: BTreeMap<String, String> = config
            .env
            .0
            .unwrap_or_default()
            .0
            .into_iter()
            .chain(std::iter::once((
                "PORT".to_string(),
                allocated_port.to_string(),
            )))
            .collect();

        blueprint_sdk::debug!(
            ?args,
            ?env_vars,
            package = %config.package,
            runtime = ?config.runtime,
            "Starting MCP server with args"
        );
        let ct = match config.runtime {
            crate::McpRuntime::Python => {
                PythonRunner
                    .start(
                        ctx,
                        service_id,
                        config.package.clone(),
                        args.clone(),
                        env_vars.clone(),
                        config.transport_adapter,
                    )
                    .await?
            }
            crate::McpRuntime::Javascript => {
                JsRunner
                    .start(
                        ctx,
                        service_id,
                        config.package.clone(),
                        args.clone(),
                        env_vars.clone(),
                        config.transport_adapter,
                    )
                    .await?
            }
            crate::McpRuntime::Docker => {
                DockerRunner
                    .start(
                        ctx,
                        service_id,
                        config.package.clone(),
                        args.clone(),
                        env_vars.clone(),
                        config.transport_adapter,
                    )
                    .await?
            }
            crate::McpRuntime::Unknown => {
                return Err(Error::UnknownRuntime);
            }
        };
        let server = McpServer {
            runtime: config.runtime,
            package: config.package,
            args,
            env_vars,
            cancellation_token: Some(ct),
        };
        let endpoint = format!("http://127.0.0.1:{allocated_port}");
        self.servers.insert(service_id, server);
        self.owners.insert(service_id, owner);
        self.endpoints.insert(service_id, endpoint.clone());
        blueprint_sdk::debug!(
            %endpoint,
            "MCP server started"
        );
        Ok(endpoint)
    }
    /// Stop the MCP server with the given service_id.
    #[tracing::instrument(skip(self), fields(service_id))]
    pub async fn stop_server(&mut self, service_id: u64) -> Result<bool, Error> {
        blueprint_sdk::debug!("Stopping MCP server");
        if let Some(mut server) = self.servers.remove(&service_id) {
            if let Some(ct) = server.cancellation_token.take() {
                ct.cancel();
                ct.cancelled().await;
                blueprint_sdk::debug!("MCP server cancelled");
            }
            self.owners.remove(&service_id);
            self.endpoints.remove(&service_id);
            blueprint_sdk::debug!("MCP server stopped");
            Ok(true)
        } else {
            blueprint_sdk::debug!("MCP server not found");
            Ok(false)
        }
    }
}
