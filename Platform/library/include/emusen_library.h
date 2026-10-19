/* emusen_library.h - the player's library, version 1.0. Owned by the library component, emusen-library.
 *
 * The library of games below every frontend: the scan of the player's ROM folders, a dump's identity, and games.db
 * with the player's records (favourites, play time, collections, metadata edits, a crop stored for the frontend, cover
 * lookups, the hash cache by which a renamed game keeps its row). It runs with no session, and no session needs it; it
 * never calls the frontend API. It is exported from the platform library beside emusen_platform.h, under its own
 * version. The normative text is EmuSen_Backend.md in the EmuSen manual, section 28; where the two disagree the page
 * wins and this header is in error.
 *
 * Names: exports emusen_library_*, types emusen_library_*, constants EMUSEN_LIBRARY_*.
 *
 * Rules every function follows (EmuSen_Backend.md sections 3.3 and 28.4):
 *  - Version. emusen_library_abi_version() is (MAJOR << 16) | MINOR. The major is 1 and is expected never to change;
 *    a minor adds functions, appends fields to the config, adds values to open enumerations (a get's "what", a
 *    change's "op") and keys to JSON. Nothing is removed. A caller checks the major before calling anything else.
 *  - Pull, never push. The library calls nothing of the client's; a scan's progress is drained.
 *  - Statuses. A negative return is a status; zero or more is success: a length or a count, as each function says.
 *    The words for a failure are read with emusen_library_last_error on the same thread.
 *  - Text is UTF-8, passed as a pointer and a length in bytes and never assumed to end in a NUL.
 *  - Copy out. A function that fills a buffer takes (out, out_len) and returns the whole length; it writes only when
 *    the buffer holds all of it, and a null out asks the length. An answer is made once: one that did not fit is kept
 *    on the calling thread, and the next call of the same function with the same request takes it rather than reading
 *    or changing anything again.
 *  - Shapes. Requests and answers are JSON objects, keys in snake_case; a caller skips keys it does not know. A time
 *    is .NET's round-trip text of a local time, "2026-10-18T14:03:11.2510000+02:00"; a change that records one takes
 *    it as "at", and the library's clock gives it in the same form when "at" is left out. A double JSON has no number
 *    for crosses as the string "NaN", "Infinity" or "-Infinity".
 *  - Threads. Any function may be called from any thread. The handle serialises calls on its database, except
 *    emusen_library_progress and emusen_library_cancel, which never wait. A scan and a reclaim read files without
 *    holding the database, so reads are answered while one runs; a second while one runs is EMUSEN_LIBRARY_BUSY.
 *  - SQLite. The host lends the SQLite it has loaded, so one process holds one copy and its locks on a file are one
 *    set (EmuSen_RustPlatform.md section 2.8 and Q25). The library never loads SQLite itself.
 *  - Nothing unwinds.
 */
#ifndef EMUSEN_LIBRARY_H
#define EMUSEN_LIBRARY_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define EMUSEN_LIBRARY_MAJOR 1u
#define EMUSEN_LIBRARY_MINOR 0u
#define EMUSEN_LIBRARY_VERSION ((EMUSEN_LIBRARY_MAJOR << 16) | EMUSEN_LIBRARY_MINOR)

/* ---- Statuses: the band -1140 to -1159 (EmuSen_Backend.md D12), beside the platform's shared codes ------------- */
#define EMUSEN_LIBRARY_NULL          (-1)    /* a null pointer where one was required */
#define EMUSEN_LIBRARY_BAD_STRING    (-5)    /* text that is not UTF-8 */
#define EMUSEN_LIBRARY_NOT_SUPPORTED (-256)  /* a function this build does not have */
#define EMUSEN_LIBRARY_NEWER_FILE    (-1140) /* games.db is of a newer schema; refused, nothing written */
#define EMUSEN_LIBRARY_DATABASE      (-1141) /* SQLite refused: busy past its wait, a damaged file, a full disc */
#define EMUSEN_LIBRARY_NO_SQLITE     (-1142) /* no SQLite was lent, or the one lent lacks a function */
#define EMUSEN_LIBRARY_BAD_REQUEST   (-1143) /* a request that is not what this header says */
#define EMUSEN_LIBRARY_CANCELLED     (-1144) /* a scan or a reclaim was cancelled */
#define EMUSEN_LIBRARY_BUSY          (-1145) /* a scan or a reclaim is already running on the handle */
#define EMUSEN_LIBRARY_IO            (-1146) /* a file the call had to read could not be read */

/* A handle: one games.db, its last scan, and what a running call has done. */
typedef struct emusen_library emusen_library;

/* What emusen_library_open is given. size is sizeof as the caller knows it; fields are only appended. */
typedef struct emusen_library_config {
    uint32_t size;
    const uint8_t *database;  /* games.db's path; its folder is made when absent */
    size_t database_len;
    void *sqlite;             /* the platform loader's handle of the SQLite the host has loaded, lent for the handle's life */
} emusen_library_config;

