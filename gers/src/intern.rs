//! Interned String.
use std::{
    borrow::{Borrow, Cow},
    cell::RefCell,
    collections::HashSet,
    fmt,
    ops::Deref,
    rc::Rc,
};

// ----------------------------------------------------------------------------
/// An interned string.
///
/// ```
/// # use gers::InternStr;
/// // Create from &str or String
/// let a = InternStr::from("a");
/// let b = InternStr::from(String::from("b"));
///
/// // Two interned strings with the same contents are considered equal
/// // and point to the same memory.
/// let a2 = InternStr::from("a");
/// assert_eq!(a, a2);
/// assert!(a.ptr_eq(&a2));
/// ```
///
/// An interned string is a string value that is stored only once in memory in a
/// global string intern table. It reduces memory usage when the same string is
/// used multiple times, and improves performance for string comparisons.
///
/// In order for a string to be interned, it must be immutable. It is identified
/// in the intern table by its hash.
///
/// The `InternStr` struct is implemented as a thin wrapper around `Rc<str>`,
/// which allows it to behave like a normal `&str` in most cases. It can be
/// created from a `String` or a `&str` using the `from()` functions.
///
/// Interning strings comes with memory overhead, as the interned strings must be
/// stored in a global table to be shared accross the program. Therefore, it is
/// recommended to use `InternStr` sparingly and only when it is expected to
/// provide a significant benefit over regular strings.
///
/// ## Threading
///
/// The intern table is global, but thread local. This avoids locking to improve
/// performance in multithreaded environments, at the cost of more memory usage.
/// To move an `InternStr` between threads, copy it to a new `String`.
///
/// ```
/// # use gers::InternStr;
/// use std::thread;
///
/// let foobar = InternStr::from("foobar");
/// let message = foobar.to_string();
///
/// let t = thread::spawn(|| {
///     let foobar = InternStr::from(message);
///     println!("{foobar}");
/// });
/// # t.join().unwrap();
/// ```
///
/// ## Nondistinct Instances
///
/// There are cases where inserting a string into the intern table may fail. If
/// the table experiences an internal failure, a new `InternStr` instance will
/// be created regardless. It will function normally, but will not be stored only
/// once in memory.
///
/// ## Garbage Collection
///
/// The intern table holds an `Rc<str>` instance for every interned string.
/// Therefore, it is necessary to manually execute a garbage collection pass to
/// free up memory.
///
/// The [`InternStr::gc()`] function will scan the intern table for any `Rc<str>`
/// instances that are not being used by any other part of the program and remove
/// them from the table. This means that if you have an interned string that is
/// still being used by another part of your program, it will not be removed from
/// the table.
///
/// The normal `Rc` behaviour applies. The underlying `str` is deallocated once
/// the strong reference count reaches 0.
#[derive(Debug, Clone, Hash)]
pub struct InternStr(Rc<str>);

impl InternStr {
    /// Returns `true` if the two `InternStr`s point to the same allocation.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Collect garbage.
    ///
    /// Scans the global string table and drops strings that no longer have references.
    pub fn gc() {
        collect_garbage()
    }
}

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

// FIXME: Interning table of weak references.
//
// The current solution requires periodic garbage collection calls which are
// manually triggered. A better alternative would be keeping weak references
// in the global table. That way the underlying string storage is deallocated
// when the `Rc`s string referenec count reaches 0, and the table just needs
// to be cleared of `Weak<T>`.
//
// The blockers to implementing this are:
//
// - `Weak<str>` doesn't implement `PartialEq<str>`, `Eq`, or `Hash`.
//    It cannot be stored in `HashMap` or `HashSet`, and looked up using `&str`.
// - `Weak<T>` effectively has inner mutability. If the strong reference count
//   reaches 0 the underlying value is deallocated, and upgrading a `Weak` to
//   `Rc` results in `None`. Modifying a key value like this is considered
//   undefined behaviour.

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

fn collect_garbage() {
    INTERNED_STRINGS
        .try_with(|ref_cell| {
            if let Ok(mut table) = ref_cell.try_borrow_mut() {
                // The table keeps strong references, so if only one Rc is left then
                // no further references are live.
                table.retain(|intern_str| Rc::strong_count(&intern_str.0) > 1)
            }
        })
        .unwrap_or_default()
}

/// Score of the amount of unused garbage in the table.
#[allow(dead_code)]
fn garbage_score() -> f32 {
    INTERNED_STRINGS
        .try_with(|ref_cell| match ref_cell.try_borrow() {
            Ok(table) => {
                if table.is_empty() {
                    0.0
                } else {
                    let count = table
                        .iter()
                        .filter(|intern_str| Rc::strong_count(&intern_str.0) == 1)
                        .count();

                    count as f32 / table.len() as f32
                }
            }
            Err(_) => 0.0,
        })
        .unwrap_or(0.0)
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

    #[test]
    fn test_garbage_collection() {
        let _ = InternStr::from("a");
        let _ = InternStr::from("b");
        let _ = InternStr::from("c");
        let _ = InternStr::from("d");

        assert_eq!(garbage_score(), 1.0);

        collect_garbage();
        assert_eq!(garbage_score(), 0.0);
    }
}
