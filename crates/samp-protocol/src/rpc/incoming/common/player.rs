use super::wire::{Bool8, Empty, F32, I32, U8, U16, Vector3Codec, read_bool8, write_bool8};
use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, WireReadExt, WireWriteExt, types::Vector3,
};

/// MoonLoader's `onSetPlayerName` payload (RPC 11).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerName {
    pub player_id: u16,
    pub name: Vec<u8>,
    pub success: bool,
}

/// MoonLoader's `onGivePlayerWeapon` payload (RPC 22).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerWeapon {
    pub weapon_id: i32,
    pub ammo: i32,
}

/// MoonLoader's `onSetPlayerTeam` payload (RPC 69).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerTeam {
    pub player_id: u16,
    pub team_id: u8,
}

/// MoonLoader's `onSetPlayerSkin` payload (RPC 153).
///
/// Both fields stay signed so unknown skin IDs remain observable without lossy validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSkin {
    pub player_id: i32,
    pub skin_id: i32,
}

/// MoonLoader's `onPlayerDeathNotification` payload (RPC 55).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerDeathNotification {
    pub killer_id: u16,
    pub killed_id: u16,
    pub reason: u8,
}

/// MoonLoader's `onSetPlayerColor` payload (RPC 72).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerColor {
    pub player_id: u16,
    pub color: i32,
}

/// MoonLoader's `onSetPlayerSkillLevel` payload (RPC 34).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSkill {
    pub player_id: u16,
    pub skill: i32,
    pub level: u16,
}

/// MoonLoader's `onShowPlayerNameTag` payload (RPC 80).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerNameTag {
    pub player_id: u16,
    pub show: bool,
}

/// MoonLoader's `onSetPlayerFightingStyle` payload (RPC 89).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerFightingStyle {
    pub player_id: u16,
    pub style_id: u8,
}

/// MoonLoader's `onSetWeaponAmmo` payload (RPC 145).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponAmmo {
    pub weapon_id: u8,
    pub ammo: u16,
}

struct PlayerNameCodec;

struct PlayerWeaponCodec;

struct PlayerTeamCodec;

struct PlayerSkinCodec;

struct PlayerDeathNotificationCodec;

struct PlayerColorCodec;

struct PlayerSkillCodec;

struct PlayerNameTagCodec;

struct PlayerFightingStyleCodec;

struct WeaponAmmoCodec;

descriptor!(SetPlayerPos, SET_PLAYER_POS, 12, Vector3Codec, Vector3);

descriptor!(
    SetPlayerPosFindZ,
    SET_PLAYER_POS_FIND_Z,
    13,
    Vector3Codec,
    Vector3
);

descriptor!(SetPlayerHealth, SET_PLAYER_HEALTH, 14, F32, f32);

descriptor!(SetPlayerArmour, SET_PLAYER_ARMOUR, 66, F32, f32);

descriptor!(SetPlayerFacingAngle, SET_PLAYER_FACING_ANGLE, 19, F32, f32);

descriptor!(
    TogglePlayerControllable,
    TOGGLE_PLAYER_CONTROLLABLE,
    15,
    Bool8,
    bool
);

descriptor!(
    SetPlayerName,
    SET_PLAYER_NAME,
    11,
    PlayerNameCodec,
    PlayerName
);

descriptor!(GivePlayerMoney, GIVE_PLAYER_MONEY, 18, I32, i32);

descriptor!(
    GivePlayerWeapon,
    GIVE_PLAYER_WEAPON,
    22,
    PlayerWeaponCodec,
    PlayerWeapon
);

descriptor!(
    SetPlayerSkin,
    SET_PLAYER_SKIN,
    153,
    PlayerSkinCodec,
    PlayerSkin
);

descriptor!(SetInterior, SET_INTERIOR, 156, U8, u8);

descriptor!(SetPlayerArmedWeapon, SET_PLAYER_ARMED_WEAPON, 67, I32, i32);

descriptor!(SetPlayerWantedLevel, SET_PLAYER_WANTED_LEVEL, 133, U8, u8);

