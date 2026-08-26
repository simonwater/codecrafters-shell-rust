use crate::command::CommandResult;
use crate::environment::Environment;
use anyhow::Result;

pub fn complete(args: &[&str], environment: &Environment) -> Result<CommandResult> {
    let mut iter = args.iter();
    if let Some(&first) = iter.next() {
        match first {
            "-p" => {
                if let Some(&cmd) = iter.next() {
                    let res = if let Some(content) = environment.get_complete_reg(cmd) {
                        let msg = format!("complete -C '{}' {}\n", content, cmd);
                        CommandResult::new().success(msg)
                    } else {
                        CommandResult::new()
                            .error(format!("complete: {}: no completion specification\n", cmd))
                    };
                    return Ok(res);
                }
            }
            "-C" => {
                if let (Some(&content), Some(&cmd)) = (iter.next(), iter.next()) {
                    environment.reg_complete(cmd.to_string(), content.to_string());
                }
            }
            _ => {}
        }
    }
    Ok(CommandResult::new())
}
