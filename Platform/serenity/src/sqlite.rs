//! SQLite as the cache calls it, in the library the program ships: opened from its file, or lent by a host that has
//! it loaded, so that one process holds one SQLite and its locks on a file are one set. The waits and the words of a
//! failure are Microsoft.Data.Sqlite's, which the C# cache is written on. See EmuSen_RustPlatform.md §2.8 and §16.2.

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const BUSY: i32 = 5;
pub const LOCKED: i32 = 6;
pub const CORRUPT: i32 = 11;
pub const NOT_A_DATABASE: i32 = 26;
const OK: c_int = 0;
const ROW: c_int = 100;
const DONE: c_int = 101;
const OPEN_READWRITE: c_int = 0x2;
const OPEN_CREATE: c_int = 0x4;
const BLOB: c_int = 4;
const TRANSIENT: isize = -1;

macro_rules! functions {
    ($($field:ident = $name:literal: fn($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        #[allow(clippy::type_complexity)]
        struct Functions {
            $($field: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)*
        }

        /// The functions of SQLite's that are called, by their C names.
        pub const NAMES: &[&str] = &[$($name),*];

        impl Functions {
            /// # Safety
            /// `library` must be SQLite, so that each name has the signature declared for it.
            unsafe fn load(library: &libloading::Library) -> Result<Functions, String> {
                Ok(Functions {
                    $($field: *unsafe { library.get::<unsafe extern "C" fn($($arg),*) $(-> $ret)?>($name) }
                        .map_err(|_| format!("SQLite has no {}", $name))?,)*
                })
            }
        }
    };
}

functions! {
    open_v2 = "sqlite3_open_v2": fn(*const c_char, *mut *mut c_void, c_int, *const c_char) -> c_int;
    close_v2 = "sqlite3_close_v2": fn(*mut c_void) -> c_int;
    prepare_v2 = "sqlite3_prepare_v2": fn(*mut c_void, *const c_char, c_int, *mut *mut c_void, *mut *const c_char) -> c_int;
    step = "sqlite3_step": fn(*mut c_void) -> c_int;
    reset = "sqlite3_reset": fn(*mut c_void) -> c_int;
    finalize = "sqlite3_finalize": fn(*mut c_void) -> c_int;
    bind_parameter_index = "sqlite3_bind_parameter_index": fn(*mut c_void, *const c_char) -> c_int;
    bind_blob = "sqlite3_bind_blob": fn(*mut c_void, c_int, *const c_void, c_int, isize) -> c_int;
    bind_int64 = "sqlite3_bind_int64": fn(*mut c_void, c_int, i64) -> c_int;
    column_type = "sqlite3_column_type": fn(*mut c_void, c_int) -> c_int;
    column_blob = "sqlite3_column_blob": fn(*mut c_void, c_int) -> *const c_void;
    column_bytes = "sqlite3_column_bytes": fn(*mut c_void, c_int) -> c_int;
    column_int64 = "sqlite3_column_int64": fn(*mut c_void, c_int) -> i64;
    changes = "sqlite3_changes": fn(*mut c_void) -> c_int;
    errmsg = "sqlite3_errmsg": fn(*mut c_void) -> *const c_char;
    errstr = "sqlite3_errstr": fn(c_int) -> *const c_char;
}

/// The SQLite library and its functions.
pub struct Sqlite {
    f: Functions,
    _library: ManuallyDrop<libloading::Library>,
    owned: bool,
}

// SQLite is built to serialise its own connections, as the C# relies on.
unsafe impl Send for Sqlite {}
unsafe impl Sync for Sqlite {}

impl Sqlite {
    /// SQLite from its file.
    pub fn open(path: &str) -> Result<Sqlite, String> {
        let library = unsafe { libloading::Library::new(path) }.map_err(|e| format!("{path} could not be loaded: {e}"))?;
        let f = unsafe { Functions::load(&library) }?;
        Ok(Sqlite { f, _library: ManuallyDrop::new(library), owned: true })
    }

    /// SQLite as the host has it loaded, by its loader's handle.
    ///
    /// # Safety
    /// `handle` must be a live handle to SQLite from the platform's loader, kept loaded for as long as this lives.
    pub unsafe fn lent(handle: *mut c_void) -> Result<Sqlite, String> {
        if handle.is_null() {
            return Err("no SQLite handle".to_string());
        }
        #[cfg(unix)]
        let library: libloading::Library = unsafe { libloading::os::unix::Library::from_raw(handle) }.into();
        #[cfg(windows)]
        let library: libloading::Library = unsafe { libloading::os::windows::Library::from_raw(handle as isize) }.into();
        let f = unsafe { Functions::load(&library) }?;
        Ok(Sqlite { f, _library: ManuallyDrop::new(library), owned: false })
    }
}

impl Drop for Sqlite {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ManuallyDrop::drop(&mut self._library) };
        }
    }
}

/// A statement that failed: SQLite's primary result code and Microsoft.Data.Sqlite's words for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub code: i32,
    pub message: String,
}

/// A value bound to a named parameter.
pub enum Bound<'a> {
    Blob(&'a [u8]),
    Integer(i64),
}

