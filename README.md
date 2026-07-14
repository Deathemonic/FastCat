# FastCat

Fast string concatenation, with const-str support built in and a fork of [fast-concat](https://github.com/tigerros/fast-concat)

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
    println!("{greeting}"); // "hello world"
}
```

```rust
use fastcat::fconcat;

fn main() {
    let base = "/usr/local";
    let user = "bob";
    // to intantiate a separator it need to be first and ends with ;
    let greeting = fconcat!("/"; base, user, "bin"); 
    println!("{greeting}"); // "/usr/local/bob/bin"
}
```

### Acknowledgement

- [tigerros/fast-concat](https://github.com/tigerros/fast-concat)
- [rossmacarthur/constcat](https://github.com/rossmacarthur/constcat)