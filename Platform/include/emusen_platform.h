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
 *    kept after the call, except SDL's handle and a simulated pad or set plugged into another, which are said so
 *    where they are passed; the only pointers returned are handles the library made.
 *  - A return below zero is a status. EMUSEN_PLATFORM_ABSENT is an answer, not a failure: the thing asked for does
 *    not exist. The words for a failure are read with emusen_platform_last_error on the same thread.
 *  - Every call of Galaxia's is safe from any thread, concurrently with any other. A resampler, a router and an
 *    interface player of Endymion's are used by one thread at a time; a rate control, an audio player, a simulated
 *    pad, a set of them and a pads handle lock themselves and may be used from any thread, as the programs use
 *    them; Endymion's other calls are safe from any thread.
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

#define EMUSEN_PLATFORM_ABI_VERSION 4u

/* Statuses. Zero and above is success: a count or a length. */
#define EMUSEN_PLATFORM_NULL (-1)            /* a null pointer where one was required */
#define EMUSEN_PLATFORM_BAD_STRING (-5)      /* text that is not UTF-8 */
#define EMUSEN_PLATFORM_NOT_SUPPORTED (-256) /* an operation this build does not have */
#define EMUSEN_PLATFORM_IO (-1024)           /* reading or writing failed */
#define EMUSEN_PLATFORM_PARSE (-1025)        /* text that is not JSON as .NET reads it, or not what its model allows */
/* -1026 to -1029 are kept for newer-file, device and compile failures of later components. */
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
#define EMUSEN_GALAXIA_DIR_LOG_DEFAULT 19u    /* ErrorLog.DefaultRoot */

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

/* ---- Galaxia: config files (EmuSen_Config_Reference.md, section 2) ---- */

/*
 * A config file is named by its file name and, where a kind of config has many files, a category, which is a
 * subdirectory of the config directory; a null category is the config directory itself. Its path is
 * emusen_galaxia_config_path, and whether it exists EMUSEN_GALAXIA_TEST_FILE_EXISTS of that path.
 */

/* Deletes a config file. 1 when it is gone afterwards, whether or not it was there; 0 when its folder is not there or it would not go. */
int32_t emusen_galaxia_config_delete(const uint8_t *category, size_t category_len, const uint8_t *file, size_t file_len);

/*
 * A config file's text, for a host that binds the type itself (a file whose types belong to a frontend). A file
 * that is not there but sits where config was kept before Galaxia is copied in first, never moved, and read from
 * the old place when the copy cannot be made; only a file with no category ever sat there. The bytes are decoded
 * as .NET's File.ReadAllText decodes them (emusen_galaxia_text_decode). ABSENT for no file; IO, with the system's
 * words in emusen_platform_last_error, for one that will not read.
 */
int64_t emusen_galaxia_config_read(const uint8_t *category, size_t category_len, const uint8_t *file, size_t file_len, uint8_t *out, size_t len);

/* Writes text as a config file: its folder made, the text written to <file>.tmp and renamed over the file. 0 or IO. Null text fails after the folder is made. Nothing is queued as a diagnostic. */
int32_t emusen_galaxia_config_write(const uint8_t *category, size_t category_len, const uint8_t *file, size_t file_len, const uint8_t *text, size_t text_len);

/* ---- Galaxia: the models (EmuSen_Config_Reference.md, section 3) ---- */

/* The config models whose schemas the library owns, each mirroring a C# class of EmuSen.Galaxia.Models. */
#define EMUSEN_GALAXIA_MODEL_APP_SETTINGS 0u    /* AppSettings, appsettings.json */
#define EMUSEN_GALAXIA_MODEL_AUDIO_CONFIG 1u    /* AudioConfig, audio.json */
#define EMUSEN_GALAXIA_MODEL_GRAPHICS_CONFIG 2u /* GraphicsConfig, graphics.json */
#define EMUSEN_GALAXIA_MODEL_CHEAT_FILE 3u      /* CheatFile, a file of the cheats category */

/* Flags of the calls that read or make a model. */
#define EMUSEN_GALAXIA_MODEL_UPGRADE 1u /* apply the model's own upgrade, as AppSettings.Load does and ConfigFile<T>.Load does not */

/*
 * A model crosses as a JSON document. Out of the library it is the bound document: an object holding every field
 * the model can set, in the model's order, with a number as the token the file had. Into the library it is any
 * JSON the model binds, by the rules of reading a file. A value the model computes and writes but never reads
 * (CheatFileEntry.IsRomPatch and EffectiveWrites) is left out on the way out and ignored on the way in.
 *
 * Reading follows System.Text.Json under ConfigJson.Options, measured rule by rule (EmuSen_RustPlatform.md,
 * sections 5.2 and 11.3): comments and a trailing comma are allowed, names match without case, the last of two
 * members wins whole, an unknown member is ignored, and a field with no member keeps its value in a new instance.
 */

