// The Mesen2 backend: the only file in the probe that includes a Mesen header.
// See EmuSen_Debugging_Tools_Reference_v5.md §3.45.
#pragma once

#include "../ProbeBackend.h"

#include <memory>

// Built only when the probe is compiled against a Mesen2 checkout; the factory
// returns null otherwise, so the policy layer can report a missing backend
// rather than failing to link.
std::unique_ptr<IProbeBackend> CreateMesenBackend();
