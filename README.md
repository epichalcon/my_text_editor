# Texty

This proyect is a rust written text editor. The goal is to write a text editor following the ["Build your own text editor"](https://viewsourcecode.org/snaptoken/kilo/index.html) porting the code to rust. This way I intend to learn what goes in writing a text editor and I can practice and carry on learning rust.

---

## Building the proyect

As the proyect is writen in rust and uses the Cargo package manager, rust and cargo must be installed in the system. To install them follow the instructions in this [link](https://doc.rust-lang.org/cargo/getting-started/installation.html) 

To build the proyect do the following steps:
- clone the repo
- compile the proyect (this will generate the executable in the target/release folder)
- copy the executable to the root folder

```
git clone https://github.com/epichalcon/texty.git
cd texty 
cargo build --release
cp target/release/texty .
```

## Executing the proyect

To execute the proyect run the following comand 

`texty [file_name]`

If a file name is provided, the text editor will open the file, if the file does not exist, it will be created. If a file name is not provided, a new file will be created.

## Functionality

The commands to use the editor are the following:
- Ctrl-Q: Quit (if the file has not been saved, a warning message will be displayed)
- Ctrl-S: Save (if it is a new file the user will be promped to provide a name for the file)
- Ctrl-F: Search

Navegation will be done with the arrow keys.
