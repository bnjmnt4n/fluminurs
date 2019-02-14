use crate::api::{Api, ApiData, Data};
use crate::Result;
use serde::Deserialize;
use reqwest::Method;

#[derive(Debug, Deserialize)]
struct Access {
    #[serde(rename = "access_Full")]
    full: bool,
    #[serde(rename = "access_Read")]
    read: bool,
    #[serde(rename = "access_Create")]
    create: bool,
    #[serde(rename = "access_Update")]
    update: bool,
    #[serde(rename = "access_Delete")]
    delete: bool,
    #[serde(rename = "access_Settings_Read")]
    settings_read: bool,
    #[serde(rename = "access_Settings_Update")]
    settings_update: bool,
}

#[derive(Debug, Deserialize)]
pub struct Announcement {
    pub title: String,
    pub description: String
}

#[derive(Debug, Deserialize)]
pub struct Module {
    pub id: String,
    #[serde(rename = "name")]
    pub code: String,
    #[serde(rename = "courseName")]
    pub name: String,
    access: Access,
    pub term: String,
}

impl Module {
    pub fn is_teaching(&self) -> bool {
        let access = &self.access;
        access.full || access.create || access.update || access.delete || access.settings_read || access.settings_update
    }

    pub fn get_announcements(&self, api: &Api, archived: bool) -> Result<Vec<Announcement>> {
        let path = format!("/announcement/{}/{}?sortby=displayFrom%20ASC", if archived { "Archived" } else { "NonArchived" }, self.id);
        let api_data: ApiData = api.api_as_json(&path, Method::GET, None)?;
        if let Data::Announcements(announcements) = api_data.data {
            Ok(announcements)
        } else if let Data::Empty(_) = api_data.data {
            Ok(Vec::new())
        } else {
            Err("Invalid API response from server: type mismatch")
        }
    }

    pub fn as_file(&self, api: &Api, preload_children: bool) -> Result<File> {
        let mut file = File { id: self.id.to_owned(), name: self.code.to_owned(), is_directory: true, children: None };
        if preload_children {
            file.load_children(api)?;
        }
        Ok(file)
    }
}

#[derive(Clone)]
pub struct File {
    id: String,
    pub name: String,
    pub is_directory: bool,
    pub children: Option<Vec<File>>,
}

fn sanitise_filename(name: String) -> String {
    ["\0", "/"].iter().fold(name, |acc, x| acc.replace(x, "-"))
}

impl File {
    pub fn load_children(&mut self, api: &Api) -> Result<bool> {
        if !self.is_directory {
            self.children = Some(Vec::new());
            return Ok(true);
        }
        if self.children.is_some() {
            return Ok(true);
        }
        let subdirs_data: ApiData = api.api_as_json(&format!("/files/?ParentID={}", self.id), Method::GET, None)?;
        let files_data: ApiData = api.api_as_json(&format!("/files/{}/file", self.id), Method::GET, None)?;
        let mut subdirs = match subdirs_data.data {
            Data::ApiFileDirectory(subdirs) => subdirs.into_iter().map(|s|File { id: s.id, name: sanitise_filename(s.name), is_directory: true, children: None }).collect(),
            _ => Vec::new(),
        };
        let mut files = match files_data.data {
            Data::ApiFileDirectory(files) => files.into_iter().map(|s|File { id: s.id, name: sanitise_filename(s.name), is_directory: false, children: Some(Vec::new()) }).collect(),
            _ => Vec::new(),
        };
        subdirs.append(&mut files);
        self.children = Some(subdirs);
        Ok(true)
    }
}
