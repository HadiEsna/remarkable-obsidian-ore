use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::process::Command;
use tide::{Request, Response, StatusCode};
use walkdir::WalkDir;

// let obsidian_folder = "/Users/working/Library/Mobile Documents/iCloud~md~obsidian/Documents/notes/plugin";

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct FileData {
    name: String,
    hash: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct SyncData {
    files: Vec<FileData>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/").get(|_| async { Ok("Hello, world123!") });
    app.at("/sync").get(sync);

    app.listen("localhost:8081").await?;
    Ok(())
}

async fn sync(_: Request<()>) -> tide::Result {
    println!("Running command");

    let command = "sh";
    // let command = "ls";
    let args: [&str; 1] = ["src/bash/getUpdates.sh"];
    let output = Command::new(command)
        .args(&args)
        .output()
        .expect("Failed to execute command");

    // Check if the command executed successfully
    if output.status.success() {
        // Convert the output to a string and print it
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command executed successfully:\n{}", stdout);
        check_downloaded_files().await;
    } else {
        // Handle the error
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command failed with error:\n{}", stderr);
    }
    Ok(Response::builder(StatusCode::Ok).body("Hello!").build())
}

async fn check_downloaded_files() {
    let data = fs::read_to_string("trackingFiles.json");
    let data = match data {
        Ok(data) => data,
        Err(_) => {
            panic!("Error reading trackingFiles.json");
        }
    };
    let data: SyncData = serde_json::from_str(&data).unwrap();
    println!("{:?}", data);

    let mut files = Vec::new();
    for entry in WalkDir::new("./Obsidian") {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().display().to_string().contains(".zip") {
            files.push(entry.path().display().to_string());
        }
    }
    println!("{:?}", files);
    let mut files_to_remove = Vec::new();
    let mut files_to_add = Vec::new();
    for file_path in &files {
        let file = File::open(file_path.clone()).unwrap();
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        let result = hasher.finalize();
        println!("{:x}", result);
        if !data.files.iter().any(|x| x.name == *file_path) {
            files_to_add.push(FileData {
                name: file_path.clone(),
                hash: format!("{:x}", result),
            });
        }
        if data.files.iter().any(|x| x.name == *file_path) {
            let file_data = data.files.iter().find(|x| x.name == *file_path).unwrap();
            if file_data.hash != format!("{:x}", result) {
                files_to_add.push(FileData {
                    name: file_path.clone(),
                    hash: format!("{:x}", result),
                });
            }
        }
    }
    println!("{:?}", files_to_add);
    for files_to_add in &files_to_add {
        let command = "sh";
        let file_path = "/Users/working/sources/Hadi/remarkable-obsidian/remarkable-obsidian-ore/"
            .to_string()
            + &files_to_add.name;

        let zip_add = file_path.replace("./", "");
        let dest_folder = zip_add
            .replace(".zip", "")
            .replace("/Obsidian/", "/Obsidian_processed/");
        let folder_add = format!("{}:/store", dest_folder);

        println!("zip_add: {}", zip_add);
        println!("dest_folder: {}", dest_folder);
        println!("folder_add: {}", folder_add);
        let args: [&str; 4] = [
            "src/bash/openZipAndConvert.sh",
            zip_add.as_str(),
            dest_folder.as_str(),
            folder_add.as_str(),
        ];
        let output = Command::new(command)
            .args(&args)
            .output()
            .expect("Failed to execute command");

        // Check if the command executed successfully
        if output.status.success() {
            // Convert the output to a string and print it
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Command executed successfully:\n{}", stdout);
            let files_in_folder = WalkDir::new(format!("{}/out", dest_folder))
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().display().to_string())
                .collect::<Vec<String>>();
            println!("files in dest {:?}", files_in_folder[0]);
            let mut file_name = files_in_folder[0].replace(
                "/Users/working/sources/Hadi/remarkable-obsidian/remarkable-obsidian-ore/Obsidian_processed/",
                "",
            );
            file_name = file_name.split("/").collect::<Vec<&str>>()[0].to_string();
            let command = "mv";
            let ar = format!("/Users/working/Library/Mobile Documents/iCloud~md~obsidian/Documents/notes/notes/8. remarkable notes/{}.pdf", file_name);
            let args: [&str; 2] = [files_in_folder[0].as_str(), ar.as_str()];
            let output = Command::new(command)
                .args(&args)
                .output()
                .expect("Failed to execute command");
            if output.status.success() {
                // Convert the output to a string and print it
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("Command executed successfully:\n{}", stdout);
            } else {
                // Handle the error
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("Command failed with error:\n{}", stderr);
            }
        } else {
            // Handle the error

            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);

            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Command failed with error:\n{}", stderr);
        }
    }
    for file_data in &data.files {
        if !files.iter().any(|x| x == &file_data.name) {
            files_to_remove.push(file_data.name.clone());
        }
    }
}
