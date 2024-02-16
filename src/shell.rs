use alloc::{string::{ToString, String}, vec::Vec, format};
use pc_keyboard::DecodedKey;
use crate::{print, println, io::vga_buffer::{WRITER, writer::BUFFER_HEIGHT}};

use super::{exit_qemu, QemuExitCode};

static mut COMMAND_DRAFT: String = String::new();
static mut COMMAND: String = String::new();
static mut HISTORY: Vec<String> = Vec::new();
static mut POINT: usize = 0;

fn check_or_add() {
    unsafe {
        match HISTORY.get(POINT) {
            Some(_) => (),
            None => {
                HISTORY.push(String::new());
                ()
            }
        }
    }
}

fn update_history() {
    unsafe {
        match HISTORY.get(POINT) {
            Some(_) => HISTORY[POINT] = COMMAND_DRAFT.clone(),
            None => ()
        }
    }
}

fn update_display() {
    WRITER.lock().clear_row(BUFFER_HEIGHT-1);
    unsafe { print!("\n> {}", COMMAND_DRAFT); }
}

pub fn get_char(key: DecodedKey) {
    check_or_add();
    match key {
        DecodedKey::Unicode(character) => {
            if character == '\n' {
                unsafe {
                    let topush = COMMAND_DRAFT.clone();

                    COMMAND = COMMAND_DRAFT.to_string();
                    COMMAND_DRAFT = String::new();
                    
                    // update history
                    HISTORY.remove(POINT);
                    HISTORY.insert(0, topush);
                    POINT = 0;
                };

                println!();
                run();
                print!("\n> ");
            } else if character == '\x08' {
                // \x08 -> \b
                // rust doesn't support \b directly
                unsafe { COMMAND_DRAFT.pop(); }

                update_display();
            } else {
                unsafe { COMMAND_DRAFT += character.to_string().as_str(); };
                print!("{}", character);
            }
        }
        DecodedKey::RawKey(rawkey) => {
            let name = format!("{:?}", rawkey);
            unsafe {
            if name == "ArrowUp" {
                    if POINT == 0 {
                        return;
                    }

                    POINT -= 1;
                    COMMAND_DRAFT = HISTORY[POINT].clone();
            } else if name == "ArrowDown" {
                if COMMAND_DRAFT == "" {
                    return;
                }
                COMMAND_DRAFT = HISTORY[POINT].clone(); // save the old draft
                POINT += 1;
                check_or_add();
                COMMAND_DRAFT = HISTORY[POINT].clone(); // load the new draft
            }
            }

            update_display();
        },
    }

    update_history();
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

    if command == "" { // do nothing
    } else if command == "exit" {
        exit_qemu(QemuExitCode::Success); // ! not working
        super::hlt_loop();
    } else if command == "clear" {
        for i in 1..BUFFER_HEIGHT {
            WRITER.lock().clear_row(i);
        }
    }else if command == "echo" {
        println!("{}", args.join(" "));
    } else if command == "rand" {
        // let rand = x86_64::instructions::random::RdRand(());
        // println!("{:?}", rand);
    } else {
        println!("{} is not a command", command);
    }
}
