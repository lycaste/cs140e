// FIXME: Make me compile! Diff budget: 2 lines.

// Do not modify the inner type &'a T.
struct RefWrapper<'a, T: 'a>(&'a T);

// Do not modify the inner type &'b RefWrapper<'a, T>.
struct RefWrapperWrapper<'a: 'b, 'b, T: 'a>(&'b RefWrapper<'a, T>);

/* Note:
 * 'a: 'b means 'a outlives 'b, or be more clear 'long: 'short
 * reference:
 * http://www.howtobuildsoftware.com/index.php/how-do/hzJ/rust-lifetime-does-a-b-a-mean-that-the-lifetime-b-must-outlive-the-lifetime-a
 */

pub fn main() { }
