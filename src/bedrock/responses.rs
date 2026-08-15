//! Bedrock response packet builders

use crate::util::Buffer;

/// Build PlayStatus packet (0x02)
/// Bedrock packet format: VarInt(packet_id) + u32BE(status)
pub fn build_play_status(status: u32) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x02); // PlayStatus packet ID (VarInt encoded)
    let _ = buf.write_u32(status); // status code (Big Endian)
    buf.to_vec()
}

/// Build Disconnect packet (0x05)
/// Format: VarInt(packet_id) + SignedVarInt(reason) + Bool(hideReason) + VarString(message) + VarString(filteredMessage)
pub fn build_disconnect(reason: i32, message: &str) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x05); // Disconnect packet ID (VarInt encoded)
    let _ = buf.write_signed_var_int(reason); // reason code
    let _ = buf.write_bool(false); // hideReason
    let _ = buf.write_var_string(message); // message
    let _ = buf.write_var_string(message); // filteredMessage (same as message)
    buf.to_vec()
}

/// Build NetworkSettings packet (0x8f)
/// Format: VarInt(packet_id) + LShort(compression_threshold) + LShort(compression_algo) + Bool(throttle) + Byte(throttle_threshold) + LFloat(throttle_rate)
pub fn build_network_settings() -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x8f); // NetworkSettings packet ID (VarInt encoded)
    let _ = buf.write_u16_le(1); // compression threshold (LE short)
    let _ = buf.write_u16_le(0); // compression algorithm (0 = zlib)
    let _ = buf.write_bool(false); // client throttle enabled
    let _ = buf.write_u8(0); // client throttle threshold
    let _ = buf.write_f32_le(0.0); // client throttle rate
    buf.to_vec()
}

/// Build ResourcePacksInfo packet (0x06), adapting to the client's protocol version.
/// - < 766 (pre 1.21.50): Bool(required) + Bool(has_addons) + Bool(has_scripts) + U16(pack_count)
/// - 766..2167: + Bool(force_disable_vibrant_visuals) + U64 x 2 (world template UUID) + VarString(world template version) + U16(pack_count)
/// - >= 2168: same as 766..2167 but the pack count is encoded as VarUInt
pub fn build_resource_packs_info(protocol: u32) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x06); // ResourcePacksInfo packet ID
    let _ = buf.write_bool(false); // must_accept
    let _ = buf.write_bool(false); // has_addons
    let _ = buf.write_bool(false); // has_scripts
    if protocol >= 766 {
        let _ = buf.write_bool(false); // force_disable_vibrant_visuals
        let _ = buf.write_u64(0); // world template UUID (most significant)
        let _ = buf.write_u64(0); // world template UUID (least significant)
        let _ = buf.write_var_string(""); // world template version
    }
    if protocol >= 2168 {
        let _ = buf.write_var_int(0); // resource pack count (varuint)
    } else {
        let _ = buf.write_u16_le(0); // resource pack count (u16)
    }
    buf.to_vec()
}

/// Build ResourcePackStack packet (0x07) - protocol 2168 format
/// Format: VarInt(0x07) + Bool(must_accept) + VarUInt(pack_stack_count)
///         + VarString(base_game_version) + U32LE(experiments_count)
///         + Bool(experiments_previously_toggled) + Bool(include_editor_packs)
pub fn build_resource_pack_stack(game_version: &str) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x07); // ResourcePackStack packet ID
    let _ = buf.write_bool(false); // must_accept
    let _ = buf.write_var_int(0); // resource pack stack count
    let _ = buf.write_var_string(game_version); // base game version
    let _ = buf.write_u32_le(0); // experiments count
    let _ = buf.write_bool(false); // experiments previously toggled
    let _ = buf.write_bool(false); // include editor packs
    buf.to_vec()
}

