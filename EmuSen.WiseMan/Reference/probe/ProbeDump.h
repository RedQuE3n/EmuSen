// Everything a dump set is made of, with no knowledge of which emulator made it
// - see EmuSen_Debugging_Tools_Reference_v5.md §3.45.
#pragma once

#include "ProbeBackend.h"

#include <fstream>
#include <string>
#include <vector>

namespace ProbeDump
{
	// <dir>/<backend>_<space>_f<frame>.bin - the layout every consumer already
	// globs for, which is why the Mesen backend's dumps keep their old names.
	void WriteBlob(const std::string& dir, const std::string& backend, const std::string& space,
		uint32_t frame, const void* data, size_t bytes);

	// The same path shape, plus the 8-byte magic that tells a trace parser which
	// record layout it is looking at (ESCT/ESGT/ESAW).
	void WriteTrace(const std::string& dir, const std::string& backend, const std::string& kind,
		uint32_t frame, const char* magic, const std::vector<uint8_t>& payload, size_t recordSize);

	// <backend>_manifest_f<frame>.json. Additive, and the reason a dump set from
	// an emulator this project has never seen is still readable: it names every
	// space, its size and file, and the screen's dimensions and pixel format.
	void WriteManifest(const std::string& dir, const std::string& backend, const std::string& system,
		const std::string& romPath, uint32_t frame, const std::vector<MemorySpace>& spaces,
		const ScreenView* screen, const ProbeIdentity& identity);

	uint32_t Crc32(const void* data, size_t bytes);

	// One row per frame of hashes, which is what locating a divergence needs and
	// what 900 full dumps would cost too much to give - see §3.48.
	//
	// Memory spaces only, plus a screen column the consumer must not trust unless
	// both sides declare the same pixel format: bytes are bytes everywhere, but a
	// palette index and an RGBA quad describe the same picture and never hash
	// alike. The limitation is declared in the header rather than worked around,
	// because memory divergence precedes screen divergence in every case this was
	// built for.
	class SignatureWriter
	{
	public:
		bool Open(const std::string& path);
		void WriteHeader(const std::string& backend, const std::string& system, const std::string& romPath,
			const ProbeIdentity& identity, const ScreenView* screen, const std::vector<MemorySpace>& spaces);
		void WriteRow(uint32_t frame, const std::vector<MemorySpace>& spaces, const ScreenView* screen);
		bool IsOpen() const { return _file.is_open(); }

	private:
		std::ofstream _file;
		std::vector<std::string> _columns;
	};

	// The "RAM @ $0A00: xx xx ..." report line, off a named space rather than a
	// backend-specific pointer.
	std::string HexLine(const std::string& label, const MemorySpace& space, uint32_t addr, uint32_t length);

	const MemorySpace* Find(const std::vector<MemorySpace>& spaces, const std::string& name);
}
