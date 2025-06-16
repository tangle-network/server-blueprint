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
    pub package: String,
    /// A list of arguments to pass to the mcp server
    #[serde(default)]
    pub args: Optional<List<String>>,
    /// The port to bind the mcp server to
    /// This is a list of tuples, where the first element is the host port and the second element is the
    /// container port (if applicable)
    #[serde(default)]
    pub port_bindings: Optional<List<(u16, u16)>>,
    /// Environment variables for the MCP server
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
            crate::error::Error::Io(std::io::Error::other(
                format!("Failed to create Docker client: {}", e),
            ))
        })?;
        Ok(Self {
            env,
            mcp_server_manager: Arc::new(Mutex::new(McpServerManager::default())),
            docker: docker_builder.client(),
        })
    }
}

/// The request parameters for this blueprint
pub type BlueprintRequestParams = TangleArg<RequestParams>;
