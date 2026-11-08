//! The build host's notes, warnings and errors, in the form MSBuild's Exec reads as its own.

pub fn note(text: &str) {
    println!("xtask: {text}");
}

pub fn warn(text: &str) {
    println!("xtask : warning : {text}");
}

pub fn error(text: &str) {
    eprintln!("xtask : error : {text}");
}
