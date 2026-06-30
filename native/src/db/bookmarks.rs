//! 书签（Bookmarks）模块

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::AppResult;

/// 书签结构
#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub folder_id: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

/// 书签文件夹结构
#[derive(Debug, Clone)]
pub struct BookmarkFolder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
}

/// 创建书签表
pub fn create_bookmarks_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS bookmarks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            favicon TEXT,
            folder_id TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS bookmark_folders (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_bookmarks_folder ON bookmarks(folder_id);
        CREATE INDEX IF NOT EXISTS idx_bookmarks_order ON bookmarks(sort_order);
        CREATE INDEX IF NOT EXISTS idx_bookmark_folders_parent ON bookmark_folders(parent_id);
        "#
    )?;
    
    Ok(())
}

/// 列出书签
pub fn list_bookmarks(conn: &Connection, folder_id: Option<&str>) -> AppResult<Vec<Bookmark>> {
    let sql = match folder_id {
        Some(_) => "SELECT id, title, url, favicon, folder_id, sort_order, created_at
                    FROM bookmarks WHERE folder_id = ?1 ORDER BY sort_order ASC",
        None => "SELECT id, title, url, favicon, folder_id, sort_order, created_at
                 FROM bookmarks WHERE folder_id IS NULL ORDER BY sort_order ASC",
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = match folder_id {
        Some(f) => stmt.query_map(params![f], map_bookmark_row)?,
        None => stmt.query_map([], map_bookmark_row)?,
    };

    let mut bookmarks = Vec::new();
    for bookmark in rows {
        bookmarks.push(bookmark?);
    }

    Ok(bookmarks)
}

fn map_bookmark_row(row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
    Ok(Bookmark {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        favicon: row.get(3)?,
        folder_id: row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
    })
}

/// 创建书签
pub fn create_bookmark(
    conn: &Connection,
    title: String,
    url: String,
    favicon: Option<String>,
    folder_id: Option<String>,
) -> AppResult<Bookmark> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let max_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM bookmarks WHERE folder_id IS ?1",
        params![folder_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO bookmarks (id, title, url, favicon, folder_id, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, title, url, favicon, folder_id.clone(), max_order + 1, now],
    )?;

    Ok(Bookmark {
        id,
        title,
        url,
        favicon,
        folder_id,
        sort_order: max_order + 1,
        created_at: now,
    })
}

/// 删除书签
pub fn delete_bookmark(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
    Ok(())
}

/// 列出书签文件夹
pub fn list_bookmark_folders(conn: &Connection, parent_id: Option<&str>) -> AppResult<Vec<BookmarkFolder>> {
    let sql = match parent_id {
        Some(_) => "SELECT id, name, parent_id, sort_order FROM bookmark_folders WHERE parent_id = ?1 ORDER BY sort_order ASC",
        None => "SELECT id, name, parent_id, sort_order FROM bookmark_folders WHERE parent_id IS NULL ORDER BY sort_order ASC",
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = match parent_id {
        Some(p) => stmt.query_map(params![p], map_folder_row)?,
        None => stmt.query_map([], map_folder_row)?,
    };

    let mut folders = Vec::new();
    for folder in rows {
        folders.push(folder?);
    }

    Ok(folders)
}

fn map_folder_row(row: &rusqlite::Row) -> rusqlite::Result<BookmarkFolder> {
    Ok(BookmarkFolder {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        sort_order: row.get(3)?,
    })
}

/// 创建书签文件夹
pub fn create_bookmark_folder(
    conn: &Connection,
    name: String,
    parent_id: Option<String>,
) -> AppResult<BookmarkFolder> {
    let id = Uuid::new_v4().to_string();
    
    let max_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM bookmark_folders WHERE parent_id IS ?1",
        params![parent_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO bookmark_folders (id, name, parent_id, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, parent_id.clone(), max_order + 1],
    )?;

    Ok(BookmarkFolder {
        id,
        name,
        parent_id,
        sort_order: max_order + 1,
    })
}

/// 删除书签文件夹（级联删除子文件夹和书签）
pub fn delete_bookmark_folder(conn: &Connection, id: &str) -> AppResult<()> {
    // 先删除该文件夹下的所有书签
    conn.execute("DELETE FROM bookmarks WHERE folder_id = ?1", params![id])?;
    // 再删除子文件夹
    conn.execute("DELETE FROM bookmark_folders WHERE parent_id = ?1", params![id])?;
    // 最后删除文件夹本身
    conn.execute("DELETE FROM bookmark_folders WHERE id = ?1", params![id])?;
    Ok(())
}
