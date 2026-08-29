use super::*;

/// The maximum number of rows that R1 menus can expose per column.
pub const MAX_MENU_ROWS: usize = 12;

/// The R1 client accepts at most two menu columns.
pub const MAX_MENU_COLUMNS: usize = 2;

/// One column in an R1 menu initialization payload.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuColumn {
    pub width: f32,
    pub title: [u8; 32],
    pub rows: Vec<[u8; 32]>,
}

/// R1's `onInitMenu` payload (RPC 76).
#[derive(Clone, Debug, PartialEq)]
pub struct InitMenu {
    pub menu_id: u8,
    pub two_columns: bool,
    pub title: [u8; 32],
    pub position: Vector2,
    pub columns: Vec<MenuColumn>,
    pub rows: [i32; MAX_MENU_ROWS],
    pub menu: bool,
}

/// R1's `onToggleSelectTextDraw` payload (RPC 83).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleSelectTextDraw {
    pub enabled: bool,
    pub hover_color: i32,
}

/// The R1 textdraw shape and content sent by `onShowTextDraw`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextDraw {
    pub flags: u8,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_color: i32,
    pub line_width: f32,
    pub line_height: f32,
    pub box_color: i32,
    pub shadow: u8,
    pub outline: u8,
    pub background_color: i32,
    pub style: u8,
    pub selectable: u8,
    pub position: Vector2,
    pub model_id: u16,
    pub rotation: Vector3,
    pub zoom: f32,
    pub color1: i16,
    pub color2: i16,
    pub text: Vec<u8>,
}

/// R1's `onShowTextDraw` payload (RPC 134).
#[derive(Clone, Debug, PartialEq)]
pub struct ShowTextDraw {
    pub textdraw_id: u16,
    pub textdraw: TextDraw,
}

struct InitMenuCodec;

struct ToggleSelectTextDrawCodec;

struct ShowTextDrawCodec;

struct TextDrawHideCodec;

descriptor!(
    InitMenuRpc,
    INIT_MENU,
    76,
    InitMenuCodec,
    InitMenu,
    ExactBytesPolicy
);

descriptor!(
    ToggleSelectTextDrawRpc,
    TOGGLE_SELECT_TEXT_DRAW,
    83,
    ToggleSelectTextDrawCodec,
    ToggleSelectTextDraw,
    ExactBitsPolicy
);

descriptor!(
    ShowTextDrawRpc,
    SHOW_TEXT_DRAW,
    134,
    ShowTextDrawCodec,
    ShowTextDraw,
    ExactBytesPolicy
);

descriptor!(
    TextDrawHideRpc,
    TEXT_DRAW_HIDE,
    135,
    TextDrawHideCodec,
    u16,
    ExactBytesPolicy
);

r1_codec!(InitMenuCodec, InitMenu, decode_init_menu, encode_init_menu);

r1_codec!(
    ToggleSelectTextDrawCodec,
    ToggleSelectTextDraw,
    decode_toggle_select_text_draw,
    encode_toggle_select_text_draw
);

r1_codec!(
    ShowTextDrawCodec,
    ShowTextDraw,
    decode_show_text_draw,
    encode_show_text_draw
);

r1_codec!(TextDrawHideCodec, u16, decode_u16, encode_u16);

fn decode_menu_column<R: BitRead>(
    reader: &mut R,
    width: f32,
) -> Result<MenuColumn, DecodeError<R::Error>> {
    let title = read_fixed(reader)?;
    let row_count = usize::from(reader.read_u8()?);
    if row_count > MAX_MENU_ROWS {
        return Err(DecodeError::LengthExceedsLimit {
            length: row_count,
            limit: MAX_MENU_ROWS,
        });
    }
    let mut rows = Vec::with_capacity(row_count);
    for _ in 0..row_count {
        rows.push(read_fixed(reader)?);
    }
    Ok(MenuColumn { width, title, rows })
}

fn encode_menu_column<W: BitWrite>(
    writer: &mut W,
    value: &MenuColumn,
) -> Result<(), EncodeError<W::Error>> {
    if value.rows.len() > MAX_MENU_ROWS {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.rows.len(),
            limit: MAX_MENU_ROWS,
        });
    }
    writer.write_bytes(&value.title)?;
    writer.write_u8(value.rows.len() as u8)?;
    for row in &value.rows {
        writer.write_bytes(row)?;
    }
    Ok(())
}

fn decode_init_menu<R: BitRead>(reader: &mut R) -> Result<InitMenu, DecodeError<R::Error>> {
    let menu_id = reader.read_u8()?;
    let two_columns = read_bool32(reader)?;
    let title = read_fixed(reader)?;
    let position = reader.read_vector2_le()?;
    let first_width = reader.read_f32_le()?;
    let second_width = two_columns.then(|| reader.read_f32_le()).transpose()?;
    let menu = read_bool32(reader)?;
    let mut rows = [0; MAX_MENU_ROWS];
    for row in &mut rows {
        *row = reader.read_i32_le()?;
    }
    let mut columns = Vec::with_capacity(if two_columns { 2 } else { 1 });
    columns.push(decode_menu_column(reader, first_width)?);
    if let Some(width) = second_width {
        columns.push(decode_menu_column(reader, width)?);
    }
    Ok(InitMenu {
        menu_id,
        two_columns,
        title,
        position,
        columns,
        rows,
        menu,
    })
}

