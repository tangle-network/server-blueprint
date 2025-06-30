use crate::manager::McpServerManager;
use blueprint_sdk::macros::context::ServicesContext;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::tangle::extract::{List, Optional, TangleArg};
use docktopus::bollard::Docker;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Different types of errors that can occur in the mcp server
mod error;
/// Blueprint Jobs
mod jobs;
/// The mcp server manager
mod manager;
/// The MCP Transport converter
mod transport;

pub use jobs::{MCP_START_JOB_ID, MCP_STOP_JOB_ID, mcp_start, mcp_stop};

/// Represents the runtime of the MCP server (Python, JS, Docker etc.)
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpRuntime {
    /// Unknown runtime
    #[default]
    Unknown,
    /// Will use uvx to run the mcp server
    Python,
    /// Will use bunx to run the mcp server
    Javascript,
    /// using a docker container to run the mcp server
    Docker,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    /// The different runtimes that can be used to run the mcp server
    pub runtime: McpRuntime,
    /// The package to use for the mcp server or the docker image
    ///
    /// Example: `mcp-server@x.y.z` for Python or JS, or `mcp-server:latest` for Docker
    pub package: String,
    /// A list of arguments to pass to the mcp server
    /// This is optional and can be empty
    #[serde(default)]
    pub args: Optional<List<String>>,
    /// Environment variables for the MCP server
    /// This is optional and can be empty
    #[serde(default)]
    pub env: Optional<List<(String, String)>>,
    /// The transport adapter to use for the MCP server
    #[serde(default)]
    pub transport_adapter: SupportedTransportAdapter,
}

/// The supported transport adapters for the MCP server
#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SupportedTransportAdapter {
    /// Converts the MCP server's stdout and stderr to Server-Sent Events (SSE) using our built-in SSE server
    #[default]
    StdioToSSE,
    /// No transport adapter, the MCP server will handle communication directly and give us a url to interact with
    None,
}

impl SupportedTransportAdapter {
    /// Returns `true` if the supported transport adapter is [`None`].
    ///
    /// [`None`]: SupportedTransportAdapter::None
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns `true` if the supported transport adapter is [`StdioToSSE`].
    ///
    /// [`StdioToSSE`]: SupportedTransportAdapter::StdioToSSE
    #[must_use]
    pub fn is_stdio_to_sse(&self) -> bool {
        matches!(self, Self::StdioToSSE)
    }
}

/// The Service Request Parameters
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestParams {
    pub config: McpServerConfig,
}

#[derive(Clone, ServicesContext)]
pub struct MyContext {
    #[config]
    env: BlueprintEnvironment,
    pub mcp_server_manager: Arc<Mutex<McpServerManager>>,
    pub docker: Arc<Docker>,
}

impl MyContext {
    pub async fn new(env: BlueprintEnvironment) -> Result<Self, error::Error> {
        let docker_builder = docktopus::DockerBuilder::new().await.map_err(|e| {
            crate::error::Error::Io(std::io::Error::other(format!(
                "Failed to create Docker client: {e}"
            )))
        })?;
        Ok(Self {
            env,
            mcp_server_manager: Arc::new(Mutex::new(McpServerManager::default())),
            docker: docker_builder.client(),
        })
    }
    /// Finds the next available port by binding to localhost:0 and retrieving the assigned port.
    ///
    /// This function uses the OS's ability to assign an available port when binding to port 0.
    /// The TCP listener is immediately closed after retrieving the port number.
    ///
    /// # Important: Race Condition Warning
    ///
    /// There is an inherent race condition in this approach: between the time the port is
    /// released (when the listener is dropped) and when the caller attempts to use the port,
    /// another process may bind to the same port. This is a fundamental limitation of this
    /// approach and should be considered when using this function, but this scenario is
    /// unlikely in practice for most applications.
    ///
    /// For critical applications, consider keeping the listener alive and passing it to the
    /// caller, or using a more sophisticated port management strategy.
    ///
    /// # Returns
    ///
    /// Returns the port number that was available at the time of testing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unable to bind to localhost (network issues, permission problems)
    /// - Unable to retrieve the local address from the bound socket
    pub async fn next_available_port(&self) -> Result<u16, error::Error> {
        let tcp = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(|e| {
                error::Error::Io(std::io::Error::other(format!(
                    "Failed to bind TCP listener: {e}"
                )))
            })?;
        let local_addr = tcp.local_addr().map_err(|e| {
            error::Error::Io(std::io::Error::other(format!(
                "Failed to get local address: {e}"
            )))
        })?;
        // Close the listener immediately after getting the port
        drop(tcp);
        Ok(local_addr.port())
    }
}

/// The request parameters for this blueprint
pub type BlueprintRequestParams = TangleArg<RequestParams>;
