use alloc::{string::{ToString, String}, vec::Vec};
use pc_keyboard::DecodedKey;
use crate::{print, println, io::vga_buffer::{WRITER, writer::BUFFER_HEIGHT}};

use super::exit_qemu;

static mut COMMAND_DRAFT: String = String::new();
static mut COMMAND: String = String::new();

pub fn get_char(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(character) => {
            if character == '\n' {
                unsafe {
                    COMMAND = COMMAND_DRAFT.to_string();
                    COMMAND_DRAFT = String::new();
                };
                println!();
                run();
                print!("\n> ");
            } else if character == '\x08' {
                // \x08 -> \b
                // rust doesn't support \b directly
                unsafe { COMMAND_DRAFT.pop(); }

                // update the vga buffer
                WRITER.lock().clear_row(BUFFER_HEIGHT-1);
                unsafe { print!("\n> {}", COMMAND_DRAFT); }
            } else {
                unsafe { COMMAND_DRAFT += character.to_string().as_str(); };
                print!("{}", character);
            }
        }
        DecodedKey::RawKey(_rawkey) => todo!(),
    }
}

fn run() {
    let (command, args): (String, Vec<&str>) =  {
        let mut temp = unsafe { COMMAND.split(' ') };
        let cmd = match temp.next() {
            Some(i) => i,
            None => ""
        };
        // exterme parsing, ik
        let mut arg = Vec::new();
        #[allow(while_true)]
        while true {
            let item = match temp.next() {
                Some(i) => i,
                None => "\x08"
            };
            if item == "\x08" {
                break;
            }
            arg.push(item);
        }

        (cmd.to_lowercase(), arg)
    };

    if command == "exit" {
        exit_qemu(super::QemuExitCode::Success);
    } else if command == "clear" {
        for i in 1..BUFFER_HEIGHT {
            WRITER.lock().clear_row(i);
        }
    }else if command == "echo" {
        println!("{}", args.join(" "));
    } else {
        println!("{} is not a command", command);
    }
}
