use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    pub order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkFolder {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub order: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookmarkRequest {
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub folder_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

impl Database {
    pub fn list_bookmarks(&self, folder_id: Option<&str>) -> AppResult<Vec<Bookmark>> {
        let mut stmt = match folder_id {
            Some(_) => self.conn().prepare(
                "SELECT id, title, url, favicon, folder_id, sort_order, created_at
                 FROM bookmarks WHERE folder_id = ?1 ORDER BY sort_order ASC",
            )?,
            None => self.conn().prepare(
                "SELECT id, title, url, favicon, folder_id, sort_order, created_at
                 FROM bookmarks WHERE folder_id IS NULL ORDER BY sort_order ASC",
            )?,
        };

        let rows = match folder_id {
            Some(fid) => stmt.query_map(params![fid], Self::map_bookmark)?,
            None => stmt.query_map([], Self::map_bookmark)?,
        };

        let mut bookmarks = Vec::new();
        for row in rows {
            bookmarks.push(row?);
        }
        Ok(bookmarks)
    }

    fn map_bookmark(row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
        Ok(Bookmark {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            favicon: row.get(3)?,
            folder_id: row.get(4)?,
            order: row.get(5)?,
            created_at: row.get(6)?,
        })
    }

    pub fn create_bookmark(&self, req: &CreateBookmarkRequest) -> AppResult<Bookmark> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let max_order: i64 = self.conn().query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM bookmarks WHERE folder_id IS ?1",
            params![req.folder_id],
            |row| row.get(0),
        )?;

        self.conn().execute(
            "INSERT INTO bookmarks (id, title, url, favicon, folder_id, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, req.title, req.url, req.favicon, req.folder_id, max_order + 1, now],
        )?;

        Ok(Bookmark {
            id,
            title: req.title.clone(),
            url: req.url.clone(),
            favicon: req.favicon.clone(),
            folder_id: req.folder_id.clone(),
            order: max_order + 1,
            created_at: now,
        })
    }

    pub fn delete_bookmark(&self, id: &str) -> AppResult<()> {
        let affected = self.conn().execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Bookmark {} not found", id)));
        }
        Ok(())
    }

    pub fn list_bookmark_folders(&self, parent_id: Option<&str>) -> AppResult<Vec<BookmarkFolder>> {
        let mut stmt = match parent_id {
            Some(_) => self.conn().prepare(
                "SELECT id, name, parent_id, sort_order FROM bookmark_folders
                 WHERE parent_id = ?1 ORDER BY sort_order ASC",
            )?,
            None => self.conn().prepare(
                "SELECT id, name, parent_id, sort_order FROM bookmark_folders
                 WHERE parent_id IS NULL ORDER BY sort_order ASC",
            )?,
        };

        let rows = match parent_id {
            Some(pid) => stmt.query_map(params![pid], Self::map_folder)?,
            None => stmt.query_map([], Self::map_folder)?,
        };

        let mut folders = Vec::new();
        for row in rows {
            folders.push(row?);
        }
        Ok(folders)
    }

    fn map_folder(row: &rusqlite::Row) -> rusqlite::Result<BookmarkFolder> {
        Ok(BookmarkFolder {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            order: row.get(3)?,
        })
    }

    pub fn create_bookmark_folder(&self, req: &CreateFolderRequest) -> AppResult<BookmarkFolder> {
        let id = uuid::Uuid::new_v4().to_string();

        let max_order: i64 = self.conn().query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM bookmark_folders WHERE parent_id IS ?1",
            params![req.parent_id],
            |row| row.get(0),
        )?;

        self.conn().execute(
            "INSERT INTO bookmark_folders (id, name, parent_id, sort_order)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, req.name, req.parent_id, max_order + 1],
        )?;

        Ok(BookmarkFolder {
            id,
            name: req.name.clone(),
            parent_id: req.parent_id.clone(),
            order: max_order + 1,
        })
    }

    pub fn delete_bookmark_folder(&self, id: &str) -> AppResult<()> {
        self.conn().execute("DELETE FROM bookmarks WHERE folder_id = ?1", params![id])?;
        self.conn().execute("DELETE FROM bookmark_folders WHERE parent_id = ?1", params![id])?;
        let affected = self.conn().execute("DELETE FROM bookmark_folders WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("Bookmark folder {} not found", id)));
        }
        Ok(())
    }
}
