# with-checked-bytes

A Rust extension trait for `&mut str` to simplify manipulating UTF-8 strings as if they were
plain ASCII bytes. The result is only applied to the original string if the end result of
the modification is valid UTF-8.

## Examples

```rust
use with_checked_bytes::WithCheckedBytes;

let mut my_string = String::from("hello");
my_string.with_checked_bytes_mut(|s| {
    s[1] += 1;
}).unwrap();
assert_eq!(my_string, "hfllo");
```

```rust
let mut my_string = String::from("hello");
let old_value = my_string.with_checked_bytes_mut(|s| {
    std::mem::replace(&mut s[3], b'z')
}).unwrap();
assert_eq!(old_value, b'l');
assert_eq!(my_string, "helzo");
```

```rust
let mut my_string = String::from("hello");
my_string.with_checked_bytes_mut(|s| {
    s[1] = 0xff; // not valid UTF-8
}).unwrap(); // will panic - original string remains unmodified
```

## Licence

Apache 2.0. See `LICENSE`.