/// Build StartGame packet (0x0b) - protocol 2168 format
pub fn build_start_game(game_version: &str) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x0b); // StartGame packet ID

    // Entity Unique ID (Varint64, zigzag)
    let _ = buf.write_signed_var_i64(1);
    // Entity Runtime ID (Varuint64)
    let _ = buf.write_var_u64(1);

    let _ = buf.write_signed_var_int(1); // player gamemode (1 = Creative)

    // Player position (Vec3 float LE)
    let _ = buf.write_f32_le(0.0); // X
    let _ = buf.write_f32_le(4.0); // Y
    let _ = buf.write_f32_le(0.0); // Z

    // Pitch, Yaw (floats LE)
    let _ = buf.write_f32_le(0.0);
    let _ = buf.write_f32_le(0.0);

    // World settings
    let _ = buf.write_u64_le(12345); // seed (Int64 LE)
    let _ = buf.write_u16_le(0); // spawn biome type (Int16)
    let _ = buf.write_var_string("plains"); // custom biome name
    let _ = buf.write_signed_var_int(0); // dimension (0 = Overworld)
    let _ = buf.write_signed_var_int(1); // generator (1 = infinite)
    let _ = buf.write_signed_var_int(1); // world game mode (1 = Creative)
    let _ = buf.write_bool(false); // hardcore
    let _ = buf.write_signed_var_int(1); // difficulty (1 = Normal)

    // Spawn position (BlockPos: 3x VarInt32, zigzag)
    let _ = buf.write_signed_var_int(0); // X
    let _ = buf.write_signed_var_int(4); // Y
    let _ = buf.write_signed_var_int(0); // Z

    let _ = buf.write_bool(false); // achievements disabled
    let _ = buf.write_signed_var_int(0); // editor world type
    let _ = buf.write_bool(false); // created in editor
    let _ = buf.write_bool(false); // exported from editor
    let _ = buf.write_signed_var_int(0); // day cycle lock time
    let _ = buf.write_var_int(0); // education edition offer
    let _ = buf.write_bool(false); // education features enabled
    let _ = buf.write_var_string(""); // education product ID
    let _ = buf.write_f32_le(0.0); // rain level
    let _ = buf.write_f32_le(0.0); // lightning level
    let _ = buf.write_bool(false); // confirmed platform locked content
    let _ = buf.write_bool(true); // multiplayer game
    let _ = buf.write_bool(true); // LAN broadcast enabled
    let _ = buf.write_signed_var_int(4); // XBL broadcast mode
    let _ = buf.write_signed_var_int(4); // platform broadcast mode
    let _ = buf.write_bool(true); // commands enabled
    let _ = buf.write_bool(false); // texture pack required

    // GameRules (VarUInt count, then rules)
    let _ = buf.write_var_int(0); // 0 gamerules

    // Experiments (u32 LE count, then experiments)
    let _ = buf.write_u32_le(0); // 0 experiments
    let _ = buf.write_bool(false); // experiments run previously

    let _ = buf.write_bool(false); // bonus chest
    let _ = buf.write_bool(false); // start with map
    let _ = buf.write_u8(1); // player permissions (1 = Member)
    let _ = buf.write_u32_le(4); // server chunk tick range (Int32 LE)
    let _ = buf.write_bool(false); // behavior pack locked
    let _ = buf.write_bool(false); // resource pack locked
    let _ = buf.write_bool(false); // from locked world template
    let _ = buf.write_bool(false); // MSA gamer tags only
    let _ = buf.write_bool(false); // from world template
    let _ = buf.write_bool(false); // world template settings locked
    let _ = buf.write_bool(false); // only spawn v1 villagers
    let _ = buf.write_bool(false); // persona disabled
    let _ = buf.write_bool(false); // custom skins disabled
    let _ = buf.write_bool(false); // emote chat muted
    let _ = buf.write_var_string(game_version); // base game version
    let _ = buf.write_u32_le(0); // limited world width (Int32 LE)
    let _ = buf.write_u32_le(0); // limited world depth (Int32 LE)
    let _ = buf.write_bool(true); // new nether
    let _ = buf.write_var_string(""); // education resource button name
    let _ = buf.write_var_string(""); // education resource link URI
    let _ = buf.write_bool(false); // force experimental gameplay
    let _ = buf.write_u8(0); // chat restriction level (Uint8)
    let _ = buf.write_bool(false); // disable player interactions
    let _ = buf.write_signed_var_int(0); // server editor connection policy
    let _ = buf.write_bool(false); // allow anonymous block drops in editor worlds

    let _ = buf.write_var_string(""); // level ID
    let _ = buf.write_var_string("AndroServeMC"); // world name
    let _ = buf.write_var_string(""); // template content identity
    let _ = buf.write_bool(false); // trial

    // Player movement settings (Varint32 + Bool)
    let _ = buf.write_signed_var_int(0); // rewind history size
    let _ = buf.write_bool(false); // server auth block break

    let _ = buf.write_u64_le(0); // time (Int64 LE)
    let _ = buf.write_signed_var_int(0); // enchantment seed

    // Block Properties (palette)
    let _ = buf.write_var_int(0);

    let _ = buf.write_var_string(""); // multiplayer correlation ID
    let _ = buf.write_bool(false); // inventory server authoritative
    let _ = buf.write_var_string(game_version); // game version

    // Modern properties
    let _ = buf.write_u8(0); // empty NBT compound (player property data)
    let _ = buf.write_u64_le(0); // block registry checksum
    let _ = buf.write_u64_le(0); // world template ID UUID (most significant)
    let _ = buf.write_u64_le(0); // world template ID UUID (least significant)
    let _ = buf.write_bool(false); // client-side generation
    let _ = buf.write_bool(false); // use block network ID hashes
    let _ = buf.write_bool(false); // server authoritative sound
    let _ = buf.write_bool(false); // server join information present

    let _ = buf.write_var_string(""); // server ID
    let _ = buf.write_var_string(""); // scenario ID
    let _ = buf.write_var_string(""); // world ID
    let _ = buf.write_var_string(""); // owner ID

    buf.to_vec()
}

