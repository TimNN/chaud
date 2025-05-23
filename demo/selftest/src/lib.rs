mod crashlog;

pub fn init_log() {
    crashlog::init();
}

pub fn run(root_version: impl Fn() -> u32) {
    assert_eq!(root_version(), 1001);
    assert_eq!(mid::version(), 2001);
    assert_eq!(mid::leaf_version(), 3001);
    assert_eq!(mid::counters(), (0, 0));
    assert_eq!(mid::counters(), (1, 1));

    eprintln!("{:?}", std::env::current_dir());
}
