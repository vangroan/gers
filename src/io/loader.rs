//! Wren module loader.
use rust_wren::prelude::*;
use smol_str::SmolStr;
use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    iter,
};

pub struct WrenModuleLoader {
    /// Loader internal state is infallible
    /// because one root must be defined.
    root: PathBuf,
    rest: Vec<PathBuf>,
}

impl WrenModuleLoader {
    pub fn from_root<P>(root: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            root: root.as_ref().to_path_buf(),
            rest: vec![],
        }
    }

    pub fn add_root<P>(&mut self, root: P)
    where
        P: AsRef<Path>,
    {
        self.rest.push(root.as_ref().to_path_buf());
    }

    pub fn with_root<P>(mut self, root: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.add_root(root);
        self
    }

    /// An iterator of root paths.
    pub fn iter_roots(&self) -> impl iter::Iterator<Item=&Path> {
        iter::once(self.root.as_path()).chain(self.rest.iter().map(|path_buf| path_buf.as_path()))
    }
}

impl ModuleLoader for WrenModuleLoader {
    fn load(&mut self, name: &str) -> Option<String> {
        let mod_path = match ModulePath::new(name) {
            Ok(mod_path) => mod_path,
            Err(err) => {
                log::error!("Module load error: {}", err);
                return None;
            }
        };
        log::debug!("Loading module: {}", mod_path);

        let file_path = mod_path.as_path();

        for dir_path in self.iter_roots() {
            let path = dir_path.join(file_path);
            log::debug!("Attempting '{}'", path.to_string_lossy());

            if path.is_file() {
                return match fs::read_to_string(path) {
                    Ok(source) => { Some(source) }
                    Err(err) => {
                        log::error!("Module load error: {}", err);
                        None
                    }
                };
            } else {
                log::trace!("Module path is not a file: {}", path.to_string_lossy());
            }
        }
        
        log::warn!("Module not found: {}", mod_path);
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModulePath {
    mod_path: SmolStr,
    file_path: PathBuf,
}

impl ModulePath {
    const DELIMITER: char = '.';
    const WREN_EXT: &'static str = ".wren";

    pub fn new<S>(module_name: S) -> Result<Self, ModuleNameError>
    where
        S: AsRef<str>,
    {
        use ModuleNameParseKind as E;

        // Convert to file path.
        let mut ident_buf = String::new();
        let mut path_buf = PathBuf::new();

        for (pos, c) in module_name.as_ref().char_indices() {
            match c {
                Self::DELIMITER => {
                    if ident_buf.is_empty() {
                        return Err(ModuleNameError {
                            kind: E::ModuleNameMissing,
                            pos,
                            snippet: Self::make_snippet(module_name.as_ref(), pos),
                        });
                    } else {
                        path_buf.push(&ident_buf);
                        ident_buf.clear();
                    }
                }
                ' ' | '\t' | '\r' | '\n' => {
                    return Err(ModuleNameError {
                        kind: E::Whitespace,
                        pos,
                        snippet: Self::make_snippet(module_name.as_ref(), pos),
                    });
                }
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                    ident_buf.push(c);
                }
                _ => {
                    return Err(ModuleNameError {
                        kind: E::InvalidCharacter,
                        pos,
                        snippet: Self::make_snippet(module_name.as_ref(), pos),
                    });
                }
            }
        }

        // Last path part, which is also considered the filename.
        if ident_buf.is_empty() {
            let pos = module_name.as_ref().len() - 1;
            return Err(ModuleNameError {
                kind: E::UnexpectedEOS,
                pos,
                snippet: Self::make_snippet(module_name.as_ref(), pos),
            });
        } else {
            ident_buf.push_str(Self::WREN_EXT);
            path_buf.push(&ident_buf);
        }

        Ok(Self {
            mod_path: SmolStr::from(module_name.as_ref()),
            file_path: path_buf,
        })
    }

    fn make_snippet(source: &str, pos: usize) -> String {
        if pos == 0 {
            // Start of string.
            format!("-->{}", source)
        } else if pos == source.len() - 1 {
            // End of string.
            format!("{}<--", source)
        } else {
            format!("{}-->{}", &source[..pos], &source[pos..],)
        }
    }

    /// Return the parsed file system path.
    #[inline]
    pub fn as_path(&self) -> &Path {
        self.file_path.as_path()
    }
}

impl AsRef<Path> for ModulePath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.mod_path, f)
    }
}

#[derive(Debug, PartialEq)]
pub struct ModuleNameError {
    pub kind: ModuleNameParseKind,
    /// Character position in the string where the parse error occurred.
    pub pos: usize,
    /// Current or last character where error occurred.
    pub snippet: String,
}

impl fmt::Display for ModuleNameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ModuleNameParseKind as E;

        write!(f, "parse error: ")?;

        match self.kind {
            E::UnexpectedEOS => write!(f, "unexpected end of module path at '{}'", self.snippet),
            E::ModuleNameMissing => write!(f, "module name expected at character {} '{}'", self.pos+1, self.snippet),
            E::Whitespace => write!(f, "whitespace not allowed in module path '{}'", self.snippet),
            E::InvalidCharacter => write!(f, "invalid character at '{}'", self.snippet),
        }
    }
}

impl Error for ModuleNameError {}

#[derive(Debug, PartialEq, Eq)]
pub enum ModuleNameParseKind {
    /// Unexpected end of string.
    UnexpectedEOS,
    /// Module name expected.
    ModuleNameMissing,
    /// Whitespace not allowed.
    Whitespace,
    /// Unexpected character.
    InvalidCharacter,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_module_name_parse() {
        let path = ModulePath::new("gers.graphics").unwrap();
        assert_eq!(path.as_path(), PathBuf::from("gers/graphics.wren").as_path());
    }

    #[test]
    fn test_module_name_parse_err() {
        use ModuleNameParseKind as E;
        assert_eq!(
            ModulePath::new(".gers.graphics"),
            Err(ModuleNameError {
                kind: E::ModuleNameMissing,
                pos: 0,
                snippet: "-->.gers.graphics".to_string(),
            })
        );
        assert_eq!(
            ModulePath::new("gers.graphics."),
            Err(ModuleNameError {
                kind: E::UnexpectedEOS,
                pos: 13,
                snippet: "gers.graphics.<--".to_string(),
            })
        );
        assert_eq!(
            ModulePath::new("gers..graphics"),
            Err(ModuleNameError {
                kind: E::ModuleNameMissing,
                pos: 5,
                snippet: "gers.-->.graphics".to_string(),
            })
        );
    }
}
