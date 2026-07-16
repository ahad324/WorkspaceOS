use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub state: TunnelState,
    pub public_url: Option<String>,
    pub provider: String,
    pub latency_ms: u32,
    pub reconnect_count: u32,
}

pub trait TunnelProvider: Send + Sync {
    fn name(&self) -> &str;
    fn connect(&mut self) -> Result<String, String>;
    fn disconnect(&mut self) -> Result<(), String>;
    fn get_status(&self) -> TunnelStatus;
    fn update_metrics(&mut self);
}

pub struct MockTunnelProvider {
    name: String,
    state: TunnelState,
    public_url: Option<String>,
    latency_ms: u32,
    reconnect_count: u32,
}

impl MockTunnelProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            state: TunnelState::Disconnected,
            public_url: None,
            latency_ms: 0,
            reconnect_count: 0,
        }
    }
}

impl TunnelProvider for MockTunnelProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn connect(&mut self) -> Result<String, String> {
        self.state = TunnelState::Connecting;
        self.state = TunnelState::Connected;
        let url = format!(
            "https://{}.workspaceos.dev/mcp-session",
            self.name.to_lowercase()
        );
        self.public_url = Some(url.clone());
        self.latency_ms = 12;
        Ok(url)
    }

    fn disconnect(&mut self) -> Result<(), String> {
        self.state = TunnelState::Disconnected;
        self.public_url = None;
        self.latency_ms = 0;
        Ok(())
    }

    fn get_status(&self) -> TunnelStatus {
        TunnelStatus {
            state: self.state,
            public_url: self.public_url.clone(),
            provider: self.name.clone(),
            latency_ms: self.latency_ms,
            reconnect_count: self.reconnect_count,
        }
    }

    fn update_metrics(&mut self) {
        if self.state == TunnelState::Connected {
            // Simulate slight variations in latency
            let rand_val = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() % 20)
                .unwrap_or(0) as u32;
            self.latency_ms = 10 + rand_val;
        }
    }
}

pub struct TunnelManager {
    provider: Arc<Mutex<Box<dyn TunnelProvider>>>,
    active: Arc<Mutex<bool>>,
}

impl TunnelManager {
    pub fn new(provider_name: &str) -> Self {
        let provider: Box<dyn TunnelProvider> = Box::new(MockTunnelProvider::new(provider_name));
        Self {
            provider: Arc::new(Mutex::new(provider)),
            active: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start_tunnel(&self) -> Result<String, String> {
        let mut prov = self.provider.lock().unwrap();
        let url = prov.connect()?;

        let was_inactive = !*self.active.lock().unwrap();
        *self.active.lock().unwrap() = true;

        if was_inactive {
            let provider_clone = self.provider.clone();
            let active_clone = self.active.clone();
            tokio::spawn(async move {
                while *active_clone.lock().unwrap() {
                    sleep(Duration::from_secs(2)).await;
                    if *active_clone.lock().unwrap() {
                        let mut prov = provider_clone.lock().unwrap();
                        prov.update_metrics();
                    }
                }
            });
        }

        Ok(url)
    }

    pub fn stop_tunnel(&self) -> Result<(), String> {
        let mut prov = self.provider.lock().unwrap();
        prov.disconnect()?;
        *self.active.lock().unwrap() = false;
        Ok(())
    }

    pub fn get_status(&self) -> TunnelStatus {
        let prov = self.provider.lock().unwrap();
        prov.get_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tunnel_state_transitions() {
        let manager = TunnelManager::new("Cloudflare");

        // Initial state
        let status = manager.get_status();
        assert_eq!(status.state, TunnelState::Disconnected);
        assert_eq!(status.public_url, None);

        // Connect
        let url = manager.start_tunnel().unwrap();
        assert!(url.contains("cloudflare.workspaceos.dev"));

        let status = manager.get_status();
        assert_eq!(status.state, TunnelState::Connected);
        assert_eq!(status.public_url, Some(url));

        // Disconnect
        manager.stop_tunnel().unwrap();
        let status = manager.get_status();
        assert_eq!(status.state, TunnelState::Disconnected);
        assert_eq!(status.public_url, None);
    }
}
