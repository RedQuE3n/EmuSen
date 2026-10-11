//! Builds the seven record programs that carry the release build 158's code (tests/support/records.rs) from the corpus
//! `EMUSEN_DIAMOND_PS1TESTS` names: `records [OUT]` writes each one's source and PS-EXE into OUT, by default
//! `~/.cache/emusen/probe/diamond/records/`, for the referees (Diamond_Records.md §1.1). Never into the tree.

#[path = "../tests/support/records.rs"]
mod records;

fn main() {
    let corpus = records::corpus().unwrap_or_else(|| panic!("{} names no ps1-tests build 158", records::CORPUS_VARIABLE));
    let out = std::env::args().nth(1).map(std::path::PathBuf::from).unwrap_or_else(|| {
        std::path::PathBuf::from(std::env::var("HOME").expect("HOME")).join(".cache/emusen/probe/diamond/records")
    });
    std::fs::create_dir_all(&out).expect("the output folder");
    let dialect = blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR));
    for record in &records::RECORDS {
        let source = records::source(record, &corpus).unwrap_or_else(|e| panic!("{e}"));
        let words = emusen_mips::assemble(&source, 0x8001_0000, &dialect).unwrap_or_else(|e| panic!("{}: {e}", record.name));
        std::fs::write(out.join(format!("{}.s", record.name)), &source).expect("written");
        std::fs::write(out.join(format!("{}.exe", record.name)), diamond::exe::build(&words)).expect("written");
        println!("{}: {} words", record.name, words.len());
    }
}
