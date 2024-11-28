use aarch64::current_el;
use fat32::traits::Entry;
use fat32::traits::Metadata;
use shim::io::Write;
use shim::path::Path;
use stack_vec::StackVec;

use fat32::traits::Dir;
use fat32::traits::FileSystem;

use crate::console::{kprint, kprintln, CONSOLE};
use crate::FILESYSTEM;

use core::prelude::rust_2024::derive;

use core::fmt::Debug;
use core::iter::Iterator;
use core::result::Result;
use core::result::Result::{Err, Ok};

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

// const WELCOME_TXT: &str = r#"
//                                   _
//                                _ooOoo_
//                               o8888888o
//                               88" . "88
//                               (| -_- |)
//                               O\  =  /O
//                            ____/`---'\____
//                          .'  \\|     |//  `.
//                         /  \\|||  :  |||//  \
//                        /  _||||| -:- |||||_  \
//                        |   | \\\  -  /'| |   |
//                        | \_|  `\`---'//  |_/ |
//                        \  .-\__ `-. -'__/-.  /
//                      ___`. .'  /--.--\  `. .'___
//                   ."" '<  `.___\_<|>_/___.' _> \"".
//                  | | :  `- \`. ;`. _/; .'/ /  .' ; |
//                  \  \ `-.   \_\_`. _.'_/_/  -' _.' /
// ==================`-.`___`-.__\ \___  /__.-'_.'_.-'================
//  ____  _     _     _ _                _   _            ___  ____  _
// / ___|(_) __| | __| | |__   __ _ _ __| |_| |__   __ _ / _ \/ ___|| |
// \___ \| |/ _` |/ _` | '_ \ / _` | '__| __| '_ \ / _` | | | \___ \| |
//  ___) | | (_| | (_| | | | | (_| | |  | |_| | | | (_| | |_| |___) |_|
// |____/|_|\__,_|\__,_|_| |_|\__,_|_|   \__|_| |_|\__,_|\___/|____/(_)"#;

