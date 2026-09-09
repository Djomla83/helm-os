/* LAUNCH-EXEC-01 process-tree negative control.
 *
 * Disposable spike code. Forks a descendant that outlives its parent, so the
 * experiment can establish honestly that helm-launch 0.1 provides NO
 * process-tree containment.
 *
 * Two deliberately different descendant shapes, because the P-series and the
 * O6/P4 pair test different things and one construction cannot serve both:
 *
 *   --release-stdio  (P1, P2)  The descendant's FIRST THREE ACTIONS after fork
 *                    are to close 0, 1 and 2 and reopen them on /dev/null, so
 *                    it can never hold a launcher pipe open. A lifecycle
 *                    control must not be able to hang a mandatory case.
 *
 *   --retain-stdio   (O6, P4)  The descendant KEEPS descriptors 1 and 2. This
 *                    is the state that makes the launcher's own drain loop
 *                    unable to observe end-of-file, and it is the whole point
 *                    of those cases. It is bounded by POST_EXIT_DRAIN_MS on the
 *                    launcher side, never by containment here.
 *
 * The descendant always _exits after its bounded lifetime, and the harness
 * confirms no descendant remains before declaring the run complete.
 *
 * Build: cc -O2 -Wall -Wextra -static -o helper_fork helper_fork.c
 */
#define _GNU_SOURCE
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

static void sleep_ms(long ms)
{
    struct timespec ts;
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (ms % 1000) * 1000000L;
    nanosleep(&ts, NULL);
}

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

int main(int argc, char **argv)
{
    int retain_stdio = 0, do_setsid = 0, parent_exit = 0;
    long lifetime_ms = 20000, prewrite = 0;
    const char *fifo = NULL;

    for (int i = 1; i < argc; i++) {
        const char *a = argv[i];
        const char *v = (i + 1 < argc) ? argv[i + 1] : NULL;
        if (!strcmp(a, "--retain-stdio")) { retain_stdio = 1; }
        else if (!strcmp(a, "--release-stdio")) { retain_stdio = 0; }
        else if (!strcmp(a, "--setsid")) { do_setsid = 1; }
        else if (!strcmp(a, "--parent-exit") && v) { parent_exit = atoi(v); i++; }
        else if (!strcmp(a, "--lifetime-ms") && v) { lifetime_ms = atol(v); i++; }
        else if (!strcmp(a, "--prewrite") && v) { prewrite = atol(v); i++; }
        else if (!strcmp(a, "--liveness-fifo") && v) { fifo = v; i++; }
    }

    /* O6 writes a known payload BEFORE the direct child exits, so the launcher
     * has real bytes to account for in a stream that never reaches EOF. */
    if (prewrite > 0) {
        for (long i = 0; i < prewrite; i++) {
            unsigned char b = (unsigned char)((i * 251 + 1) % 256);
            write_all(1, (const char *)&b, 1);
        }
    }

    pid_t pid = fork();
    if (pid < 0) {
        return 90; /* the harness scores a failed fork as INVALID, not a result */
    }

    if (pid == 0) {
        if (!retain_stdio) {
            close(0);
            close(1);
            close(2);
            int devnull = open("/dev/null", O_RDWR);
            if (devnull >= 0) {
                if (devnull != 0) { dup2(devnull, 0); }
                dup2(0, 1);
                dup2(0, 2);
                if (devnull > 2) { close(devnull); }
            }
        }
        if (do_setsid) {
            setsid();
        }
        /* Liveness is signalled out of band, by pathname, because this is a
         * negative-control fixture and not the mechanism under test. */
        if (fifo) {
            int f = open(fifo, O_WRONLY | O_CLOEXEC);
            if (f >= 0) {
                write_all(f, "L", 1);
                close(f);
            }
        }
        sleep_ms(lifetime_ms);
        _exit(0);
    }

    /* The direct child exits immediately and does not reap the descendant. */
    return parent_exit;
}
