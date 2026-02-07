//! SFU (Selective Forwarding Unit) for RustChat Calls
//!
//! Routes audio/video tracks between participants in a call.
//! Each participant sends one stream and receives streams from all other participants.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::rtp_transceiver::RTCRtpTransceiver;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocalWriter;
use webrtc::track::track_remote::TrackRemote;

use crate::config::CallsConfig;

pub mod manager;
pub mod signaling;
pub mod tracks;

pub use manager::SFUManager;
use signaling::{SignalingMessage, SignalingServer};
use tracks::TrackManager;

/// Represents a participant in the SFU
pub struct Participant {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub peer_connection: Arc<RTCPeerConnection>,
    pub audio_track: Option<Arc<TrackLocalStaticRTP>>,
    pub video_track: Option<Arc<TrackLocalStaticRTP>>,
    pub screen_track: Option<Arc<TrackLocalStaticRTP>>,
    pub signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
}

/// SFU manages all peer connections and routes media
pub struct SFU {
    config: CallsConfig,
    participants: Arc<RwLock<HashMap<Uuid, Participant>>>,
    track_manager: Arc<TrackManager>,
    signaling: Arc<SignalingServer>,
}

impl SFU {
    /// Create a new SFU instance
    pub async fn new(
        config: CallsConfig,
    ) -> Result<Arc<Self>, Box<dyn std::error::Error + Send + Sync>> {
        let participants = Arc::new(RwLock::new(HashMap::new()));
        let track_manager = Arc::new(TrackManager::new());
        let signaling = Arc::new(SignalingServer::new());

        Ok(Arc::new(Self {
            config,
            participants,
            track_manager,
            signaling,
        }))
    }

    /// Add a new participant to the SFU
    pub async fn add_participant(
        &self,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<
        (
            Arc<RTCPeerConnection>,
            mpsc::UnboundedReceiver<SignalingMessage>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        // Create media engine with codec support
        let mut m = MediaEngine::default();
        m.register_default_codecs()?;

        // Create interceptor registry
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)?;

        // Create API
        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        // Create ICE servers configuration
        let ice_servers = self.build_ice_servers();

        // Create peer connection configuration
        let config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        // Create peer connection
        let peer_connection = Arc::new(api.new_peer_connection(config).await?);

        // Create signaling channel
        let (signaling_tx, signaling_rx) = mpsc::unbounded_channel();

        // Set up track handling
        self.setup_track_handlers(&peer_connection, user_id, session_id)
            .await?;

        // Set up ICE handling
        self.setup_ice_handlers(&peer_connection, user_id, signaling_tx.clone())
            .await?;

        // Store participant
        let participant = Participant {
            user_id,
            session_id,
            peer_connection: peer_connection.clone(),
            audio_track: None,
            video_track: None,
            screen_track: None,
            signaling_tx,
        };

        self.participants
            .write()
            .await
            .insert(session_id, participant);

        Ok((peer_connection, signaling_rx))
    }

    /// Remove a participant from the SFU
    pub async fn remove_participant(
        &self,
        session_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut participants = self.participants.write().await;

        if let Some(participant) = participants.remove(&session_id) {
            // Close peer connection
            participant.peer_connection.close().await?;

            // Remove tracks from manager
            self.track_manager
                .remove_participant_tracks(session_id)
                .await;
        }

        Ok(())
    }

    /// Handle offer from client
    pub async fn handle_offer(
        &self,
        session_id: Uuid,
        offer: RTCSessionDescription,
    ) -> Result<RTCSessionDescription, Box<dyn std::error::Error + Send + Sync>> {
        let participants = self.participants.read().await;

        let participant = participants
            .get(&session_id)
            .ok_or("Participant not found")?;

        // Set remote description (the offer)
        participant
            .peer_connection
            .set_remote_description(offer)
            .await?;

        // Create answer
        let answer = participant.peer_connection.create_answer(None).await?;

        // Set local description
        participant
            .peer_connection
            .set_local_description(answer.clone())
            .await?;

        // Wait for ICE gathering to complete (or timeout)
        // In production, you'd want to handle trickle ICE instead
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Get the final answer with ICE candidates
        let final_answer = participant
            .peer_connection
            .local_description()
            .await
            .ok_or("No local description")?;

        Ok(final_answer)
    }

    /// Handle ICE candidate from client
    pub async fn handle_ice_candidate(
        &self,
        session_id: Uuid,
        candidate: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let participants = self.participants.read().await;

        let participant = participants
            .get(&session_id)
            .ok_or("Participant not found")?;

        // Parse and add ICE candidate
        // Note: webrtc-rs expects the candidate in a specific format
        // For now, we'll skip detailed parsing and just forward it
        // In production, use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit

        Ok(())
    }

