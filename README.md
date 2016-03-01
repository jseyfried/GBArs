# rsGBA

A GBA emulator written in Rust.

Thanks to a guy named Ferris and his project [Rustendo 64](https://github.com/yupferris/rustendo64), many people got motivated to write their own emulators in Rust. Even I wasn't spared, so here it is, my GBA emulator.

And why GBA?

- It is ARM-based and ARM is sexy.
- I want to play Metroid Zero Mission and Fusion with it.
- It can handle GBC games as well.

# Features

- Integreated disassembler.
	- Disassemble individual instructions via command line:
	- Disassemble sections of the loaded ROM.
- TODO

# Build and Run

Building the emulator currently requires a nightly Rust installation, version 1.8 or higher. If you have it, you can `cd` into the source directory and do one of the following:

- To run all tests, execute `cargo test`.
- To generate an HTML documentation for the source code, execute `cargo doc`.
- To build the emulator, execute `cargo build --release`.
- To run the emulator, execute `cargo run -- ARGS...` with any command line arguments `ARGS...`. You may want to try `cargo run -- --help` to get a list of all supported command line arguments.

# Tools
- **[wxHexEditor](http://www.wxhexeditor.org/)**
	
	> a free hex editor / disk editor for Linux, Windows and MacOSX
	
	So far my favourite hex editor. It is obviously a GUI application based on [wxWidgets](http://www.wxwidgets.org/).
- **[MSYS2](https://sourceforge.net/p/msys2/wiki/MSYS2%20installation/)**
	
	> A Cygwin-derived software distro for Windows using Arch Linux's Pacman
	
	If you work on a Windows machine but find yourself wanting to use Bash, binutils, and other Linux tools, then this might be just what you needed.
- **[Notepad++ Rust Syntax](https://github.com/pfalabella/Rust-notepadplusplus)**
	
	> Notepad++ syntax highlighting for Rust.
	
	There are some missing highlights like the keyword `crate`, but these are easily added by hand.
	
- **[Online Disassembler](https://onlinedisassembler.com/odaweb/)**
	
	> Disassembly in the cloud.
	
	As the name states: A website disassembling your binaries. Just copy and paste sections of your ROM files and have fun. Note that the GBA works in Little Endian mode.

- **[ARM7TDMI Instruction Set Reference](http://morrow.ece.wisc.edu/ECE353/arm7tdmi_instruction_set_reference.pdf)**
	
	> Department of Electrical and Computer Engineering University of Wisconsin-Madison
	
	Thanks to those guys, here is an instruction set reference manual for the ARM7TDMI, which is the very CPU used in the GBA.


# References
- **[ProblemKaputt.de - GBATEK](http://problemkaputt.de/gbatek.htm)**
	
	> GBATEK written 2001-2014 by Martin Korth, programming specs for the GBA and NDS hardware
	
	A pretty detailed one-page with almost all the information you need.
- **[GBADEV.org](http://www.gbadev.org/docs.php)**
	
	> It started as a website aiming to share development information about the then unreleased GBA system. Here we are, 16 years later, doing the same thing. Who would have thought?
	
	The first GBA reference site I stumbled upon.
- **[Rust Programming Language](https://www.rust-lang.org/)**
	
	> Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.
	
	This language is just beautiful. Try it! Use it! It even provides an online book and other useful resources to learn and master this language. And if you use DuckDuckGo as a search engine, you can browse the standard library documentation using `!rust` for Rust stable or `!rustn` for Rust nightly.
- **[Cargo Package Manager](https://crates.io/)**
	
	> The Rust community’s crate host
	
	Rust's package manager. Here you can find almost all Rust packages, called "crates".
- **[Rustendo64](https://github.com/yupferris/rustendo64)**
	
	> Livecoding a Nintendo 64 emulator in Rust :D
	
	This project inspired me to make this emulator. [Here](https://www.youtube.com/channel/UC4mpLlHn0FOekNg05yCnkzQ/videos) is a YouTube channel containing recordings of Ferris' Rustendo64 streams.

# License

Licensed under the [Mozilla Public License 2.0](./LICENSE-MPL.md).
