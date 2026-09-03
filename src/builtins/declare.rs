use crate::Environment;
use crate::ShellOutput;
use anyhow::Error;
use anyhow::Result;

pub(crate) fn declare(args: &[&str], environment: &mut Environment) -> Result<ShellOutput> {
    let mut iter = args.iter();
    if let Some(&arg1) = iter.next() {
        match arg1 {
            "-p" => {
                let Some(&var_name) = iter.next() else {
                    return Ok(ShellOutput::null());
                };
                let var_value = environment
                    .get_variable(var_name)
                    .ok_or_else(|| Error::msg(format!("declare: {}: not found\n", var_name)))?;
                let out = format!("declare -- {}=\"{}\"\n", var_name, var_value);
                return Ok(ShellOutput::new().success(out));
            }
            kv => {
                if let Some((k, v)) = kv.split_once("=") {
                    environment.add_variable(k.into(), v.into());
                }
                return Ok(ShellOutput::null());
            }
        };
    }
    Ok(ShellOutput::null())
}
