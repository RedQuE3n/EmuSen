// The reference probe's policy layer: arguments, the press script, the state
// anchor, the report loop and the dump format. It names no emulator - every
// machine-specific act goes through IProbeBackend.
// See EmuSen_Debugging_Tools_Reference_v5.md §3.45.
#include "ProbeBackend.h"
#include "ProbeDump.h"

// Which backends are compiled in is a build-time choice, because linking one is
// not free: the Mesen backend pulls in a 14 MB MesenCore.so, the libretro one
// needs nothing but dlopen. One binary per backend, one policy layer for all.
#ifdef PROBE_BACKEND_MESEN
#include "backends/MesenBackend.h"
#endif
#ifdef PROBE_BACKEND_LIBRETRO
#include "backends/LibretroBackend.h"
#endif

#ifndef PROBE_DEFAULT_BACKEND
#define PROBE_DEFAULT_BACKEND "mesen"
#endif

#include <cctype>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <string>
#include <thread>
#include <vector>

namespace
{
	const uint32_t NoFrame = 0xFFFFFFFF;

	void Usage()
	{
		printf("usage: probe <rom> <outDir> <startFrame> <endFrame> [stride] [anchorAddr] [traceUntilFrame] [flags]\n");
		printf("  --backend NAME   which reference emulator to drive (default: " PROBE_DEFAULT_BACKEND ")\n");
		printf("  --core PATH      libretro backend only: the *_libretro.so to drive\n");
		printf("  Writes <backend>_<space>_f<frame>.bin for every memory space the backend\n");
		printf("  exposes, plus <backend>_screen_f<frame>.bin and a manifest naming all of\n");
		printf("  them with sizes and the screen's pixel format.\n");
		printf("  --press F:BTN[:DUR]  holds BTN on port 1 from frame F for DUR frames\n");
		printf("                       (default 4), repeatable - needed to reach any scene\n");
		printf("                       behind a menu. A/B/X/Y/L/R/Up/Down/Left/Right/Start/Select\n");
		printf("  --pressuntil BTN:ADDR:VALUE[:CAP[:EVERY]]  taps BTN until the word at ADDR\n");
		printf("                       in the backend's anchor space equals VALUE (all hex).\n");
		printf("                       Anchor on a value both emulators already agree about.\n");
		printf("  --after N        report for N frames past a --pressuntil anchor, no trace\n");
		printf("  --cputrace F     record every CPU instruction from boot to frame F\n");
		printf("  --apulog F       record every $4000-$4017 write from boot to frame F\n");
		printf("  --wav PATH       record mixed audio - the only ground truth for silence\n");
		printf("  --ramstate zeros|ones|random   power-on RAM fill (default zeros)\n");
		printf("  --sig            also write <backend>_sig.csv: one row of CRCs per frame\n");
		printf("                       from frame 0, which is what locates a divergence\n");
		printf("                       rather than measuring one at the end - see §3.48\n");
	}

	std::vector<std::string> Split(const std::string& text, char separator)
	{
		std::vector<std::string> fields;
		size_t at = 0;
		while(at <= text.size()) {
			size_t next = text.find(separator, at);
			if(next == std::string::npos) { fields.push_back(text.substr(at)); break; }
			fields.push_back(text.substr(at, next - at));
			at = next + 1;
		}
		return fields;
	}

	// Reads the little-endian word the anchor watches, wrapping the way the space
	// itself does so an address past the end is a mirror rather than a crash.
	uint32_t AnchorWord(const MemorySpace& space, uint32_t addr)
	{
		if(space.Size == 0) { return 0; }
		return space.Data[(addr + 1) & (space.Size - 1)] << 8 | space.Data[addr & (space.Size - 1)];
	}
}

