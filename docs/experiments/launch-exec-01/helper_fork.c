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
 * --fixture-health-fifo (P1, P2, P4 only; Trial #3 correction candidate,
 * review finding R-I1). A missing liveness byte used to be read as a dead
 * descendant even when the liveness fixture itself had failed. The descendant
 * now proves, on a separate harness-armed FIFO, that it reached the liveness
 * probe point, and reports a liveness open or write failure there. See
 * liveness_probe() below for the exact order and why the direct child waits.
 *
 * The descendant always _exits after its bounded lifetime, and the harness
 * confirms no descendant remains before declaring the run complete.
 *
 * Build: cc -O2 -Wall -Wextra -static -o helper_fork helper_fork.c
 */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
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

/* 0 when every byte was written; -1 with errno set otherwise. */
static int write_all(int fd, const char *s, size_t n)
{
    size_t off = 0;
    while (off < n) {
        ssize_t w = write(fd, s + off, n - off);
        if (w <= 0) {
            if (w == 0) {
                errno = EIO;
            }
            return -1;
        }
        off += (size_t)w;
    }
    return 0;
}

/* Trial #2: a fixture-signal open that failed was silent, so a broken fixture
 * path looked exactly like a descendant that never reached the signalling
 * point. A failure is now one line on the descendant's own standard error: the
 * failed step and the errno NUMBER, never the path. For a retained-stdio
 * fixture that descriptor is the launcher's stderr pipe, which the harness
 * reads as a diagnosis of an unestablished fixture, never as its proof; for a
 * released one it is /dev/null. No descriptor is added and the signal byte is
 * unchanged. */
#define FIXTURE_SIGNAL_FAILED "HELM-LAUNCH-EXEC-01-FIXTURE-SIGNAL-FAILED:"

static void fixture_signal_failed(const char *step, int err)
{
    char line[96];
    int n = snprintf(line, sizeof(line), "%s%s:%d\n", FIXTURE_SIGNAL_FAILED,
                     step, err);
    if (n > 0 && (size_t)n < sizeof(line)) {
        (void)write_all(2, line, (size_t)n);
    }
}

/* The closed fixture-health vocabulary. Each token is ONE write of at most 32
 * bytes into an otherwise empty pipe, so it arrives whole or not at all, and it
 * never names a path. */
#define LIVENESS_PROBE_REACHED "PROBE_REACHED\n"
#define LIVENESS_OPEN_FAILED "LIVENESS_OPEN_FAILED"
#define LIVENESS_WRITE_FAILED "LIVENESS_WRITE_FAILED"
#define LIVENESS_UNEXPECTED_OPEN "LIVENESS_UNEXPECTED_OPEN\n"

static void health_token(int health, const char *token)
{
    if (health >= 0) {
        (void)write_all(health, token, strlen(token));
    }
}

static void health_failure(int health, const char *step, int err)
{
    char line[48];
    int n = snprintf(line, sizeof(line), "%s:%d\n", step, err);
    if (health >= 0 && n > 0 && (size_t)n < sizeof(line)) {
        (void)write_all(health, line, (size_t)n);
    }
}

/* R-I1: reach the liveness probe point and say so, BEFORE the direct child may
 * exit. Runs in the descendant after any stdio release and setsid.
 *
 * The harness opened the health FIFO's read end before the launcher existed,
 * so the non-blocking open below completes at once. The liveness FIFO is then
 * checked without blocking: ENXIO means it is a FIFO this process may open for
 * writing and that nobody reads yet -- exactly the state the post-return
 * rendezvous needs. Only then is PROBE_REACHED written. A path that cannot be
 * opened is reported as LIVENESS_OPEN_FAILED instead, and an object that opens
 * without a waiting reader as LIVENESS_UNEXPECTED_OPEN; neither attempts the
 * rendezvous.
 *
 * Closing the gate's write end last is what lets the direct child exit. The
 * launcher sweeps only after the direct child exited, so whichever health
 * token was written is buffered in the FIFO before the sweep can kill this
 * descendant. Returns whether the rendezvous may be attempted. */
static int liveness_probe(const char *health_path, const char *fifo, int *health,
                          int gate_write)
{
    int probed = 0;
    *health = open(health_path, O_WRONLY | O_NONBLOCK | O_CLOEXEC);
    if (fifo) {
        int pre = open(fifo, O_WRONLY | O_NONBLOCK | O_CLOEXEC);
        if (pre >= 0) {
            close(pre);
            health_token(*health, LIVENESS_UNEXPECTED_OPEN);
        } else if (errno != ENXIO) {
            health_failure(*health, LIVENESS_OPEN_FAILED, errno);
        } else {
            health_token(*health, LIVENESS_PROBE_REACHED);
            probed = 1;
        }
    }
    if (gate_write >= 0) {
        close(gate_write);
    }
    return probed;
}

int main(int argc, char **argv)
{
    int retain_stdio = 0, do_setsid = 0, parent_exit = 0;
    long lifetime_ms = 20000, prewrite = 0;
    const char *fifo = NULL, *health_path = NULL;

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
        else if (!strcmp(a, "--fixture-health-fifo") && v) { health_path = v; i++; }
    }

    /* O6 writes a known payload BEFORE the direct child exits, so the launcher
     * has real bytes to account for in a stream that never reaches EOF. */
    if (prewrite > 0) {
        for (long i = 0; i < prewrite; i++) {
            unsigned char b = (unsigned char)((i * 251 + 1) % 256);
            write_all(1, (const char *)&b, 1);
        }
    }

    /* R-I1: the direct child waits on this pipe until its descendant has
     * written its health token. Created only with a health channel, inside
     * this image and after exec, close-on-exec, and closed by both processes;
     * nothing crosses an exec. */
    int gate[2] = { -1, -1 };
    if (health_path && pipe2(gate, O_CLOEXEC) != 0) {
        gate[0] = -1;
        gate[1] = -1;
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
        int health = -1, rendezvous = 1;
        if (health_path) {
            if (gate[0] >= 0) { close(gate[0]); }
            rendezvous = liveness_probe(health_path, fifo, &health, gate[1]);
        }
        /* Liveness is signalled out of band, by pathname, because this is a
         * negative-control fixture and not the mechanism under test. */
        if (fifo && rendezvous) {
            int f = open(fifo, O_WRONLY | O_CLOEXEC);
            if (f < 0) {
                int err = errno;
                fixture_signal_failed("open", err);
                health_failure(health, LIVENESS_OPEN_FAILED, err);
            } else {
                if (write_all(f, "L", 1) != 0) {
                    int err = errno;
                    fixture_signal_failed("write", err);
                    health_failure(health, LIVENESS_WRITE_FAILED, err);
                }
                close(f);
            }
        }
        if (health >= 0) {
            close(health);
        }
        sleep_ms(lifetime_ms);
        _exit(0);
    }

    /* The direct child exits without reaping the descendant: immediately, or,
     * with a health channel, as soon as the descendant closed the gate (or
     * died). */
    if (gate[1] >= 0) {
        close(gate[1]);
    }
    if (gate[0] >= 0) {
        char byte;
        while (read(gate[0], &byte, 1) < 0 && errno == EINTR) {
        }
        close(gate[0]);
    }
    return parent_exit;
}
