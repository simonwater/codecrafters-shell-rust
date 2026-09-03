use crate::Environment;
use anyhow::{Context, Result};

pub fn scan(s: &str, ctx: &Environment) -> Result<Vec<String>> {
    let tokens = shell_words::split(&s).context("command line parse error.")?;
    let tokens = tokens
        .into_iter()
        .map(|t| expand_token(t, ctx))
        .filter(|t| !t.is_empty())
        .collect::<Vec<String>>();

    Ok(tokens)
}

fn expand_token(s: String, ctx: &Environment) -> String {
    let mut ans = String::with_capacity(s.len());
    let mut left = 0usize;
    let mut open = false;
    let mut need_expand = false;
    let mut iter = s.char_indices().peekable();
    while let Some((i, cur_char)) = iter.next() {
        if cur_char == '$' && !open {
            need_expand = true;
            ans.push_str(&s[left..i]);
            if let Some((next_idx, _)) = iter.next_if_eq(&(i + 1, '{')) {
                left = next_idx + 1;
                open = true;
            } else {
                let p = &s[i + 1..];
                let value = ctx.get_variable(p).map(|v| v.as_str()).unwrap_or("");
                ans.push_str(value);
                left = s.len();
                break;
            }
        } else if cur_char == '}' && open {
            let p = &s[left..i];
            let value = ctx.get_variable(p).map(|v| v.as_str()).unwrap_or("");
            ans.push_str(value);
            open = false;
            left = i + 1;
        }
    }

    if !need_expand {
        return s;
    }
    if !open {
        // 括号正确关闭
        ans.push_str(&s[left..]);
    }
    ans
}