/*
 * A model's file as its bound document. The text "null" for a file whose document is null, which is no settings
 * and no error. ABSENT for no file. PARSE or IO for a file that will not load, with the words for the diagnostic
 * in emusen_platform_last_error; the host reports "<path>: <words> Falling back to defaults." and uses defaults.
 */
int64_t emusen_galaxia_model_load(uint32_t model, const uint8_t *category, size_t category_len, const uint8_t *file, size_t file_len, uint32_t flags, uint8_t *out, size_t len);

/*
 * Saves a document as a model's file: the folder is made, the document is written as the bytes System.Text.Json
 * writes for the class, and they are put in place by a rename. A null pointer for the document fails, after the
 * folder is made, as a class C# could not serialize fails. 0 or IO. Nothing is queued as a diagnostic.
 */
int32_t emusen_galaxia_model_save(uint32_t model, const uint8_t *category, size_t category_len, const uint8_t *file, size_t file_len, const uint8_t *document, size_t document_len);

/* A new instance of a model: every field at its initial value. */
int64_t emusen_galaxia_model_new(uint32_t model, uint32_t flags, uint8_t *out, size_t len);

/* Text bound to a model with no file involved: the bound document, "null" for a document that is null, or PARSE with the words. */
int64_t emusen_galaxia_model_bind(uint32_t model, const uint8_t *text, size_t text_len, uint32_t flags, uint8_t *out, size_t len);

/* A document as the bytes of the model's file. PARSE, with the words, for one that cannot be written (a number JSON has no token for, a cheat with no list of writes). */
int64_t emusen_galaxia_model_format(uint32_t model, const uint8_t *document, size_t document_len, uint8_t *out, size_t len);

/* A model's fields, one to a line: Class.Name, a tab, the type, and after another tab "derived" or "skipped when null" where that applies. Each class once, the classes it holds after it. */
int64_t emusen_galaxia_model_schema(uint32_t model, uint8_t *out, size_t len);

/* A model read from any path, as CheatFile.LoadFrom reads one. ABSENT for a file that is missing, unreadable or will not bind; nothing is said of it. */
int64_t emusen_galaxia_model_load_from(uint32_t model, const uint8_t *file, size_t file_len, uint8_t *out, size_t len);

/* A document written straight to any path, its folder made first, with no temp file: as CheatFile.SaveTo writes a file the player picked. 0 or IO. */
int32_t emusen_galaxia_model_save_to(uint32_t model, const uint8_t *file, size_t file_len, const uint8_t *document, size_t document_len);

/* The saved cheat lists by name, in the order of StringComparer.OrdinalIgnoreCase, each followed by a NUL. */
int64_t emusen_galaxia_cheat_names(uint8_t *out, size_t len);

/* Whether a cheat list may have this name: 1 or 0. It is checked and never rewritten. */
int32_t emusen_galaxia_cheat_name_valid(const uint8_t *name, size_t name_len);

/* ---- Galaxia: the error log (EmuSen_Settings_Reference.md, section 4.70) ---- */

/*
 * The log's folder: the directory given, else the LogDirectory setting when that folder can be made, else the
 * default. Reading the setting reads appsettings.json; if that will not load, the diagnostic is queued.
 */
int64_t emusen_galaxia_log_root(const uint8_t *directory, size_t directory_len, uint8_t *out, size_t len);

/* Whether a folder is named and can be made: 1 or 0. It is made by the asking. */
int32_t emusen_galaxia_log_usable(const uint8_t *directory, size_t directory_len);

/* A day's file in a root: <root>/emusen_<day>.log. The day is text the caller formatted (yyyyMMdd). */
int64_t emusen_galaxia_log_path(const uint8_t *root, size_t root_len, const uint8_t *day, size_t day_len, uint8_t *out, size_t len);

/*
 * One entry as text: "<stamp> <level> [<area>] <message>", then "    context: <context>" when the context is not
 * blank, then a fault's text with each line indented by four. A null area is empty; a null message is NULL. A
 * line break inside the message or the context is
 * replaced so that each stays on its line. The stamp is text the caller formatted, so the library holds no clock
 * and no calendar. The host may redact the text before it appends it.
 */
int64_t emusen_galaxia_log_format(const uint8_t *stamp, size_t stamp_len, const uint8_t *level, size_t level_len, const uint8_t *area, size_t area_len, const uint8_t *message, size_t message_len,
                                  const uint8_t *context, size_t context_len, const uint8_t *fault, size_t fault_len, uint8_t *out, size_t len);

