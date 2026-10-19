/* emusen_scrape.h - scraping, version 1.0. Owned by the scraping component, emusen-scrape.
 *
 * Scraping below every frontend: the ScreenScraper client and its quota, the queue's run on the player's word alone,
 * OpenEmu's sources as the failover, the media store and media.db, and the order in which a game's pictures are
 * found. It runs with no session and calls no frontend; it is exported from the platform library beside
 * emusen_platform.h, under its own version. The normative text is EmuSen_Backend.md in the EmuSen manual, section 30;
 * where the two disagree the page wins and this header is in error.
 *
 * Names: exports emusen_scrape_*, types emusen_scrape_*, constants EMUSEN_SCRAPE_*.
 *
 * Rules every function follows (EmuSen_Backend.md sections 3.3 and 30.3):
 *  - Version. emusen_scrape_abi_version() is (MAJOR << 16) | MINOR. The major is 1 and is expected never to change;
 *    a minor adds functions, appends fields to the config, adds values to open enumerations (a get's "what", a
 *    change's "op") and keys to JSON. Nothing is removed. A caller checks the major before calling anything else.
 *  - Pull, never push: nothing calls the host back; a run's progress is drained.
 *  - Length-query idiom: a null out asks the length; a short buffer is written nothing, and the answer, made once,
 *    waits on the calling thread for the same call with the same request, which then copies it.
 *  - Signed returns: zero or more is success (a count or a length); a negative is a status below, with the words in
 *    emusen_scrape_last_error. Every word this library gives has passed through its redactor: no credential.
 *  - Text is UTF-8 as pointer and length. Requests and answers are JSON.
 *  - Threads: any call from any thread; a handle serialises calls on media.db.
 */
#ifndef EMUSEN_SCRAPE_H
#define EMUSEN_SCRAPE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define EMUSEN_SCRAPE_MAJOR 1u
#define EMUSEN_SCRAPE_MINOR 0u
#define EMUSEN_SCRAPE_VERSION ((EMUSEN_SCRAPE_MAJOR << 16) | EMUSEN_SCRAPE_MINOR)

#define EMUSEN_SCRAPE_NULL (-1)
#define EMUSEN_SCRAPE_BAD_STRING (-5)
#define EMUSEN_SCRAPE_NOT_SUPPORTED (-256)
#define EMUSEN_SCRAPE_NEWER_FILE (-1160)  /* media.db is of a newer schema; refused, nothing written */
#define EMUSEN_SCRAPE_DATABASE (-1161)    /* SQLite refused; the words are SQLite's */
#define EMUSEN_SCRAPE_NO_SQLITE (-1162)   /* no SQLite was lent */
#define EMUSEN_SCRAPE_BAD_REQUEST (-1163) /* not JSON, an unknown what or op, a field missing */
#define EMUSEN_SCRAPE_BUSY (-1164)        /* a run, or OpenVGDB's download, already going */
#define EMUSEN_SCRAPE_IO (-1165)          /* a file of the store or a credential file could not be read or written */
#define EMUSEN_SCRAPE_NO_LIBRARY (-1166)  /* a read that needs the lent library, with none lent */

typedef struct emusen_scrape emusen_scrape;
struct emusen_library;

/* Settings: {"roms", "art", "esde_media", "openemu", "user_agent", "developer_files", "test": {"connect", "roots",
 * "clock", "offline"}}, every member optional; "test" is for tests alone, reached by no setting, variable or command. */
typedef struct emusen_scrape_config {
    uint32_t size;                           /* sizeof, so fields may be appended */
    const uint8_t *root; size_t root_len;    /* the media store; media.db in it */
    void *sqlite;                            /* the host's loaded e_sqlite3, lent for the handle's life */
    const struct emusen_library *library;    /* lent, may be null: a cover chosen from another game, edits, cover lookups */
    const uint8_t *embedded; size_t embedded_len; /* the program's embedded developer blob, or none */
    const uint8_t *settings; size_t settings_len;
} emusen_scrape_config;

/* (MAJOR << 16) | MINOR. */
uint32_t emusen_scrape_abi_version(void);

/* The words of this thread's last failing call, redacted. */
int64_t emusen_scrape_last_error(uint8_t *out, size_t len);

/* Opens the store at root, migrating media.db; a file of a newer schema is refused unwritten. */
int32_t emusen_scrape_open(const emusen_scrape_config *config, emusen_scrape **out);

/* Closes a handle: a run is cancelled and its workers joined, then media.db is closed. */
int32_t emusen_scrape_close(emusen_scrape *s);

/* One read, {"what": ...}:
 *   kinds                    [{"type", "folder", "screenscraper": [...], "payload"}], ES-DE's eleven kinds
 *   record {path}            the found record of the path's game, or null
 *   game {md5, bytes}        the record by its hashes, of any state, or null
 *   found                    every found record by path
 *   outcomes                 {path: {"state", "has_cover"}}
 *   stored {md5, bytes}      [{"type", "path", "region", "sha1"}]
 *   queue                    [{"path", "system", "priority", "attempts"}]
 *   orphans {games}          the store's files whose game is not among games [{"path", "system"}], under their systems
 *   moves                    [{"from", "to", "how"}], the layout step's record
 *   quota                    the quota's snapshot
 *   account                  {"signed_in", "name", "verified"}, never a password
 *   developer                {"origin": "tree" | "user" | "embedded" | null}
 *   folder {path}            a ROM's game folder below its console's */
int64_t emusen_scrape_get(emusen_scrape *s, const uint8_t *request, size_t len, uint8_t *out, size_t out_len);

/* One change, {"op": ...}:
 *   settings {...}           the settings' members laid over what was set; a stop for the credentials is lifted
 *   forget {path, system, folder}  a game's answer and pictures, from the store only; the files deleted
 *   forget_unfound {path}    an Unknown or Error answer of the path's game forgotten
 *   clean_up {relatives, at} each file moved to CLEANUP/<local date_time>/: {"moved", "bytes", "folder"}
 *   sign_out                 the member's file deleted: {"had"} */
int64_t emusen_scrape_change(emusen_scrape *s, const uint8_t *request, size_t len, uint8_t *out, size_t out_len);

#ifdef __cplusplus
}
#endif

#endif
