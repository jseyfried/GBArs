# GBArs

A GBA emulator written in Rust.

Thanks to a guy named Ferris and his project [Rustendo 64](https://github.com/yupferris/rustendo64), many people got motivated to write their own emulators in Rust. Even I wasn't spared, so here it is, my GBA emulator.

And why GBA?

- It is ARM-based and ARM is sexy.
- I want to play Metroid Zero Mission and Fusion with it.
- It can handle GBC games as well.

# Features

- A stupid name, pronounced: "G-B-Ars"
- Integreated disassembler.
	- Disassemble individual instructions via command line.
	- Disassemble sections of the loaded ROM via command line.
	- Mimics the standard ARM assembly language, although some parts of the syntax work differently. The integrated disassembler also does not support pseudo instructions like `push`/`pop`, to make the code simpler.
- Optional optimised BIOS routines.
- It is entirely written in Rust, a safe and awesome language.
- TODO

# Build and Run

Building the emulator currently requires a nightly Rust installation, version 1.8 or higher. If you have it, you can `cd` into the source directory and do one of the following:

- To run all tests, execute `cargo test`.
- To generate an HTML documentation for the source code, execute `cargo doc`.
- To build the emulator, execute `cargo build --release`.
- To run the emulator, execute `cargo run -- ARGS...` with any command line arguments `ARGS...`. You may want to try `cargo run -- --help` to get a list of all supported command line arguments.

# Examples

- **Change the log file**
			
			GBArs --log path/to/logfile.log
			
- **Run a game**
	
	The SRAM file contains any saved data.
			
			GBArs --rom ./ZeroMission.gba --load-sram
			
- **Disassemble an ARM state instruction**
	
	The instruction to disassemble must be given in big endian hexadecimal format without base.
			
			GBArs --dasm-arm DEADBEEF
			
- **Disassemble a section of the BIOS**
	
	Disassembles the first 128 bytes, 32 ARM state instructions. This also loads a BIOS ROM from any given file. If no such file is given, an internal default BIOS ROM will be loaded.
			
			GBArs --bios ./GbaBios.gba --dasm-bios 0..80

# Tools
- **[wxHexEditor](http://www.wxhexeditor.org/)**

	> a free hex editor / disk editor for Linux, Windows and MacOSX

	So far my favourite hex editor. It is obviously a GUI application based on [wxWidgets](http://www.wxwidgets.org/).
- **[ATOM](https://atom.io/)**

    > A hackable text editor

    This is quite a decent source code editor and it's what I'm currently using to develop GBArs. For this project here I at least recommend using these packages:

    - `editorconfig` for tab settings and what not.
    - `language-rust` for Rust language support.
    - `linter-rust` for showing warnings etc. in the status bar.
- **[Online Disassembler](https://onlinedisassembler.com/odaweb/)**

	> Disassembly in the cloud.

	As the name states: A website disassembling your binaries. Just copy and paste sections of your ROM files and have fun. To use it, apply the following settings:
	
	- **Arch**: ARMv4T (T = THUMB support)
	- **Endian**: Little
	- **Force THUMB Mode**: Default (Might need to toggle in case the disassembly is wrong.)
	- **Register Names**: reg-names-std (Just a matter of taste.)
	
	A hint on how you recognise whether the website erroneously disassembles ARM instructions as THUMB instructions: The first instruction of a BIOS or GamePak ROM file is almost always an unconditional ARM state branch like `b loc_00000058`.
	
	Also note that ARMv4 is just the "instruction set version". Although the GBA's CPU is an ARM7TDMI (again, T = THUMB), its instruction set is ARMv4.
- **[ARM7TDMI Technical Reference Manual](http://www.atmel.com/Images/ddi0029g_7tdmi_r3_trm.pdf)**
	
	> This document is Open Access. This document has no restriction on distribution.
	
	This PDF also includes a data sheet on the THUMB state instruction set. There aren't, however, no detailed notes on what the different instructions actually do, as this manual focuses more on delays and signals.
- **[ARM7TDMI Instruction Set Reference](http://morrow.ece.wisc.edu/ECE353/arm7tdmi_instruction_set_reference.pdf)**

	> Department of Electrical and Computer Engineering University of Wisconsin-Madison

	Thanks to those guys, here is an instruction set reference manual for the ARM7TDMI, which is the very CPU used in the GBA.


# References
- **[Rustendo64](https://github.com/yupferris/rustendo64)**

	> Livecoding a Nintendo 64 emulator in Rust :D

	This project inspired me to make this emulator. [Here](https://www.youtube.com/channel/UC4mpLlHn0FOekNg05yCnkzQ/videos) is a YouTube channel containing recordings of Ferris' Rustendo64 streams. You'll find stuff like the GitHub repository link in the video descriptions.
- **[ProblemKaputt.de - GBATEK](http://problemkaputt.de/gbatek.htm)**

	> GBATEK written 2001-2014 by Martin Korth, programming specs for the GBA and NDS hardware

	A pretty detailed one-page with almost all the information you need.
- **[Rust Programming Language](https://www.rust-lang.org/)**

	> Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.

	This language is just beautiful. Try it! Use it! It even provides an online book and other useful resources to learn and master this language. And if you use DuckDuckGo as a search engine, you can browse the standard library documentation using `!rust` for Rust stable or `!rustn` for Rust nightly.
- **[Cargo Package Manager](https://crates.io/)**

	> The Rust community’s crate host

	Rust's package manager. Here you can find almost all Rust packages, called "crates".

# License

Licensed under the [Apache License 2.0](./LICENSE-APACHE.md).

- [https://www.apache.org/licenses/LICENSE-2.0.html](https://www.apache.org/licenses/LICENSE-2.0.html)
- [https://tldrlegal.com/license/apache-license-2.0-(apache-2.0)](https://tldrlegal.com/license/apache-license-2.0-%28apache-2.0%29)