use std::{
    io::{self, Read, stdin, Write},
    process::exit,
};

use clap::Parser;

use basehan::BaseHanError;
use basehan::v1::{BaseHanDecoder, BaseHanEncoder};

// Base-Han is a command line tool to encode/decode binary data to/from Base-Han.
#[derive(Debug, Parser)]
#[command(author, about, version)]
struct Args {
    // Whether encode or docode
    #[clap(short, long, default_value = "false")]
    decode: bool,
    #[clap(short, long, default_value = "false")]
    interactive: bool,
    #[clap(short, long, default_value = "3145728")]
    chunk_size: usize,
}

const ENCODE_PROMPT: &str = "encode> ";
const DECODE_PROMPT: &str = "decode> ";

fn interactive_shell(decode: bool) {
    println!("Interactive mode.");
    let stdin = stdin();
    loop {
        eprint!("{}", if decode { DECODE_PROMPT } else { ENCODE_PROMPT });
        match io::stdout().flush() {
            Ok(_) => (),
            Err(e) => {
                println!("Error: Failed to flush stdout: {:?}", e);
                exit(1);
            }
        };
        let mut buffer = String::new();
        let read_size = stdin.read_line(&mut buffer).unwrap();
        if read_size == 0 {
            break;
        }
        let buffer = buffer.trim();
        if buffer == "exit" {
            break;
        }
        if decode {
            let result = basehan::decode(&buffer.to_string());
            match result {
                Ok(bytes) => {
                    io::stdout().write_all(&bytes).unwrap();
                    println!();
                }
                Err(e) => {
                    println!("Error: Internal error.{:?}", e);
                }
            }
        } else {
            let result = basehan::encode(buffer);
            match result {
                Ok(result) => println!("{}", result),
                Err(err) => println!("Error: Please input a valid BaseHan cipher.{:?}", err),
            }
        }
    }
    println!("Exit");
}


fn v1(args: Args) {
    if args.decode {
        let mut buf = vec![0u8; args.chunk_size];
        let mut decoder = BaseHanDecoder::new();
        loop {
            buf.fill(0);
            let n = io::stdin().read(&mut buf).unwrap();

            if n == 0 {
                if let Some(_) = decoder.finish() {
                    panic!("The string input is corrupted!")
                }
                break;
            }
            let char_buf: Vec<char> = String::from_utf8_lossy(&buf).chars().collect();
            let out = decoder.update(char_buf).unwrap();
            io::stdout().write_all(&out).unwrap();
            io::stdout().flush().unwrap();
        }
    } else {
        let mut buf = vec![0u8; args.chunk_size];
        let mut encoder = BaseHanEncoder::new();
        loop {
            buf.fill(0);
            let n = io::stdin().read(&mut buf).unwrap();
            if n == 0 {
                let out = [encoder.finish()];
                io::stdout().write_all(String::from_iter(out).as_bytes()).unwrap();
                break;
            }
            let out = encoder.update(&buf[..n]).unwrap();
            io::stdout().write_all(String::from_iter(out).as_bytes()).unwrap();
            io::stdout().flush().unwrap();
        }
    }
    io::stdout().flush().unwrap();
}

#[allow(dead_code)]
fn v0(args: Args) {

    if args.interactive {
        return interactive_shell(args.decode);
    }

    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .map_err(|e| BaseHanError::InternalError(format!("Failed to read from stdin: {:?}", e)))
        .unwrap_or_else(|e| error_handler(e));
    if args.decode {
        // check is string
        let buffer = String::from_utf8(buffer)
            .map_err(|e| {
                BaseHanError::InternalError(format!("Failed to convert to string: {:?}", e))
            })
            .unwrap_or_else(|e| error_handler(e));
        let mut result = basehan::decode(&buffer).unwrap_or_else(|err| error_handler(err));
        // let result = String::from_utf8(result).expect("Internal bugs occurred when decoding.").to_string();
        // result.push('\n' as u8);
        io::stdout()
            .write_all(&result)
            .map_err(|e| BaseHanError::InternalError(format!("Failed to write to stdout: {:?}", e)))
            .unwrap_or_else(|e| error_handler(e));
    } else {
        let mut result = basehan::encode(buffer).unwrap_or_else(|err| error_handler(err));
        // result.push('\n');
        io::stdout()
            .write_all(result.as_bytes())
            .map_err(|e| BaseHanError::InternalError(format!("Failed to write to stdout: {:?}", e)))
            .unwrap_or_else(|e| error_handler(e));
    }
    io::stdout()
        .flush()
        .map_err(|e| BaseHanError::InternalError(format!("Failed to write to stdout: {:?}", e)))
        .unwrap_or_else(|e| error_handler(e));
}

fn main() {
    let args = Args::parse();
    return v1(args);
}

fn error_handler(err: BaseHanError) -> ! {
    match err {
        BaseHanError::InternalError(msg) => {
            eprintln!("Internal error: {}", msg);
        }
        BaseHanError::InvalidCode(code, pos) => {
            eprintln!("Invalid input: code {:#x} at pos {}", code, pos);
        }
    }
    exit(1);
}
