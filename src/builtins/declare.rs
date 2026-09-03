use crate::Environment;
use crate::ShellOutput;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;

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
                    .ok_or_else(|| anyhow!("declare: {}: not found\n", var_name))?;
                let out = format!("declare -- {}=\"{}\"\n", var_name, var_value);
                return Ok(ShellOutput::new().success(out));
            }
            kv => {
                if let Some((k, v)) = kv.split_once("=") {
                    if k.starts_with(|c: char| c.is_ascii_digit())
                        || k.chars()
                            .any(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    {
                        bail!("declare: `{}={}': not a valid identifier\n", k, v);
                    }
                    environment.add_variable(k.into(), v.into());
                }
                return Ok(ShellOutput::null());
            }
        };
    }
    Ok(ShellOutput::null())
}
