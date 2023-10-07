# rchip8
A CHIP-8 emulator/interpreter in Rust

To run the emulator just type 
```terminal
"cargo run -- -d".
```
It will run a simple inline program in main.

Commandline switches:

```terminal
-r, --rom <ROM> : Choose ROM file to execute
-d, --demo : Run the demo program
-f, --fps <FPS> : Set emulation speed [default: 60]
--debug <N> : Set degug level. [default: 0]
-h, --help           Print help
```

The program plays a sound an draw an alien on screen.
You can move the alien pressing the emulated 0, 5, 7 and 9 keys.
The actual keys from computer keyboard is W, A, D and X.

You can run any chip8 rom.
In the roms folder, there are some.

![image](https://github.com/dedraks/rchip8/assets/843727/f2fe18c1-e850-49d3-a817-b5129b5e8b31)

![image](https://github.com/dedraks/rchip8/assets/843727/5aa660e4-f3e8-4c42-a4de-81562962d132)

![image](https://github.com/dedraks/rchip8/assets/843727/d9310156-f39a-48e9-bd9f-007a627b7fa9)

![image](https://github.com/dedraks/rchip8/assets/843727/e170226f-92fd-4d20-8c9a-7bb944c72e1f)
