# Contributing to Toutui

Thanks for your interest in **Toutui**! 🦜

## ⚠️ Beta Version
This project is still in **heavy development**. There are known bugs (check [known_bugs](https://github.com/ealtun21/Toutui/blob/main/known_bugs.md) and open issues).

Check the [roadmap](https://github.com/ealtun21/Toutui?tab=readme-ov-file#%EF%B8%8F-roadmap) to see what is currently in progress.  
If you want to contribute new features but don't have any ideas, feel free to check the [future features](https://github.com/ealtun21/Toutui?tab=readme-ov-file#-future-features) section for inspiration. 

## 🔁 Branching workflow 
This project follow this [branching workflow](https://gist.github.com/digitaljhelms/4287848). 

## 🧱 Build from a local clone
```bash
git clone https://github.com/ealtun21/Toutui
cd Toutui/
mkdir -p ~/.config/toutui
cp config.example.toml ~/.config/toutui/config.toml
echo "TOUTUI_SECRET_KEY=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')" >> ~/.config/toutui/.env
cargo run --release
```

## 🧪 Run the tests

```bash
cargo test
```

`cargo nextest run` gives the same tests and it is much faster: `cargo test`
runs the test binaries one after the other, and nextest runs every test in one
pool of processes. A measurement of 2026-08-11 gave **8.7 s** with `cargo test`
and **2.2 s** with nextest.

```bash
cargo install cargo-nextest --locked   # a tool of the machine, not a dependency
cargo nextest run                      # the tests that need no server
cargo nextest run --run-ignored all    # the tests of the sandbox too
```

The tests of the sandbox carry `#[ignore]`, because they need a server. Read
[docs/TEST-SERVER.md](docs/TEST-SERVER.md) to make that server. `.config/nextest.toml`
holds them in a group of one thread, because the login of the server has a
limit of the rate. With `cargo test`, give `-- --ignored --test-threads=1`.

## 💬 How to Contribute
- **Share your theme**: Check [here](https://github.com/AlbanDAVID/Toutui-theme).
- **Suggestions/feedback**: Open an issue (feature request) or use [discussions](https://github.com/ealtun21/Toutui/discussions).
- **Bugs**: Report bugs not listed in issues or [known bugs](https://github.com/ealtun21/Toutui/blob/main/known_bugs.md). Use the appropriate issue section (Installation issue or bug report).
- **Code**: Fork the repo, create a branch, and submit a pull request. 
