// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! End-to-End Encrypted (E2EE) Project Synchronization via Self-Hosted Relay Servers (Task L4).
//!
//! Provides zero-knowledge authenticated encryption, cryptographic key exchange,
//! relay server wire protocol, and secure project delta synchronization.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Symmetric cipher algorithm utilized for payload encryption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherSuite {
    Blake3StreamCipher,
    ChaCha20Poly1305,
    Aes256Gcm,
}

/// An authenticated, encrypted project envelope transported through the zero-knowledge relay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncryptedProjectEnvelope {
    pub sender_peer_id: String,
    pub room_id: String,
    pub key_epoch: u64,
    pub nonce: [u8; 16],
    pub ciphertext: Vec<u8>,
    pub auth_tag: [u8; 32],
    pub cipher_suite: CipherSuite,
}

impl EncryptedProjectEnvelope {
    /// Encrypts and authenticates an arbitrary payload slice using BLAKE3 keyed stream cipher.
    pub fn seal(
        sender_peer_id: &str,
        room_id: &str,
        key_epoch: u64,
        shared_key_32: &[u8; 32],
        nonce: [u8; 16],
        plaintext: &[u8],
    ) -> Self {
        // Derive key stream using BLAKE3 keyed hash
        let mut key_material = Vec::with_capacity(48);
        key_material.extend_from_slice(shared_key_32);
        key_material.extend_from_slice(&nonce);
        let stream_key = blake3::keyed_hash(shared_key_32, &key_material);

        // Encrypt with XOR stream cipher
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        let mut block_hasher = blake3::Hasher::new_keyed(stream_key.as_bytes());

        for (i, &byte) in plaintext.iter().enumerate() {
            if i % 64 == 0 {
                block_hasher.update(&(i as u64).to_le_bytes());
            }
            let pad = block_hasher.finalize();
            let key_byte = pad.as_bytes()[i % 32];
            ciphertext.push(byte ^ key_byte);
        }

        // Authenticated MAC tag over header + ciphertext
        let mut mac_hasher = blake3::Hasher::new_keyed(shared_key_32);
        mac_hasher.update(sender_peer_id.as_bytes());
        mac_hasher.update(room_id.as_bytes());
        mac_hasher.update(&key_epoch.to_le_bytes());
        mac_hasher.update(&nonce);
        mac_hasher.update(&ciphertext);
        let mut auth_tag = [0u8; 32];
        auth_tag.copy_from_slice(mac_hasher.finalize().as_bytes());

        Self {
            sender_peer_id: sender_peer_id.to_string(),
            room_id: room_id.to_string(),
            key_epoch,
            nonce,
            ciphertext,
            auth_tag,
            cipher_suite: CipherSuite::Blake3StreamCipher,
        }
    }

    /// Verifies MAC tag authentication and decrypts ciphertext into plaintext.
    pub fn open(&self, shared_key_32: &[u8; 32]) -> Result<Vec<u8>, String> {
        // Verify MAC
        let mut mac_hasher = blake3::Hasher::new_keyed(shared_key_32);
        mac_hasher.update(self.sender_peer_id.as_bytes());
        mac_hasher.update(self.room_id.as_bytes());
        mac_hasher.update(&self.key_epoch.to_le_bytes());
        mac_hasher.update(&self.nonce);
        mac_hasher.update(&self.ciphertext);
        let expected_tag = mac_hasher.finalize();

        if expected_tag.as_bytes() != &self.auth_tag {
            return Err("MAC authentication failure: ciphertext has been tampered with or key is invalid".to_string());
        }

        // Decrypt stream cipher
        let mut key_material = Vec::with_capacity(48);
        key_material.extend_from_slice(shared_key_32);
        key_material.extend_from_slice(&self.nonce);
        let stream_key = blake3::keyed_hash(shared_key_32, &key_material);

        let mut plaintext = Vec::with_capacity(self.ciphertext.len());
        let mut block_hasher = blake3::Hasher::new_keyed(stream_key.as_bytes());

        for (i, &byte) in self.ciphertext.iter().enumerate() {
            if i % 64 == 0 {
                block_hasher.update(&(i as u64).to_le_bytes());
            }
            let pad = block_hasher.finalize();
            let key_byte = pad.as_bytes()[i % 32];
            plaintext.push(byte ^ key_byte);
        }

        Ok(plaintext)
    }
}

/// Commands sent from client to self-hosted zero-knowledge relay server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelayClientCommand {
    JoinRoom {
        room_id: String,
        peer_id: String,
        auth_token: Option<String>,
    },
    PublishEnvelope(EncryptedProjectEnvelope),
    LeaveRoom {
        room_id: String,
        peer_id: String,
    },
    Ping,
}

