/*
 * emusen_platform.h - the C interface of EmuSen's platform library.
 *
 * The library holds the platform components that have moved to Rust (Galaxia first) and is loaded by the C#
 * programs, which keep their own types and forward to it. This header is the interface's specification; the rules
 * are argued in EmuSen_RustPlatform.md, section 3, and are the core ABI's (emusen_core.h) one layer up:
 *
 *  - The host calls and the library returns. The library never calls the host: what the C# reports through a
 *    delegate is returned to it, as a status, as words or as queued diagnostics.
 *  - Text is UTF-8, passed as a pointer and a length in bytes and never assumed to end in a NUL. A null pointer is
 *    a null string and is told apart from an empty one.
 *  - A result is copied into the caller's buffer by the length-query idiom: the call returns the whole length in
 *    bytes and writes only when the buffer holds all of it; a null buffer asks the length. Nothing passed in is
 *    kept after the call, and the library returns no pointer.
 *  - A return below zero is a status. EMUSEN_PLATFORM_ABSENT is an answer, not a failure: the thing asked for does
 *    not exist. The words for a failure are read with emusen_platform_last_error on the same thread.
 *  - Every call here is safe from any thread, concurrently with any other.
 *  - Nothing unwinds. The library is built to abort on a panic, after writing the crash log the host named.
 *
 * This interface is not stable. Its one version number is matched exactly by the host built from the same commit,
 * and nothing outside the repository may rely on it (EmuSen_RustPlatform.md, section 3.2).
 */
#ifndef EMUSEN_PLATFORM_H
#define EMUSEN_PLATFORM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define EMUSEN_PLATFORM_ABI_VERSION 1u

/* Statuses. Zero and above is success: a count or a length. */
#define EMUSEN_PLATFORM_NULL (-1)            /* a null pointer where one was required */
#define EMUSEN_PLATFORM_BAD_STRING (-5)      /* text that is not UTF-8 */
#define EMUSEN_PLATFORM_NOT_SUPPORTED (-256) /* an operation this build does not have */
#define EMUSEN_PLATFORM_IO (-1024)           /* reading or writing failed */
/* -1025 to -1029 are kept for parsing, schema, newer-file, device and compile failures of later components. */
#define EMUSEN_PLATFORM_ABSENT (-1030)       /* what was asked for does not exist; an answer, not a failure */
#define EMUSEN_PLATFORM_NOT_FOUND (-1031)    /* a file or directory that had to exist does not */
#define EMUSEN_PLATFORM_ACCESS (-1032)       /* the system refused access */
#define EMUSEN_PLATFORM_BAD_ARGUMENT (-1033) /* a kind, an operation or a number the call does not know */

/* ---- The library ---- */

/* The interface's version. A host refuses a library whose number is not its own and calls nothing else in it. */
uint32_t emusen_platform_abi_version(void);

/* Where a panic is written before the process aborts. The first path given stands. */
int32_t emusen_platform_set_crash_log(const uint8_t *path, size_t path_len);

/* The words for the last failing call on the calling thread. */
int64_t emusen_platform_last_error(uint8_t *out, size_t len);

/*
 * The calling thread's waiting diagnostics: what the C# Galaxia reports through ConfigDiagnostics (a save that
 * could not be written, a file left in place by a migration). Each message is followed by a NUL. They are removed
 * once a buffer has taken them all; a null or short buffer leaves them waiting.
 */
int64_t emusen_platform_diagnostics(uint8_t *out, size_t len);

/* ---- Galaxia: the tree ---- */

/* The directories of the tree (EmuSen_Galaxia.md, section 3; `man hier`). */
#define EMUSEN_GALAXIA_DIR_ROOT 0u            /* ConfigRoot.Directory */
#define EMUSEN_GALAXIA_DIR_SEED 1u            /* ConfigRoot.SeedDirectory; absent outside a Mac bundle */
#define EMUSEN_GALAXIA_DIR_CONFIG 2u          /* ConfigStore.Directory */
#define EMUSEN_GALAXIA_DIR_CONFIG_PREVIOUS 3u /* ConfigStore.PreviousDirectory */
#define EMUSEN_GALAXIA_DIR_CONFIG_LEGACY 4u   /* ConfigStore.LegacyDirectory */
#define EMUSEN_GALAXIA_DIR_HOME 5u            /* DataStore.UsrHome */
#define EMUSEN_GALAXIA_DIR_LOGS 6u
#define EMUSEN_GALAXIA_DIR_SAVES 7u
#define EMUSEN_GALAXIA_DIR_SAVE_STATES 8u
#define EMUSEN_GALAXIA_DIR_LIBRARY 9u
#define EMUSEN_GALAXIA_DIR_SCREENSHOTS 10u
#define EMUSEN_GALAXIA_DIR_ARTWORK 11u
#define EMUSEN_GALAXIA_DIR_MEDIA 12u
#define EMUSEN_GALAXIA_DIR_FIRMWARE 13u
#define EMUSEN_GALAXIA_DIR_SHADERS 14u
#define EMUSEN_GALAXIA_DIR_THEMES 15u
#define EMUSEN_GALAXIA_DIR_CHEATS 16u
#define EMUSEN_GALAXIA_DIR_GAMES 17u          /* a skeleton directory of the shell's tree, not the ROM library */
#define EMUSEN_GALAXIA_DIR_LEGACY_ROOT 18u    /* DataMigration.LegacyRoot */

