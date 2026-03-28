use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

pub enum ExitAction {
    Quit,
    Cd(PathBuf),
    EditFile(PathBuf),
    Yazi(PathBuf),
}

pub struct App {
    pub mode: Mode,
    pub query: String,
    pub results: Vec<PathBuf>,
    pub selected: usize,
    /// Which path segment is highlighted. None = the leaf (file/dir name itself).
    pub path_segment: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            path_segment: None,
        }
    }

    pub fn run_search(&mut self) {
        if self.query.is_empty() {
            self.results.clear();
            self.selected = 0;
            self.path_segment = None;
            return;
        }

        let output = Command::new("locate")
            .arg("--limit=200")
            .arg("--regex")
            .arg(&self.query)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                self.results = stdout
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(PathBuf::from)
                    .collect();
                self.selected = 0;
                self.path_segment = None;
            }
            Err(_) => {
                self.results.clear();
            }
        }
    }

    pub fn move_down(&mut self) {
        if !self.results.is_empty() && self.selected < self.results.len() - 1 {
            self.selected += 1;
            self.path_segment = None;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.path_segment = None;
        }
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
        self.path_segment = None;
    }

    pub fn select_last(&mut self) {
        if !self.results.is_empty() {
            self.selected = self.results.len() - 1;
            self.path_segment = None;
        }
    }

    pub fn move_path_left(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let count = self.path_display_count();
        if count == 0 {
            return;
        }
        match self.path_segment {
            None => {
                if count > 1 {
                    self.path_segment = Some(count - 2);
                }
            }
            Some(seg) => {
                if seg > 0 {
                    self.path_segment = Some(seg - 1);
                }
            }
        }
    }

    pub fn move_path_right(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let count = self.path_display_count();
        if count == 0 {
            return;
        }
        match self.path_segment {
            None => {}
            Some(seg) => {
                if seg + 1 >= count - 1 {
                    self.path_segment = None;
                } else {
                    self.path_segment = Some(seg + 1);
                }
            }
        }
    }

    /// Number of display segments for the selected path.
    /// We split on "/" and skip the empty first element from absolute paths,
    /// giving us ["home", "user", "file.txt"] etc.
    fn path_display_count(&self) -> usize {
        let path = &self.results[self.selected];
        path_display_parts(path).len()
    }

    /// Returns the currently resolved path for preview/action.
    pub fn selected_path(&self) -> Option<PathBuf> {
        if self.results.is_empty() {
            return None;
        }
        let path = &self.results[self.selected];
        match self.path_segment {
            None => Some(path.clone()),
            Some(seg) => {
                let parts = path_display_parts(path);
                if seg < parts.len() {
                    // Reconstruct absolute path up to this segment
                    let mut resolved = PathBuf::from("/");
                    for p in &parts[..=seg] {
                        resolved.push(p);
                    }
                    Some(resolved)
                } else {
                    Some(path.clone())
                }
            }
        }
    }

    pub fn enter_selection(&self) -> Option<ExitAction> {
        let path = self.selected_path()?;
        if path.is_dir() {
            Some(ExitAction::Cd(path))
        } else if path.is_file() {
            if is_likely_text(&path) {
                Some(ExitAction::EditFile(path))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn yazi_selection(&self) -> Option<ExitAction> {
        let path = self.selected_path()?;
        // Open yazi at the file's parent dir if it's a file, or at the dir itself
        let target = if path.is_dir() {
            path
        } else {
            path.parent()?.to_path_buf()
        };
        Some(ExitAction::Yazi(target))
    }
}

/// Split an absolute path into display parts, skipping the root "/".
/// "/home/user/file.txt" -> ["home", "user", "file.txt"]
pub fn path_display_parts(path: &Path) -> Vec<String> {
    let s = path.to_string_lossy();
    s.split('/')
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

pub fn is_likely_text(path: &Path) -> bool {
    let mut buf = [0u8; 512];
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let Ok(n) = f.read(&mut buf) else {
        return false;
    };
    !buf[..n].contains(&0)
}
