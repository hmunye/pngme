# pngme

Command-line tool for embedding and retrieving messages in PNG files.

## Quick Start

```bash
git clone https://github.com/hmunye/pngme.git
cd pngme
```

```bash
cargo b --release
./target/release/pngme help
```

```
Usage: pngme <command>

Commands:
  encode  Encodes a message into the PNG file given its chunk type
  decode  Decodes a message from the PNG file given the chunk type
  remove  Removes a message from the PNG file given the chunk type
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## License

This project is licensed under the [MIT License].

[MIT License]: https://github.com/hmunye/pngme/blob/main/LICENSE

## References
[PNG Specification](https://www.w3.org/TR/png-3/)
