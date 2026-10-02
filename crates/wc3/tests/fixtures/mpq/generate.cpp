#include "StormLib.h"
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
static void check(bool ok) { if(!ok) { fprintf(stderr, "StormLib error %u\n", SErrGetLastError()); exit(1); } }
int main(int argc, char **argv) {
    if(argc == 3 && std::string(argv[1]) == "verify") {
        HANDLE archive, file; check(SFileOpenArchive(argv[2], 0, MPQ_OPEN_READ_ONLY, &archive));
        check(SFileOpenFileEx(archive, "Units\\Encrypted.bin", 0, &file));
        std::vector<unsigned char> data(17003); DWORD count;
        check(SFileReadFile(file, data.data(), data.size(), &count, nullptr));
        if(count != data.size()) return 2;
        for(size_t i=0; i<data.size(); i++) if(data[i] != i%7) return 3;
        SFileCloseFile(file);
        if(SFileVerifyFile(archive, "Units\\Encrypted.bin", SFILE_VERIFY_RAW_MD5) & VERIFY_FILE_ERROR_MASK) return 4;
        if(SFileVerifyRawData(archive, SFILE_VERIFY_HET_TABLE, nullptr)) return 5;
        if(SFileVerifyRawData(archive, SFILE_VERIFY_BET_TABLE, nullptr)) return 6;
        SFileCloseArchive(archive); return 0;
    }
    for(unsigned version=1; version<=3; version++) {
        std::string path=std::string(argv[1])+"/storm-v"+std::to_string(version+1)+".mpq";
        remove(path.c_str());
        SFILE_CREATE_MPQ info={}; info.cbSize=sizeof(info); info.dwMpqVersion=version;
        info.dwSectorSize=4096; info.dwRawChunkSize=version==3 ? 1024 : 0; info.dwMaxFileCount=16;
        HANDLE archive; check(SFileCreateArchive2(path.c_str(), &info, &archive));
        std::vector<unsigned char> data(17003); for(size_t i=0; i<data.size(); i++) data[i]=i%7;
        HANDLE file; check(SFileCreateFile(archive, "Units\\Encrypted.bin", 0, data.size(), 0,
            MPQ_FILE_COMPRESS|MPQ_FILE_ENCRYPTED|MPQ_FILE_FIX_KEY, &file));
        check(SFileWriteFile(file, data.data(), data.size(), MPQ_COMPRESSION_ZLIB));
        check(SFileFinishFile(file));
        check(SFileCreateFile(archive, "stored.bin", 0, 7, 0, 0, &file));
        check(SFileWriteFile(file, "payload", 7, 0)); check(SFileFinishFile(file));
        check(SFileCloseArchive(archive));
    }
}