/// Build BiomeDefinitionList packet (0x7a)
pub fn build_biome_definitions() -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x7a);
    let _ = buf.write_u8(0); // Empty NBT tag compound
    buf.to_vec()
}

/// Build AvailableEntityIdentifiers packet (0x77)
pub fn build_entity_identifiers() -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x77);
    let _ = buf.write_u8(0); // Empty NBT tag compound
    buf.to_vec()
}

/// Build ChunkRadiusUpdated packet (0x46)
pub fn build_chunk_radius_updated(radius: i32) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x46);
    let _ = buf.write_var_int(radius as u32);
    buf.to_vec()
}

/// Build NetworkChunkPublisherUpdate packet (0x79)
pub fn build_network_chunk_publisher_update(x: i32, y: i32, z: i32, radius: u32) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x79);
    let _ = buf.write_signed_var_int(x);
    let _ = buf.write_signed_var_int(y);
    let _ = buf.write_signed_var_int(z);
    let _ = buf.write_var_int(radius);
    let _ = buf.write_u32_le(0); // saved chunks count
    buf.to_vec()
}

/// Build LevelChunk packet (0x3a) - empty Overworld chunk
pub fn build_level_chunk(chunk_x: i32, chunk_z: i32) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x3a);
    let _ = buf.write_signed_var_int(chunk_x); // chunk X (Varint32)
    let _ = buf.write_signed_var_int(chunk_z); // chunk Z (Varint32)
    let _ = buf.write_signed_var_int(0); // dimension (Varint32)
    let _ = buf.write_var_int(0); // sub_chunk_count (Varuint32) = 0
    let _ = buf.write_bool(false); // sub_chunk_limit present (optional)
    let _ = buf.write_bool(false); // cache_enabled
    let _ = buf.write_var_int(0); // blob hashes count (0)
    let _ = buf.write_var_int(0); // payload length (0)
    buf.to_vec()
}

/// Build ItemRegistry packet (0xa2) - empty registry (gophertunnel example server compatible)
pub fn build_item_registry() -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0xa2);
    let _ = buf.write_var_int(0); // item count (0)
    buf.to_vec()
}

/// Build CreativeContent packet (0x91) - empty creative inventory
pub fn build_creative_content() -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x91);
    let _ = buf.write_var_int(0); // groups count (0)
    let _ = buf.write_var_int(0); // items count (0)
    buf.to_vec()
}

/// Build Text packet (0x09) - Chat Message
pub fn build_text_packet(source: &str, message: &str) -> Vec<u8> {
    let mut buf = Buffer::new();
    let _ = buf.write_var_int(0x09);
    let _ = buf.write_u8(1); // Type: Chat/Raw
    let _ = buf.write_bool(false); // needs translation
    let _ = buf.write_var_string(source);
    let _ = buf.write_var_string(message);
    let _ = buf.write_var_string(""); // xbox user id
    let _ = buf.write_var_string(""); // platform chat id
    buf.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_game_matches_2168_wire_format() {
        let pkt = build_start_game("1.26.40");

        // packet ID (0x0b), EntityUniqueID Varint64(1)->0x02,
        // EntityRuntimeID Varuint64(1)->0x01, PlayerGameMode Varint32(1)->0x02,
        // then Vec3 position floats (0, 4, 0) little-endian
        let expected_head = [
            0x0b, 0x02, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x40, 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(&pkt[..expected_head.len()], &expected_head);
        assert_eq!(pkt.len(), 190);
    }

    #[test]
    fn signed_var_int_is_zigzag() {
        let mut buf = Buffer::new();
        buf.write_signed_var_int(4).unwrap();
        assert_eq!(buf.as_slice(), &[0x08]);
        let mut buf = Buffer::new();
        buf.write_signed_var_int(-1).unwrap();
        assert_eq!(buf.as_slice(), &[0x01]);
    }

    #[test]
    fn item_registry_matches_2168_wire_format() {
        assert_eq!(build_item_registry(), vec![0xa2, 0x01, 0x00]);
    }

    #[test]
    fn creative_content_matches_2168_wire_format() {
        assert_eq!(build_creative_content(), vec![0x91, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn level_chunk_matches_2168_wire_format() {
        // 0x3a, X(0), Z(0), dimension(0), subchunk_count(0),
        // subchunk_limit present flag(false), cache_enabled(false),
        // blob hashes count(0), payload length(0)
        assert_eq!(
            build_level_chunk(0, 0),
            vec![0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            build_level_chunk(1, -2),
            vec![0x3a, 0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }
}
