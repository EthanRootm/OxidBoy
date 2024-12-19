
![Logo](./assets/logo.png)

OxidBoy is a GameBoy/ GameBoy Color emulator written in pure rust using SDL2 libraries. This is mostly a research project that I will most likely continue to work on, but I have other projects to do.


## Usage

Install OxidBoy

```bash
  git clone https://github.com/EthanRootm/OxidBoy
  cd OxidBoy
```
Run
```bash
  cargo run -- release -- "your/rom/here"
```
Extra Options
```text
  -s, --scale    Scale the Window
```
    
## Dependencies

- [argparse](https://github.com/tailhook/rust-argparse)
- [blip_buf](https://docs.rs/blip_buf/latest/blip_buf/)
- [bytemuck](https://github.com/Lokathor/bytemuck)
- [cpal](https://github.com/RustAudio/cpal)
- [SDL2](https://github.com/rust-sdl2/rust-sdl2)

## Controls

| Key   | Input  |
| ----- | ------ |
| Left  | Left   |
| Down  | Down   |
| Right | Right  |
| Up    | Up     |
| Z     | A      |
| X     | B      |
| C     | Select |
| V     | Start  |


## Roadmap

- Enable controller support

- Settings

