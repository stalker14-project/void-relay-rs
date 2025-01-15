pub mod whitelist;
pub mod notes;
pub mod ban;

use std::str::FromStr;

pub enum DiscordCommandType {
    Whitelist,
    Notes,
    Ban
    // todo
}

impl FromStr for DiscordCommandType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "whitelist" => Ok(Self::Whitelist),
            "notes" => Ok(Self::Notes),
            "bans" => Ok(Self::Ban),
            _ => Err(())
        }
    }
}