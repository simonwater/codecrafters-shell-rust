use crate::command::ShellOutput;
use crate::environment::Environment;
use anyhow::Result;

pub fn complete(args: &[&str], environment: &Environment) -> Result<ShellOutput> {
    let mut iter = args.iter();
    if let Some(&first) = iter.next() {
        match first {
            "-p" => {
                if let Some(&cmd) = iter.next() {
                    let res = if let Some(content) = environment.get_complete_reg(cmd) {
                        let msg = format!("complete -C '{}' {}\n", content, cmd);
                        ShellOutput::new().success(msg)
                    } else {
                        ShellOutput::new()
                            .error(format!("complete: {}: no completion specification\n", cmd))
                    };
                    return Ok(res);
                }
            }
            "-r" => {
                if let Some(&cmd) = iter.next() {
                    environment.remove_complete_reg(cmd);
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
    Ok(ShellOutput::null())
}
