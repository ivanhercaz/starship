use ansi_term::Color;
use std::path::Path;
use std::process::Command;
use std::str;

use std::ffi::OsStr;
use std::ops::Deref;

use super::{Context, Module};

/// A module which shows the latest (or pinned) version of the dotnet SDK
///
/// Will display if any of the following file extensions are present in
/// the current directory: .sln, .csproj, .fsproj, .xproj
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    const DOTNET_SYMBOL: &str = "•NET ";

    let mut module = context.new_module("dotnet");

    let dotnet_files = get_local_dotnet_files(context).ok()?;

    if dotnet_files.len() == 0 {
        return None;
    }

    let get_file_of_type = |t: FileType| dotnet_files.iter().find(|f| f.file_type == t);
    let relevant_file = get_file_of_type(FileType::ProjectJson)
        .or_else(|| get_file_of_type(FileType::GlobalJson))
        .or_else(|| get_file_of_type(FileType::ProjectFile))
        .or_else(|| get_file_of_type(FileType::SolutionFile))?;

    match relevant_file.file_type {
        FileType::ProjectJson => {}
        FileType::GlobalJson => {}
        FileType::ProjectFile => {}
        FileType::SolutionFile => {}
    }

    let version = get_dotnet_version()?;
    module.set_style(Color::Blue.bold());
    module.new_segment("symbol", DOTNET_SYMBOL);
    module.new_segment("version", &version);

    Some(module)
}

struct Version(String);

impl Deref for Version {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn get_pinned_sdk_version(path: &Path) -> Option<Version> {
    let json_text = crate::utils::read_file(path).ok()?;
    let parsed_json: serde_json::Value = serde_json::from_str(&json_text).ok()?;

    match parsed_json {
        serde_json::Value::Object(root) => {
            let sdk = root.get("sdk")?;
            match sdk {
                serde_json::Value::Object(sdk) => {
                    let version = sdk.get("version")?;
                    match version {
                        serde_json::Value::String(version_string) => {
                            Some(Version(version_string.clone()))
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

struct DotNetFile<'a> {
    path: &'a Path,
    file_type: FileType,
}

#[derive(PartialEq)]
enum FileType {
    ProjectJson,
    ProjectFile,
    GlobalJson,
    SolutionFile,
}

fn get_local_dotnet_files<'a>(context: &'a Context) -> Result<Vec<DotNetFile<'a>>, std::io::Error> {
    Ok(context
        .get_dir_files()?
        .iter()
        .filter_map(|p| {
            get_dotnet_file_type(p).map(|t| DotNetFile {
                path: p.as_ref(),
                file_type: t,
            })
        })
        .collect())
}

fn get_dotnet_file_type(path: &Path) -> Option<FileType> {
    let file_name_lower = map_to_lower(path.file_name());

    match file_name_lower.as_ref().map(|f| f.as_ref()) {
        Some("global.json") => return Some(FileType::GlobalJson),
        Some("project.json") => return Some(FileType::ProjectJson),
        _ => (),
    };

    let extension_lower = map_to_lower(path.extension());

    match extension_lower.as_ref().map(|f| f.as_ref()) {
        Some("sln") => return Some(FileType::SolutionFile),
        Some("csproj") | Some("fsproj") | Some("xproj") => return Some(FileType::ProjectFile),
        _ => (),
    };

    None
}

fn map_to_lower(value: Option<&OsStr>) -> Option<String> {
    Some(value?.to_str()?.to_ascii_lowercase())
}

fn get_dotnet_version() -> Option<String> {
    let version_output = Command::new("dotnet").arg("--version").output().ok()?;
    let version = str::from_utf8(version_output.stdout.as_slice())
        .ok()?
        .trim();

    let mut buffer = String::with_capacity(version.len() + 1);
    buffer.push('v');
    buffer.push_str(version);

    Some(buffer)
}