/* The redirects a test moves. */
#define EMUSEN_GALAXIA_OVERRIDE_CONFIG 0u /* ConfigStore.OverrideDirectory */
#define EMUSEN_GALAXIA_OVERRIDE_DATA 1u   /* DataStore.OverrideDirectory */
#define EMUSEN_GALAXIA_OVERRIDE_LEGACY 2u /* ConfigStore.OverrideLegacyDirectory */

/*
 * Names the program's directory (AppContext.BaseDirectory, which a managed host knows and the library cannot
 * find) and .NET's SpecialFolder.ApplicationData, before the tree is first asked for. A null application_data
 * leaves the library to work it out. Returns 0, or 1 when the tree was already computed and the call changed
 * nothing. A host that never calls this gets a tree rooted from the library's own executable.
 */
int32_t emusen_galaxia_init(const uint8_t *base, size_t base_len, const uint8_t *application_data, size_t application_data_len);

/* A directory of the process's tree, by its EMUSEN_GALAXIA_DIR_* number. */
int64_t emusen_galaxia_directory(uint32_t which, uint8_t *out, size_t len);

/* Moves a redirect, by its EMUSEN_GALAXIA_OVERRIDE_* number; a null directory restores the real location. */
int32_t emusen_galaxia_set_override(uint32_t which, const uint8_t *directory, size_t directory_len);

/* A config file's path; a null category is the config directory itself, else its subdirectory of that name. */
int64_t emusen_galaxia_config_path(const uint8_t *category, size_t category_len, const uint8_t *file, size_t file_len, uint8_t *out, size_t len);

/* Where a config file sat before Galaxia. ABSENT when config is redirected and the legacy directory is not. */
int64_t emusen_galaxia_legacy_config_path(const uint8_t *file, size_t file_len, uint8_t *out, size_t len);

/* The root a program in `base` would have, with the platform and the home directory as parameters. */
int64_t emusen_galaxia_root_for(const uint8_t *base, size_t base_len, int32_t mac_os, const uint8_t *user_home, size_t user_home_len, uint8_t *out, size_t len);

/* A Mac bundle's Contents directory when `base` is its Contents/MacOS, else ABSENT. */
int64_t emusen_galaxia_bundle_contents_for(const uint8_t *base, size_t base_len, uint8_t *out, size_t len);

/* The skeleton a bundle carries, Contents/Resources/home; ABSENT outside a bundle or when mac_os is 0. */
int64_t emusen_galaxia_seed_directory_for(const uint8_t *base, size_t base_len, int32_t mac_os, uint8_t *out, size_t len);

/* Library/Application Support/EmuSen under a home directory. */
int64_t emusen_galaxia_mac_data_directory_for(const uint8_t *user_home, size_t user_home_len, uint8_t *out, size_t len);

/* Where the tree lived under a root before it collapsed to <root>/home. */
int64_t emusen_galaxia_legacy_root_for(const uint8_t *config_root, size_t config_root_len, uint8_t *out, size_t len);

/* ---- Galaxia: the save library's names (EmuSen_Galaxia.md, section 5) ---- */

#define EMUSEN_GALAXIA_SAVE_SRAM 0u         /* Saves/<console>/<stem>.srm; extra is the console, required */
#define EMUSEN_GALAXIA_SAVE_FLAT_SRAM 1u    /* Saves/<stem>.srm, read to copy across and never written */
#define EMUSEN_GALAXIA_SAVE_STATE 2u        /* <stem>.state for slot 1, <stem>.slotN.state otherwise; extra is the directory override */
#define EMUSEN_GALAXIA_SAVE_RESUME_STATE 3u /* <stem>.resume.state; extra is the directory override */
#define EMUSEN_GALAXIA_SAVE_PICTURE 4u      /* rom is a state's path; the same name with .png */

/* A save file's path. A null or blank directory override is the tree's own Save States directory. */
int64_t emusen_galaxia_save_path(uint32_t kind, const uint8_t *rom, size_t rom_len, const uint8_t *extra, size_t extra_len, int32_t slot, uint8_t *out, size_t len);

/* ---- Galaxia: files (EmuSen_Galaxia.md, section 4) ---- */

