//! Bedrock-layer packet handling (login capture, network settings, responses).

use tracing::{debug, info, warn};

use super::RakNetServer;

impl RakNetServer {
    /// Process a single Bedrock packet (already decompressed)
    pub(super) async fn process_single_bedrock_packet(
        &self,
        data: &[u8],
        from: std::net::SocketAddr,
    ) {
        if data.is_empty() {
            return;
        }

        if !self.is_active_connection(from).await {
            debug!("Ignoring Bedrock packet from closed session {}", from);
            return;
        }

        use crate::util::Buffer;
        let mut buf = Buffer::from(data);

        // Try to read packet ID as VarInt first
        let packet_id = match buf.read_var_int() {
            Ok(id) => {
                debug!("VarInt packet ID: 0x{:02X} from {}", id, from);
                id
            }
            Err(_) => {
                // Fallback: use first byte as packet ID (for direct protocol packets)
                let id = data[0] as u32;
                debug!("Direct packet ID: 0x{:02X} from {}", id, from);
                id
            }
        };

        // Handle specific packet types
        match packet_id {
            0x01 => {
                // LOGIN packet (0x01)
                debug!("LOGIN packet from {}", from);
                self.handle_login_packet(&mut buf, from).await;
            }
            0x02 => {
                // PLAY_STATUS packet (0x02) - typically sent by server
                debug!("PLAY_STATUS packet from {}", from);
            }
            0x03 => {
                // SERVER_TO_CLIENT_HANDSHAKE
                debug!("SERVER_TO_CLIENT_HANDSHAKE (0x03) received from {}", from);
            }
            0x04 => {
                // CLIENT_TO_SERVER_HANDSHAKE
                debug!("CLIENT_TO_SERVER_HANDSHAKE (0x04) received from {}", from);
            }
            0x08 => {
                // RESOURCE_PACK_CLIENT_RESPONSE (0x08)
                debug!("RESOURCE_PACK_CLIENT_RESPONSE from {}", from);
                self.handle_resource_pack_client_response(&mut buf, from).await;
            }
            0x09 => {
                // Text packet (chat)
                debug!("Text packet from {}", from);
                self.handle_text_packet(&mut buf, from).await;
            }
            0x17 => {
                // TICK_SYNC (0x17)
                debug!("TICK_SYNC from {}", from);
                self.handle_tick_sync(&mut buf, from).await;
            }
            0x45 | 0x61 => {
                // REQUEST_CHUNK_RADIUS (0x45 or 0x61 depending on protocol version)
                debug!("REQUEST_CHUNK_RADIUS (0x{:02X}) from {}", packet_id, from);
                self.handle_request_chunk_radius(&mut buf, from).await;
            }
            0x71 => {
                // SET_LOCAL_PLAYER_AS_INITIALIZED (0x71)
                debug!("SET_LOCAL_PLAYER_AS_INITIALIZED from {}", from);
                self.handle_player_initialized(from).await;
            }
            0xc1 => {
                // REQUEST_NETWORK_SETTINGS (0xc1)
                let proto = buf
                    .read_u32()
                    .unwrap_or_else(|_| self.effective_protocol_version() as u32);
                let ver = crate::bedrock::resolve_version(proto);
                info!("RequestNetworkSettings proto={} ({})", proto, ver);

                {
                    let mut conns = self.connections.write().await;
                    if let Some(session) = conns.get_mut(&from.to_string()) {
                        let was_enabled = session.compression_enabled;
                        session.bedrock_protocol = proto as u16;
                        session.bedrock_version = ver.clone();
                        // Mark compression as negotiated immediately
                        session.compression_enabled = true;
                        session.compression_algo = None;
                        tracing::debug!(
                            "Compression state update on C1: {} -> {} (from={})",
                            was_enabled,
                            session.compression_enabled,
                            from
                        );
                    }
                }

                self.send_network_settings_response(from).await;
                info!("Sent NetworkSettings to {}", from);
            }
            _ => {
                debug!("Unhandled Bedrock packet: 0x{:02X}", packet_id);
            }
        }
    }

