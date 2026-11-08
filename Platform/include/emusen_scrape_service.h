/*
 * emusen_scrape_service.h - the emusen.scrape service, version 2.0: scraping's one surface. The table speaks for the
 * process's one scraper, which the platform library holds: the ScreenScraper client and its quota, the queue's run on
 * the player's word alone, OpenEmu's sources as the failover, the media store and media.db, and the order in which a
 * game's pictures are found. A front end gets the table from emusen_scraping_service() and opens the scraper with
 * emusen_scraping_open (emusen_platform.h); the platform library hands the same table to every shell it makes, which
 * copies it into each call unread, since a DianaSh module links neither the scraper nor Galaxia. Where no front end
 * opened one, the platform library opens one on first use from the settings file; after any open or close it never
 * does, so a process has one scraper and one quota. Since 2.0 the library's reads are the emusen.games service's
 * (emusen_games_service.h), and the handle API of emusen_scrape.h is gone (EmuSen_RustPlatform.md, section 47.25).
 *
 * Rules every call keeps (EmuSen_LibraryTools.md, section 8):
 *  - Pull, never push: nothing calls the host back except a theme loader it named at open; a run's progress is drained.
 *  - Length-query idiom: a null out asks the length; a short buffer is written nothing, and the answer, made once,
 *    waits on the calling thread for the same call with the same request, which then copies it.
 *  - Signed returns: zero or more is success (a count or a length); a negative is a status below, its words read with
 *    last_error on the same thread. Every word has passed through the scraper's redactor: no credential.
 *  - Text is UTF-8 as pointer and length. Requests and answers are JSON. Any call from any thread.
 *
 * get {"what": ...}: kinds; record {path}; game {md5, bytes} (or {fingerprint, bytes}); found; outcomes; stored {md5,
 * bytes}; queue; orphans {games}; moves; quota; account (never a password); developer; folder {path}; plan {games, one,
 * refresh, kinds, enabled}; media {path, system, type, folder, folder_path}; present {system, games}; stamp {system,
 * choice}; metadata {path, scraped_names}; openvgdb; now; delays.
 * change {"op": ...}: settings {...}; forget {path, system, folder}; forget_unfound {path}; clean_up {relatives, at};
 * sign_out; sign_in {user, password}; check; search {path, system, text}; art_added {console, stem, path}; playing
 * {running}; openvgdb_download; openvgdb_remove. A change that asks the server blocks for that one request.
 * start {"games": [{"path", "system"}], "priority", "one", "resume", "refresh", "pick", "enabled", "choices"}: the
 * games queued; a run already going is EMUSEN_SCRAPE_BUSY, and a scraper opened for its sources alone refuses every
 * run with EMUSEN_SCRAPE_BAD_REQUEST.
 * progress {"since": n}: the run as it stands, its results after serial n. wait: until the run's serial passes serial,
 * at most 250 ms so that a close is not held; the serial then. pause: 1 when it changed. cancel: 1 when a run was going.
 * theme_kinds: a system's ES-DE name answered with the media types the front end's active theme draws, a JSON list, or
 * EMUSEN_SCRAPE_NOT_SUPPORTED where none was named at open. The members of each request and answer are
 * EmuSen_Backend.md, section 30.3's.
 */
#ifndef EMUSEN_SCRAPE_SERVICE_H
#define EMUSEN_SCRAPE_SERVICE_H

#include <stddef.h>
#include <stdint.h>

#define EMUSEN_SCRAPE_SERVICE "emusen.scrape"
#define EMUSEN_SCRAPE_SERVICE_MAJOR 2u
#define EMUSEN_SCRAPE_SERVICE_MINOR 0u

#define EMUSEN_SCRAPE_NULL (-1)
#define EMUSEN_SCRAPE_BAD_STRING (-5)
#define EMUSEN_SCRAPE_NOT_SUPPORTED (-256)
#define EMUSEN_SCRAPE_NEWER_FILE (-1160)   /* media.db is of a newer schema; refused, nothing written */
#define EMUSEN_SCRAPE_DATABASE (-1161)     /* SQLite refused; the words are SQLite's */
#define EMUSEN_SCRAPE_NO_SQLITE (-1162)    /* no SQLite could be loaded */
#define EMUSEN_SCRAPE_BAD_REQUEST (-1163)  /* not JSON, an unknown what or op, a field missing */
#define EMUSEN_SCRAPE_BUSY (-1164)         /* a run, or OpenVGDB's download, already going; or no scraper open now */
#define EMUSEN_SCRAPE_IO (-1165)           /* a file of the store or a credential file could not be read or written */
#define EMUSEN_SCRAPE_NO_LIBRARY (-1166)   /* a read that needs games.db, with none named at open */
#define EMUSEN_SCRAPE_STORE_IN_USE (-1167) /* another process holds the store */

typedef struct {
    uint32_t size; /* sizeof this table, so that a later minor may add calls at its end */
    int64_t (*get)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*change)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*start)(const uint8_t *request, size_t len);
    int64_t (*progress)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*wait)(uint64_t serial, uint32_t timeout_ms);
    int32_t (*pause)(int32_t paused);
    int32_t (*cancel)(void);
    int64_t (*theme_kinds)(const uint8_t *system, size_t len, uint8_t *out, size_t out_len);
    int64_t (*last_error)(uint8_t *out, size_t len);
} emusen_scrape_service_v2;

#endif
