type Result<T> = std::result::Result<T, &'static str>;

mod api;

use api::Api;
use api::module::File;
use std::collections::HashSet;
use std::io;
use std::io::Write;

fn flush_stdout() {
    io::stdout().flush().expect("Unable to flush stdout");
}

fn get_input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    flush_stdout();
    io::stdin().read_line(&mut input).expect("Unable to get input");
    input.trim().to_string()
}

fn get_password(prompt: &str) -> String {
    print!("{}", prompt);
    flush_stdout();
    rpassword::read_password().expect("Unable to get non-echo input mode for password")
}

fn print_files(file: &File, api: &Api, prefix: &str) -> Result<bool> {
    if file.is_directory {
        for mut child in file.children.clone().ok_or("children must be preloaded")?.into_iter() {
            child.load_children(api)?;
            print_files(&child, api, &format!("{}/{}", prefix, file.name))?;
        }
    } else {
        println!("{}/{}", prefix, file.name);
    }
    Ok(true)
}

fn main() {
    let username = get_input("Username: ");
    let password = get_password("Password: ");
    let api = Api::with_login(&username, &password).expect("Unable to login");
    println!("Your name is {}", api.name().expect("Unable to read name"));
    for module in api.modules(true).expect("Unable to retrieve modules") {
        println!("# {} {}, teaching: {}", module.code, module.name, module.is_teaching());
        println!();
        println!("## Announcements");
        for announcement in module.get_announcements(&api, false).expect("Unable to retrieve announcements") {
            println!("=== {} ===", announcement.title);
            let stripped = ammonia::Builder::new().tags(HashSet::new()).clean(&announcement.description).to_string();
            let decoded = htmlescape::decode_html(&stripped).expect("Unable to decode HTML Entities");
            println!("{}", decoded);
        }
        print_files(&module.as_file(&api, true).expect("Unable to retrieve files"), &api, "").expect("Unable to print files");
    }
}