/*
 * Appends an entry to a day's file (emusen_galaxia_log_path). On the first append of a process the folder's day
 * files last written more than 14 days before `now_unix_ms` are removed. A file of 8 MB takes nothing more; the
 * entry that would fill it is written with one last notice after it. 1 when the entry was written, 0 when it was
 * not, for any reason: a log that cannot be written is never a second fault. A null entry is one the host could
 * not put into UTF-8; it fails where C# fails with such text, at the write, after the pruning and with the file made.
 */
int32_t emusen_galaxia_log_append(const uint8_t *file, size_t file_len, const uint8_t *stamp, size_t stamp_len, int64_t now_unix_ms, const uint8_t *entry, size_t entry_len);

/* Lets the next append prune again; for a test. */
int32_t emusen_galaxia_log_reset(void);

/* ---- Galaxia: "Did you mean ...?" (EmuSen_Config_Reference.md, section 6) ---- */

#define EMUSEN_GALAXIA_SUGGEST_HINT 0u    /* the end of a sentence: " Did you mean 'x'?", or nothing */
#define EMUSEN_GALAXIA_SUGGEST_NEAREST 1u /* the nearest names, each followed by a NUL */

/* How far apart two names are: edits counted in UTF-16 code units, a swap of two neighbours counting once, case ignored. */
int32_t emusen_galaxia_suggest_distance(const uint8_t *a, size_t a_len, const uint8_t *b, size_t b_len);

/* The candidates nearest a name that matched none of them. The candidates are given each followed by a NUL. */
int64_t emusen_galaxia_suggest(uint32_t what, const uint8_t *typed, size_t typed_len, const uint8_t *candidates, size_t candidates_len, int32_t max, uint8_t *out, size_t len);

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

#define EMUSEN_GALAXIA_NUMBER_DOUBLE 0u /* bits is a double's 64 */
#define EMUSEN_GALAXIA_NUMBER_SINGLE 1u /* the low 32 of bits are a float's */

/* A number as System.Text.Json writes it. ABSENT for an infinity or a NaN, which it refuses to write. */
int64_t emusen_galaxia_number_format(uint32_t kind, uint64_t bits, uint8_t *out, size_t len);

/* A file's bytes as File.ReadAllText decodes them: a byte-order mark chooses UTF-8, UTF-16 or UTF-32 and is dropped; what is not valid becomes U+FFFD. */
int64_t emusen_galaxia_text_decode(const uint8_t *bytes, size_t bytes_len, uint8_t *out, size_t len);

/* ---- Endymion: the logic half (EmuSen_Audio_Sync.md, sections 1 to 3; EmuSen_Input.md, sections 7 and 8) ---- */

/*
 * A call that changes a handle and makes output (samples, or changes for the core) takes the caller's buffer and
 * returns the whole count. When the buffer holds it all it is filled; otherwise the output waits in the handle and
 * the matching *_take copies it, so that a call that changes state is never made twice to learn its length. Output
 * not taken is dropped by the next call that makes some.
 */

typedef struct emusen_endymion_resampler emusen_endymion_resampler;
typedef struct emusen_endymion_rate emusen_endymion_rate;
typedef struct emusen_endymion_router emusen_endymion_router;

/* A phase-continuous linear resampler for interleaved stereo: LinearResampler. */
emusen_endymion_resampler *emusen_endymion_resampler_new(void);
int32_t emusen_endymion_resampler_free(emusen_endymion_resampler *resampler);
int32_t emusen_endymion_resampler_reset(emusen_endymion_resampler *resampler);

/*
 * Resamples `len` samples by `ratio` output frames per input frame: the count of samples made. A ratio of zero or
 * below is BAD_ARGUMENT, with the words "Resample ratio must be positive."; a ratio that is NaN or infinite is
 * BAD_ARGUMENT too, with "Resample ratio must be finite.", where the C# takes it and never returns.
 */
int64_t emusen_endymion_resampler_run(emusen_endymion_resampler *resampler, const int16_t *input, size_t len, double ratio, int16_t *out, size_t cap);
int64_t emusen_endymion_resampler_take(emusen_endymion_resampler *resampler, int16_t *out, size_t cap);

/* Absorbs clock drift by resampling, never by discarding: DynamicRateControl. */
emusen_endymion_rate *emusen_endymion_rate_new(int32_t target_queued_frames);
int32_t emusen_endymion_rate_free(emusen_endymion_rate *rate);
int32_t emusen_endymion_rate_reset(emusen_endymion_rate *rate);

