mod mcp_start;
mod mcp_stop;

pub const MCP_START_JOB_ID: u8 = 0;
pub const MCP_STOP_JOB_ID: u8 = 1;

pub use mcp_start::mcp_start;
pub use mcp_stop::mcp_stop;