    /// Handle LOGIN packet (0x01)
    pub(super) async fn handle_login_packet(
        &self,
        buf: &mut crate::util::Buffer,
        from: std::net::SocketAddr,
    ) {
        let remaining = buf.read_remaining();
        match crate::bedrock::login::parse_login_packet(&remaining) {
            Ok(Some(parsed)) => {
                info!("Handling Bedrock Login Packet...");
                info!("Client Protocol: {}", parsed.protocol);
                info!("Detected Player: {}", parsed.player_name);
                
                self.sync_client_version(parsed.protocol as u16);

                let session_key = from.to_string();
                let version = crate::bedrock::resolve_version(parsed.protocol);
                
                {
                    let mut conns = self.connections.write().await;
                    if let Some(session) = conns.get_mut(&session_key) {
                        session.username = Some(parsed.player_name.clone());
                        session.bedrock_protocol = parsed.protocol as u16;
                        session.bedrock_version = version.clone();
                        session.connected = true;
                    }
                }

                info!(
                    "LOGIN metadata: xuid={:?} device_os={:?} device_model={:?} playfab_id={:?}",
                    parsed.metadata.xuid,
                    parsed.metadata.device_os,
                    parsed.metadata.device_model,
                    parsed.metadata.playfab_id,
                );

                // Instead of disconnecting immediately, send PlayStatus(Success) + ResourcePacksInfo
                self.send_login_success_sequence(from).await;
            }
            Ok(None) => {
                warn!("Login packet from {} did not match any known layout", from);
            }
            Err(e) => {
                warn!("Login packet parse error from {}: {}", from, e);
            }
        }
    }

    /// Resolve the Bedrock protocol version a client negotiated during login.
    async fn session_protocol(&self, addr: &std::net::SocketAddr) -> u32 {
        let conns = self.connections.read().await;
        conns
            .get(&addr.to_string())
            .map(|s| s.bedrock_protocol as u32)
            .unwrap_or_else(|| self.effective_protocol_version() as u32)
    }

    /// Process RESOURCE_PACK_CLIENT_RESPONSE (0x08)
    pub(super) async fn handle_resource_pack_client_response(
        &self,
        buf: &mut crate::util::Buffer,
        from: std::net::SocketAddr,
    ) {
        let proto = self.session_protocol(&from).await;
        let status = buf.read_var_int().unwrap_or(0);

        if proto >= 2168 {
            // Protocol 2168+: VarUInt(status) + VarString(name)
            let name = buf.read_var_string().unwrap_or_default();
            info!(
                "ResourcePackClientResponse status={} ({}) from {}",
                status, name, from
            );
        } else {
            // Legacy (pre-2168): single byte status
            info!(
                "ResourcePackClientResponse (legacy) status={} from {}",
                status, from
            );
        }

        // Enum mapping differs by protocol era:
        //   >= 2168: 2 = DownloadingFinished, 3 = StackFinished
        //   < 2168:  3 = AllPacksDownloaded, 4 = Completed
        let (send_stack, send_start) = if proto >= 2168 {
            (status == 2, status == 3)
        } else {
            (status == 3, status == 4)
        };

        if send_stack {
            // Server sends ResourcePackStack (0x07)
            self.send_resource_pack_stack(from).await;
        } else if send_start {
            // Server sends StartGame (0x0b), BiomeDefinitionList (0x7a), AvailableEntityIdentifiers (0x77), PlayStatus(Spawned=3)
            self.send_start_game_sequence(from).await;
        } else {
            warn!("Unhandled ResourcePackClientResponse status={} from {}", status, from);
        }
    }

    /// Handle client RequestChunkRadius (0x45 or 0x61)
    pub(super) async fn handle_request_chunk_radius(
        &self,
        buf: &mut crate::util::Buffer,
        from: std::net::SocketAddr,
    ) {
        let radius = buf.read_var_int().unwrap_or(4);
        info!("RequestChunkRadius: radius={} from {}", radius, from);

        // Send ChunkRadiusUpdated (0x46)
        let radius_updated = crate::bedrock::build_chunk_radius_updated(radius as i32);
        self.send_bedrock_response(&radius_updated, from, false).await;

        // Send NetworkChunkPublisherUpdate (0x79)
        let publisher_update = crate::bedrock::build_network_chunk_publisher_update(0, 4, 0, radius as u32 * 16);
        self.send_bedrock_response(&publisher_update, from, false).await;

        // Send a few flat chunk packets around (0,0) so terrain builds successfully
        for x in -1..=1 {
            for z in -1..=1 {
                let chunk = crate::bedrock::build_level_chunk(x, z);
                self.send_bedrock_response(&chunk, from, false).await;
            }
        }
        info!("Sent basic Overworld flat chunks to {}", from);

        // Per vanilla flow: PlayStatus(PlayerSpawn) + CreativeContent after chunks are sent
        let play_status = crate::bedrock::build_play_status(3); // 3 = PlayerSpawn
        self.send_bedrock_response(&play_status, from, false).await;
        let creative = crate::bedrock::build_creative_content();
        self.send_bedrock_response(&creative, from, false).await;
    }