#define EMUSEN_ENDYMION_RATE_TARGET_QUEUED_FRAMES 0u /* a whole number of frames, given as a double */
#define EMUSEN_ENDYMION_RATE_MAX_DEVIATION 1u
#define EMUSEN_ENDYMION_RATE_SHEDDING_ENTRY_FACTOR 2u
#define EMUSEN_ENDYMION_RATE_SHEDDING_EXIT_FACTOR 3u
#define EMUSEN_ENDYMION_RATE_NOMINAL_RATIO 4u

int32_t emusen_endymion_rate_set(emusen_endymion_rate *rate, uint32_t which, double value);

/* A rate control's whole state. The caller sets `size`; one smaller than this struct is refused. */
typedef struct emusen_endymion_rate_state {
    uint32_t size;
    int32_t target_queued_frames;
    int32_t shedding_events;
    uint32_t is_shedding;
    int64_t total_input_frames;
    int64_t total_output_frames;
    double max_deviation;
    double shedding_entry_factor;
    double shedding_exit_factor;
    double nominal_ratio;
    double last_ratio;
} emusen_endymion_rate_state;

int32_t emusen_endymion_rate_read(emusen_endymion_rate *rate, emusen_endymion_rate_state *out);

/* The ratio a queue of `queued_frames` calls for. */
int32_t emusen_endymion_rate_compute(emusen_endymion_rate *rate, int32_t queued_frames, double *out);

/* One batch of samples against the queue's fill: the count of samples to hand the device; none while shedding. A null input sets the last ratio and is NULL. */
int64_t emusen_endymion_rate_process(emusen_endymion_rate *rate, const int16_t *input, size_t len, int32_t queued_frames, int16_t *out, size_t cap);
int64_t emusen_endymion_rate_take(emusen_endymion_rate *rate, int16_t *out, size_t cap);

/*
 * Every player's pad and the keyboard routed to the game's controller ports: PortRouter. The router reads no
 * device: the host reads each pad and hands the reading in, and the router answers with the changes to make to the
 * core, in the order the C# makes them: each port past the first's controller, then for each port the buttons that
 * changed in PadButton's order and every read axis in the order the reset gave them.
 */
emusen_endymion_router *emusen_endymion_router_new(void);
int32_t emusen_endymion_router_free(emusen_endymion_router *router);

#define EMUSEN_ENDYMION_ROUTER_KEYBOARD_PLAYER 0u          /* 1-based; settable */
#define EMUSEN_ENDYMION_ROUTER_MIRROR_PLAYER1_TO_PLAYER2 1u /* 0 or 1; settable */
#define EMUSEN_ENDYMION_ROUTER_PORTS 2u
#define EMUSEN_ENDYMION_ROUTER_PLAYERS_READ 3u             /* the players a poll reads: the ports, at most eight */
#define EMUSEN_ENDYMION_ROUTER_AXES 4u                     /* how many axes the reset gave */

int32_t emusen_endymion_router_get(emusen_endymion_router *router, uint32_t which);
int32_t emusen_endymion_router_set(emusen_endymion_router *router, uint32_t which, int32_t value);

#define EMUSEN_ENDYMION_CHANGE_CONNECTED 0u /* a port past the first holds a controller (on) or not */
#define EMUSEN_ENDYMION_CHANGE_BUTTON 1u    /* which is a PadButton, held (on) or let go */
#define EMUSEN_ENDYMION_CHANGE_AXIS 2u      /* which is a PadAxis, its value in value */

typedef struct emusen_endymion_change {
    uint32_t kind;
    int32_t port;
    uint32_t which;
    uint32_t on;
    double value;
} emusen_endymion_change;

/* A game just started: its ports and the axes it reads, by PadAxis, with nothing held and nothing sent. */
int32_t emusen_endymion_router_reset(emusen_endymion_router *router, int32_t ports, const uint32_t *axes, size_t count);

/* The game's ports changed in number: a port taken away lets go of its buttons. */
int64_t emusen_endymion_router_resize(emusen_endymion_router *router, int32_t ports, emusen_endymion_change *out, size_t cap);

/*
 * The pads as read, then every change. `held` has a mask by PadButton for each of `players` players from player 1;
 * `axes` has, for each player in turn, the value of each axis the reset gave. `keyboard` is a mask by PadControl of
 * the keys held; `seated` a mask of the players, from bit 0 for player 1, whose pad the game may hear: seated,
 * connected or kept, and not cut off by the interface's first-controller-only setting.
 */
int64_t emusen_endymion_router_poll(emusen_endymion_router *router, const uint16_t *held, size_t players, const double *axes, size_t axes_len, uint32_t keyboard, uint32_t seated,
                                    emusen_endymion_change *out, size_t cap);

/* Every change with the pads as last read, as after a key went down or up. */
int64_t emusen_endymion_router_send(emusen_endymion_router *router, uint32_t keyboard, uint32_t seated, emusen_endymion_change *out, size_t cap);
int64_t emusen_endymion_router_take(emusen_endymion_router *router, emusen_endymion_change *out, size_t cap);

