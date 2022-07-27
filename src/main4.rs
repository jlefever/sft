use std::str::FromStr;

use glob::Pattern;

fn main() {
    let abs_pat = Pattern::from_str("/**/*").unwrap();
    let rel_pat = Pattern::from_str("**/*").unwrap();

    let abs_str = "/db/foo/bar/baz.cc";
    let rel_str = "db/foo/bar/baz.cc";

    println!("Absolute pattern matches an absolute string?  {}", abs_pat.matches(abs_str));
    println!("Absolute pattern matches a  relative string?  {}", abs_pat.matches(rel_str));
    println!("Relative pattern matches an absolute string?  {}", rel_pat.matches(abs_str));
    println!("Relative pattern matches a  relative string?  {}", rel_pat.matches(rel_str));
}