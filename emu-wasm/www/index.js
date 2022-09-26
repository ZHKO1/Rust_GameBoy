import { Gameboy, JoyPadKey } from "../pkg/rust_gameboy_wasm.js";
import { memory } from '../pkg/rust_gameboy_wasm_bg';

class Emulator {
  constructor() {
    this.lcd_width = Gameboy.lcd_width();
    this.lcd_height = Gameboy.lcd_height();
    this.gameboy = null;
    this.running = false;

    this.canvas = document.getElementById("game-of-life-canvas");
    this.ctx = this.canvas.getContext("2d");

    this.romPicker = document.getElementById("rompicker");
    this.startButton = document.getElementById("start");
    this.startButton.onclick = () => this.start();
  }

  init(romBuffer) {
    const romData = new Uint8Array(romBuffer);

    try {
      this.gameboy = new Gameboy([], romData);
    } catch (e) {
      console.error(e);
      throw e;
    }

    console.log("Gameboy loaded!");
  }

  async loadRom() {
    if (this.romPicker.files.length == 0) {
      alert("Please load a ROM first!");
      return;
    }

    const romFile = this.romPicker.files[0];
    this.romName = romFile.name;

    return await romFile.arrayBuffer();
  }

  start() {
    this.loadRom().then((romBuffer) => {
      this.init(romBuffer);
      this.run();
      // this.renderFrame()
    });
  }

  run() {
    if (this.frameTimer != null) {
      clearInterval(this.frameTimer);
    }

    this.frameTimer = window.setInterval(() => {
      this.renderFrame()
    }, 16.7504);

    this.running = true;
  }

  renderFrame() {
    const frameBufferPtr = this.gameboy.frame();
    const frameBuffer = new Uint8Array(memory.buffer, frameBufferPtr,
      this.lcd_width * this.lcd_height * 4);
    const imageData = this.ctx.createImageData(this.lcd_width, this.lcd_height);
    const data = imageData.data;

    for (var x = 0; x < this.lcd_width; x += 1) {
      for (var y = 0; y < this.lcd_height; y += 1) {
        const source_idx = y * this.lcd_width * 4 + x * 4;
        const red = frameBuffer[source_idx];
        const green = frameBuffer[source_idx + 1];
        const blue = frameBuffer[source_idx + 2];
        const dest_idx = y * this.lcd_width * 4 + x * 4;
        data[dest_idx] = red;
        data[dest_idx + 1] = green;
        data[dest_idx + 2] = blue;
        data[dest_idx + 3] = 255; // alpha
        // console.log(`${red}${green}${blue}`);
      }
    }
    this.ctx.putImageData(imageData, 0, 0);
  }
  mapKeyCodeToInput(keycode) {
    let joypad_input = null;

    switch (keycode) {
      case "ArrowUp":
        joypad_input = JoyPadKey.Up;
        break;
      case "ArrowDown":
        joypad_input = JoyPadKey.Down;
        break;
      case "ArrowLeft":
        joypad_input = JoyPadKey.Left;
        break;
      case "ArrowRight":
        joypad_input = JoyPadKey.Right;
        break;
      case "KeyX":
        joypad_input = JoyPadKey.A;
        break;
      case "KeyZ":
        joypad_input = JoyPadKey.B;
        break;
      case "Enter":
        joypad_input = JoyPadKey.Start;
        break;
      case "Backspace":
        joypad_input = JoyPadKey.Select;
        break;
      default:
        break;
    }

    return joypad_input;
  }

  handleKey(keyEvent, down) {
    if (this.gameboy == null) {
      return;
    }

    const keyCode = keyEvent.code;
    const joypad_input = this.mapKeyCodeToInput(keyCode);

    if (joypad_input != null && this.running) {
      this.gameboy.input(joypad_input, down);
    }
  }
}

const emulator = new Emulator();

document.addEventListener("keydown", (event) => {
  emulator.handleKey(event, true);
});

document.addEventListener("keyup", (event) => {
  emulator.handleKey(event, false);
});
