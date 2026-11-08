/*
 * emusen_games_service.h - the emusen.games service, version 1.0: the player's library as a DianaSh command module
 * reaches it. A module links neither the library nor Galaxia, so the platform library hands every shell it makes this
 * table beside emusen.scrape's, and the shell copies it into each call unread. The table speaks for the process's
 * games.db: the one a front end named at emusen_scraping_open (emusen_platform.h), shared with the scraper, else the
 * tree's, opened on first use. A front end keeps emusen_library.h for its own handle; the two connections on one file
 * are safe, as EmuSen_RustPlatform.md, section 47.25 shows. Until 2.0 these calls were emusen.scrape's library_get and
 * games (EmuSen_LibraryTools.md, section 8).
 *
 * get is emusen_library.h's emusen_library_get on that games.db, with its requests and answers. games is the
 * settings' ROM folder, with Packages and Compressed beside it, scanned for the systems the front end named at open
 * (else every system of the table) on a database of its own: emusen_library_games's answer for the filter given, with
 * "esde", each game's ES-DE system or null, added. Statuses are emusen_library.h's; text is UTF-8 with a length,
 * answers come by the length-query idiom, and a failure's words are read with last_error on the same thread.
 */
#ifndef EMUSEN_GAMES_SERVICE_H
#define EMUSEN_GAMES_SERVICE_H

#include <stddef.h>
#include <stdint.h>

#define EMUSEN_GAMES_SERVICE "emusen.games"
#define EMUSEN_GAMES_SERVICE_MAJOR 1u
#define EMUSEN_GAMES_SERVICE_MINOR 0u

typedef struct {
    uint32_t size; /* sizeof this table, so that a later minor may add calls at its end */
    int64_t (*get)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*games)(const uint8_t *filter, size_t len, uint8_t *out, size_t out_len);
    int64_t (*last_error)(uint8_t *out, size_t len);
} emusen_games_service_v1;

#endif
