use crate::command::CommandResult;
use anyhow::Result;

pub fn complete(args: &[&str]) -> Result<CommandResult> {
    if args.len() >= 2 {
        match args[0] {
            "-p" => {
                let cmd = args[1];
                return Ok(CommandResult::new()
                    .error(format!("complete: {}: no completion specification\n", cmd)));
            }
            _ => {}
        }
    }
    Ok(CommandResult::new())
}