descriptor!(
    SetPlayerTeam,
    SET_PLAYER_TEAM,
    69,
    PlayerTeamCodec,
    PlayerTeam
);

descriptor!(PlayerStreamOut, PLAYER_STREAM_OUT, 163, U16, u16);

descriptor!(ResetPlayerMoney, RESET_PLAYER_MONEY, 20, Empty, ());

descriptor!(ResetPlayerWeapons, RESET_PLAYER_WEAPONS, 21, Empty, ());

descriptor!(SetPlayerDrunk, SET_PLAYER_DRUNK, 35, I32, i32);

descriptor!(
    PlayerDeathNotificationRpc,
    PLAYER_DEATH_NOTIFICATION,
    55,
    PlayerDeathNotificationCodec,
    PlayerDeathNotification
);

descriptor!(
    SetPlayerColor,
    SET_PLAYER_COLOR,
    72,
    PlayerColorCodec,
    PlayerColor
);

descriptor!(
    SetPlayerSkillLevel,
    SET_PLAYER_SKILL_LEVEL,
    34,
    PlayerSkillCodec,
    PlayerSkill
);

descriptor!(
    ShowPlayerNameTag,
    SHOW_PLAYER_NAME_TAG,
    80,
    PlayerNameTagCodec,
    PlayerNameTag
);

descriptor!(
    SetPlayerDrunkVisuals,
    SET_PLAYER_DRUNK_VISUALS,
    92,
    I32,
    i32
);

descriptor!(
    SetPlayerDrunkHandling,
    SET_PLAYER_DRUNK_HANDLING,
    150,
    I32,
    i32
);

descriptor!(ClearPlayerAnimation, CLEAR_PLAYER_ANIMATION, 87, U16, u16);

descriptor!(
    SetPlayerSpecialAction,
    SET_PLAYER_SPECIAL_ACTION,
    88,
    U8,
    u8
);

descriptor!(
    SetPlayerFightingStyle,
    SET_PLAYER_FIGHTING_STYLE,
    89,
    PlayerFightingStyleCodec,
    PlayerFightingStyle
);

descriptor!(
    SetPlayerVelocity,
    SET_PLAYER_VELOCITY,
    90,
    Vector3Codec,
    Vector3
);

descriptor!(
    SetWeaponAmmo,
    SET_WEAPON_AMMO,
    145,
    WeaponAmmoCodec,
    WeaponAmmo
);

descriptor!(PlayerDeath, PLAYER_DEATH, 166, U16, u16);

wire_codec!(
    PlayerNameCodec,
    PlayerName,
    read_player_name,
    write_player_name
);

wire_codec!(
    PlayerWeaponCodec,
    PlayerWeapon,
    read_player_weapon,
    write_player_weapon
);

wire_codec!(
    PlayerTeamCodec,
    PlayerTeam,
    read_player_team,
    write_player_team
);

wire_codec!(
    PlayerSkinCodec,
    PlayerSkin,
    read_player_skin,
    write_player_skin
);

wire_codec!(
    PlayerDeathNotificationCodec,
    PlayerDeathNotification,
    read_player_death_notification,
    write_player_death_notification
);

wire_codec!(
    PlayerColorCodec,
    PlayerColor,
    read_player_color,
    write_player_color
);

wire_codec!(
    PlayerSkillCodec,
    PlayerSkill,
    read_player_skill,
    write_player_skill
);

wire_codec!(
    PlayerNameTagCodec,
    PlayerNameTag,
    read_player_name_tag,
    write_player_name_tag
);

wire_codec!(
    PlayerFightingStyleCodec,
    PlayerFightingStyle,
    read_player_fighting_style,
    write_player_fighting_style
);

wire_codec!(
    WeaponAmmoCodec,
    WeaponAmmo,
    read_weapon_ammo,
    write_weapon_ammo
);

fn read_player_name<R: BitRead>(reader: &mut R) -> Result<PlayerName, DecodeError<R::Error>> {
    Ok(PlayerName {
        player_id: reader.read_u16_le()?,
        name: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        success: read_bool8(reader)?,
    })
}

