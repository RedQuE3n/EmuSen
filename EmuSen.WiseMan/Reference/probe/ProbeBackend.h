// The contract every reference emulator implements - see EmuSen_Debugging_Tools_Reference_v5.md §3.45.
//
// Nothing above this line includes an emulator header. A backend is the only
// place that knows what "Mesen" or "libretro" means; the policy layer above
// (ProbeMain.cpp) parses arguments, schedules presses, anchors on state and
// writes dumps without ever naming one.
#pragma once

#include <cstdint>
#include <string>
#include <vector>

enum class ProbeButton { A, B, X, Y, L, R, Up, Down, Left, Right, Start, Select, Count };

// How to read the bytes a backend hands back for the screen. A dump is always
// raw - what the emulator actually had - and the manifest records which of
// these it is, so a consumer never has to guess from the file size.
enum class ScreenFormat { None, PaletteIndex16, Bgr555, Rgb565, Xrgb8888 };

enum class TraceKind { Cpu, Gsu, ApuWrites };

// One named block of emulator memory. Name is lowercase and becomes the middle
// field of <backend>_<name>_f<frame>.bin, which is why it is a string rather
// than an enum: a backend may expose spaces this tool has never heard of.
struct MemorySpace
{
	std::string Name;
	const uint8_t* Data;
	uint32_t Size;
};

struct ScreenView
{
	const void* Data;
	uint32_t Width;
	uint32_t Height;
	uint32_t Bytes;
	ScreenFormat Format;
};

// What the backend decided the machine *is*, as opposed to what it did. A
// differential run compares this before it compares a single pixel, because two
// emulators can resolve the same file to different boards and neither will say
// so - see EmuSen_Debugging_Tools_Reference_v5.md §3.48.
//
// Every field may be left empty. "Unknown" is a real answer here and a useful
// one: a libretro core cannot report its mapper, and a gate that treats silence
// as agreement is worse than one that reports reduced confidence.
struct ProbeIdentity
{
	std::string Board;
	std::string Region;
	std::string HeaderTrust;
	uint64_t PrgBytes = 0;
	uint64_t ChrBytes = 0;
	bool SaveLoaded = false;
};

struct ProbeOptions
{
	std::string RamState = "zeros";
	std::string WavPath;
	// Kept beside the dumps, so a probe run never touches a real emulator profile
	// and its .srm never silently changes what the ROM boots into.
	std::string HomeFolder;
};

// A --press script, in frames and buttons and nothing else. Parsing it is policy;
// *sampling* it has to be the backend's job, because only the backend knows when
// its emulator polls the pad - and a probe that free-runs between reports cannot
// stop precisely enough to set the pad itself without dropping presses.
struct InputSchedule
{
	struct Entry { uint32_t StartFrame; uint32_t EndFrame; ProbeButton Button; };
	std::vector<Entry> Entries;

	void Add(uint32_t start, uint32_t duration, ProbeButton button)
	{
		Entries.push_back({ start, start + duration, button });
	}

	bool HeldAt(uint32_t frame, ProbeButton button) const
	{
		for(const Entry& e : Entries) {
			if(e.Button == button && frame >= e.StartFrame && frame < e.EndFrame) { return true; }
		}
		return false;
	}
};

class IProbeBackend
{
public:
	virtual ~IProbeBackend() = default;

	// Becomes the dump prefix, so an existing Mesen dump set keeps its filenames.
	virtual const char* Name() const = 0;
	// Which machine the loaded ROM turned out to be: "nes", "snes", ...
	virtual const char* System() const = 0;

	// What this backend resolved the cartridge to be. Empty fields are honest.
	virtual ProbeIdentity Identity() { return {}; }

	virtual bool Load(const std::string& romPath, const ProbeOptions& options) = 0;
	virtual void Shutdown() {}

	// Advances to at least `frame`, then stops on a frame boundary. Every other
	// call here is only valid while stopped, and that is the whole point: the
	// old probe raced a free-running emulator, which is why --pressuntil needed
	// an atomic live key and why §3.39 warns about cached frame counters.
	virtual void RunUntil(uint32_t frame) = 0;
	virtual uint32_t FrameCount() = 0;

	virtual std::vector<MemorySpace> Spaces() = 0;
	// Which space --pressuntil watches when none is named: GSU RAM on a SNES,
	// internal RAM on an NES. Empty means this backend cannot be anchored.
	virtual const char* AnchorSpace() const = 0;

	// The frame-indexed script, fixed before boot. Sampled by the backend.
	virtual void SetSchedule(const InputSchedule& schedule) { (void)schedule; }
	// The live override --pressuntil decides at runtime, on top of the schedule.
	virtual void SetButton(ProbeButton button, bool held) = 0;
	virtual bool Screen(ScreenView& out) { (void)out; return false; }

	// The emulator-internal register decode for one report. Anything derivable
	// from Spaces() belongs to the policy layer instead, not here.
	virtual std::string StateLine() { return {}; }

	// Optional by nature: a per-instruction trace cannot be had from an
	// arbitrary emulator without patching it, so a backend that has no such
	// hook says so instead of silently producing an empty file. Arming is
	// valid before Load, which is what the CPU and APU traces need.
	virtual bool BeginTrace(TraceKind kind) { (void)kind; return false; }
	virtual bool EndTrace(TraceKind kind, std::vector<uint8_t>& out) { (void)kind; (void)out; return false; }
};

// Inline because they are pure tables, and a whole translation unit for two
// lookups is not worth the build rule.
inline const char* ProbeButtonName(ProbeButton button)
{
	static const char* names[] = { "A", "B", "X", "Y", "L", "R", "Up", "Down", "Left", "Right", "Start", "Select" };
	return button < ProbeButton::Count ? names[(int)button] : "?";
}

inline bool ProbeButtonFromName(const std::string& name, ProbeButton& out)
{
	for(int i = 0; i < (int)ProbeButton::Count; i++) {
		if(name == ProbeButtonName((ProbeButton)i)) { out = (ProbeButton)i; return true; }
	}
	return false;
}

inline const char* ScreenFormatName(ScreenFormat format)
{
	switch(format) {
		case ScreenFormat::PaletteIndex16: return "PaletteIndex16";
		case ScreenFormat::Bgr555: return "Bgr555";
		case ScreenFormat::Rgb565: return "Rgb565";
		case ScreenFormat::Xrgb8888: return "Xrgb8888";
		default: return "None";
	}
}
