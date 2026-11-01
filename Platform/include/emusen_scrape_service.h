/*
 * emusen_scrape_service.h - the emusen.scrape service, version 1.0: the process's one scraper as a DianaSh command
 * module reaches it. A module links neither the scraper nor Galaxia, so the platform library hands every shell it makes
 * this table, which the shell copies into each call unread. The table's calls take no handle: each speaks for the
 * scraper the process holds, which is the host's while it lends one with emusen_scraping_lend (emusen_platform.h), and
 * otherwise one the platform library opens on first use from the settings file. After any lend the platform library
 * never opens one of its own, so a process has one scraper and one quota.
 *
 * get, change, start, progress, wait, pause and cancel are emusen_scrape.h's calls of the same names, with its requests,
 * answers and statuses. library_get is emusen_library.h's emusen_library_get on the scraper's library. games is the
 * settings' ROM folder scanned for the systems the build opens, emusen_library_games's answer with "esde", each game's
 * ES-DE system or null, added. theme_kinds answers a system's ES-DE name with the media types the host's active theme
 * draws for it, a JSON list, or EMUSEN_SCRAPE_NOT_SUPPORTED where the host loads no themes. Every call keeps
 * emusen_scrape.h's rules: UTF-8 with a length, the length-query idiom, words redacted and read with last_error on the
 * same thread. See EmuSen_LibraryTools.md, section 8.
 */
#ifndef EMUSEN_SCRAPE_SERVICE_H
#define EMUSEN_SCRAPE_SERVICE_H

#include <stddef.h>
#include <stdint.h>

#define EMUSEN_SCRAPE_SERVICE "emusen.scrape"
#define EMUSEN_SCRAPE_SERVICE_MAJOR 1u
#define EMUSEN_SCRAPE_SERVICE_MINOR 0u

typedef struct {
    uint32_t size; /* sizeof this table, so that a later minor may add calls at its end */
    int64_t (*get)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*change)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*start)(const uint8_t *request, size_t len);
    int64_t (*progress)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*wait)(uint64_t serial, uint32_t timeout_ms); /* at most 250 ms, so that a withdrawal is not held */
    int32_t (*pause)(int32_t paused);
    int32_t (*cancel)(void);
    int64_t (*library_get)(const uint8_t *request, size_t len, uint8_t *out, size_t out_len);
    int64_t (*games)(const uint8_t *filter, size_t len, uint8_t *out, size_t out_len);
    int64_t (*theme_kinds)(const uint8_t *system, size_t len, uint8_t *out, size_t out_len);
    int64_t (*last_error)(uint8_t *out, size_t len);
} emusen_scrape_service_v1;

#endif
