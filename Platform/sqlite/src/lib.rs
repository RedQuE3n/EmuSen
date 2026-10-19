//! SQLite as the platform's crates call it, in the library the program ships: opened from its file, or lent by a host
//! that has it loaded, so that one process holds one SQLite and its locks on a file are one set. The waits and the
//! words of a failure are Microsoft.Data.Sqlite's, which the C# twins are written on. Moved out of Serenity for the
//! library to share. See EmuSen_RustPlatform.md §2.8, §16.2 and EmuSen_Backend.md §28.3.

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
const OPEN_READONLY: c_int = 0x1;
const OPEN_READWRITE: c_int = 0x2;
const OPEN_CREATE: c_int = 0x4;
const INTEGER: c_int = 1;
const FLOAT: c_int = 2;
const TEXT: c_int = 3;
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
    bind_double = "sqlite3_bind_double": fn(*mut c_void, c_int, f64) -> c_int;
    bind_text = "sqlite3_bind_text": fn(*mut c_void, c_int, *const c_char, c_int, isize) -> c_int;
    bind_null = "sqlite3_bind_null": fn(*mut c_void, c_int) -> c_int;
    column_type = "sqlite3_column_type": fn(*mut c_void, c_int) -> c_int;
    column_blob = "sqlite3_column_blob": fn(*mut c_void, c_int) -> *const c_void;
    column_bytes = "sqlite3_column_bytes": fn(*mut c_void, c_int) -> c_int;
    column_int64 = "sqlite3_column_int64": fn(*mut c_void, c_int) -> i64;
    column_double = "sqlite3_column_double": fn(*mut c_void, c_int) -> f64;
    column_text = "sqlite3_column_text": fn(*mut c_void, c_int) -> *const u8;
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

/// A value bound to a named parameter, by its type as Microsoft.Data.Sqlite binds a .NET value.
pub enum Bound<'a> {
    Blob(&'a [u8]),
    Integer(i64),
    Real(f64),
    Text(&'a str),
    Null,
}

/// The type a column of a row holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Integer,
    Real,
    Text,
    Blob,
    Null,
}

/// A row being read, its columns read as SqliteDataReader's getters read them: SQLite's own conversions.
pub struct Row<'a> {
    f: &'a Functions,
    statement: *mut c_void,
}

impl Row<'_> {
    pub fn kind(&self, index: i32) -> Kind {
        match unsafe { (self.f.column_type)(self.statement, index) } {
            INTEGER => Kind::Integer,
            FLOAT => Kind::Real,
            TEXT => Kind::Text,
            BLOB => Kind::Blob,
            _ => Kind::Null,
        }
    }

    pub fn is_null(&self, index: i32) -> bool {
        self.kind(index) == Kind::Null
    }

    pub fn int64(&self, index: i32) -> i64 {
        unsafe { (self.f.column_int64)(self.statement, index) }
    }

    pub fn double(&self, index: i32) -> f64 {
        unsafe { (self.f.column_double)(self.statement, index) }
    }

    pub fn text(&self, index: i32) -> String {
        unsafe {
            let text = (self.f.column_text)(self.statement, index);
            let length = (self.f.column_bytes)(self.statement, index).max(0) as usize;
            if text.is_null() || length == 0 { String::new() } else { String::from_utf8_lossy(std::slice::from_raw_parts(text, length)).into_owned() }
        }
    }

    /// The column as text, or none where it is null: `IsDBNull` then `GetString`.
    pub fn text_or_null(&self, index: i32) -> Option<String> {
        if self.is_null(index) { None } else { Some(self.text(index)) }
    }

    pub fn blob(&self, index: i32) -> Vec<u8> {
        unsafe {
            let bytes = (self.f.column_blob)(self.statement, index);
            let length = (self.f.column_bytes)(self.statement, index).max(0) as usize;
            if bytes.is_null() || length == 0 { Vec::new() } else { std::slice::from_raw_parts(bytes.cast::<u8>(), length).to_vec() }
        }
    }
}