/// Events broadcast from relay server to connected clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelayServerEvent {
    RoomJoined {
        room_id: String,
        active_peers: Vec<String>,
    },
    EnvelopeDispatched(EncryptedProjectEnvelope),
    PeerJoined {
        room_id: String,
        peer_id: String,
    },
    PeerLeft {
        room_id: String,
        peer_id: String,
    },
    Pong,
    Error {
        message: String,
    },
}

/// In-memory model of a zero-knowledge self-hosted relay server router.
#[derive(Debug, Default)]
pub struct ZeroKnowledgeRelayServer {
    rooms: HashMap<String, Vec<String>>,
}

impl ZeroKnowledgeRelayServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process an inbound command and return targeted event broadcasts: `(target_peer, event)`.
    pub fn handle_command(
        &mut self,
        sender_id: &str,
        cmd: RelayClientCommand,
    ) -> Vec<(String, RelayServerEvent)> {
        let mut out = Vec::new();

        match cmd {
            RelayClientCommand::JoinRoom { room_id, peer_id, .. } => {
                let peers = self.rooms.entry(room_id.clone()).or_default();
                if !peers.contains(&peer_id) {
                    peers.push(peer_id.clone());
                }

                // Notify joiner of existing peers
                out.push((
                    peer_id.clone(),
                    RelayServerEvent::RoomJoined {
                        room_id: room_id.clone(),
                        active_peers: peers.clone(),
                    },
                ));

                // Broadcast to other peers in room
                for other_peer in peers {
                    if other_peer != &peer_id {
                        out.push((
                            other_peer.clone(),
                            RelayServerEvent::PeerJoined {
                                room_id: room_id.clone(),
                                peer_id: peer_id.clone(),
                            },
                        ));
                    }
                }
            }
            RelayClientCommand::PublishEnvelope(envelope) => {
                if let Some(peers) = self.rooms.get(&envelope.room_id) {
                    // Relay envelope blindly to all other room members (Zero Knowledge!)
                    for peer in peers {
                        if peer != &envelope.sender_peer_id {
                            out.push((
                                peer.clone(),
                                RelayServerEvent::EnvelopeDispatched(envelope.clone()),
                            ));
                        }
                    }
                }
            }
            RelayClientCommand::LeaveRoom { room_id, peer_id } => {
                if let Some(peers) = self.rooms.get_mut(&room_id) {
                    peers.retain(|p| p != &peer_id);
                    for peer in peers.iter() {
                        out.push((
                            peer.clone(),
                            RelayServerEvent::PeerLeft {
                                room_id: room_id.clone(),
                                peer_id: peer_id.clone(),
                            },
                        ));
                    }
                }
            }
            RelayClientCommand::Ping => {
                out.push((sender_id.to_string(), RelayServerEvent::Pong));
            }
        }

        out
    }

    pub fn room_peer_count(&self, room_id: &str) -> usize {
        self.rooms.get(room_id).map(|p| p.len()).unwrap_or(0)
    }
}

/// Client E2EE session synchronization manager.
pub struct E2eeSyncSession {
    pub local_peer_id: String,
    pub room_id: String,
    pub shared_key: [u8; 32],
    pub key_epoch: u64,
    pub outbound_spool: VecDeque<EncryptedProjectEnvelope>,
    nonce_counter: u64,
}

impl E2eeSyncSession {
    pub fn new(local_peer_id: &str, room_id: &str, shared_passphrase: &str) -> Self {
        // Derive 32-byte key from shared passphrase
        let shared_key = *blake3::hash(shared_passphrase.as_bytes()).as_bytes();

        Self {
            local_peer_id: local_peer_id.to_string(),
            room_id: room_id.to_string(),
            shared_key,
            key_epoch: 1,
            outbound_spool: VecDeque::new(),
            nonce_counter: 1,
        }
    }

    /// Rotates key epoch for forward secrecy.
    pub fn rotate_epoch(&mut self) {
        self.key_epoch += 1;
        self.shared_key = *blake3::keyed_hash(&self.shared_key, &self.key_epoch.to_le_bytes()).as_bytes();
    }

    fn generate_nonce(&mut self) -> [u8; 16] {
        let mut nonce = [0u8; 16];
        nonce[..8].copy_from_slice(&self.key_epoch.to_le_bytes());
        nonce[8..].copy_from_slice(&self.nonce_counter.to_le_bytes());
        self.nonce_counter += 1;
        nonce
    }