/* Whether a player's pad held a button at the last poll: 1 or 0. */
int32_t emusen_endymion_router_pad_held(emusen_endymion_router *router, int32_t player, uint32_t button);

/* Whether a port holds a controller, given the seated mask as for a poll: 1 or 0. */
int32_t emusen_endymion_router_connected(emusen_endymion_router *router, int32_t port, uint32_t seated);

/*
 * Which pad is which player: PlayerSlots's rules over the eight seats as the host holds them. A seat is empty, or
 * a pad connected or gone with SDL's GUID and the device path; a null path is no path, which is not an empty one.
 */
#define EMUSEN_ENDYMION_SEAT_EMPTY 0u
#define EMUSEN_ENDYMION_SEAT_OPEN 1u
#define EMUSEN_ENDYMION_SEAT_CLOSED 2u

typedef struct emusen_endymion_seat {
    uint32_t state;
    const uint8_t *guid;
    size_t guid_len;
    const uint8_t *path;
    size_t path_len;
} emusen_endymion_seat;

/* The seat a pad just connected takes: its own reserved one by GUID and path, else by GUID, else the lowest with no pad connected. The player it becomes, or 0. */
int32_t emusen_endymion_slots_seat(const emusen_endymion_seat *seats, const uint8_t *guid, size_t guid_len, const uint8_t *path, size_t path_len);

/*
 * A pad seated as `from` (0 for none) moved to `player` (0 for none): trading seats with whoever held it, or out of
 * every seat. 1 when the seats changed, with `out` holding them: -1 for an empty seat, 0 to 7 for the pad that was
 * in that seat, 8 for the pad moved. 0 when nothing changed. A player outside 0 to 8 is BAD_ARGUMENT.
 */
int32_t emusen_endymion_slots_move(const emusen_endymion_seat *seats, uint32_t mover_open, int32_t from, int32_t player, int32_t *out);

/* Lets go of a reservation: 1 when there was one, 0 otherwise. */
int32_t emusen_endymion_slots_forget(const emusen_endymion_seat *seats, int32_t player);

/* The highest player with a pad seated, connected or reserved; 0 for none. */
int32_t emusen_endymion_slots_highest(const emusen_endymion_seat *seats);

/* The pad's rules, PadControls, for the parity tests: an axis from its reading and the controls held as a mask by PadControl; two directions combined; a console's bindable controls. */
int32_t emusen_endymion_pad_resolve(uint32_t axis, double analog, uint32_t held, double *out);
int32_t emusen_endymion_pad_combine(double analog, uint32_t negative, uint32_t positive, double *out);
int64_t emusen_endymion_pad_controls_for(const uint32_t *buttons, size_t buttons_len, const uint32_t *axes, size_t axes_len, uint32_t *out, size_t cap);

/* ---- Endymion: the device half (EmuSen_Audio_Sync.md, section 7; EmuSen_Input.md, sections 4 and 8) ---- */

/*
 * SDL3 is the host's: it lends the handle its loader returned, once, and the library calls the same SDL through it,
 * so both share its subsystems, hints and devices. Nothing that needs SDL is made before it is lent; such a call
 * returns null, or NOT_SUPPORTED. The library never closes it.
 */
int32_t emusen_endymion_sdl_lend(void *handle);

/* A hint as the lent SDL has it; ABSENT when unset. */
int64_t emusen_endymion_sdl_hint(const uint8_t *name, size_t name_len, uint8_t *out, size_t cap);

/*
 * Simulated pads, which a test presses and plugs into a set in a real pad's place: SimulatedPad and SimulatedPads.
 * A set holds its own reference to each pad plugged in, and a pads handle its own to its set, so either may be
 * freed while the other is in use.
 */
typedef struct emusen_endymion_sim_pad emusen_endymion_sim_pad;
typedef struct emusen_endymion_sim_set emusen_endymion_sim_set;

/* A pad named "Simulated pad", with no GUID of its own, at `path` (null for none). */
emusen_endymion_sim_pad *emusen_endymion_sim_pad_new(const uint8_t *path, size_t path_len);
int32_t emusen_endymion_sim_pad_free(emusen_endymion_sim_pad *pad);

#define EMUSEN_ENDYMION_SIM_PAD_NAME 0u /* never null */
#define EMUSEN_ENDYMION_SIM_PAD_GUID 1u /* null: the MD5 of the name, as the model's */
#define EMUSEN_ENDYMION_SIM_PAD_PATH 2u

