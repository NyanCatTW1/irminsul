#[cfg(windows)]
mod pktmon_backend;

use std::fmt::{Debug, Display};

use anyhow::Error;
use async_trait::async_trait;

pub const PORT_RANGE: (u16, u16) = (22101, 22102);

#[derive(Debug)]
#[allow(dead_code)]
pub enum CaptureError {
    Filter(Error),
    Capture { has_captured: bool, error: Error },
    CaptureClosed,
    ChannelClosed,
}

impl Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::Filter(e) => write!(f, "Filter error: {}", e),
            CaptureError::Capture {
                has_captured,
                error,
            } => write!(
                f,
                "Capture error (has_captured = {}): {}",
                has_captured, error
            ),
            CaptureError::CaptureClosed => write!(f, "Capture closed"),
            CaptureError::ChannelClosed => write!(f, "Channel closed"),
        }
    }
}

pub type Result<T> = std::result::Result<T, CaptureError>;

#[async_trait]
pub trait CaptureBackend: Send {
    async fn next_packet(&mut self) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub enum BackendType {
    Pktmon,
    Pcap,
}

#[cfg(windows)]
pub const DEFAULT_CAPTURE_BACKEND_TYPE: BackendType = BackendType::Pktmon;
#[cfg(not(windows))]
pub const DEFAULT_CAPTURE_BACKEND_TYPE: BackendType = BackendType::Pcap;

pub fn create_capture(backend: BackendType) -> Result<Box<dyn CaptureBackend>> {
    match backend {
        #[cfg(windows)]
        BackendType::Pktmon => Ok(Box::new(pktmon_backend::PktmonBackend::new()?)),
        BackendType::Pcap => todo!(),
        #[allow(unreachable_patterns)]
        _ => Err(CaptureError::Capture {
            has_captured: false,
            error: anyhow::anyhow!(
                "Capture backend type {:?} not supported on this operating system",
                backend
            ),
        }),
    }
}
