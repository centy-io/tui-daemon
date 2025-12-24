use std::time::Duration;

use tonic::transport::Channel;

use super::daemon::{
    daemon_service_client::DaemonServiceClient, ControlCommand, ControlRequest, ControlResponse,
    MetricsRequest, MetricsResponse, StatusRequest, StatusResponse,
};

/// Wrapper around the gRPC client with connection management
pub struct DaemonClient {
    client: Option<DaemonServiceClient<Channel>>,
    address: String,
}

impl DaemonClient {
    /// Create a new daemon client
    pub fn new(address: String) -> Self {
        Self {
            client: None,
            address,
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    /// Connect to the daemon
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let channel = Channel::from_shared(self.address.clone())?
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .connect()
            .await?;

        self.client = Some(DaemonServiceClient::new(channel));
        Ok(())
    }

    /// Disconnect from the daemon
    pub fn disconnect(&mut self) {
        self.client = None;
    }

    /// Get daemon status
    pub async fn get_status(
        &mut self,
    ) -> Result<StatusResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = self
            .client
            .as_mut()
            .ok_or("Not connected to daemon")?;

        let response = client.get_status(StatusRequest {}).await?;
        Ok(response.into_inner())
    }

    /// Get daemon metrics
    pub async fn get_metrics(
        &mut self,
    ) -> Result<MetricsResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = self
            .client
            .as_mut()
            .ok_or("Not connected to daemon")?;

        let response = client.get_metrics(MetricsRequest {}).await?;
        Ok(response.into_inner())
    }

    /// Send a control command
    pub async fn control(
        &mut self,
        command: ControlCommand,
    ) -> Result<ControlResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = self
            .client
            .as_mut()
            .ok_or("Not connected to daemon")?;

        let response = client
            .control(ControlRequest {
                command: command.into(),
            })
            .await?;
        Ok(response.into_inner())
    }
}
