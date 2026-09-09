/* LAUNCH-EXEC-01 dynamically linked helper (case E7).
 *
 * Disposable spike code. This is the ONE helper that is deliberately NOT
 * statically linked, and it exists because every other helper is.
 *
 * Static linking is a precondition elsewhere: it removes loader activity from
 * the trace, so the negative claims "no other descriptor was present" and "no
 * other file was opened" rest on a complete trace rather than on filtering.
 * The side effect is that without E7 no preregistered case would execute a
 * dynamic object at all -- while every real HELM subject, Wine included, is
 * dynamic.
 *
 * What E7 establishes, from the trace of this helper's execution:
 *
 *   - the kernel resolves this object's PT_INTERP BY PATHNAME at exec time;
 *   - ld.so then resolves every DT_NEEDED object BY NAME;
 *   - neither is measured, and neither appears in the receipt.
 *
 * So pre_exec_body_sha256 identifies the pinned object and NOT the loaded-code
 * closure -- and it identifies no more of that closure under an empty
 * environment than under any other, which is why D-10 is hardening and not
 * provenance.
 *
 * Build: cc -O2 -Wall -Wextra -o helper_dynamic helper_dynamic.c
 *        (deliberately no -static; `ldd` must report a real interpreter)
 */
#define _GNU_SOURCE
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define SENTINEL "HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n"

extern char **environ;

int main(int argc, char **argv)
{
    (void)argc;
    (void)argv;

    /* snprintf is a libc call, so a DT_NEEDED object must have been resolved
     * by name for this line to work at all. Its success is part of E7's
     * evidence, not incidental. */
    char line[512];
    int env_count = 0;
    for (char **e = environ; e && *e; e++) {
        env_count++;
    }
    int n = snprintf(line, sizeof(line),
                     "{\"marker\":\"helper_dynamic\",\"linkage\":\"dynamic\","
                     "\"environ_count\":%d}\n", env_count);

    ssize_t ignored = write(1, SENTINEL, strlen(SENTINEL));
    (void)ignored;
    if (n > 0) {
        ignored = write(1, line, (size_t)n);
        (void)ignored;
    }
    return 0;
}
