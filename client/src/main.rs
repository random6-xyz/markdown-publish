use reqwest::{blocking::Client, StatusCode};
use std::{env, fs::File, process};

enum Command {
    Upload,
    Remove,
    List,
    Publish,
    Unknown,
}

struct Config {
    command: Command,
    file_names: Vec<String>,
}

impl Config {
    fn new() -> Self {
        Self {
            command: Command::Unknown,
            file_names: Vec::new(),
        }
    }
}

enum Status {
    Success,
    Unprocessable,
    FileNotFound,
    NetworkError,
}

struct FileStatus {
    file_name: String,
    status: Status,
}

const IPADDRESS: &str = "127.0.0.1:8080";

fn main() {
    let config = parse_args();
    let result: Vec<FileStatus> = match config.command {
        Command::Upload => command_upload(config.file_names),
        Command::Remove => command_remove(config.file_names),
        Command::List => command_list(config.file_names),
        Command::Publish => command_publish(config.file_names),
        Command::Unknown => {
            command_help();
            process::exit(0);
        }
    };
}

fn parse_args() -> Config {
    let mut config = Config::new();

    let args: Vec<String> = env::args().collect();
    let command = args[1].clone();

    config.command = match command.as_str() {
        "upload" | "u" => Command::Upload,
        "remove" | "h" => Command::Remove,
        "list" | "l" => Command::List,
        "publish" | "p" => Command::Publish,
        _ => Command::Unknown,
    };

    for file_names in args[2..].iter() {
        config.file_names.push(file_names.to_string());
    }

    config
}

fn command_help() {
    println!(
        r#"markdown publish - Simple Markdown Publisher
    # Usage
        markdown_publish <command> [<args>]

    # Command
        - 'upload' or 'u'  -> upload markdown files to the server
        - 'remove' or 'r'  -> remove markdown files form the server
        - 'list' or 'l'    -> list markdown files in server
        - 'publish' or 'p' -> publish markdwon files in the server
        
    # Args
        - give list of files
    
    # Examples
        - markdown_publish upload ./file1.md ./file2.md
        - markdown_publish r ./file1.md
        - markdown_publish list
        - markdown_publish p ./file1 ./file2"#
    );
}

fn command_upload(file_names: Vec<String>) -> Vec<FileStatus> {
    let mut file_status: Vec<FileStatus> = Vec::new();

    for file_name in file_names {
        let file = match File::open(file_name.clone()) {
            Err(_) => {
                file_status.push(FileStatus {
                    file_name: file_name,
                    status: Status::FileNotFound,
                });
                continue;
            }
            Ok(result) => result,
        };

        let client = Client::new();
        let response = match client
            .post(format!("{}/upload/{}", IPADDRESS, file_name))
            .body(file)
            .send()
        {
            Err(_) => {
                file_status.push(FileStatus {
                    file_name: file_name,
                    status: Status::NetworkError,
                });
                continue;
            }
            Ok(result) => result,
        };

        match response.status() {
            StatusCode::ACCEPTED => {
                file_status.push(FileStatus {
                    file_name: file_name,
                    status: Status::Success,
                });
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                file_status.push(FileStatus {
                    file_name: file_name,
                    status: Status::Unprocessable,
                });
            }
            _ => {
                file_status.push(FileStatus {
                    file_name: file_name,
                    status: Status::NetworkError,
                });
            }
        }
    }

    file_status
}

fn command_remove(file_names: Vec<String>) -> Vec<FileStatus> {
    Vec::new()
}

fn command_list(file_names: Vec<String>) -> Vec<FileStatus> {
    Vec::new()
}

fn command_publish(file_names: Vec<String>) -> Vec<FileStatus> {
    Vec::new()
}
