#include <windows.h>
#include <stdio.h>
typedef HRESULT (WINAPI *pDCompositionCreateDevice)(void*, REFIID, void**);
int main(void){
    HMODULE h = LoadLibraryA("dcomp.dll");
    if(!h){ printf("{\"dcomp_dll_loaded\":false,\"note\":\"LoadLibrary failed\"}\n"); return 0; }
    pDCompositionCreateDevice f = (pDCompositionCreateDevice)GetProcAddress(h,"DCompositionCreateDevice");
    if(!f){ printf("{\"dcomp_dll_loaded\":true,\"entrypoint_found\":false}\n"); return 0; }
    void* dev=NULL;
    GUID iid = {0xC37EA93A,0xE7AA,0x450D,{0xB1,0x6F,0x97,0x46,0xCB,0x04,0x07,0xF3}};
    HRESULT hr = f(NULL,&iid,&dev);
    printf("{\"dcomp_dll_loaded\":true,\"entrypoint_found\":true,\"hresult\":\"0x%08lX\",\"is_E_NOTIMPL\":%s}\n",
           (unsigned long)hr, (hr==0x80004001L)?"true":"false");
    return 0;
}
