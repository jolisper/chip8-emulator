# Chip-8 Emulator

This project is a Chip-8 emulator written in Rust using the SDL2 library, which allows users to run Chip-8 programs and games on a modern system. Chip-8 is an interpreted programming language and virtual machine that was originally developed in the mid-1970s for use on early microcomputers like the COSMAC VIP and Telmac 1800. It was designed to make it easier to develop games and other interactive programs for these systems.


## Project Structure

The project is structured as follows:

* `chip8-avsys`: This library provides the interface with the SDL library for the emulator.
* `chip8-vm`: This library provides an implementation of the Chip-8 virtual machine.
* `chip8-cli`: This is the command-line interface for the emulator.
* `chip8-roms`: This directory contains the Chip-8 ROMs that can be loaded into the emulator.

## Building and Running

The simplest way to build and run the emulator is to use the makefile targets ended with `-with-docker`. This will use Docker to build the emulator and run it. For example:

```shell
$ make build-windows-with-docker
```

In the `build/windows` directory, you should see a `chip8.exe` executable allong with the SDL2 library.

```shell
build\windows\chip8.exe chip8-roms\INVADERS
```

<img title="" src="assets/2fd98baf4e189ff43b906e4005ee0d2fcd1562f2.gif" alt="Chip8 - Space Invaders - Made with Clipchamp.gif" data-align="center">

## Controls

The emulator supports the following keys to play the roms:
<pre>
| 1 | 2 | 3 | 4 |
| q | w | e | r |
| a | s | d | f |
| z | x | c | v |
</pre>

The action that the keys perform depends on the rom that is loaded.

## References

- Original Chip-8 documentation (http://drevernay.free.fr/hacks/chip8/C8TECH10.HTM)

## License

This project is licensed under the MIT License.