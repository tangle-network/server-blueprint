mod server_start;
mod server_stop;

pub const SERVER_START_JOB_ID: u8 = 0;
pub const SERVER_STOP_JOB_ID: u8 = 1;

pub use server_start::server_start;
pub use server_stop::server_stop;