/* (MAJOR << 16) | MINOR. */
uint32_t emusen_library_abi_version(void);

/* The words of the last failing call on this thread. */
int64_t emusen_library_last_error(uint8_t *out, size_t out_len);

/* Opens games.db, migrating it to this build's schema in one transaction a version. 0, the handle in *out. */
int32_t emusen_library_open(const emusen_library_config *config, emusen_library **out);

/* Closes a handle; nothing may be running on it. */
int32_t emusen_library_close(emusen_library *lib);

/* Scans and keeps the result in the handle; the games found. Request:
 *   {"roots": ["/home/p/Roms"], "systems": ["snes", "nes", "gb", "gbc", ...]}
 * systems are the systems this build opens, the system of each of the frontend API's shelves. A file is listed when
 * its extension is one a named system takes, or an archive's; a sheet's tracks and a list's discs are their game's.
 * Each is routed among the named systems taking its image's extension, by content and then by its folder's ES-DE
 * name; hidden entries are skipped and unreadable folders passed over; games are ordered by title ignoring case. */
int64_t emusen_library_scan(emusen_library *lib, const uint8_t *request, size_t len);

/* {"running": true, "phase": "walk" | "route" | "hash" | "", "files": n, "games": n, "hashed": n, "to_hash": n} */
int64_t emusen_library_progress(emusen_library *lib, uint8_t *out, size_t out_len);

/* Stops a running scan or reclaim at its next file; it returns EMUSEN_LIBRARY_CANCELLED and the last scan stays. */
int32_t emusen_library_cancel(emusen_library *lib);

/* The last scan, narrowed by an optional filter, {"system": "gbc"}; an empty filter is none. Answer:
 *   {"status": "ok" | "empty" | "not_found" | "no_directory", "roots": [...], "games": [{"path": "...", "system": "gbc" | null}]} */
int64_t emusen_library_games(emusen_library *lib, const uint8_t *filter, size_t len, uint8_t *out, size_t out_len);

/* What a dump is. Request {"path": "...", "systems": [...] (else the last scan's), "crc32": false, "record": false}.
 * Answer {"path", "bytes", "modified" (.NET's LastWriteTimeUtc.Ticks), "md5", "crc32" | null, "system" | null,
 * "cached"}. The MD5 comes from the hash cache while the file is the size and age it was hashed at; with crc32 the
 * file is read; with record the cache and the game's row are written. */
int64_t emusen_library_identity(emusen_library *lib, const uint8_t *request, size_t len, uint8_t *out, size_t out_len);

/* One read, {"what": ...}:
 *   record {path}            {"favourite", "last_played", "play_count", "play_seconds"} or null
 *   records                  every row so, by path
 *   game {path}              the record with "md5", "bytes", "crop", "edits", "collections", or null
 *   edits {path}             {field: value}
 *   all_edits                by path
 *   collections              [{"id", "name", "count"}] by name ignoring case
 *   members {id}             paths
 *   collections_of {path}    ids
 *   crop {path}              {"left", "right", "top", "bottom"} or null
 *   cover_lookup {path}      "Found", "Unknown", "NoArt" or null
 *   known_hash {path, bytes, modified}   the MD5 or null
 *   path_by_hash {md5}       a path or null
 *   orphans {present}        [{"path", "md5", "bytes"}]: rows with a hash whose path is not present
 *   recent {limit, games}    of the games given (else the last scan's), those with a last-played time, newest first */
int64_t emusen_library_get(emusen_library *lib, const uint8_t *request, size_t len, uint8_t *out, size_t out_len);

/* One change, {"op": ...}; the answer is null unless said:
 *   toggle_favourite {path}; set_favourite {path, favourite}; started {path, at}; played {path, seconds, max_hours}
 *   (the seconds recorded, or null where tracking is off or the launch ran past max_hours; 24 or more is no limit);
 *   set_play_stats {path, count, seconds}; set_crop {path, left, right, top, bottom}; clear_crop {path};
 *   save_edits {path, changes: {field: value | null}, at}; clear_edits {path} (the cover choice and the television
 *   standard kept); create_collection {name, at} (the id, or null for a name taken in any case);
 *   rename_collection {id, name} (true or false); delete_collection {id}; add_to_collection {id, path};
 *   remove_from_collection {id, path}; store_hash {path, bytes, modified, md5}; identify {path, md5, bytes};
 *   move {from, to} (a renamed game's row, collections, edits, cover lookup and the covers others borrow from it
 *   follow it, merged into what the new path had); record_cover_lookup {path, outcome, rom_name, at} (a "Failed"
 *   outcome is not recorded); forget_cover_lookup {path};
 *   reclaim {games}: the rows whose file is not among the games given (else the last scan's), each moved to a game of
 *   its size and contents that has no row, answering {"moved": n}; it runs as a scan does, with progress and cancel. */
int64_t emusen_library_change(emusen_library *lib, const uint8_t *request, size_t len, uint8_t *out, size_t out_len);

#ifdef __cplusplus
}
#endif

#endif