fn encode_init_menu<W: BitWrite>(
    writer: &mut W,
    value: &InitMenu,
) -> Result<(), EncodeError<W::Error>> {
    if value.columns.len() > MAX_MENU_COLUMNS {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.columns.len(),
            limit: MAX_MENU_COLUMNS,
        });
    }
    let expected_columns = if value.two_columns { 2 } else { 1 };
    if value.columns.len() != expected_columns {
        return Err(EncodeError::InvalidCollectionLength {
            length: value.columns.len(),
            expected: expected_columns,
        });
    }
    let first = &value.columns[0];
    writer.write_u8(value.menu_id)?;
    write_bool32(writer, value.two_columns)?;
    writer.write_bytes(&value.title)?;
    writer.write_vector2_le(&value.position)?;
    writer.write_f32_le(first.width)?;
    if let Some(second) = value.columns.get(1) {
        writer.write_f32_le(second.width)?;
    }
    write_bool32(writer, value.menu)?;
    for row in value.rows {
        writer.write_i32_le(row)?;
    }
    encode_menu_column(writer, first)?;
    if let Some(second) = value.columns.get(1) {
        encode_menu_column(writer, second)?;
    }
    Ok(())
}

fn decode_toggle_select_text_draw<R: BitRead>(
    reader: &mut R,
) -> Result<ToggleSelectTextDraw, DecodeError<R::Error>> {
    Ok(ToggleSelectTextDraw {
        enabled: reader.read_bit_bool()?,
        hover_color: reader.read_i32_le()?,
    })
}

fn encode_toggle_select_text_draw<W: BitWrite>(
    writer: &mut W,
    value: &ToggleSelectTextDraw,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bit_bool(value.enabled)?;
    writer.write_i32_le(value.hover_color)
}

fn decode_show_text_draw<R: BitRead>(
    reader: &mut R,
) -> Result<ShowTextDraw, DecodeError<R::Error>> {
    Ok(ShowTextDraw {
        textdraw_id: reader.read_u16_le()?,
        textdraw: TextDraw {
            flags: reader.read_u8()?,
            letter_width: reader.read_f32_le()?,
            letter_height: reader.read_f32_le()?,
            letter_color: reader.read_i32_le()?,
            line_width: reader.read_f32_le()?,
            line_height: reader.read_f32_le()?,
            box_color: reader.read_i32_le()?,
            shadow: reader.read_u8()?,
            outline: reader.read_u8()?,
            background_color: reader.read_i32_le()?,
            style: reader.read_u8()?,
            selectable: reader.read_u8()?,
            position: reader.read_vector2_le()?,
            model_id: reader.read_u16_le()?,
            rotation: reader.read_vector3_le()?,
            zoom: reader.read_f32_le()?,
            color1: reader.read_i16_le()?,
            color2: reader.read_i16_le()?,
            text: reader.read_len_prefixed_bytes_u16_le(MAX_STRING32_BYTES)?,
        },
    })
}

fn encode_show_text_draw<W: BitWrite>(
    writer: &mut W,
    value: &ShowTextDraw,
) -> Result<(), EncodeError<W::Error>> {
    let textdraw = &value.textdraw;
    writer.write_u16_le(value.textdraw_id)?;
    writer.write_u8(textdraw.flags)?;
    writer.write_f32_le(textdraw.letter_width)?;
    writer.write_f32_le(textdraw.letter_height)?;
    writer.write_i32_le(textdraw.letter_color)?;
    writer.write_f32_le(textdraw.line_width)?;
    writer.write_f32_le(textdraw.line_height)?;
    writer.write_i32_le(textdraw.box_color)?;
    writer.write_u8(textdraw.shadow)?;
    writer.write_u8(textdraw.outline)?;
    writer.write_i32_le(textdraw.background_color)?;
    writer.write_u8(textdraw.style)?;
    writer.write_u8(textdraw.selectable)?;
    writer.write_vector2_le(&textdraw.position)?;
    writer.write_u16_le(textdraw.model_id)?;
    writer.write_vector3_le(&textdraw.rotation)?;
    writer.write_f32_le(textdraw.zoom)?;
    writer.write_i16_le(textdraw.color1)?;
    writer.write_i16_le(textdraw.color2)?;
    writer.write_len_prefixed_bytes_u16_le(&textdraw.text, MAX_STRING32_BYTES)
}

fn decode_u16<R: BitRead>(reader: &mut R) -> Result<u16, DecodeError<R::Error>> {
    reader.read_u16_le()
}

fn encode_u16<W: BitWrite>(writer: &mut W, value: &u16) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(*value)
}
