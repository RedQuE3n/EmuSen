#include "ProbeDump.h"

#include <cstdio>
#include <cstring>
#include <fstream>

namespace
{
	// The bare filename a consumer reads out of the manifest, which must match
	// what WriteBlob produced byte for byte - hence one formatter, not two.
	std::string BlobName(const std::string& backend, const std::string& part, uint32_t frame, const char* extension)
	{
		char name[512];
		snprintf(name, sizeof(name), "%s_%s_f%05u.%s", backend.c_str(), part.c_str(), frame, extension);
		return name;
	}

	std::string BlobPath(const std::string& dir, const std::string& backend, const std::string& part,
		uint32_t frame, const char* extension)
	{
		return dir + "/" + BlobName(backend, part, frame, extension);
	}

	// The manifest is small and fixed-shape, so a JSON dependency would cost more
	// than it saves; only the ROM path can contain anything needing an escape.
	std::string JsonEscape(const std::string& text)
	{
		std::string out;
		for(char c : text) {
			if(c == '"' || c == '\\') { out += '\\'; out += c; }
			else if(c == '\n') { out += "\\n"; }
			else { out += c; }
		}
		return out;
	}
}

void ProbeDump::WriteBlob(const std::string& dir, const std::string& backend, const std::string& space,
	uint32_t frame, const void* data, size_t bytes)
{
	if(data == nullptr || bytes == 0) { return; }
	std::ofstream file(BlobPath(dir, backend, space, frame, "bin"), std::ios::binary);
	file.write((const char*)data, bytes);
}

void ProbeDump::WriteTrace(const std::string& dir, const std::string& backend, const std::string& kind,
	uint32_t frame, const char* magic, const std::vector<uint8_t>& payload, size_t recordSize)
{
	std::string path = BlobPath(dir, backend, kind, frame, "bin");
	std::ofstream file(path, std::ios::binary);
	file.write(magic, 8);
	file.write((const char*)payload.data(), payload.size());
	printf("  [%s %zu steps -> %s]\n", kind.c_str(), recordSize ? payload.size() / recordSize : 0, path.c_str());
}

void ProbeDump::WriteManifest(const std::string& dir, const std::string& backend, const std::string& system,
	const std::string& romPath, uint32_t frame, const std::vector<MemorySpace>& spaces, const ScreenView* screen)
{
	std::ofstream file(BlobPath(dir, backend, "manifest", frame, "json"));
	file << "{\n";
	file << "  \"backend\": \"" << JsonEscape(backend) << "\",\n";
	file << "  \"system\": \"" << JsonEscape(system) << "\",\n";
	file << "  \"rom\": \"" << JsonEscape(romPath) << "\",\n";
	file << "  \"frame\": " << frame << ",\n";
	file << "  \"spaces\": [";

	bool first = true;
	for(const MemorySpace& space : spaces) {
		if(space.Data == nullptr || space.Size == 0) { continue; }
		file << (first ? "\n" : ",\n");
		first = false;
		file << "    { \"name\": \"" << space.Name << "\", \"size\": " << space.Size
		     << ", \"file\": \"" << BlobName(backend, space.Name, frame, "bin") << "\" }";
	}

	file << (first ? "" : "\n  ") << "],\n";

	if(screen != nullptr && screen->Format != ScreenFormat::None) {
		file << "  \"screen\": { \"width\": " << screen->Width << ", \"height\": " << screen->Height
		     << ", \"bytes\": " << screen->Bytes << ", \"format\": \"" << ScreenFormatName(screen->Format)
		     << "\", \"file\": \"" << BlobName(backend, "screen", frame, "bin") << "\" }\n";
	} else {
		file << "  \"screen\": null\n";
	}

	file << "}\n";
}

std::string ProbeDump::HexLine(const std::string& label, const MemorySpace& space, uint32_t addr, uint32_t length)
{
	char head[256];
	snprintf(head, sizeof(head), "  %s @ $%05X:", label.c_str(), addr);

	std::string line = head;
	for(uint32_t i = 0; i < length; i++) {
		char byte[8];
		snprintf(byte, sizeof(byte), " %02X", addr + i < space.Size ? space.Data[addr + i] : 0);
		line += byte;
	}
	return line;
}

const MemorySpace* ProbeDump::Find(const std::vector<MemorySpace>& spaces, const std::string& name)
{
	for(const MemorySpace& space : spaces) {
		if(space.Name == name) { return &space; }
	}
	return nullptr;
}
