use core::time::Duration;
use std::thread;

#[chaud::hot]
fn version() -> u32 {
    1001
}

fn main() {
    #[cfg(feature = "selftest")]
    if true {
        selftest::init_log();
        chaud::init!();
        selftest::run(version);
        return;
    }

    env_logger::init();
    chaud::init!();

    loop {
        println!(
            "root: {} mid: {} leaf: {} counters: {:?}",
            version(),
            mid::version(),
            mid::leaf_version(),
            mid::counters(),
        );
        thread::sleep(Duration::from_secs(2));
    }
}
