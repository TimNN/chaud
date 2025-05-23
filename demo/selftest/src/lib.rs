use chaud::cycle::Track;
use std::borrow::Cow;
use std::fs;
use std::path::Path;

mod crashlog;

pub struct Selftest {
    track: Track,
}

impl Selftest {
    pub fn init_before_chaud() -> Self {
        crashlog::init();
        chaud_hot::cycle::track_init();

        Self { track: Track::new() }
    }
}

impl Selftest {
    pub fn run(self, root_version: impl Fn() -> u32) {
        run(self.track, root_version);
    }
}

pub fn run(mut track: Track, root_version: impl Fn() -> u32) {
    assert_eq!(root_version(), 1001);
    assert_eq!(mid::version(), 2001);
    assert_eq!(mid::leaf_version(), 3001);
    assert_eq!(mid::counters(), (0, 0, 0));
    assert_eq!(mid::counters(), (1, 1, 1));

    track.wait();

    patch("root/src/main.rs", "VERSION", "1002");
    track.wait();

    assert_eq!(root_version(), 1002);
    assert_eq!(mid::counters(), (2, 2, 2));

    patch("leaf/src/lib.rs", "VERSION", "3002");
    track.wait();
    assert_eq!(mid::leaf_version(), 3002);
    assert_eq!(mid::counters(), (0, 3, 3));

    patch("mid/src/lib.rs", "VERSION", "cold::initially_unused()");
    track.wait();
    assert_eq!(mid::version(), 42);
    assert_eq!(mid::counters(), (0, 4, 4));
}

#[track_caller]
fn patch(src: impl AsRef<Path>, marker: &str, patch: &str) {
    let src = src.as_ref();
    let buf = fs::read_to_string(src).expect("Failed to read src");

    let suffix = format!(" // {marker}");

    let mut did_patch = false;
    let mut lines = vec![];
    for line in buf.lines() {
        let Some(line) = line.strip_suffix(&suffix) else {
            lines.push(Cow::Borrowed(line));
            continue;
        };
        assert!(!did_patch);
        did_patch = true;

        let content = line
            .find(|c| c != ' ')
            .expect("Failed to find content of marked line");

        let whitespace = &line[..content];

        lines.push(Cow::Owned(format!("{}{}{}", whitespace, patch, suffix)));
    }
    assert!(did_patch);

    fs::write(src, lines.join("\n")).expect("Failed to write src");
}
