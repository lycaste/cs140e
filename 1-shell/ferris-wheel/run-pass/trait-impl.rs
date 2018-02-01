// FIXME: Make me pass! Diff budget: 25 lines.

use Duration::*;
#[derive(Debug)]
enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16)
}

impl Duration {
    pub fn to_mili(&self) -> u64 {
        match *self {
            Duration::MilliSeconds(i) => i,
            Duration::Seconds(i) => i as u64 * 1000,
            Duration::Minutes(i) => i as u64 * 60 * 1000
        }
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Duration) -> bool {
        self.to_mili() == other.to_mili()
    }

}

fn main() {
    assert_eq!(Seconds(120), Minutes(2));
    assert_eq!(Seconds(420), Minutes(7));
    assert_eq!(MilliSeconds(420000), Minutes(7));
    assert_eq!(MilliSeconds(43000), Seconds(43));
}
