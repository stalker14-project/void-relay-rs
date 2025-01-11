pub mod whitelistadd;
pub mod whitelistrm;
pub mod notes;

use std::str::FromStr;

pub enum DiscordCommandType {
    WhitelistAdd,
    WhitelistRm,
    Notes,
    // todo
}

impl FromStr for DiscordCommandType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "whitelistadd" => Ok(Self::WhitelistAdd),
            "notes" => Ok(Self::Notes),
            "whitelistrm" => Ok(Self::WhitelistRm),
            _ => Err(())
        }
    }
}