int32_t emusen_endymion_sim_pad_set_text(const emusen_endymion_sim_pad *pad, uint32_t which, const uint8_t *value, size_t len);
int64_t emusen_endymion_sim_pad_text(const emusen_endymion_sim_pad *pad, uint32_t which, uint8_t *out, size_t cap);

#define EMUSEN_ENDYMION_SIM_PAD_PLAYER_INDEX 0u /* the player number SDL was last asked to light, or -1 */
#define EMUSEN_ENDYMION_SIM_PAD_KIND 1u         /* SDL_GamepadType */

int32_t emusen_endymion_sim_pad_set(const emusen_endymion_sim_pad *pad, uint32_t which, int32_t value);
int32_t emusen_endymion_sim_pad_get(const emusen_endymion_sim_pad *pad, uint32_t which, int32_t *out);

/* Presses (`down` 1) or releases (0) an SDL_GamepadButton; any other `down` lets go of every button and axis. */
int32_t emusen_endymion_sim_pad_press(const emusen_endymion_sim_pad *pad, int32_t button, uint32_t down);
/* -1 to 1 for a stick, 0 to 1 for a trigger, kept as SDL's sixteen bits: clamped, scaled by 32767 and rounded half to even. */
int32_t emusen_endymion_sim_pad_set_axis(const emusen_endymion_sim_pad *pad, int32_t axis, double value);
int32_t emusen_endymion_sim_pad_held(const emusen_endymion_sim_pad *pad, int32_t button);
int32_t emusen_endymion_sim_pad_axis(const emusen_endymion_sim_pad *pad, int32_t axis);

emusen_endymion_sim_set *emusen_endymion_sim_set_new(void);
int32_t emusen_endymion_sim_set_free(emusen_endymion_sim_set *set);

/* Plugs a pad in: its id, from 1. Pulling one out removes it from under every id it was plugged in as. */
int64_t emusen_endymion_sim_set_connect(const emusen_endymion_sim_set *set, const emusen_endymion_sim_pad *pad);
int32_t emusen_endymion_sim_set_disconnect(const emusen_endymion_sim_set *set, const emusen_endymion_sim_pad *pad);

#define EMUSEN_ENDYMION_SIM_SET_FIRST 0u        /* the first attached pad's id, 0 for none */
#define EMUSEN_ENDYMION_SIM_SET_OPEN_HANDLES 1u
#define EMUSEN_ENDYMION_SIM_SET_OPENS 2u
#define EMUSEN_ENDYMION_SIM_SET_CLOSES 3u
#define EMUSEN_ENDYMION_SIM_SET_INITIALIZED 4u

int64_t emusen_endymion_sim_set_get(const emusen_endymion_sim_set *set, uint32_t which);
int64_t emusen_endymion_sim_set_attached(const emusen_endymion_sim_set *set, uint32_t *out, size_t cap);

/* The set as a device layer, IPadDevices, for a host's own manager: `handle` an opened pad's, `arg` an id, a button, an axis or an index. */
#define EMUSEN_ENDYMION_DEVICE_INIT 0u
#define EMUSEN_ENDYMION_DEVICE_QUIT 1u
#define EMUSEN_ENDYMION_DEVICE_OPEN 2u /* the handle, 0 for none */
#define EMUSEN_ENDYMION_DEVICE_CLOSE 3u
#define EMUSEN_ENDYMION_DEVICE_IS_ATTACHED 4u
#define EMUSEN_ENDYMION_DEVICE_CHANGED 5u /* 1 once for each batch of pads plugged or pulled */
#define EMUSEN_ENDYMION_DEVICE_UPDATE 6u
#define EMUSEN_ENDYMION_DEVICE_BUTTON 7u
#define EMUSEN_ENDYMION_DEVICE_AXIS 8u
#define EMUSEN_ENDYMION_DEVICE_KIND 9u
#define EMUSEN_ENDYMION_DEVICE_LABEL 10u
#define EMUSEN_ENDYMION_DEVICE_SET_PLAYER_INDEX 11u

int64_t emusen_endymion_sim_set_call(const emusen_endymion_sim_set *set, uint32_t op, uint64_t handle, int32_t arg);

#define EMUSEN_ENDYMION_DEVICE_NAME 0u
#define EMUSEN_ENDYMION_DEVICE_GUID 1u
#define EMUSEN_ENDYMION_DEVICE_PATH 2u

int64_t emusen_endymion_sim_set_text(const emusen_endymion_sim_set *set, uint32_t op, uint64_t handle, uint8_t *out, size_t cap);

/*
 * Every connected pad, opened, hot-plugged and let go, and read through the stick and trigger rules: GamepadManager's
 * device bookkeeping and ConnectedPad. Seating is the host's: a start, a poll or a change of devices returns the
 * pads opened and closed, in order, and the host seats each opened one and lights the players. A pad is known by a
 * key from 1, kept after it closes so that it is still known by name.
 */
