/* LAUNCH-EXEC-01 substitution-detection helper.
 *
 * Disposable spike code. A second helper with a DIFFERENT BODY and therefore a
 * different digest, whose only job is to be distinguishable from
 * helper_report if executable substitution ever occurs.
 *
 * E2 renames this over the pinned pathname, E4 retargets a symlink at it, and
 * E6 writes its marker into helper_report's inode. In E2 and E4 the pinned body
 * must still run and the marker must still read "helper_report"; in E6 the
 * mutated marker must appear, because the descriptor pins the inode and does
 * not freeze its contents.
 *
 * The marker below is a fixed-length string so that E6 can overwrite it in
 * place without changing the file length, which is what makes E6's mutation
 * length-preserving and ELF-valid.
 *
 * Build: cc -O2 -Wall -Wextra -static -o helper_alt helper_alt.c
 */
#define _GNU_SOURCE
#include <string.h>
#include <unistd.h>

#define SENTINEL "HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n"

/* Exactly the same length as helper_report's default marker, so a
 * length-preserving pwrite can swap one for the other inside a pinned inode. */
volatile char g_marker[16] = "helper_alt\0\0\0\0\0";

static void write_all(int fd, const char *s, size_t n)
{
    size_t off = 0;
    while (off < n) {
        ssize_t w = write(fd, s + off, n - off);
        if (w <= 0) {
            return;
        }
        off += (size_t)w;
    }
}

int main(void)
{
    char marker[sizeof(g_marker) + 1];
    for (size_t i = 0; i < sizeof(g_marker); i++) {
        marker[i] = g_marker[i];
    }
    marker[sizeof(g_marker)] = '\0';

    write_all(1, SENTINEL, strlen(SENTINEL));
    write_all(1, "{\"marker\":\"", 11);
    write_all(1, marker, strlen(marker));
    write_all(1, "\"}\n", 3);
    return 0;
}