/*
 * A file's bytes. Returns its whole length; `out` holds the file when the length returned is no more than `len`,
 * and otherwise holds nothing to rely on. A null `out` asks the length. ABSENT where there is no file. A file that
 * is there and will not read is IO, with a diagnostic queued.
 */
int64_t emusen_galaxia_file_read(const uint8_t *file, size_t file_len, uint8_t *out, size_t len);

/*
 * Writes the bytes to <file>.tmp and renames that over the file, creating the directory first. A reader or a
 * killed process sees the whole old file or the whole new one. Nothing is flushed to the device: this is not a
 * promise against power loss. A failure is IO with a diagnostic queued, and the live file is as it was.
 */
int32_t emusen_galaxia_file_write(const uint8_t *file, size_t file_len, const uint8_t *data, size_t len);

/* A file's MD5, every byte of it, as 32 lower-case hexadecimal digits. */
int64_t emusen_galaxia_rom_md5(const uint8_t *file, size_t file_len, uint8_t *out, size_t len);

/* ---- Galaxia: migration (EmuSen_Galaxia.md, section 3.2) ---- */

#define EMUSEN_GALAXIA_MIGRATE_OWN 0u        /* DataMigration.Run(): the process's own old tree, then its old config */
#define EMUSEN_GALAXIA_MIGRATE_OWN_CONFIG 1u /* DataMigration.RunConfig() */
#define EMUSEN_GALAXIA_MIGRATE_OWN_SEED 2u   /* DataMigration.SeedFromBundle() */
#define EMUSEN_GALAXIA_MIGRATE_BETWEEN 3u    /* DataMigration.Run(source, destination): the four emulator-written directories */
#define EMUSEN_GALAXIA_MIGRATE_TREE 4u       /* every file under source that destination lacks */
#define EMUSEN_GALAXIA_MIGRATE_SEED 5u       /* as TREE, and nothing when source is null */

/*
 * Copies, never moves and never overwrites. Returns how many files were copied; each file that could not be is
 * a diagnostic and is left in place. A directory that cannot be listed is a failing status. The ROM library
 * (Games, Roms) is never copied.
 */
int64_t emusen_galaxia_migrate(uint32_t what, const uint8_t *source, size_t source_len, const uint8_t *destination, size_t destination_len);

/* The old ROM folders under a legacy root that still hold files, each path followed by a NUL. A null root asks about the process's own. */
int64_t emusen_galaxia_remaining_library(const uint8_t *legacy_root, size_t legacy_root_len, uint8_t *out, size_t len);

/* ---- Galaxia: the .NET rules the tree is built from, exported for the parity tests ---- */

#define EMUSEN_GALAXIA_PATH_COMBINE 0u                     /* Path.Combine(a, b) */
#define EMUSEN_GALAXIA_PATH_FILE_NAME 1u                   /* Path.GetFileName(a) */
#define EMUSEN_GALAXIA_PATH_FILE_NAME_WITHOUT_EXTENSION 2u /* Path.GetFileNameWithoutExtension(a) */
#define EMUSEN_GALAXIA_PATH_CHANGE_EXTENSION 3u            /* Path.ChangeExtension(a, b) */
#define EMUSEN_GALAXIA_PATH_DIRECTORY_NAME 4u              /* Path.GetDirectoryName(a); ABSENT for null */
#define EMUSEN_GALAXIA_PATH_TRIM_ENDING_SEPARATOR 5u       /* Path.TrimEndingDirectorySeparator(a) */
#define EMUSEN_GALAXIA_PATH_FULL_PATH 6u                   /* Path.GetFullPath(a) */

/* One path rule as the library applies it, so a test can hold it to .NET's own answer. */
int64_t emusen_galaxia_dotnet_path(uint32_t op, const uint8_t *a, size_t a_len, const uint8_t *b, size_t b_len, uint8_t *out, size_t len);

#define EMUSEN_GALAXIA_TEST_FILE_EXISTS 0u           /* File.Exists(a) */
#define EMUSEN_GALAXIA_TEST_DIRECTORY_EXISTS 1u      /* Directory.Exists(a) */
#define EMUSEN_GALAXIA_TEST_IS_BLANK 2u              /* string.IsNullOrWhiteSpace(a) */
#define EMUSEN_GALAXIA_TEST_EQUALS_IGNORE_CASE 3u    /* string.Equals(a, b, OrdinalIgnoreCase) */
#define EMUSEN_GALAXIA_TEST_ENDS_WITH_IGNORE_CASE 4u /* a.EndsWith(b, OrdinalIgnoreCase) */

/* One yes-or-no rule as the library applies it: 1, 0 or a status. */
int32_t emusen_galaxia_dotnet_test(uint32_t op, const uint8_t *a, size_t a_len, const uint8_t *b, size_t b_len);

#ifdef __cplusplus
}
#endif

#endif /* EMUSEN_PLATFORM_H */
