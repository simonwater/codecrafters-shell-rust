use crate::command::CommandResult;
use crate::environment::Environment;
use anyhow::Result;

pub fn complete(args: &[&str], environment: &mut Environment) -> Result<CommandResult> {
    if args.len() >= 2 {
        match args[0] {
            "-p" => {
                let cmd = args[1];
                let res = if let Some(content) = environment.get_complete_reg(cmd) {
                    let msg = format!("complete -C '{}' {}\n", content, cmd);
                    CommandResult::new().success(msg)
                } else {
                    CommandResult::new()
                        .error(format!("complete: {}: no completion specification\n", cmd))
                };
                return Ok(res);
            }
            "-C" => {
                if args.len() == 3 {
                    let content = args[1];
                    let cmd = args[2];
                    environment.reg_complete(cmd.to_string(), content.to_string());
                }
            }
            _ => {}
        }
    }
    Ok(CommandResult::new())
}
