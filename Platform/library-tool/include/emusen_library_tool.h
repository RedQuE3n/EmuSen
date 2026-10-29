/*
 * emusen_library_tool.h - the emusen.library service, version 1.4: the library tool's jobs as a DianaSh command module
 * reaches them. A module never links Galaxia (EmuSen_DianaSh.md, section 16, Q11), so the tool, which reads Galaxia's
 * tree and settings, is lent to it as this table, which the platform library hands every shell it makes; the shell
 * copies it into each call unread. The calls are the platform library's own emusen_tool_job_* (emusen_platform.h),
 * whose rules they keep: text is UTF-8 with a length, results are copied by the length-query idiom, statuses are the
 * platform library's, and the words for a failure are read with last_error on the same thread. Requests and events
 * are EmuSen_GamePackages.md, section 12.3's; since 1.1 the table is unchanged and a request may also be a pkg one,
 * whose ops and package events are section 14's; since 1.2 also pkg-patch-add and pkg-patch-rm, section 15's; since
 * 1.3 pkg-make's self_contained and zip, a package's zip among pkg-check's paths, pkg-identity and pkg-zip, section
 * 19's; since 1.4 a request's output (the folder CHDs go to) and convert's beside (each CHD beside its original in the
 * library), section 20's.
 */
#ifndef EMUSEN_LIBRARY_TOOL_H
#define EMUSEN_LIBRARY_TOOL_H

#include <stddef.h>
#include <stdint.h>

#define EMUSEN_LIBRARY_TOOL_SERVICE "emusen.library"
#define EMUSEN_LIBRARY_TOOL_MAJOR 1u
#define EMUSEN_LIBRARY_TOOL_MINOR 4u

typedef struct emusen_tool_job emusen_tool_job;

typedef struct {
    uint32_t size; /* sizeof this table, so that a later minor may add calls at its end */
    int32_t (*start)(const uint8_t *request, size_t len, emusen_tool_job **job);
    int64_t (*next)(emusen_tool_job *job, uint32_t wait_ms, uint8_t *out, size_t len); /* ABSENT: none yet; 0: ended */
    int32_t (*stop)(emusen_tool_job *job);
    int32_t (*free)(emusen_tool_job *job);
    int64_t (*last_error)(uint8_t *out, size_t len);
} emusen_library_tool_v1;

#endif
