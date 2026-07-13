# FastCat

The fastest, constant able, no-std compatible way to concatenate `&str`s

It's a fork of [fast-concat](https://github.com/tigerros/fast-concat), similar to it but some sight adjustments and remove the required constcat dependency.

### Get Started

Just add this to your `Cargo.toml`

```toml
[dependencies]
fastcat = "1.0.0"
```

Then to use it
```rust

use fastcat::fconcat;

fn main() {
    let name = "world";
    let greeting = fconcat!("hello ", name);
    println!("{greeting}");
}
```

### Acknowledgement

- [tigerros/fast-concat](https://github.com/tigerros/fast-concat)
- [rossmacarthur/constcat](https://github.com/rossmacarthur/constcat)