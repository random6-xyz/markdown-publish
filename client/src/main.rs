use lazy_static::lazy_static;
use reqwest::{blocking::Client, header, StatusCode};
use std::{collections::HashMap, env, fmt, fs::File, path::Path, process};

enum Command {
    Upload,
    Remove,
    List,
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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Success => write!(f, "Success"),
            Status::Unprocessable => write!(f, "Unporcessable"),
            Status::FileNotFound => write!(f, "FileNotFound"),
            Status::NetworkError => write!(f, "NetworkError"),
        }
    }
}

struct FileStatus {
    file_name: String,
    status: Status,
}

lazy_static! {
    static ref APIKEY: String = {
        let builder = config::Config::builder().add_source(config::File::new(
            format!(
                "{}{}",
                dirs::home_dir().expect("No home directory.").display(),
                "/.config/markdown-publish-client/setting.toml"
            )
            .as_str(),
            config::FileFormat::Toml,
        ));

        let config = builder
            .build()
            .expect("No `$HOEM/.config/markdown-publish-client/setting.toml` file");
        config
            .get_string("apikey")
            .expect("No name api_key in `$HOEM/.config/markdown-publish-client/setting.toml`")
            .to_owned()
    };
    static ref IPADDRESS: String = {
        let builder = config::Config::builder().add_source(config::File::new(
            format!(
                "{}{}",
                dirs::home_dir().expect("No home directory.").display(),
                "/.config/markdown-publish-client/setting.toml"
            )
            .as_str(),
            config::FileFormat::Toml,
        ));

        let config = builder
            .build()
            .expect("No `$HOEM/.config/markdown-publish-client/setting.toml` file");
        config
            .get_string("ipaddress")
            .expect("No name api_key in `$HOEM/.config/markdown-publish-client/setting.toml`")
            .to_owned()
    };
}

fn main() {
    let config = parse_args();
    let result: Vec<FileStatus> = match config.command {
        Command::Upload => command_upload(config.file_names),
        Command::Remove => command_remove(config.file_names),
        Command::List => command_list(),
        Command::Unknown => {
            command_help();
            process::exit(0);
        }
    };

    println!("# Result");
    println!(
        "{0: <10}  {1: <40}  {2: <10}",
        "idx", "file name", "file status"
    );
    for (idx, file) in result.iter().enumerate() {
        println!(
            "{0: <10}  {1: <40}  {2: <10}",
            idx + 1,
            file.file_name,
            file.status
        );
    }
}

fn parse_args() -> Config {
    let mut config = Config::new();

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        return Config {
            command: Command::Unknown,
            file_names: Vec::new(),
        };
    }
    let command = args[1].clone();

    config.command = match command.as_str() {
        "upload" | "u" => Command::Upload,
        "remove" | "r" => Command::Remove,
        "list" | "l" => Command::List,
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
        
    # Args
        - give list of files
    
    # Examples
        - markdown_publish upload ./file1.md ./file2.md
        - markdown_publish r ./file1
        - markdown_publish list"#
    );
}

fn command_upload(file_names: Vec<String>) -> Vec<FileStatus> {
    let mut file_status: Vec<FileStatus> = Vec::new();

    for path in file_names {
        let file = match File::open(path.clone()) {
            Err(_) => {
                file_status.push(FileStatus {
                    file_name: path,
                    status: Status::FileNotFound,
                });
                continue;
            }
            Ok(result) => result,
        };

        let file_name = match Path::new(&path).file_stem() {
            None => {
                file_status.push(FileStatus {
                    file_name: path,
                    status: Status::FileNotFound,
                });
                continue;
            }
            Some(result) => match result.to_str() {
                None => {
                    file_status.push(FileStatus {
                        file_name: path,
                        status: Status::FileNotFound,
                    });
                    continue;
                }
                Some(result) => result.to_string(),
            },
        };

        let client = Client::new();
        let response = match client
            .post(format!("{}/upload/{}", IPADDRESS.as_str(), file_name))
            .body(file)
            .header("x-api-key", APIKEY.as_str())
            .header(header::CONTENT_TYPE, "application/octet-stream")
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
            StatusCode::OK => {
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
    let mut file_status: Vec<FileStatus> = Vec::new();

    for file_name in file_names {
        let client = Client::new();
        let response = match client
            .get(format!("{}/delete/{}", IPADDRESS.as_str(), file_name))
            .header("x-api-key", APIKEY.as_str())
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
            StatusCode::OK => {
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

fn command_list() -> Vec<FileStatus> {
    let mut file_status: Vec<FileStatus> = Vec::new();

    let client = Client::new();
    let response = match client
        .get(format!("{}/upload_list", IPADDRESS.as_str()))
        .header("x-api-key", APIKEY.as_str())
        .send()
    {
        Err(_) => {
            file_status.push(FileStatus {
                file_name: "NetworkError".to_string(),
                status: Status::NetworkError,
            });
            return file_status;
        }
        Ok(result) => result,
    };

    if response.status() != StatusCode::OK {
        file_status.push(FileStatus {
            file_name: "NetworkError".to_string(),
            status: Status::NetworkError,
        });
        return file_status;
    };

    let response_text = match response.text() {
        Err(_) => {
            file_status.push(FileStatus {
                file_name: "NetworkResponseError".to_string(),
                status: Status::NetworkError,
            });
            return file_status;
        }
        Ok(result) => result,
    };

    let file_list: HashMap<usize, String> = match serde_json::from_str(&response_text) {
        Err(_) => {
            file_status.push(FileStatus {
                file_name: "NetworkParseError".to_string(),
                status: Status::NetworkError,
            });
            return file_status;
        }
        Ok(result) => result,
    };

    for file in file_list {
        file_status.push(FileStatus {
            file_name: file.1,
            status: Status::Success,
        });
    }

    file_status
}
