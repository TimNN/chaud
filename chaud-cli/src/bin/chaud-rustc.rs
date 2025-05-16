#![allow(
    clippy::missing_errors_doc,
    reason = "less restrictions on build-time tools"
)]
use anyhow::{Context as _, Result, bail};
use chaud_cli::{actual_args, link_pre_args, run};
use std::env;
use std::process::Command;

fn main() -> Result<()> {
    let args = actual_args()?;
    let Some((prog, args)) = args.split_first() else {
        bail!("Too few arguments: missing `rustc` executable");
    };

    let mut cmd = Command::new(prog);
    cmd.args(args);

    if is_hot(args) {
        let extracted = extract(args).context("Failed to extract feature flags")?;

        if extracted.is_binary {
            cmd.arg("-Clink-dead-code")
                .arg(link_pre_args()?)
                .env("RUSTC_BOOTSTRAP", "1");
        }

        if env::var_os("CHAUD_FEATURE_FLAGS").is_none() {
            cmd.env("__CHAUD_RUSTC_FEATURE_FLAGS", extracted.feature_flags);
        }
    }

    run(cmd)
}

fn is_hot(args: &[String]) -> bool {
    args.iter().any(|s| s.contains("__chaud_hot_marker"))
}

struct Extracted {
    is_binary: bool,
    feature_flags: String,
}

fn extract(args: &[String]) -> Result<Extracted> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_args(args);

    let mut is_binary = false;
    let mut features = vec![];
    while let Some(arg) = parser.next()? {
        match arg {
            Long("crate-type") => {
                let val = parser.value()?.parse::<String>()?;
                is_binary |= val == "bin";
            }
            Long("cfg") => {
                let val = parser.value()?.parse::<String>()?;
                let Some(f) = val.strip_prefix("feature=") else {
                    continue;
                };
                let f = f.trim_matches('"');
                features.push(f.to_owned());
            }
            Short(_) | Long(_) => {
                parser.optional_value();
            }
            _ => {}
        }
    }

    let feature_flags = match features.as_slice() {
        [] => String::new(),
        f => format!("-F{}", f.join(",")),
    };

    Ok(Extracted { is_binary, feature_flags })
}
