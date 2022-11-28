# Rust-GameBoy
A GameBoy emulator written in Rust.
Support GameBoy and GameBoyColor
Try it here: https://rust-gameboy.netlify.app/

You can start a game with the following command.
```s
cargo run --release -- run -b ./tests/DMG_ROM.bin ./tests/Tetris.gb
```
# Play

Controls:
* Arrows for direction keys
* `Z`: A button
* `X`: B button
* `Enter`: start button
* `Backspace`: select button

## Reference
- https://www.youtube.com/watch?v=HyzD8pNlpwI&t=46m55s
- https://gbdev.io/pandocs/About.html
- http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
- https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
- https://blog.tigris.fr/2019/07/09/writing-an-emulator-the-first-steps/
- http://accu.cc/content/gameboy/preface/
- https://github.com/aksiksi/gbc/


## License
This project is open source and available under the [MIT License](LICENSE).