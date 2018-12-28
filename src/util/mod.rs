use crypto::rc4::Rc4;
use crypto::symmetriccipher::SynchronousStreamCipher;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::iter::repeat;
use std::process::Command;
mod template;

fn encode(input: String, key: String) -> Vec<u8> {
    return encode_vec(input.as_bytes().to_vec(), key);
}

fn encode_vec(input: Vec<u8>, key: String) -> Vec<u8> {
    let mut rc4 = Rc4::new(key.as_bytes());
    let mut output: Vec<u8> = repeat(0).take(input.len()).collect();
    rc4.process(&input, &mut output);
    return output.to_vec();
}

pub fn find_interp(content: &String) -> (String, String) {
    if content.starts_with("#!") {
        let lines: Vec<&str> = content.split("\n").collect();
        let first: Vec<&str> = lines[0].split(" ").collect();
        if first.len() < 2 {
            (String::from("bash"), content.to_owned())
        } else {
            let interp = String::from(
                first[first.len() - 2]
                    .split("/")
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .to_owned(),
            );
            (interp, lines[1..lines.len()].join("\n"))
        }
    } else {
        (String::from("bash"), content.to_owned())
    }
}

fn compile_it(file: &String) {
    println!("compile it ... {}", file);
    let output = Command::new("rustc")
        .arg(file)
        .output()
        .expect("failed to compile");

    let stdout = output.stdout;
    let stderr = output.stderr;
    if stdout.len() > 0 {
        println!("{}", String::from_utf8_lossy(&stdout));
    }
    if stderr.len() > 0 {
        println!("{}", String::from_utf8_lossy(&stderr));
    }
    if output.status.success() {
        println!(
            "compiled success, try it with: ./{}",
            file.replace(".rs", "")
        );
    } else {
        std::process::exit(1);
    }
}

pub fn gen_and_compile(file: &str, rs_file: &str, pass: &str) {
    let content = fs::read_to_string(file).expect("Failed to read source file");
    // we need to encode it latter
    let _encoded = encode(content.clone(), "hello".to_string());
    let (interp, content) = find_interp(&content);
    //println!("{}", content);
    let encoded_str = format!("vec!{:?}\n", content.as_bytes());
    let prog = template::prog()
        .replace("{ script_code }", &encoded_str)
        .replace("{ pass }", &pass)
        .replace("{ interp }", &interp);

    File::create(rs_file)
        .unwrap()
        .write_all(prog.as_bytes())
        .unwrap();
    compile_it(&rs_file.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;

    #[test]
    fn test_find_interp() {
        let text = String::from("#!/bin/expect -f\nsend 1 2 3");
        let (interp, _) = find_interp(&text);
        println!("interp: {}", interp);
        assert!(interp == "expect");

        let text = String::from("#!/bin/bash -f\nsend 1 2 3");
        let (interp, _) = find_interp(&text);
        println!("interp: {}", interp);
        assert!(interp == "bash");

        let text = String::from("#!/bash -f\nsend 1 2 3");
        let (interp, _) = find_interp(&text);
        println!("interp: {}", interp);
        assert!(interp == "bash");

        let text = String::from("send 1 2 3");
        let (interp, _) = find_interp(&text);
        println!("interp: {}", interp);
        assert!(interp == "bash");
    }
    #[test]
    fn test_encode_decode() {
        let content = String::from("ahah, this is hello world!");
        let encoded = encode(content.clone(), "hello".to_string());
        let decoded = encode_vec(encoded, "hello".to_string());
        let result = String::from_utf8_lossy(&decoded);
        assert!(result == content);
    }

    #[test]
    fn test_compile_run() {
        let exe = env::current_exe().unwrap();
        let mut elems: Vec<&str> = exe.to_str().unwrap().split("/").collect();
        unsafe {
            elems.set_len(elems.len() - 4);
        }
        let path = elems.join("/") + "/examples/";
        env::set_current_dir(Path::new(&path)).is_ok();
        let files = fs::read_dir(path.to_owned()).unwrap();
        for file in files {
            let p = file.unwrap().path();
            let s = p.to_str().unwrap();
            if !s.ends_with(".out") && s.contains(".") {
                let out = format!("{}.out", s.replace(".", "_"));
                println!("out: {} {}", s, out);
                gen_and_compile(s, &out.to_owned(), "");
            }
        }
    }

    #[test]
    fn test_encode() {
        struct Test {
            key: &'static str,
            input: &'static str,
            output: Vec<u8>,
        }

        fn tests() -> Vec<Test> {
            vec![
                Test {
                    key: "Key",
                    input: "Plaintext",
                    output: vec![0xBB, 0xF3, 0x16, 0xE8, 0xD9, 0x40, 0xAF, 0x0A, 0xD3],
                },
                Test {
                    key: "Wiki",
                    input: "pedia",
                    output: vec![0x10, 0x21, 0xBF, 0x04, 0x20],
                },
                Test {
                    key: "Secret",
                    input: "Attack at dawn",
                    output: vec![
                        0x45, 0xA0, 0x1F, 0x64, 0x5F, 0xC3, 0x5B, 0x38, 0x35, 0x52, 0x54, 0x4B,
                        0x9B, 0xF5,
                    ],
                },
            ]
        }

        let tests = tests();
        for t in tests.iter() {
            let result = encode(t.input.to_string(), t.key.to_string());
            assert!(result == t.output);
        }
    }
}