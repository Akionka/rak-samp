use super::*;

/// MoonLoader's `onPlayerJoin` payload (RPC 137).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerJoin {
    pub player_id: u16,
    pub color: u32,
    pub is_npc: bool,
    pub nickname: Vec<u8>,
}

/// MoonLoader's `onPlayerQuit` payload (RPC 138).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerQuit {
    pub player_id: u16,
    pub reason: u8,
}

/// MoonLoader's `onClientCheck` payload (RPC 103).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCheck {
    pub request_type: u8,
    pub subject: i32,
    pub offset: u16,
    pub length: u16,
}

struct PlayerJoinCodec;

struct PlayerQuitCodec;

struct ClientCheckCodec;

descriptor!(PlayerJoinRpc, PLAYER_JOIN, 137, PlayerJoinCodec, PlayerJoin);

descriptor!(PlayerQuitRpc, PLAYER_QUIT, 138, PlayerQuitCodec, PlayerQuit);

descriptor!(
    RequestSpawnResponse,
    REQUEST_SPAWN_RESPONSE,
    129,
    Bool8,
    bool
);

descriptor!(
    ClientCheckRpc,
    CLIENT_CHECK,
    103,
    ClientCheckCodec,
    ClientCheck
);

descriptor!(
    ServerStatisticsResponse,
    SERVER_STATISTICS_RESPONSE,
    102,
    Empty,
    ()
);

descriptor!(GamemodeRestart, GAMEMODE_RESTART, 40, Empty, ());

descriptor!(ForceClassSelection, FORCE_CLASS_SELECTION, 74, Empty, ());

descriptor!(ConnectionRejected, CONNECTION_REJECTED, 130, U8, u8);

wire_codec!(
    PlayerJoinCodec,
    PlayerJoin,
    read_player_join,
    write_player_join
);

wire_codec!(
    PlayerQuitCodec,
    PlayerQuit,
    read_player_quit,
    write_player_quit
);

wire_codec!(
    ClientCheckCodec,
    ClientCheck,
    read_client_check,
    write_client_check
);

fn read_player_join<R: BitRead>(reader: &mut R) -> Result<PlayerJoin, DecodeError<R::Error>> {
    Ok(PlayerJoin {
        player_id: reader.read_u16_le()?,
        color: reader.read_u32_le()?,
        is_npc: read_bool8(reader)?,
        nickname: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
    })
}

fn write_player_join<W: BitWrite>(
    writer: &mut W,
    value: &PlayerJoin,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u32_le(value.color)?;
    write_bool8(writer, &value.is_npc)?;
    writer.write_len_prefixed_bytes_u8(&value.nickname, usize::from(u8::MAX))
}

fn read_player_quit<R: BitRead>(reader: &mut R) -> Result<PlayerQuit, DecodeError<R::Error>> {
    Ok(PlayerQuit {
        player_id: reader.read_u16_le()?,
        reason: reader.read_u8()?,
    })
}

fn write_player_quit<W: BitWrite>(
    writer: &mut W,
    value: &PlayerQuit,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.reason)
}

fn read_client_check<R: BitRead>(reader: &mut R) -> Result<ClientCheck, DecodeError<R::Error>> {
    Ok(ClientCheck {
        request_type: reader.read_u8()?,
        subject: reader.read_i32_le()?,
        offset: reader.read_u16_le()?,
        length: reader.read_u16_le()?,
    })
}

fn write_client_check<W: BitWrite>(
    writer: &mut W,
    value: &ClientCheck,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.request_type)?;
    writer.write_i32_le(value.subject)?;
    writer.write_u16_le(value.offset)?;
    writer.write_u16_le(value.length)
}