typedef struct emusen_endymion_pads emusen_endymion_pads;

typedef struct emusen_endymion_pad_event {
    uint32_t kind;     /* 0 opened, 1 closed */
    uint32_t announce; /* for an opened pad: 0 for one present at start, which the host does not announce */
    uint64_t key;
} emusen_endymion_pad_event;

#define EMUSEN_ENDYMION_DEVICES_SDL 0u       /* SDL's pads; `only`, when not null, the ids it shows */
#define EMUSEN_ENDYMION_DEVICES_SIMULATED 1u /* `set`'s pads */

/* The pads on a device layer, not yet started; null when SDL is not lent or the devices are not given. */
emusen_endymion_pads *emusen_endymion_pads_new(uint32_t kind, const emusen_endymion_sim_set *set, const uint32_t *only, size_t only_len);

/* Frees the handle. One not closed first leaves its pads and devices open to the process's end, as a C# manager never disposed did. */
int32_t emusen_endymion_pads_free(emusen_endymion_pads *pads);

int64_t emusen_endymion_pads_start(const emusen_endymion_pads *pads, emusen_endymion_pad_event *out, size_t cap);
/* A poll is ABSENT while the devices have not started, when the C# poll does nothing and raises nothing. */
int64_t emusen_endymion_pads_poll(const emusen_endymion_pads *pads, emusen_endymion_pad_event *out, size_t cap);

/* Shows SDL's pads of these ids alone from the next start or poll, or all for a null `only`: a test's, so that it opens SDL's virtual pads and none on the desk. */
int32_t emusen_endymion_pads_show_only(const emusen_endymion_pads *pads, const uint32_t *only, size_t only_len);

/* The devices brought up to date; the host calls it after seating what a poll opened, as the C# poll ends. */
int32_t emusen_endymion_pads_update(const emusen_endymion_pads *pads);
int64_t emusen_endymion_pads_use(const emusen_endymion_pads *pads, uint32_t kind, const emusen_endymion_sim_set *set, const uint32_t *only, size_t only_len, emusen_endymion_pad_event *out, size_t cap);
int64_t emusen_endymion_pads_take(const emusen_endymion_pads *pads, emusen_endymion_pad_event *out, size_t cap);
int32_t emusen_endymion_pads_close_all(const emusen_endymion_pads *pads);

/* The open pads' keys, first controller first. */
int64_t emusen_endymion_pads_open(const emusen_endymion_pads *pads, uint64_t *out, size_t cap);

#define EMUSEN_ENDYMION_PADS_ID 0u
#define EMUSEN_ENDYMION_PADS_IS_OPEN 1u
#define EMUSEN_ENDYMION_PADS_KIND 2u
#define EMUSEN_ENDYMION_PADS_RAW_PRESSED 3u  /* arg: an SDL_GamepadButton */
#define EMUSEN_ENDYMION_PADS_AXIS_VALUE 4u   /* arg: an SDL_GamepadAxis; SDL's sixteen bits */
#define EMUSEN_ENDYMION_PADS_LABEL 5u        /* arg: an SDL_GamepadButton; 0 for none */
#define EMUSEN_ENDYMION_PADS_FRONTEND_COUNT 6u
#define EMUSEN_ENDYMION_PADS_STARTED 7u
#define EMUSEN_ENDYMION_PADS_ANY_PRESSED 8u  /* the first SDL button held on a pad the interface reads, -1 for none */
#define EMUSEN_ENDYMION_PADS_LAST_RESCAN 9u  /* .NET ticks since the handle was made */

int64_t emusen_endymion_pads_get(const emusen_endymion_pads *pads, uint64_t key, uint32_t which, int32_t arg);

#define EMUSEN_ENDYMION_PADS_GUID 0u
#define EMUSEN_ENDYMION_PADS_PATH 1u /* ABSENT for none */
#define EMUSEN_ENDYMION_PADS_NAME 2u

int64_t emusen_endymion_pads_text(const emusen_endymion_pads *pads, uint64_t key, uint32_t which, uint8_t *out, size_t cap);

#define EMUSEN_ENDYMION_AXIS_RAW 0u /* by SDL_GamepadAxis, -1 to 1 */
#define EMUSEN_ENDYMION_AXIS_PAD 1u /* by PadAxis, zero below the analog deadzone */

int32_t emusen_endymion_pads_axis(const emusen_endymion_pads *pads, uint64_t key, uint32_t which, uint32_t axis, double *out);

