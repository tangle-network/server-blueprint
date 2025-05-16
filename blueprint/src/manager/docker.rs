use std::collections::BTreeMap;
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::manager::McpRunner;

pub struct DockerRunner;

impl McpRunner for DockerRunner {
    async fn start(
        &self,
        _package: String,
        _args: Vec<String>,
        _port_bindings: Vec<(u16, Option<u16>)>,
        _env_vars: BTreeMap<String, String>,
    ) -> Result<(CancellationToken, String), Error> {
        todo!("Docker runner not implemented");
    }

    async fn check(&self) -> Result<bool, Error> {
        todo!("Docker check not implemented");
    }

    async fn install(&self) -> Result<(), Error> {
        todo!("Docker install not implemented");
    }
}
