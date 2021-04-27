#!/bin/bash

cargo build
arm-none-eabi-objcopy -O binary -S target/thumbv7em-none-eabihf/debug/daisy-synth daisy-synth.bin
dfu-util -a 0 -s 0x08000000:leave -D daisy-synth.bin --device 0483:df11