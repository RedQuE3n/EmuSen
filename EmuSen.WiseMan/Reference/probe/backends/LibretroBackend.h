// The libretro backend: one binary that drives any core exposing the libretro
// ABI, with no per-emulator code at all. See EmuSen_Debugging_Tools_Reference_v5.md §3.45.
//
// This is the breadth instrument. It reaches bsnes, snes9x, Nestopia, Gambatte,
// mGBA and a dozen others, but only through what libretro standardises: system
// RAM, save RAM, video RAM and the framebuffer. There is no OAM, no palette, no
// nametable, no CHR and no instruction trace, because the ABI has no way to ask
// for them - for those, an emulator needs a native backend of its own.
#pragma once

#include "../ProbeBackend.h"

#include <memory>
#include <string>

std::unique_ptr<IProbeBackend> CreateLibretroBackend(const std::string& corePath);
