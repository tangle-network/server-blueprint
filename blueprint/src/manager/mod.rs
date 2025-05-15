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

use crate::McpRuntime;
use crate::error::Error;

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
    /// The port to bind the mcp server to
    /// This is a list of tuples, where the first element is the host port and the second element is the
    /// container port (if applicable)
    #[serde(default)]
    pub port_bindings: Vec<(u16, Option<u16>)>,
    /// Environment variables to pass to the mcp server
    #[serde(default)]
    pub env_vars: BTreeMap<String, String>,

    /// The process handle for the mcp server
    /// This is used to kill the process when the server is stopped
    #[serde(skip)]
    pub process: Option<std::process::Child>,
}

pub trait McpRunner {
    /// Start the mcp server
    /// Returns (Child process, endpoint)
    fn start(
        &self,
        package: String,
        args: Vec<String>,
        port_bindings: Vec<(u16, Option<u16>)>,
        env_vars: BTreeMap<String, String>,
    ) -> Result<(std::process::Child, String), Error>;

    /// Check if the runtime is installed and available
    fn check(&self) -> Result<bool, Error>;

    /// Install the runtime if not present
    fn install(&self) -> Result<(), Error>;
}

impl McpServerManager {
    #[tracing::instrument(skip(self, config), fields(service_id, %owner))]
    pub fn start_server(
        &mut self,
        service_id: u64,
        owner: AccountId32,
        config: crate::McpServerConfig,
    ) -> Result<String, Error> {
        use crate::manager::docker::DockerRunner;
        use crate::manager::js::JsRunner;
        use crate::manager::python::PythonRunner;

        let args = config.args.0.unwrap_or_default().0.clone();

        let port_bindings: Vec<(u16, Option<u16>)> = config
            .port_bindings
            .0
            .unwrap_or_default()
            .0
            .into_iter()
            .map(|(host_port, container_port)| {
                let container_port = if container_port == 0 {
                    None
                } else {
                    Some(container_port)
                };
                (host_port, container_port)
            })
            .collect();

        let env_vars: BTreeMap<String, String> =
            config.env.0.unwrap_or_default().0.into_iter().collect();

        blueprint_sdk::debug!(
            ?args,
            ?port_bindings,
            ?env_vars,
            package = %config.package,
            runtime = ?config.runtime,
            "Starting MCP server with args"
        );
        let (process, endpoint) = match config.runtime {
            crate::McpRuntime::Python => PythonRunner.start(
                config.package.clone(),
                args.clone(),
                port_bindings.clone(),
                env_vars.clone(),
            )?,
            crate::McpRuntime::Javascript => JsRunner.start(
                config.package.clone(),
                args.clone(),
                port_bindings.clone(),
                env_vars.clone(),
            )?,
            crate::McpRuntime::Docker => DockerRunner.start(
                config.package.clone(),
                args.clone(),
                port_bindings.clone(),
                env_vars.clone(),
            )?,
            crate::McpRuntime::Unknown => {
                return Err(Error::UnknownRuntime);
            }
        };
        let server = McpServer {
            runtime: config.runtime,
            package: config.package,
            args,
            port_bindings,
            env_vars,
            process: Some(process),
        };
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
    pub fn stop_server(&mut self, service_id: u64) -> Result<bool, Error> {
        blueprint_sdk::debug!("Stopping MCP server");
        if let Some(mut server) = self.servers.remove(&service_id) {
            if let Some(mut child) = server.process.take() {
                match child.kill() {
                    Ok(_) => {
                        // Optionally wait for the process to ensure it's fully terminated,
                        // though kill() usually suffices. Could log error if wait fails.
                        _ = child.wait();
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::InvalidInput => {
                        blueprint_sdk::debug!("Process already exited, no need to kill: {e}");
                    }
                    Err(e) => {
                        // Failed to kill, proceed to remove maps to prevent re-attempts.
                        // In a real scenario, might re-insert server or handle error more gracefully.
                        blueprint_sdk::warn!("Failed to kill process: {e}");
                    }
                }
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