fn write_player_name<W: BitWrite>(
    writer: &mut W,
    value: &PlayerName,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_len_prefixed_bytes_u8(&value.name, usize::from(u8::MAX))?;
    write_bool8(writer, &value.success)
}

fn read_player_weapon<R: BitRead>(reader: &mut R) -> Result<PlayerWeapon, DecodeError<R::Error>> {
    Ok(PlayerWeapon {
        weapon_id: reader.read_i32_le()?,
        ammo: reader.read_i32_le()?,
    })
}

fn write_player_weapon<W: BitWrite>(
    writer: &mut W,
    value: &PlayerWeapon,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.weapon_id)?;
    writer.write_i32_le(value.ammo)
}

fn read_player_team<R: BitRead>(reader: &mut R) -> Result<PlayerTeam, DecodeError<R::Error>> {
    Ok(PlayerTeam {
        player_id: reader.read_u16_le()?,
        team_id: reader.read_u8()?,
    })
}

fn write_player_team<W: BitWrite>(
    writer: &mut W,
    value: &PlayerTeam,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.team_id)
}

fn read_player_skin<R: BitRead>(reader: &mut R) -> Result<PlayerSkin, DecodeError<R::Error>> {
    Ok(PlayerSkin {
        player_id: reader.read_i32_le()?,
        skin_id: reader.read_i32_le()?,
    })
}

fn write_player_skin<W: BitWrite>(
    writer: &mut W,
    value: &PlayerSkin,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.player_id)?;
    writer.write_i32_le(value.skin_id)
}

fn read_player_death_notification<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerDeathNotification, DecodeError<R::Error>> {
    Ok(PlayerDeathNotification {
        killer_id: reader.read_u16_le()?,
        killed_id: reader.read_u16_le()?,
        reason: reader.read_u8()?,
    })
}

fn write_player_death_notification<W: BitWrite>(
    writer: &mut W,
    value: &PlayerDeathNotification,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.killer_id)?;
    writer.write_u16_le(value.killed_id)?;
    writer.write_u8(value.reason)
}

fn read_player_color<R: BitRead>(reader: &mut R) -> Result<PlayerColor, DecodeError<R::Error>> {
    Ok(PlayerColor {
        player_id: reader.read_u16_le()?,
        color: reader.read_i32_le()?,
    })
}

fn write_player_color<W: BitWrite>(
    writer: &mut W,
    value: &PlayerColor,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_i32_le(value.color)
}

fn read_player_skill<R: BitRead>(reader: &mut R) -> Result<PlayerSkill, DecodeError<R::Error>> {
    Ok(PlayerSkill {
        player_id: reader.read_u16_le()?,
        skill: reader.read_i32_le()?,
        level: reader.read_u16_le()?,
    })
}

fn write_player_skill<W: BitWrite>(
    writer: &mut W,
    value: &PlayerSkill,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_i32_le(value.skill)?;
    writer.write_u16_le(value.level)
}

fn read_player_name_tag<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerNameTag, DecodeError<R::Error>> {
    Ok(PlayerNameTag {
        player_id: reader.read_u16_le()?,
        show: read_bool8(reader)?,
    })
}

fn write_player_name_tag<W: BitWrite>(
    writer: &mut W,
    value: &PlayerNameTag,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    write_bool8(writer, &value.show)
}

fn read_player_fighting_style<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerFightingStyle, DecodeError<R::Error>> {
    Ok(PlayerFightingStyle {
        player_id: reader.read_u16_le()?,
        style_id: reader.read_u8()?,
    })
}

fn write_player_fighting_style<W: BitWrite>(
    writer: &mut W,
    value: &PlayerFightingStyle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.style_id)
}

fn read_weapon_ammo<R: BitRead>(reader: &mut R) -> Result<WeaponAmmo, DecodeError<R::Error>> {
    Ok(WeaponAmmo {
        weapon_id: reader.read_u8()?,
        ammo: reader.read_u16_le()?,
    })
}

fn write_weapon_ammo<W: BitWrite>(
    writer: &mut W,
    value: &WeaponAmmo,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.weapon_id)?;
    writer.write_u16_le(value.ammo)
}
