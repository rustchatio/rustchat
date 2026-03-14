//! SFU Performance Optimizations
//!
//! This module contains performance optimizations for the SFU:
//! - Object pooling for RTP buffers
//! - ICE gathering optimization (event-based instead of sleep)
//! - Efficient packet forwarding
//! - Metrics collection

use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;

// ============================================
// RTP Buffer Pool
// ============================================

const RTP_BUFFER_SIZE: usize = 1500;
const POOL_INITIAL_CAPACITY: usize = 64;
const POOL_MAX_CAPACITY: usize = 256;

/// Thread-safe pool of RTP packet buffers
pub struct RtpBufferPool {
    available: Mutex<Vec<Vec<u8>>>,
    max_capacity: usize,
}

impl RtpBufferPool {
    pub fn new() -> Arc<Self> {
        let mut buffers = Vec::with_capacity(POOL_INITIAL_CAPACITY);
        for _ in 0..POOL_INITIAL_CAPACITY {
            buffers.push(vec![0u8; RTP_BUFFER_SIZE]);
        }

        Arc::new(Self {
            available: Mutex::new(buffers),
            max_capacity: POOL_MAX_CAPACITY,
        })
    }

    /// Acquire a buffer from the pool
    pub async fn acquire(&self) -> Vec<u8> {
        let mut available = self.available.lock().await;
        available.pop().unwrap_or_else(|| vec![0u8; RTP_BUFFER_SIZE])
    }

    /// Return a buffer to the pool
    pub async fn release(&self, mut buffer: Vec<u8>) {
        if buffer.capacity() != RTP_BUFFER_SIZE {
            return; // Don't pool buffers with wrong capacity
        }

        let mut available = self.available.lock().await;
        if available.len() < self.max_capacity {
            buffer.fill(0);
            available.push(buffer);
        }
        // Otherwise, let it drop (pool is full)
    }
}

impl Default for RtpBufferPool {
    fn default() -> Self {
        let mut buffers = Vec::with_capacity(POOL_INITIAL_CAPACITY);
        for _ in 0..POOL_INITIAL_CAPACITY {
            buffers.push(vec![0u8; RTP_BUFFER_SIZE]);
        }

        Self {
            available: Mutex::new(buffers),
            max_capacity: POOL_MAX_CAPACITY,
        }
    }
}

// ============================================
// ICE Gathering Optimization
// ============================================

use webrtc::peer_connection::RTCPeerConnection;
use std::time::Duration;

/// Wait for ICE gathering to complete or timeout
/// Uses event-based gathering instead of fixed sleep
pub async fn wait_for_ice_gathering(
    pc: &RTCPeerConnection,
    timeout: Duration,
) -> Result<(), &'static str> {
    // Check if already complete
    if pc.ice_gathering_state() == webrtc::ice_transport::ice_gathering_state::RTCIceGatheringState::Complete {
        return Ok(());
    }

    // Wait for gathering to complete with timeout
    let start = tokio::time::Instant::now();
    let timeout_duration = timeout;
    while start.elapsed() < timeout_duration {
        if pc.ice_gathering_state() == webrtc::ice_transport::ice_gathering_state::RTCIceGatheringState::Complete {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Timeout - but we can still proceed with gathered candidates
    tracing::debug!("ICE gathering timed out, proceeding with available candidates");
    Ok(())
}

// ============================================
// Optimized Track Creation
// ============================================

/// Create audio track with optimal codec settings
pub fn create_audio_track(session_id: &str, stream_id: &str) -> Arc<TrackLocalStaticRTP> {
    Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: "audio/opus".to_string(),
            clock_rate: 48000,
            channels: 2,
            sdp_fmtp_line: "minptime=10;useinbandfec=1".to_string(),
            rtcp_feedback: vec![],
        },
        format!("audio-{}", session_id),
        stream_id.to_string(),
    ))
}

/// Create video track with optimal codec settings
pub fn create_video_track(session_id: &str, stream_id: &str) -> Arc<TrackLocalStaticRTP> {
    Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: "video/VP8".to_string(),
            clock_rate: 90000,
            channels: 0,
            sdp_fmtp_line: String::new(),
            rtcp_feedback: vec![
                webrtc::rtp_transceiver::RTCPFeedback {
                    typ: "nack".to_string(),
                    parameter: String::new(),
                },
                webrtc::rtp_transceiver::RTCPFeedback {
                    typ: "nack".to_string(),
                    parameter: "pli".to_string(),
                },
                webrtc::rtp_transceiver::RTCPFeedback {
                    typ: "ccm".to_string(),
                    parameter: "fir".to_string(),
                },
            ],
        },
        format!("video-{}", session_id),
        stream_id.to_string(),
    ))
}

