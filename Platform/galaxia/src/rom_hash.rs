//! A ROM file's identity across a rename: the MD5 of every byte, header included. See EmuSen_Galaxia.md §5.4.

use md5::{Digest, Md5};
use std::io::Read;

/// The file's MD5 as 32 lower-case hexadecimal digits.
pub fn md5(path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use crate::test_support::TempDir;

    #[test]
    fn the_hash_is_md5s_own_test_vectors() {
        let temp = TempDir::new("md5");
        let file = temp.join("rom.bin");
        std::fs::write(&file, b"").unwrap();
        assert_eq!(super::md5(&file).unwrap(), "d41d8cd98f00b204e9800998ecf8427e");
        std::fs::write(&file, b"abc").unwrap();
        assert_eq!(super::md5(&file).unwrap(), "900150983cd24fb0d6963f7d28e17f72");
        std::fs::write(&file, vec![b'a'; 1_000_000]).unwrap();
        assert_eq!(super::md5(&file).unwrap(), "7707d6ae4e027c70eea2a935c2296f21");
        assert!(super::md5(&temp.join("absent.bin")).is_err());
    }
}