/// A column of a row read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Column {
    Blob(Vec<u8>),
    Integer(i64),
    /// Anything a blob was asked for and is not, which the C#'s cast refuses.
    Other,
}

/// How long a statement is tried again while the database is busy, and the pause between tries: Microsoft.Data.Sqlite's
/// default timeout as the cache sets it, one second, and its 150 ms.
const TIMEOUT: Duration = Duration::from_secs(1);
const PAUSE: Duration = Duration::from_millis(150);

/// One connection, closed when dropped.
pub struct Connection {
    sqlite: Arc<Sqlite>,
    db: *mut c_void,
}

unsafe impl Send for Connection {}

impl Connection {
    /// Opens the file for reading and writing, making it when it is not there.
    pub fn open(sqlite: Arc<Sqlite>, path: &str) -> Result<Connection, Failure> {
        let Ok(name) = CString::new(path) else { return Err(Failure { code: 14, message: "SQLite Error 14: 'unable to open database file'.".to_string() }) };
        let mut db = std::ptr::null_mut();
        let code = unsafe { (sqlite.f.open_v2)(name.as_ptr(), &mut db, OPEN_READWRITE | OPEN_CREATE, std::ptr::null()) };
        let connection = Connection { sqlite, db };
        if code != OK {
            return Err(connection.failure(code));
        }
        Ok(connection)
    }

    /// Microsoft.Data.Sqlite's exception for a result code: its primary code and `SQLite Error <code>: '<words>'.`
    fn failure(&self, code: c_int) -> Failure {
        let words = unsafe {
            let text = if self.db.is_null() { (self.sqlite.f.errstr)(code) } else { (self.sqlite.f.errmsg)(self.db) };
            if text.is_null() { String::new() } else { String::from_utf8_lossy(CStr::from_ptr(text).to_bytes()).into_owned() }
        };
        Failure { code: code & 0xFF, message: format!("SQLite Error {}: '{words}'.", code & 0xFF) }
    }

    fn busy(code: c_int) -> bool {
        matches!(code & 0xFF, BUSY | LOCKED)
    }

    /// Runs every statement of `sql` in turn with the same parameters bound by name, handing each row of each to
    /// `row`; a busy database is tried again for a second. The rows the last statement changed.
    pub fn run(&self, sql: &str, parameters: &[(&str, Bound)], mut row: impl FnMut(&dyn Fn(i32, bool) -> Column)) -> Result<i32, Failure> {
        let text = CString::new(sql).expect("the cache's own SQL has no NUL");
        let f = &self.sqlite.f;
        let started = Instant::now();
        let mut rest = text.as_ptr();
        let mut changed = 0;
        unsafe {
            while *rest != 0 {
                let mut statement = std::ptr::null_mut();
                let mut tail = std::ptr::null();
                let mut code;
                loop {
                    code = (f.prepare_v2)(self.db, rest, -1, &mut statement, &mut tail);
                    if !Self::busy(code) || started.elapsed() >= TIMEOUT {
                        break;
                    }
                    std::thread::sleep(PAUSE);
                }
                if code != OK {
                    return Err(self.failure(code));
                }
                rest = tail;
                // White space or a comment after the last statement prepares to nothing.
                if statement.is_null() {
                    continue;
                }
                for (name, value) in parameters {
                    let name = CString::new(*name).expect("a parameter's name has no NUL");
                    let index = (f.bind_parameter_index)(statement, name.as_ptr());
                    if index == 0 {
                        continue;
                    }
                    match value {
                        Bound::Blob(bytes) => (f.bind_blob)(statement, index, if bytes.is_empty() { c"".as_ptr().cast() } else { bytes.as_ptr().cast() }, bytes.len() as c_int, TRANSIENT),
                        Bound::Integer(number) => (f.bind_int64)(statement, index, *number),
                    };
                }
                loop {
                    code = (f.step)(statement);
                    if code == ROW {
                        let column = |index: i32, blob: bool| {
                            if !blob {
                                return Column::Integer((f.column_int64)(statement, index));
                            }
                            if (f.column_type)(statement, index) != BLOB {
                                return Column::Other;
                            }
                            let bytes = (f.column_blob)(statement, index);
                            let length = (f.column_bytes)(statement, index).max(0) as usize;
                            Column::Blob(if bytes.is_null() || length == 0 { Vec::new() } else { std::slice::from_raw_parts(bytes.cast::<u8>(), length).to_vec() })
                        };
                        row(&column);
                        continue;
                    }
                    if Self::busy(code) && started.elapsed() < TIMEOUT {
                        (f.reset)(statement);
                        std::thread::sleep(PAUSE);
                        continue;
                    }
                    break;
                }
                if code != DONE {
                    let failure = self.failure(code);
                    (f.finalize)(statement);
                    return Err(failure);
                }
                changed = (f.changes)(self.db);
                (f.finalize)(statement);
            }
        }
        Ok(changed)
    }

    /// A statement with no parameters and no rows wanted.
    pub fn execute(&self, sql: &str) -> Result<i32, Failure> {
        self.run(sql, &[], |_| {})
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if !self.db.is_null() {
            unsafe { (self.sqlite.f.close_v2)(self.db) };
        }
    }
}