/// A column of a row read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Column {
    Blob(Vec<u8>),
    Integer(i64),
    /// Anything a blob was asked for and is not, which the C#'s cast refuses.
    Other,
}

/// How long a statement is tried again while the database is busy, and the pause between tries: one second, as the
/// SPIR-V cache sets its timeout, and Microsoft.Data.Sqlite's 150 ms. `Connection::set_timeout` gives another.
const TIMEOUT: Duration = Duration::from_secs(1);
const PAUSE: Duration = Duration::from_millis(150);

/// Microsoft.Data.Sqlite's default command timeout, 30 seconds, for a connection that keeps it.
pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// One connection, closed when dropped.
pub struct Connection {
    sqlite: Arc<Sqlite>,
    db: *mut c_void,
    timeout: Duration,
}

unsafe impl Send for Connection {}

impl Connection {
    /// Opens the file for reading and writing, making it when it is not there.
    pub fn open(sqlite: Arc<Sqlite>, path: &str) -> Result<Connection, Failure> {
        Connection::open_with(sqlite, path, OPEN_READWRITE | OPEN_CREATE)
    }

    /// Opens the file for reading only, as Microsoft.Data.Sqlite's `Mode=ReadOnly`: a missing file is refused, nothing made.
    pub fn open_read_only(sqlite: Arc<Sqlite>, path: &str) -> Result<Connection, Failure> {
        Connection::open_with(sqlite, path, OPEN_READONLY)
    }

    fn open_with(sqlite: Arc<Sqlite>, path: &str, flags: c_int) -> Result<Connection, Failure> {
        let Ok(name) = CString::new(path) else { return Err(Failure { code: 14, message: "SQLite Error 14: 'unable to open database file'.".to_string() }) };
        let mut db = std::ptr::null_mut();
        let code = unsafe { (sqlite.f.open_v2)(name.as_ptr(), &mut db, flags, std::ptr::null()) };
        let connection = Connection { sqlite, db, timeout: TIMEOUT };
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

    /// How long a busy database is tried again before the statement fails.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    fn busy(code: c_int) -> bool {
        matches!(code & 0xFF, BUSY | LOCKED)
    }

    /// Runs every statement of `sql` in turn with the same parameters bound by name, handing each row of each to
    /// `row`; a busy database is tried again for the connection's timeout. The rows the last statement changed.
    pub fn run(&self, sql: &str, parameters: &[(&str, Bound)], mut row: impl FnMut(&dyn Fn(i32, bool) -> Column)) -> Result<i32, Failure> {
        self.query(sql, parameters, |r| {
            let column = |index: i32, blob: bool| {
                if !blob {
                    return Column::Integer(r.int64(index));
                }
                if r.kind(index) != Kind::Blob {
                    return Column::Other;
                }
                Column::Blob(r.blob(index))
            };
            row(&column);
        })
    }

    /// As `run`, each row handed over to be read by its typed getters.
    pub fn query(&self, sql: &str, parameters: &[(&str, Bound)], mut row: impl FnMut(&Row)) -> Result<i32, Failure> {
        let text = CString::new(sql).expect("the platform's own SQL has no NUL");
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
                    if !Self::busy(code) || started.elapsed() >= self.timeout {
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
                        Bound::Real(number) => (f.bind_double)(statement, index, *number),
                        Bound::Text(text) => (f.bind_text)(statement, index, if text.is_empty() { c"".as_ptr() } else { text.as_ptr().cast() }, text.len() as c_int, TRANSIENT),
                        Bound::Null => (f.bind_null)(statement, index),
                    };
                }
                loop {
                    code = (f.step)(statement);
                    if code == ROW {
                        row(&Row { f, statement });
                        continue;
                    }
                    if Self::busy(code) && started.elapsed() < self.timeout {
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