const WELCOME_TXT: &str = r#"
                            __
                           /  `~-,
                          /     /
                 ___     {     /
              ,-'   `-.  ;    :
             /         \/    ;'
            !           |   |
          ,-|        ,--+-. \,--.
         /  {       /      \/    \
         |   \     /        ;     )
         |   /`-.__|        |    _!_
         \  (      |        )  ,'   `.
          `-|       \      / )/       \
           _(        `-..-'  !         |
         ,'  \  ___  /,-"""-.|         |
        /     ,'   `./       \\       /
        |    /       \        )`-._,-'
        (    |       |        |    |
         \   \       /        ).  /
         /`-._:-._,-'\       /  \'
        (        ,'   `-._,-'    |
        (       /        \       |
         \     (          )     /
          `+.__|          |__,-';
           |   (          )     |
           \  / \        /      |
            `-|  `-.__,-'      /
              |       |`-.__,-'
              \       /   |
               `-._.-'    )
                 \       /
                  `-._,-' 
            _      _ _        ___  ____  
           | | ___| | |_   _ / _ \/ ___| 
        _  | |/ _ \ | | | | | | | \___ \ 
       | |_| |  __/ | | |_| | |_| |___) |
        \___/ \___|_|_|\__, |\___/|____/ 
                       |___/                     
"#;

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

use alloc::vec::Vec;
/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
use core::str::from_utf8;
const ROOT_NAME: &str = "/";
use crate::alloc::string::ToString;
const MAX_LINE_LENGTH: usize = 512;
pub fn shell(prefix: &str) {
    let mut pwd = Path::new(ROOT_NAME).to_path_buf();

    kprintln!("{}", WELCOME_TXT);

    let mut pwd_dir = FILESYSTEM.open_dir(&pwd).expect("directory");

    let mut console = CONSOLE.lock();
    'exit: loop {
        kprint!("({}) {} ", pwd.display().to_string().to_uppercase(), prefix);
        let mut storage = [0; MAX_LINE_LENGTH]; // maxiumum command size
        let mut line: StackVec<u8> = StackVec::new(&mut storage);
        let mut idx = 0;

        // get bytes
        loop {
            match console.read_byte() {
                b'\r' | b'\n' => break,
                8 | 127 => {
                    if idx != 0 {
                        console.write_byte(8u8);
                        console.write_byte(b' ');
                        console.write_byte(8u8);
                        idx -= 1;
                        line.pop();
                    }
                }
                byte if (byte as char).is_ascii() && idx < MAX_LINE_LENGTH => match line.push(byte)
                {
                    Ok(()) => {
                        kprint!("{}", byte as char);
                        idx += 1;
                    }
                    Err(()) => {
                        console
                            .write("failed".as_bytes())
                            .expect("failed to write to console");
                    }
                },
                _ => {
                    console.write_byte(7u8); // rings the bell
                }
            }
        }
        kprintln!("");

        match from_utf8(line.into_slice()) {
            Ok(command_string) if command_string.len() != 0 => {
                let mut buf = [""; 64];
                match Command::parse(command_string, &mut buf) {
                    Ok(command) if command.path() == "echo" => {
                        command.args.iter().skip(1).for_each(|s| kprint!("{} ", *s));
                        kprintln!("");
                    }
                    Ok(command) if command.path() == "welcome" => {
                        kprintln!("{}", WELCOME_TXT);
                    }
                    Ok(command) if command.path() == "ls" => {
                        let hide = if command.args.contains(&"-a") {
                            false
                        } else {
                            true
                        };

                        let dir_result = if command.args.len() > (1 + (!hide as usize)) {
                            let path = Path::new(command.args.last().expect("parse error"));
                            let mut curr_path = pwd.clone();
                            if let Ok(fat32::vfat::Entry::DirEntry(new_dir)) =
                                pwd_dir.open_path(path, &mut curr_path)
                            {
                                Some(new_dir)
                            } else {
                                kprintln!(
                                    "error: dir {} not found",
                                    path.display().to_string().to_uppercase()
                                );
                                None
                            }
                        } else {
                            None
                        };

                        let current_dir = dir_result.as_ref().unwrap_or(&pwd_dir);

                        current_dir
                            .entries()
                            .expect("entries interator")
                            .collect::<Vec<_>>()
                            .iter()
                            .for_each(|entry| {
                                if !(hide && entry.metadata().hidden()) {
                                    kprintln!("{}", entry);
                                }
                            });
                    }
                    Ok(command) if command.path() == "pwd" => {
                        kprintln!("{}", pwd.display().to_string().to_uppercase());
                    }
                    Ok(command) if command.path() == "cd" => {
                        let path = Path::new(command.args[1]);

                        if let Ok(fat32::vfat::Entry::DirEntry(new_dir)) =
                            pwd_dir.open_path(path, &mut pwd)
                        {
                            pwd_dir = new_dir;
                        } else {
                            kprintln!(
                                "error: dir {} not found",
                                path.display().to_string().to_uppercase()
                            );
                        }
                    }
                    Ok(command) if command.path() == "cat" => {
                        for p in command.args.iter().skip(1) {
                            let path = Path::new(p);
                            if let Ok(fat32::vfat::Entry::FileEntry(entry)) =
                                pwd_dir.open_path(path, &mut pwd.clone())
                            {
                                kprintln!("{}", entry);
                            } else {
                                kprintln!(
                                    "error: file {} not found",
                                    path.display().to_string().to_uppercase()
                                );
                            }
                        }
                    }
                    Ok(command) if command.path() == "exit" => {
                        break 'exit;
                    }
                    Ok(command) if command.path() == "sleep" => {
                        use crate::Duration;
                        if command.args.len() == 2
                            && let Some(ms) =
                                command.args.last().and_then(|v| v.parse::<u32>().ok())
                        {
                            // kprintln!("elapsed_ms:{}", kernel_api::syscall::sleep(Duration::from_millis(ms as u64)).expect("sleep failed").as_millis());
                        }
                    }
                    Ok(command) => {
                        kprintln!("unknown command: {}", command.path());
                    }
                    Err(Error::TooManyArgs) => {
                        kprintln!("error: too many arguments");
                    }
                    _ => {
                        kprintln!("error: failed to parse");
                    }
                }
            }
            _ => {}
        }
    }
}
