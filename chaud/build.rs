use core::error::Error;
use std::{env, fs};

static STYLE: &str = r"
<style>
blockquote {
  border-left: 0.25em solid crimson;
  padding-left: 1em;
  margin: 0 0 0.75em 0;
}
</style>
";

static DOCTEST_HEADER: &str = r"
```no_run
# mod some_other_crate {
#   pub fn do_something() -> u32 { 42 }
# }
#";

fn main() -> Result<(), Box<dyn Error>> {
    let mut readme = fs::read_to_string("README.md")?;

    readme = readme.replace("[!CAUTION]", "**Caution:**");

    readme = readme.replace("```rust", DOCTEST_HEADER);

    readme = readme.replace(
        "(https://docs.rs/chaud/latest/chaud/struct.Handle.html)",
        "",
    );

    readme.truncate(
        readme
            .find("<!-- readme-license-begin -->")
            .ok_or("Missing license marker")?,
    );

    readme.push_str(STYLE);

    let mut out_file = env::var("OUT_DIR")?;
    out_file.push_str("/README.processed.md");

    fs::write(&out_file, readme.as_bytes())?;

    println!("cargo::rustc-env=README={out_file}");

    Ok(())
}
