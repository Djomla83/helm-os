#include <windows.h>
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    if (argc != 2 || (strcmp(argv[1], "good") && strcmp(argv[1], "broken")))
        return 64;
    const int broken = strcmp(argv[1], "broken") == 0;
    const char *name = broken ? "HELM_G0_2_DELIBERATELY_MISSING_EXPORT" : "GetCurrentProcessId";
    HMODULE module = LoadLibraryA("kernel32.dll");
    if (!module) {
        printf("{\"control\":\"%s\",\"module_loaded\":false,\"win32_error\":%lu}\n",
               argv[1], (unsigned long)GetLastError());
        return 3;
    }
    SetLastError(0);
    FARPROC entry = GetProcAddress(module, name);
    if (!entry) {
        DWORD error = GetLastError();
        printf("{\"control\":\"%s\",\"module_loaded\":true,\"entrypoint_found\":false,\"win32_error\":%lu}\n",
               argv[1], (unsigned long)error);
        return 2;
    }
    if (broken) {
        printf("{\"control\":\"broken\",\"module_loaded\":true,\"entrypoint_found\":true}\n");
        return 0;
    }
    DWORD value = ((DWORD (WINAPI *)(void))entry)();
    printf("{\"control\":\"good\",\"module_loaded\":true,\"entrypoint_found\":true,\"call_nonzero\":%s}\n",
           value ? "true" : "false");
    return value ? 0 : 4;
}
