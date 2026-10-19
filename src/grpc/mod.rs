pub mod client;

// Include the generated protobuf code
#[allow(clippy::result_large_err)] // generated tonic 0.12 code; fixed upstream in 0.14
pub mod daemon {
    tonic::include_proto!("daemon");
}

pub use client::DaemonClient;