    /// Build ICE servers from configuration
    fn build_ice_servers(&self) -> Vec<RTCIceServer> {
        let mut servers = vec![];

        // Add STUN servers
        for stun_url in &self.config.stun_servers {
            servers.push(RTCIceServer {
                urls: vec![stun_url.clone()],
                ..Default::default()
            });
        }

        // Add TURN server if enabled
        if self.config.turn_server_enabled {
            servers.push(RTCIceServer {
                urls: vec![self.config.turn_server_url.clone()],
                username: self.config.turn_server_username.clone(),
                credential: self.config.turn_server_credential.clone(),
                ..Default::default()
            });
        }

        servers
    }

    /// Set up track handlers for a peer connection
    async fn setup_track_handlers(
        &self,
        peer_connection: &Arc<RTCPeerConnection>,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let track_manager = self.track_manager.clone();
        let participants = self.participants.clone();

        // Handle incoming tracks
        peer_connection.on_track(Box::new(
            move |track: Arc<TrackRemote>,
                  _receiver: Arc<RTCRtpReceiver>,
                  _transceiver: Arc<RTCRtpTransceiver>| {
                let track_manager = track_manager.clone();
                let participants = participants.clone();
                let session_id = session_id;

                tokio::spawn(async move {
                    // Register the track
                    track_manager
                        .register_track(session_id, track.clone())
                        .await;

                    // Forward track to other participants
                    Self::forward_track(track, track_manager, participants, session_id).await;
                });

                Box::pin(async {})
            },
        ));

        Ok(())
    }

    /// Forward a track to all other participants
    async fn forward_track(
        track: Arc<TrackRemote>,
        track_manager: Arc<TrackManager>,
        participants: Arc<RwLock<HashMap<Uuid, Participant>>>,
        sender_session_id: Uuid,
    ) {
        // Read RTP packets from the track
        let mut rtp_buffer = vec![0u8; 1500];

        loop {
            // Read RTP packet
            match track.read(&mut rtp_buffer).await {
                Ok((packet, _)) => {
                    // Get the packet data length
                    let n = packet.payload.len();
                    if n == 0 {
                        continue;
                    }

                    // Forward to other participants
                    let participants_guard = participants.read().await;

                    for (session_id, participant) in participants_guard.iter() {
                        if *session_id == sender_session_id {
                            continue; // Don't send back to sender
                        }

                        // Forward based on track kind
                        match track.kind() {
                            webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Audio => {
                                if let Some(audio_track) = &participant.audio_track {
                                    // Write RTP packet to track
                                    // This is simplified - in production you'd use a proper RTP writer
                                    let _ = audio_track.write_rtp(&packet).await;
                                }
                            }
                            webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Video => {
                                if let Some(video_track) = &participant.video_track {
                                    // Write RTP packet to track
                                    let _ = video_track.write_rtp(&packet).await;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(_) => {
                    // Track closed or error
                    break;
                }
            }
        }

        // Unregister track when done
        track_manager
            .unregister_track(sender_session_id, &track.id())
            .await;
    }

    /// Set up ICE handlers for a peer connection
    async fn setup_ice_handlers(
        &self,
        peer_connection: &Arc<RTCPeerConnection>,
        _user_id: Uuid,
        signaling_tx: mpsc::UnboundedSender<SignalingMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Clone signaling_tx for each handler since it will be moved into closures
        let signaling_tx_ice = signaling_tx.clone();
        let signaling_tx_ice_state = signaling_tx.clone();
        let signaling_tx_conn_state = signaling_tx;

        // Handle ICE candidates
        peer_connection.on_ice_candidate(Box::new(
            move |candidate: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
                if let Some(candidate) = candidate {
                    // Send ICE candidate to client via signaling
                    let _ = signaling_tx_ice.send(SignalingMessage::IceCandidate {
                        candidate: candidate
                            .to_json()
                            .ok()
                            .map(|j| j.candidate)
                            .unwrap_or_default(),
                    });
                }

                Box::pin(async {})
            },
        ));

        // Handle ICE connection state changes
        peer_connection.on_ice_connection_state_change(Box::new(
            move |state: webrtc::ice_transport::ice_connection_state::RTCIceConnectionState| {
                let _ = signaling_tx_ice_state.send(SignalingMessage::IceConnectionState {
                    state: format!("{:?}", state),
                });
                Box::pin(async {})
            },
        ));

        // Handle peer connection state changes (using ICE connection state as proxy)
        peer_connection.on_peer_connection_state_change(Box::new(
            move |state: webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState| {
                let _ = signaling_tx_conn_state.send(SignalingMessage::ConnectionState {
                    state: format!("{:?}", state),
                });
                Box::pin(async {})
            },
        ));

        Ok(())
    }

    /// Get all participant sessions in the SFU
    pub async fn get_participant_sessions(&self) -> Vec<Uuid> {
        self.participants.read().await.keys().cloned().collect()
    }

    /// Get participant count
    pub async fn get_participant_count(&self) -> usize {
        self.participants.read().await.len()
    }
}

impl Drop for SFU {
    fn drop(&mut self) {
        // Cleanup when SFU is dropped
        // In production, you'd want to properly close all peer connections
    }
}