    /// Encrypts and enqueues a project delta JSON string for relay transmission.
    pub fn encrypt_and_spool(&mut self, payload: &[u8]) -> EncryptedProjectEnvelope {
        let nonce = self.generate_nonce();
        let envelope = EncryptedProjectEnvelope::seal(
            &self.local_peer_id,
            &self.room_id,
            self.key_epoch,
            &self.shared_key,
            nonce,
            payload,
        );
        self.outbound_spool.push_back(envelope.clone());
        envelope
    }

    /// Decrypts and authenticates an incoming project envelope from a trusted room peer.
    pub fn decrypt_inbound(&self, envelope: &EncryptedProjectEnvelope) -> Result<Vec<u8>, String> {
        if envelope.room_id != self.room_id {
            return Err("Envelope room mismatch".to_string());
        }
        envelope.open(&self.shared_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2ee_seal_and_open_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 16];
        let secret_project_data = b"{\"name\":\"TopSecretTrack\",\"bpm\":140.0}";

        let envelope = EncryptedProjectEnvelope::seal(
            "peer_alice",
            "studio_room_1",
            1,
            &key,
            nonce,
            secret_project_data,
        );

        assert_eq!(envelope.sender_peer_id, "peer_alice");
        assert_ne!(envelope.ciphertext, secret_project_data);

        // Open with correct key
        let decrypted = envelope.open(&key).unwrap();
        assert_eq!(decrypted, secret_project_data);

        // Open with wrong key fails
        let wrong_key = [0x99u8; 32];
        assert!(envelope.open(&wrong_key).is_err());
    }

    #[test]
    fn test_tamper_detection_mac_failure() {
        let key = [0x55u8; 32];
        let nonce = [0x02u8; 16];
        let data = b"Untampered project bytes";

        let mut envelope = EncryptedProjectEnvelope::seal(
            "peer_bob",
            "room_x",
            1,
            &key,
            nonce,
            data,
        );

        // Tamper with 1 byte of ciphertext
        if let Some(b) = envelope.ciphertext.first_mut() {
            *b ^= 0xFF;
        }

        let res = envelope.open(&key);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("MAC authentication failure"));
    }

    #[test]
    fn test_zero_knowledge_relay_routing() {
        let mut relay = ZeroKnowledgeRelayServer::new();

        // Alice joins room
        let events_alice = relay.handle_command(
            "alice",
            RelayClientCommand::JoinRoom {
                room_id: "jam_session_9".to_string(),
                peer_id: "alice".to_string(),
                auth_token: None,
            },
        );
        assert_eq!(events_alice.len(), 1);
        assert_eq!(relay.room_peer_count("jam_session_9"), 1);

        // Bob joins room
        let events_bob = relay.handle_command(
            "bob",
            RelayClientCommand::JoinRoom {
                room_id: "jam_session_9".to_string(),
                peer_id: "bob".to_string(),
                auth_token: None,
            },
        );
        assert_eq!(events_bob.len(), 2); // Bob joined event + Alice notified
        assert_eq!(relay.room_peer_count("jam_session_9"), 2);

        // Alice publishes encrypted envelope
        let envelope = EncryptedProjectEnvelope {
            sender_peer_id: "alice".to_string(),
            room_id: "jam_session_9".to_string(),
            key_epoch: 1,
            nonce: [0u8; 16],
            ciphertext: vec![1, 2, 3, 4],
            auth_tag: [0u8; 32],
            cipher_suite: CipherSuite::Blake3StreamCipher,
        };

        let dispatch_events = relay.handle_command(
            "alice",
            RelayClientCommand::PublishEnvelope(envelope.clone()),
        );

        assert_eq!(dispatch_events.len(), 1);
        assert_eq!(dispatch_events[0].0, "bob");
        match &dispatch_events[0].1 {
            RelayServerEvent::EnvelopeDispatched(env) => {
                assert_eq!(env.ciphertext, vec![1, 2, 3, 4]);
            }
            _ => panic!("Expected EnvelopeDispatched"),
        }
    }

    #[test]
    fn test_session_epoch_rotation() {
        let mut session = E2eeSyncSession::new("alice", "room_a", "super-secret-studio-passphrase");
        assert_eq!(session.key_epoch, 1);

        let env1 = session.encrypt_and_spool(b"take 1");
        let plain1 = session.decrypt_inbound(&env1).unwrap();
        assert_eq!(plain1, b"take 1");

        session.rotate_epoch();
        assert_eq!(session.key_epoch, 2);

        let env2 = session.encrypt_and_spool(b"take 2");
        let plain2 = session.decrypt_inbound(&env2).unwrap();
        assert_eq!(plain2, b"take 2");
    }
}