/* A PadButton as the game hears it on the pad: the stick for the d-pad, a trigger past half for L2 and R2, else `bound`, the player's SDL button, or -1 for none. */
int32_t emusen_endymion_pads_pressed(const emusen_endymion_pads *pads, uint64_t key, uint32_t button, int32_t bound);
int32_t emusen_endymion_pads_set_player_index(const emusen_endymion_pads *pads, uint64_t key, int32_t index);

#define EMUSEN_ENDYMION_SETTING_FIRST_CONTROLLER_ONLY 0u /* flags: nonzero is true */
#define EMUSEN_ENDYMION_SETTING_STICK_AS_DPAD 1u
#define EMUSEN_ENDYMION_SETTING_STICK_DEADZONE 2u
#define EMUSEN_ENDYMION_SETTING_LEFT_STICK_IS_ANALOG 3u
#define EMUSEN_ENDYMION_SETTING_ANALOG_DEADZONE 4u

int32_t emusen_endymion_pads_set_setting(const emusen_endymion_pads *pads, uint32_t which, double value);
int32_t emusen_endymion_pads_setting(const emusen_endymion_pads *pads, uint32_t which, double *out);

/* Whether a rescan is due at `now` after one at `last`, in .NET ticks: a second apart. */
int32_t emusen_endymion_pads_rescan_due(int64_t now, int64_t last);

/*
 * The game's sound to SDL's default playback device: AudioPlayer. The rate control is the host's, made by
 * emusen_endymion_rate_new and lent to each call; the player sets its target when the device opens. Disposing closes
 * the device and lets go of SDL's audio, and may be done more than once, as the C#'s Dispose may; freeing one never
 * disposed leaves its device open to the process's end.
 */
typedef struct emusen_endymion_audio emusen_endymion_audio;

emusen_endymion_audio *emusen_endymion_audio_new(emusen_endymion_rate *rate, int32_t sample_rate, int32_t target_latency_ms, int32_t buffer_frames);
int32_t emusen_endymion_audio_free(emusen_endymion_audio *audio);
int32_t emusen_endymion_audio_dispose(emusen_endymion_audio *audio);

/* Interleaved stereo at `sample_rate`; a new rate reopens the device. A null `samples` does nothing. BAD_ARGUMENT for a ratio the rate control refuses. */
int32_t emusen_endymion_audio_submit(emusen_endymion_audio *audio, emusen_endymion_rate *rate, const int16_t *samples, size_t len, int32_t sample_rate);

#define EMUSEN_ENDYMION_AUDIO_QUEUED_FRAMES 0u
#define EMUSEN_ENDYMION_AUDIO_AVAILABLE 1u
#define EMUSEN_ENDYMION_AUDIO_SAMPLE_RATE 2u

int64_t emusen_endymion_audio_get(const emusen_endymion_audio *audio, uint32_t which);
int32_t emusen_endymion_audio_set_volume(emusen_endymion_audio *audio, float volume);
float emusen_endymion_audio_volume(const emusen_endymion_audio *audio);

/* The interface's short sounds on a stream of their own, opened by the first: UiSoundPlayer. */
typedef struct emusen_endymion_ui emusen_endymion_ui;

emusen_endymion_ui *emusen_endymion_ui_new(void);
int32_t emusen_endymion_ui_free(emusen_endymion_ui *ui);
int32_t emusen_endymion_ui_dispose(emusen_endymion_ui *ui);

#define EMUSEN_ENDYMION_UI_PLAY 0u    /* replaces the sound still playing */
#define EMUSEN_ENDYMION_UI_PRELOAD 1u /* decodes and keeps it */

int32_t emusen_endymion_ui_sound(emusen_endymion_ui *ui, uint32_t which, const uint8_t *key, size_t key_len);
int32_t emusen_endymion_ui_remember(emusen_endymion_ui *ui, const uint8_t *key, size_t key_len, const uint8_t *samples, size_t samples_len);

#define EMUSEN_ENDYMION_UI_IS_OPEN 0u
#define EMUSEN_ENDYMION_UI_QUEUED 1u /* bytes */

int64_t emusen_endymion_ui_get(const emusen_endymion_ui *ui, uint32_t which);
int32_t emusen_endymion_ui_set_volume(emusen_endymion_ui *ui, float volume);
float emusen_endymion_ui_volume(const emusen_endymion_ui *ui);

/* A WAV file as 48 kHz stereo 32-bit float: ABSENT when SDL cannot read it. Bytes that do not fit wait on this thread for emusen_endymion_ui_decode_take. */
int64_t emusen_endymion_ui_decode(const uint8_t *path, size_t path_len, uint8_t *out, size_t cap);
int64_t emusen_endymion_ui_decode_take(uint8_t *out, size_t cap);

#ifdef __cplusplus
}
#endif

#endif /* EMUSEN_PLATFORM_H */
