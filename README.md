# voiche

A naive voice changer library.

## Examples

### Voice change from wav file

``` sh
cargo run --release --example main -- something.wav
```

### Voice change from mic to speaker

Assuming you already have PulseAudio installed, you can run it with the following command:

``` sh
$ parec -r --raw --format=s16ne --channels=1 | cargo run --release --example stdinout 2> /dev/null | pacat --raw --format=s16ne --channels=1
```

## Author

* carrotflakes (carrotflakes@gmail.com)

## Copyright

Copyright (c) 2022 carrotflakes (carrotflakes@gmail.com)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