    /// Handle TickSync (0x17)
    pub(super) async fn handle_tick_sync(
        &self,
        buf: &mut crate::util::Buffer,
        from: std::net::SocketAddr,
    ) {
        let req_time = buf.read_u64_le().unwrap_or(0);
        let resp_time = buf.read_u64_le().unwrap_or(0);

        let mut res = crate::util::Buffer::new();
        let _ = res.write_var_int(0x17);
        let _ = res.write_u64_le(req_time);
        let _ = res.write_u64_le(resp_time);

        self.send_bedrock_response(res.as_slice(), from, false).await;
    }

    /// Handle SetLocalPlayerAsInitialized (0x71)
    pub(super) async fn handle_player_initialized(&self, from: std::net::SocketAddr) {
        let username = {
            let conns = self.connections.read().await;
            conns.get(&from.to_string())
                .and_then(|s| s.username.clone())
                .unwrap_or_else(|| "Player".to_string())
        };

        info!("Player initialized: {} ({})", username, from);

        // Broadcast join message to everyone
        let join_msg = format!("{} joined AndroServeMC!", username);
        let chat_packet = crate::bedrock::build_text_packet("Server", &join_msg);
        self.broadcast_bedrock_packet(&chat_packet, None).await;
    }

    /// Handle client text packet (0x09)
    pub(super) async fn handle_text_packet(
        &self,
        buf: &mut crate::util::Buffer,
        from: std::net::SocketAddr,
    ) {
        let msg_type = buf.read_u8().unwrap_or(0);
        let _needs_translation = buf.read_u8().unwrap_or(0) != 0;

        if msg_type == 1 || msg_type == 2 {
            let sender = buf.read_var_string().unwrap_or_default();
            let message = buf.read_var_string().unwrap_or_default();

            info!("<{}> {}", sender, message);

            // Broadcast message to everyone else
            let chat_packet = crate::bedrock::build_text_packet(&sender, &message);
            self.broadcast_bedrock_packet(&chat_packet, Some(from)).await;
        }
    }

