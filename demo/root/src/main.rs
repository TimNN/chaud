use core::time::Duration;
use std::thread;

#[chaud::hot]
fn version() -> u32 {
    1001 // VERSION
}

fn main() {
    #[cfg(feature = "selftest")]
    if true {
        let st = selftest::Selftest::init_before_chaud();
        chaud::init!();
        return st.run(version);
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
