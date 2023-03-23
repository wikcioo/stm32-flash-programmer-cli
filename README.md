# STM32 flash programmer

A command-line program that runs on the host PC to communicate with [this](https://github.com/wikcioo/stm32f446xx-bootloader) stm32f4 custom bootloader.

## Implemented commands:
- [x] BL_GET_VER
- [x] BL_GET_HELP
- [x] BL_GET_DEV_ID
- [x] BL_GET_RDP_LEVEL
- [x] BL_JMP_ADDR
- [x] BL_FLASH_ERASE
- [x] BL_MEM_WRITE
- [x] BL_MEM_READ
- [x] BL_SET_RW_PROTECT
- [x] BL_GET_RW_PROTECT

## How to run?
**Note**: this project has only been tested on Linux

### Install cargo
```sh
curl https://sh.rustup.rs -sSf | sh

```

### Clone the repository
```sh
git clone https://github.com/wikcioo/stm32-flash-programmer-cli.git
cd stm32-flash-programmer-cli
```

### Run
```sh
cargo run
```
