use std::collections::BTreeMap;
use std::process::Child;

use crate::error::Error;
use crate::manager::McpRunner;

pub struct DockerRunner;

impl McpRunner for DockerRunner {
    fn start(
        &self,
        _package: String,
        _args: Vec<String>,
        _port_bindings: Vec<(u16, Option<u16>)>,
        _env_vars: BTreeMap<String, String>,
    ) -> Result<(Child, String), Error> {
        todo!("Docker runner not implemented");
    }

    fn check(&self) -> Result<bool, Error> {
        todo!("Docker check not implemented");
    }

    fn install(&self) -> Result<(), Error> {
        todo!("Docker install not implemented");
    }
}
