use actix_web::{HttpRequest, HttpResponse};
use log::info;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::sync::Arc;

// todo: add more flags as needed (none yet)
pub enum Flags {}

/// Holds a file's data in conjunction with it's HTTP file type.
pub struct FileData {
    pub content_type: String,
    pub data: Vec<u8>,
}

pub async fn index(req: HttpRequest) -> HttpResponse {
    let path = req.path().to_string();
    let data = req.app_data::<Arc<HashMap<String, FileData>>>();

    if let Some(data) = data {
        if let Some(response) = data.get(&path) {
            let content_type = response.content_type.clone();
            let file = response.data.clone();

            return HttpResponse::Ok().content_type(content_type).body(file);
        } else if let Some(_) = data.get(&(path.clone() + "/")) {
            return HttpResponse::Found()
                .append_header(("Location", path.clone() + "/"))
                .finish();
        }
    }

    HttpResponse::NotFound().body(format!("Couldn't find '{}'", path))
}

/// Returns a map of the files (recursive) in this directory with the path
/// as a key and its contents + HTTP file type as the value.
///
/// Directories which have an `index.html` inside them will also add it's
/// directory path as a key with it's value holding a copy of `index.html`'s data.
pub fn load_website(dir: &str) -> Result<HashMap<String, FileData>, Error> {
    info!("Loading from: {}", dir);

    let mut path_map = HashMap::new();
    let files = list_files(dir)?;
    let index_re = Regex::new(r"(.*)index\.html$").unwrap();

    for path in files {
        // make sure paths use '/' (thanks windows)
        let mut path_formatted = path.replacen(dir, "", 1).replace("\\", "/");
        if !path_formatted.starts_with("/") {
            path_formatted = "/".to_string() + &path_formatted;
        }

        info!("Loading: {}", &path_formatted);

        let content_type = path_to_content_type(&path_formatted).to_string();
        let file_contents = fs::read(&path)?;

        path_map.insert(
            path_formatted.clone(),
            FileData {
                content_type: content_type.clone(),
                data: file_contents.clone(),
            },
        );

        // check if this is an 'index.html' file, if so add directory to list of paths
        if let Some(captures) = index_re.captures(&path_formatted) {
            if let Some(path_match) = captures.get(1) {
                let dir_path = path_match.as_str().to_string();
                info!("Loading: {}", &dir_path);

                path_map.insert(
                    dir_path.clone(),
                    FileData {
                        content_type: content_type.clone(),
                        data: file_contents.clone(),
                    },
                );
            }
        }
    }

    Ok(path_map)
}

/// Returns a list of file paths found under this directory with a recursive search.
///
/// This function will NOT follow links.
fn list_files(dir: &str) -> Result<Vec<String>, Error> {
    let mut file_names = Vec::new();

    for entry in fs::read_dir(dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();

            if let (Ok(file_type), Some(path)) = (entry.file_type(), path.to_str()) {
                if file_type.is_file() {
                    file_names.push(path.into());
                } else if file_type.is_dir() {
                    if let Ok(mut files) = list_files(&path) {
                        file_names.append(&mut files);
                    }
                }
            }
        }
    }

    Ok(file_names)
}

/// Matches a path's extension to a matching HTTP content type.
///
/// Defaults to `application/octet-stream`.
fn path_to_content_type(path: &str) -> &str {
    let file_type_re = Regex::new(r".*\.([^.]*)$").unwrap();

    if let Some(captures) = file_type_re.captures(path) {
        if let Some(ext_match) = captures.get(1) {
            let ext_match = ext_match.as_str();
            match ext_match {
                "html" => return "text/html",
                "css" => return "text/css",
                "xml" => return "text/xml",
                "txt" => return "text/plain",
                "csv" => return "text/csv",
                "js" => return "application/javascript",
                "json" => return "application/json",
                "pdf" => return "application/pdf",
                "zip" => return "application/zip",
                "gif" => return "image/gif",
                "jpeg" => return "image/jpeg",
                "jpg" => return "image/jpeg",
                "png" => return "image/png",
                "ico" => return "image/vnd.microsoft.icon",
                _ => return "application/octet-stream",
            };
        }
    }

    // if it doesn't have an extension
    "application/octet-stream"
}