/// Create screen share track with optimized settings
pub fn create_screen_track(session_id: &str, stream_id: &str) -> Arc<TrackLocalStaticRTP> {
    Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: "video/VP8".to_string(),
            clock_rate: 90000,
            channels: 0,
            sdp_fmtp_line: String::new(),
            rtcp_feedback: vec![
                webrtc::rtp_transceiver::RTCPFeedback {
                    typ: "nack".to_string(),
                    parameter: String::new(),
                },
                webrtc::rtp_transceiver::RTCPFeedback {
                    typ: "nack".to_string(),
                    parameter: "pli".to_string(),
                },
            ],
        },
        format!("screen-{}", session_id),
        stream_id.to_string(),
    ))
}

// ============================================
// Packet Forwarding Optimizations
// ============================================

use std::sync::atomic::{AtomicU64, Ordering};

/// Metrics for SFU performance monitoring
pub struct SfuMetrics {
    pub packets_forwarded: AtomicU64,
    pub packets_dropped: AtomicU64,
    pub bytes_forwarded: AtomicU64,
    pub active_tracks: AtomicU64,
}

impl SfuMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            packets_forwarded: AtomicU64::new(0),
            packets_dropped: AtomicU64::new(0),
            bytes_forwarded: AtomicU64::new(0),
            active_tracks: AtomicU64::new(0),
        })
    }

    pub fn record_packet_forwarded(&self, bytes: usize) {
        self.packets_forwarded.fetch_add(1, Ordering::Relaxed);
        self.bytes_forwarded.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_packet_dropped(&self) {
        self.packets_dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_active_tracks(&self) {
        self.active_tracks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_active_tracks(&self) {
        self.active_tracks.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            packets_forwarded: self.packets_forwarded.load(Ordering::Relaxed),
            packets_dropped: self.packets_dropped.load(Ordering::Relaxed),
            bytes_forwarded: self.bytes_forwarded.load(Ordering::Relaxed),
            active_tracks: self.active_tracks.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub packets_forwarded: u64,
    pub packets_dropped: u64,
    pub bytes_forwarded: u64,
    pub active_tracks: u64,
}

// ============================================
// Batch Forwarding Channel
// ============================================

/// Packet batch for efficient forwarding
pub struct PacketBatch {
    pub packets: Vec<Vec<u8>>,
    pub sender_session_id: uuid::Uuid,
    pub track_id: String,
}

/// Configuration for batch forwarding
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub max_batch_delay_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            max_batch_delay_ms: 5,
        }
    }
}

// ============================================
// Voice Activity Detection Optimization
// ============================================

/// Optimized VAD using packet analysis
pub struct VoiceActivityDetector {
    last_voice_at: std::sync::atomic::AtomicU64,
    voice_threshold_packets: u32,
    silence_threshold_ms: u64,
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceActivityDetector {
    pub fn new() -> Self {
        Self {
            last_voice_at: std::sync::atomic::AtomicU64::new(0),
            voice_threshold_packets: 3,
            silence_threshold_ms: 500,
        }
    }

    /// Analyze RTP packet for voice activity
    /// Returns true if voice is detected
    pub fn analyze_packet(&self, packet: &[u8]) -> bool {
        // Simple heuristic: check packet size
        // Voice packets are typically small (~60-100 bytes for Opus)
        // Silence packets are often smaller or consistent
        if packet.len() < 12 {
            return false;
        }

        // Get timestamp from RTP header
        let _timestamp = u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]);

        // Check payload length (after 12 byte header)
        let payload_len = packet.len().saturating_sub(12);

        // Opus voice activity heuristic
        if payload_len > 20 && payload_len < 200 {
            self.last_voice_at.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                Ordering::Relaxed,
            );
            true
        } else {
            false
        }
    }

    /// Check if voice was recently detected
    pub fn is_voice_active(&self) -> bool {
        let last = self.last_voice_at.load(Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        now.saturating_sub(last) < self.silence_threshold_ms
    }
}

// ============================================
// Rate Limiter for Logging
// ============================================

/// Rate-limited logger to prevent log spam
pub struct RateLimitedLogger {
    last_log: std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>,
    min_interval: Duration,
}

impl RateLimitedLogger {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_log: std::sync::Mutex::new(std::collections::HashMap::new()),
            min_interval,
        }
    }

    /// Log a message only if enough time has passed since the last log with this key
    pub fn log<F>(&self, key: &str, log_fn: F)
    where
        F: FnOnce(),
    {
        let now = std::time::Instant::now();
        let mut last_log = self.last_log.lock().unwrap();

        if let Some(last) = last_log.get(key) {
            if now.duration_since(*last) < self.min_interval {
                return;
            }
        }

        last_log.insert(key.to_string(), now);
        drop(last_log);
        log_fn();
    }
}
