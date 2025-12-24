pub mod client;

// Include the generated protobuf code
pub mod daemon {
    tonic::include_proto!("daemon");
}

pub use client::DaemonClient;
