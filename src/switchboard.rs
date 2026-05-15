use crate::registry::{self, RegistryOutput};

#[derive(Debug)]
pub struct SwitchboardOptions {
    pub command: SwitchboardCommand,
}

#[derive(Debug)]
pub enum SwitchboardCommand {
    List { json: bool },
    Enable { id: String },
    Disable { id: String },
}

pub fn run(options: &SwitchboardOptions) -> Result<RegistryOutput, String> {
    match &options.command {
        SwitchboardCommand::List { json } => registry::switchboard_list(*json),
        SwitchboardCommand::Enable { id } => registry::enable(id),
        SwitchboardCommand::Disable { id } => registry::disable(id),
    }
}