    /// Broadcast Bedrock response to all active clients
    pub(super) async fn broadcast_bedrock_packet(
        &self,
        packet: &[u8],
        exclude: Option<std::net::SocketAddr>,
    ) {
        let conns = self.connections.read().await;
        for (addr_str, session) in conns.iter() {
            if !session.connected {
                continue;
            }
            if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                if let Some(exc) = exclude {
                    if addr == exc {
                        continue;
                    }
                }
                self.send_bedrock_response(packet, addr, false).await;
            }
        }
    }

    /// Send PlayStatus + ResourcePacksInfo
    pub(super) async fn send_login_success_sequence(&self, to: std::net::SocketAddr) {
        let play_status = crate::bedrock::build_play_status(0); // 0 = Success
        let proto = self.session_protocol(&to).await;
        let packs_info = crate::bedrock::build_resource_packs_info(proto);

        self.send_bedrock_bundle(&[play_status, packs_info], to).await;
        info!("Sent login success bundle (PlayStatus + ResourcePacksInfo) to {}", to);
    }

    /// Send ResourcePackStack
    pub(super) async fn send_resource_pack_stack(&self, to: std::net::SocketAddr) {
        let version = self.effective_version();
        let pack_stack = crate::bedrock::build_resource_pack_stack(&version);
        self.send_bedrock_response(&pack_stack, to, false).await;
        info!("Sent ResourcePackStack to {}", to);
    }

    /// Send StartGame sequence
    pub(super) async fn send_start_game_sequence(&self, to: std::net::SocketAddr) {
        let version = self.effective_version();
        let start_game = crate::bedrock::build_start_game(&version);
        let item_registry = crate::bedrock::build_item_registry();
        let biomes = crate::bedrock::build_biome_definitions();
        let entities = crate::bedrock::build_entity_identifiers();

        self.send_bedrock_bundle(&[start_game, item_registry, biomes, entities], to).await;
        info!("Sent game start sequence bundle (StartGame, ItemRegistry, Biomes, Entities) to {}", to);
    }

    pub(super) async fn send_bedrock_bundle(&self, packets: &[Vec<u8>], to: std::net::SocketAddr) {
        use crate::util::Buffer;

        let compression_enabled = {
            let conns = self.connections.read().await;
            conns
                .get(&to.to_string())
                .map(|s| s.compression_enabled)
                .unwrap_or(false)
        };

        let mut combined = Buffer::new();
        for packet in packets {
            if combined.write_var_int(packet.len() as u32).is_err() {
                return;
            }
            if combined.write_bytes(packet).is_err() {
                return;
            }
        }

        let mut body = vec![0xfe];
        if compression_enabled {
            match crate::bedrock::compress_batch(combined.as_slice()) {
                Ok(mut compressed) => body.append(&mut compressed),
                Err(e) => {
                    warn!("Failed to compress login bundle: {}", e);
                    body.push(0x00);
                    body.extend_from_slice(combined.as_slice());
                }
            }
        } else {
            body.extend_from_slice(combined.as_slice());
        }

        let _ = self.send_frame(&body, 3, true, to).await;
    }

    /// Send PLAY_STATUS response
    #[allow(dead_code)]
    pub(super) async fn send_play_status_response(&self, status: u32, to: std::net::SocketAddr) {
        let response = crate::bedrock::build_play_status(status);
        debug!("Sending PLAY_STATUS (status={}) to {}", status, to);
        self.send_bedrock_response(&response, to, false).await;
    }

    /// Send DISCONNECT response
    #[allow(dead_code)]
    pub(super) async fn send_disconnect_response(
        &self,
        reason: i32,
        message: &str,
        to: std::net::SocketAddr,
    ) {
        let response = crate::bedrock::build_disconnect(reason, message);
        debug!("Sending DISCONNECT (reason={}) to {}", reason, to);
        self.send_bedrock_response(&response, to, false).await;
    }

    /// Send RakNet DISCONNECTION_NOTIFICATION (0x15)
    #[allow(dead_code)]
    pub(super) async fn send_raknet_disconnect_notification(&self, to: std::net::SocketAddr) {
        debug!("Sending RakNet DISCONNECTION_NOTIFICATION to {}", to);
        let _ = self.send_frame(&[0x15], 3, true, to).await;
    }

    /// Send NETWORK_SETTINGS response
    pub(super) async fn send_network_settings_response(&self, to: std::net::SocketAddr) {
        let response = crate::bedrock::build_network_settings();
        debug!("Sending NETWORK_SETTINGS to {}", to);
        self.send_bedrock_response(&response, to, true).await;
    }

    /// Send a Bedrock response (wraps in RakNet Batch frame)
    pub(super) async fn send_bedrock_response(
        &self,
        packet: &[u8],
        to: std::net::SocketAddr,
        force_uncompressed: bool,
    ) {
        // Decide per-session whether compression has been negotiated
        let compression_enabled = if force_uncompressed {
            false
        } else {
            let conns = self.connections.read().await;
            conns
                .get(&to.to_string())
                .map(|s| s.compression_enabled)
                .unwrap_or(false)
        };

        // Helper to wrap a sub-packet with VarInt length prefix
        fn wrap_with_length(packet: &[u8]) -> Vec<u8> {
            let mut buf = Vec::new();
            let mut len = packet.len();
            loop {
                let mut byte = (len & 0x7F) as u8;
                len >>= 7;
                if len != 0 {
                    byte |= 0x80;
                }
                buf.push(byte);
                if len == 0 {
                    break;
                }
            }
            buf.extend_from_slice(packet);
            buf
        }

        let wrapped = wrap_with_length(packet);

        if !compression_enabled {
            // Send raw (no compression) with length-prefixed subpacket
            let mut batch = vec![0xfe];
            batch.extend_from_slice(&wrapped);
            debug!(
                "Sending Batch (uncompressed) to {} ({} bytes)",
                to,
                packet.len()
            );
            self.send_frame(&batch, 3, true, to).await.ok();
            return;
        }

        // Compression negotiated: wrap then compress
        match crate::bedrock::compress_batch(&wrapped) {
            Ok(mut compressed) => {
                let mut batch = vec![0xfe];
                batch.append(&mut compressed);
                debug!(
                    "Sending Batch (compressed) to {} (original: {} bytes, compressed: {} bytes)",
                    to,
                    packet.len(),
                    batch.len() - 1
                );
                self.send_frame(&batch, 3, true, to).await.ok();
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to compress Bedrock packet: {}. Sending uncompressed.",
                    e
                );
                let mut batch = vec![0xfe];
                batch.extend_from_slice(&wrapped);
                self.send_frame(&batch, 3, true, to).await.ok();
            }
        }
    }
}