int main(int argc, char** argv)
{
	if(argc < 5) { Usage(); return 1; }

	std::string romPath = argv[1];
	std::string dumpDir = argv[2];
	uint32_t startFrame = (uint32_t)atoi(argv[3]);
	uint32_t endFrame = (uint32_t)atoi(argv[4]);

	// A "-" flag ends the optional positionals, so --press can follow endFrame
	// directly without being read as a stride.
	auto positional = [&](int i) { return argc > i && argv[i][0] != '-'; };
	uint32_t stride = positional(5) ? (uint32_t)atoi(argv[5]) : 1;
	uint32_t anchorAddr = positional(6) ? (uint32_t)strtoul(argv[6], nullptr, 16) : 0x0A00;
	uint32_t traceFrame = positional(7) ? (uint32_t)atoi(argv[7]) : NoFrame;

	std::string backendName = PROBE_DEFAULT_BACKEND;
	std::string corePath;
	ProbeOptions options;
	options.HomeFolder = dumpDir + "/mesenhome";

	InputSchedule schedule;
	uint32_t cpuTraceFrame = NoFrame, apuLogFrame = NoFrame, afterFrames = 0;
	bool wantSignature = false;
	bool haveUntil = false;
	ProbeButton untilButton = ProbeButton::A;
	uint32_t untilAddr = 0, untilValue = 0, untilCap = 6000, untilEvery = 40;

	for(int i = 5; i < argc; i++) {
		std::string flag = argv[i];
		bool hasValue = i + 1 < argc;

		if(flag == "--sig") { wantSignature = true; continue; }
		if(flag == "--backend" && hasValue) { backendName = argv[++i]; continue; }
		if(flag == "--core" && hasValue) { corePath = argv[++i]; continue; }
		if(flag == "--after" && hasValue) { afterFrames = (uint32_t)atoi(argv[++i]); continue; }
		if(flag == "--cputrace" && hasValue) { cpuTraceFrame = (uint32_t)atoi(argv[++i]); continue; }
		if(flag == "--apulog" && hasValue) { apuLogFrame = (uint32_t)atoi(argv[++i]); continue; }
		if(flag == "--wav" && hasValue) { options.WavPath = argv[++i]; continue; }
		if(flag == "--ramstate" && hasValue) { options.RamState = argv[++i]; continue; }

		if(flag == "--pressuntil" && hasValue) {
			std::vector<std::string> f = Split(argv[++i], ':');
			if(f.size() < 3) { printf("[ERROR] --pressuntil wants BTN:ADDR:VALUE[:CAP[:EVERY]]\n"); return 1; }
			if(!ProbeButtonFromName(f[0], untilButton)) { printf("[ERROR] unknown button '%s'\n", f[0].c_str()); return 1; }
			untilAddr = (uint32_t)strtoul(f[1].c_str(), nullptr, 16);
			untilValue = (uint32_t)strtoul(f[2].c_str(), nullptr, 16);
			if(f.size() >= 4) { untilCap = (uint32_t)atoi(f[3].c_str()); }
			if(f.size() >= 5) { untilEvery = (uint32_t)atoi(f[4].c_str()); }
			haveUntil = true;
			continue;
		}

		if(flag == "--press" && hasValue) {
			std::vector<std::string> f = Split(argv[++i], ':');
			if(f.size() < 2) { printf("[ERROR] --press wants F:BTN[:DUR]\n"); return 1; }
			ProbeButton button;
			if(!ProbeButtonFromName(f[1], button)) { printf("[ERROR] unknown button '%s'\n", f[1].c_str()); return 1; }
			uint32_t start = (uint32_t)atoi(f[0].c_str());
			uint32_t duration = f.size() >= 3 ? (uint32_t)atoi(f[2].c_str()) : 4;
			schedule.Add(start, duration, button);
			printf("[INFO] press %s frames %u..%u\n", f[1].c_str(), start, start + duration - 1);
			continue;
		}
	}

	std::unique_ptr<IProbeBackend> backend;
#ifdef PROBE_BACKEND_MESEN
	if(backendName == "mesen") { backend = CreateMesenBackend(); }
#endif
#ifdef PROBE_BACKEND_LIBRETRO
	if(backendName == "libretro") {
		if(corePath.empty()) { printf("[ERROR] --backend libretro needs --core <path to *_libretro.so>\n"); return 1; }
		backend = CreateLibretroBackend(corePath);
	}
#endif
	if(!backend) {
		printf("[ERROR] backend '%s' is not compiled into this probe\n", backendName.c_str());
		return 1;
	}

	backend->SetSchedule(schedule);

	// Armed before Load, which is what starts execution - the reset vector and
	// the whole boot sequence are the point of these two traces.
	if(cpuTraceFrame != NoFrame && !backend->BeginTrace(TraceKind::Cpu)) {
		printf("[WARN] backend '%s' has no CPU trace hook; --cputrace ignored\n", backend->Name());
		cpuTraceFrame = NoFrame;
	}
	if(apuLogFrame != NoFrame && !backend->BeginTrace(TraceKind::ApuWrites)) {
		printf("[WARN] backend '%s' has no APU write hook; --apulog ignored\n", backend->Name());
		apuLogFrame = NoFrame;
	}

	if(!backend->Load(romPath, options)) {
		printf("[ERROR] failed to load %s\n", romPath.c_str());
		return 1;
	}

	std::string prefix = backend->Name();
	std::string system = backend->System();
	ProbeIdentity identity = backend->Identity();

	printf("[INFO] backend %s, system %s\n", prefix.c_str(), system.c_str());
	printf("[INFO] identity board=%s region=%s headerTrust=%s prg=%llu chr=%llu save=%d\n",
		identity.Board.empty() ? "?" : identity.Board.c_str(),
		identity.Region.empty() ? "?" : identity.Region.c_str(),
		identity.HeaderTrust.empty() ? "?" : identity.HeaderTrust.c_str(),
		(unsigned long long)identity.PrgBytes, (unsigned long long)identity.ChrBytes, identity.SaveLoaded ? 1 : 0);
	for(const MemorySpace& space : backend->Spaces()) {
		printf("[INFO]   space %-10s %u bytes\n", space.Name.c_str(), space.Size);
	}

	ProbeDump::SignatureWriter signature;
	if(wantSignature) {
		std::string sigPath = dumpDir + "/" + prefix + "_sig.csv";
		if(!signature.Open(sigPath)) {
			printf("[WARN] could not open %s; --sig ignored\n", sigPath.c_str());
		} else {
			ScreenView view = {};
			bool have = backend->Screen(view);
			signature.WriteHeader(prefix, system, romPath, identity, have ? &view : nullptr, backend->Spaces());
			signature.WriteRow(backend->FrameCount(), backend->Spaces(), have ? &view : nullptr);
			printf("[INFO] signature stream -> %s\n", sigPath.c_str());
		}
	}

	// Every frame gets a row, because a divergence hides anywhere inside a stride
	// and the whole point of the stream is finding the first one - see §3.48.
	auto advanceTo = [&](uint32_t target) {
		if(!signature.IsOpen()) { backend->RunUntil(target); return; }
		while(backend->FrameCount() < target) {
			backend->RunUntil(backend->FrameCount() + 1);
			ScreenView view = {};
			bool have = backend->Screen(view);
			signature.WriteRow(backend->FrameCount(), backend->Spaces(), have ? &view : nullptr);
		}
	};

	// Anchors the run on game state instead of a frame count, so the same scene
	// is reached here and in the harness even though the two emulators do not
	// agree on how many frames it takes - see §3.15d and §3.40.
	if(haveUntil) {
		std::string anchorName = backend->AnchorSpace();
		const MemorySpace* anchor = ProbeDump::Find(backend->Spaces(), anchorName);
		if(anchor == nullptr) {
			printf("[ERROR] --pressuntil has no %s to watch\n", anchorName.c_str());
			return 1;
		}

		uint32_t lastPress = 0, taps = 0, frame = 0;
		bool reached = false;
		while(frame < untilCap) {
			frame = backend->FrameCount();
			if(AnchorWord(*anchor, untilAddr) == untilValue) { reached = true; break; }

			// Held four frames then released; a held button reads as one press to
			// most menus, exactly as the harness's own `tapuntil` does.
			uint32_t phase = frame - lastPress;
			if(phase >= untilEvery) { backend->SetButton(untilButton, true); lastPress = frame; taps++; }
			else if(phase >= 4) { backend->SetButton(untilButton, false); }

			advanceTo(frame + 1);
		}
		backend->SetButton(untilButton, false);

		printf(reached
			? "[PRESSUNTIL] %s $%04X reached $%04X after %u tap(s) at frame %u\n"
			: "[PRESSUNTIL] %s $%04X NOT reached ($%04X) after %u tap(s), frame %u\n",
			anchorName.c_str(), untilAddr, untilValue, taps, frame);
		fflush(stdout);
		if(!reached) { backend->Shutdown(); return 2; }

		// Report from where the anchor landed; traceUntilFrame is re-read as how
		// many frames of trace after it, and --after is the same span with none.
		startFrame = backend->FrameCount();
		endFrame = startFrame + (traceFrame != NoFrame ? traceFrame : afterFrames);
		if(traceFrame != NoFrame) { traceFrame = endFrame; }
	}

	if(traceFrame != NoFrame) { backend->BeginTrace(TraceKind::Gsu); }

	uint32_t nextReport = startFrame;
	while(true) {
		advanceTo(nextReport);
		uint32_t frame = backend->FrameCount();

		std::vector<MemorySpace> spaces = backend->Spaces();
		ScreenView screen = {};
		bool haveScreen = backend->Screen(screen);

		printf("frame %5u | %s", frame, backend->StateLine().c_str());
		if(const MemorySpace* anchor = ProbeDump::Find(spaces, backend->AnchorSpace())) {
			std::string label = anchor->Name;
			for(char& c : label) { c = (char)toupper(c); }
			printf("%s\n", ProbeDump::HexLine(label, *anchor, anchorAddr, 16).c_str());
		}

		for(const MemorySpace& space : spaces) {
			ProbeDump::WriteBlob(dumpDir, prefix, space.Name, frame, space.Data, space.Size);
		}
		if(haveScreen) { ProbeDump::WriteBlob(dumpDir, prefix, "screen", frame, screen.Data, screen.Bytes); }
		ProbeDump::WriteManifest(dumpDir, prefix, system, romPath, frame, spaces, haveScreen ? &screen : nullptr, identity);

		std::vector<uint8_t> trace;
		if(apuLogFrame != NoFrame && frame >= apuLogFrame && backend->EndTrace(TraceKind::ApuWrites, trace)) {
			ProbeDump::WriteTrace(dumpDir, prefix, "apulog", frame, "ESAW\1\0\0\0", trace, 12);
			apuLogFrame = NoFrame;
		}
		if(cpuTraceFrame != NoFrame && frame >= cpuTraceFrame && backend->EndTrace(TraceKind::Cpu, trace)) {
			ProbeDump::WriteTrace(dumpDir, prefix, "cputrace", frame, "ESCT\2\0\0\0", trace, 24);
			cpuTraceFrame = NoFrame;
		}
		if(traceFrame != NoFrame && frame >= traceFrame && backend->EndTrace(TraceKind::Gsu, trace)) {
			ProbeDump::WriteTrace(dumpDir, prefix, "gsutrace", frame, "ESGT\1\0\0\0", trace, 48);
			traceFrame = NoFrame;
		}

		fflush(stdout);
		nextReport = frame + stride;
		if(frame >= endFrame) { break; }
	}

	backend->Shutdown();
	return 0;
}
