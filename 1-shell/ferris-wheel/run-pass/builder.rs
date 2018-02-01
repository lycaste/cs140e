// FIXME: Make me pass! Diff budget: 30 lines.

/* Note
 * to_* methods consume object and return a new type
 * as_* return a reference of new type into original object
 */
struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    pub fn default() -> Builder {
        Builder{string: None, number: None}
    }

    pub fn to_string(self) -> String {
        let mut ret = "".to_string();
        if let Some(s) = self.string {
            ret.push_str(s.as_str())
        }
        if let Some(n) = self.number {
            if !ret.is_empty() {
                ret += " ";
            }
            ret.push_str(n.to_string().as_str());
        }
        ret
    }

    pub fn string<T : std::string::ToString>(mut self, s: T) -> Builder {
        self.string = Some(s.to_string());
        self
    }

    pub fn number(mut self, n: usize) -> Builder {
        self.number = Some(n);
        self
    }
}

// Do not modify this function.
fn main() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default()
        .string("heap!".to_owned())
        .to_string();

    assert_eq!(c, "heap!");
}
