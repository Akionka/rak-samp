/// A copied dialog request that is safe to retain until the game-thread pump
/// can call the private native client backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDialogRequest {
    pub id: u16,
    pub style: LocalDialogStyle,
    pub title: Vec<u8>,
    pub text: Vec<u8>,
    pub button1: Vec<u8>,
    pub button2: Vec<u8>,
}

/// A copied chat entry that is safe to retain until the game-thread pump can
/// call the private R1 chat backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalChatMessageRequest {
    pub style: LocalChatMessageStyle,
    pub text: Vec<u8>,
    pub prefix: Vec<u8>,
    pub text_colour: u32,
    pub prefix_colour: u32,
}

/// A copied death-window entry that is safe to retain until the game-thread
/// pump can call the private R1 death-window backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDeathMessageRequest {
    pub killer: Vec<u8>,
    pub victim: Vec<u8>,
    pub killer_colour: u32,
    pub victim_colour: u32,
    pub weapon: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalDialogStyle {
    MessageBox,
    Input,
    List,
    Password,
    TabList,
    HeadersList,
}

impl LocalDialogStyle {
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::MessageBox),
            1 => Some(Self::Input),
            2 => Some(Self::List),
            3 => Some(Self::Password),
            4 => Some(Self::TabList),
            5 => Some(Self::HeadersList),
            _ => None,
        }
    }

    pub const fn as_raw(self) -> u32 {
        match self {
            Self::MessageBox => 0,
            Self::Input => 1,
            Self::List => 2,
            Self::Password => 3,
            Self::TabList => 4,
            Self::HeadersList => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalChatMessageStyle {
    Chat,
    Info,
    Debug,
}

impl LocalChatMessageStyle {
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            2 => Some(Self::Chat),
            4 => Some(Self::Info),
            8 => Some(Self::Debug),
            _ => None,
        }
    }

    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Chat => 2,
            Self::Info => 4,
            Self::Debug => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalChatMessageStyle, LocalDialogStyle};

    #[test]
    fn direct_dialog_style_accepts_only_the_six_native_values() {
        assert_eq!(
            LocalDialogStyle::from_raw(0),
            Some(LocalDialogStyle::MessageBox)
        );
        assert_eq!(
            LocalDialogStyle::from_raw(5),
            Some(LocalDialogStyle::HeadersList)
        );
        assert_eq!(LocalDialogStyle::from_raw(6), None);
    }

    #[test]
    fn direct_chat_style_accepts_only_the_three_native_values() {
        assert_eq!(
            LocalChatMessageStyle::from_raw(2),
            Some(LocalChatMessageStyle::Chat)
        );
        assert_eq!(
            LocalChatMessageStyle::from_raw(8),
            Some(LocalChatMessageStyle::Debug)
        );
        assert_eq!(LocalChatMessageStyle::from_raw(1), None);
    }
}
