//! Interned String.
use std::{
    borrow::{Borrow, Cow},
    cell::RefCell,
    collections::HashSet,
    fmt,
    ops::Deref,
    rc::Rc,
};

/// An immutable interned string.
#[derive(Debug, Clone, Hash)]
pub struct InternStr(Rc<str>);

/// Internal functions for constructing an [`InterStr`]
/// without inserting it into the
impl InternStr {
    fn from_cow(cow: Cow<'_, str>) -> Self {
        // NOTE: Rc will copy a borrowed &str.
        let rc: Rc<str> = match cow {
            Cow::Borrowed(borrowed) => Rc::from(borrowed),
            Cow::Owned(owned) => Rc::from(owned),
        };
        InternStr(rc)
    }

    #[allow(dead_code)]
    fn from_string(owned: String) -> Self {
        let rc: Rc<str> = Rc::from(owned);
        InternStr(rc)
    }
}

impl Deref for InternStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for InternStr {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for InternStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<String> for InternStr {
    fn from(owned: String) -> InternStr {
        get_or_insert_cow(owned)
    }
}

impl<'a> From<&'a str> for InternStr {
    fn from(borrowed: &'a str) -> Self {
        get_or_insert_cow(borrowed)
    }
}

impl PartialEq<InternStr> for InternStr {
    fn eq(&self, other: &InternStr) -> bool {
        // If the two RCs have the same pointer, then the string contents will be equal.
        Rc::ptr_eq(&self.0, &other.0)
        // Multiple instances of InterStr can exist outside of the global table.
        // Short circuiting spares us from hopping the pointer and doing a string comparison. 
            || self.0.eq(&other.0)
    }
}

impl PartialEq<str> for InternStr {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref().eq(other)
    }
}

impl Eq for InternStr {}

impl Borrow<str> for InternStr {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

// ----------------------------------------------------------------------------
/// Intern string table type.
///
/// A `HashSet` allows for quick lookup of existing strings, when the table
/// grows really large.
type InternStrTable = HashSet<InternStr>;

thread_local! {
    /// Global table of interned strings.
    static INTERNED_STRINGS: RefCell<InternStrTable> = Default::default();
}

/// Get an existing string out of the global table, or insert it.
///
/// A `Cow` is used so that &str can be used as a key to lookup an
/// existing string, without having to copy it.
///
/// If an `InternStr` cannot be found, the given string will be copied.
fn get_or_insert_cow<'a>(string: impl Into<Cow<'a, str>>) -> InternStr {
    let s = string.into();

    INTERNED_STRINGS.with(|ref_cell| match ref_cell.try_borrow_mut() {
        Ok(mut table) => match table.get(s.as_ref()) {
            Some(intern_str) => intern_str.clone(),
            None => {
                let intern_str = InternStr::from_cow(s);
                table.insert(intern_str.clone());
                intern_str
            }
        },
        Err(_) => {
            log::error!("interned string table already borrowed");
            InternStr::from_cow(s)
        }
    })
}

// ----------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_convert() {
        let a = InternStr::from("foobar");
        let b = InternStr::from("foobar");

        assert_eq!(a, b);
        // the same Rc must be returned for the same given string
        assert_eq!(Rc::as_ptr(&a.0), Rc::as_ptr(&b.0));
    }

    #[test]
    fn test_eq() {
        let a = InternStr::from_string("foobar".to_string());
        let b = InternStr::from_string("foobar".to_string());

        // equality test must pass even though the Rcs are distinct
        assert_eq!(a, b);
        assert_ne!(Rc::as_ptr(&a.0), Rc::as_ptr(&b.0));
    }

    /// Ensure that `InternStr` can be compared to `&str`.
    #[test]
    fn test_collections() {
        let mut map: HashMap<Box<str>, InternStr> = HashMap::new();
        map.insert("foobar".into(), InternStr::from_string("foobar".into()));
        map.get("foobar").expect("get by &'static str");
        map.get("foobar".to_string().as_str()).expect("get by as_str()");

        let mut set: HashSet<InternStr> = HashSet::new();
        set.insert(InternStr::from_string("foobar".into()));
        set.get("foobar".to_string().as_str()).expect("get by as_str()");
    }

    #[test]
    fn test_iter() {
        let foobar_intern = InternStr::from("foobar");
        let foobar = "foobar";
        for (a, b) in foobar_intern.chars().zip(foobar.chars()) {
            assert_eq!(a, b);
        }
    }
}
