/* LAUNCH-EXEC-01 primary synthetic helper.
 *
 * Disposable pre-implementation spike code. NOT crates/helm-launch, not a
 * Cargo workspace member, defines no product API, and must not be promoted
 * into one.
 *
 * Reports its own observed process boundary as JSON from inside the executed
 * image. The report travels on DESCRIPTOR 1 ONLY, behind the frozen sentinel,
 * because no descriptor above 2 is ever passed to a helper: a harness-controlled
 * report descriptor would have to survive close_range into the executed image
 * and would contradict the very invariant F1 and F4 measure.
 *
 * Payload bytes requested by argv precede the sentinel; report bytes follow it.
 *
 * Build: cc -O2 -Wall -Wextra -static -o helper_report helper_report.c
 */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

extern char **environ;

#define SENTINEL "HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n"
#define REPORT_CAP (256 * 1024)
#define CHUNK 4096

static char report[REPORT_CAP];
static size_t report_len;

static void emit(const char *s, size_t n)
{
    if (report_len + n >= REPORT_CAP) {
        n = REPORT_CAP - report_len - 1;
    }
    memcpy(report + report_len, s, n);
    report_len += n;
}

static void emit_str(const char *s) { emit(s, strlen(s)); }

static void emit_fmt(const char *fmt, ...)
{
    char line[4096];
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(line, sizeof(line), fmt, ap);
    va_end(ap);
    if (n > 0) {
        emit(line, (size_t)n < sizeof(line) ? (size_t)n : sizeof(line) - 1);
    }
}

/* JSON string escaping. Environment values and argv are arbitrary bytes; any
 * byte outside printable ASCII is emitted as \u00XX so the report is valid
 * JSON and byte-faithful. */
static void emit_json_string(const char *s, size_t len)
{
    emit_str("\"");
    for (size_t i = 0; i < len; i++) {
        unsigned char c = (unsigned char)s[i];
        if (c == '"' || c == '\\') {
            char pair[2] = { '\\', (char)c };
            emit(pair, 2);
        } else if (c >= 0x20 && c < 0x7f) {
            emit((const char *)&c, 1);
        } else {
            emit_fmt("\\u%04x", c);
        }
    }
    emit_str("\"");
}

/* One field from /proc/self/status, e.g. "SigBlk" or "NoNewPrivs". Returns 1
 * on success. This is the kernel-backed observation D-11 requires; there is no
 * userspace substitute for it. */
static int proc_status_field(const char *name, char *out, size_t cap)
{
    int fd = open("/proc/self/status", O_RDONLY | O_CLOEXEC);
    if (fd < 0) {
        return 0;
    }
    static char buf[65536];
    ssize_t total = 0, n;
    while ((n = read(fd, buf + total, sizeof(buf) - 1 - (size_t)total)) > 0) {
        total += n;
        if ((size_t)total >= sizeof(buf) - 1) {
            break;
        }
    }
    close(fd);
    if (total <= 0) {
        return 0;
    }
    buf[total] = '\0';

    char key[64];
    int klen = snprintf(key, sizeof(key), "%s:", name);
    if (klen <= 0) {
        return 0;
    }
    char *line = buf;
    while (line && *line) {
        if (strncmp(line, key, (size_t)klen) == 0) {
            char *value = line + klen;
            while (*value == ' ' || *value == '\t') {
                value++;
            }
            char *end = strchr(value, '\n');
            size_t vlen = end ? (size_t)(end - value) : strlen(value);
            if (vlen >= cap) {
                vlen = cap - 1;
            }
            memcpy(out, value, vlen);
            out[vlen] = '\0';
            return 1;
        }
        line = strchr(line, '\n');
        if (line) {
            line++;
        }
    }
    return 0;
}

/* Frozen output recipe: byte[i] = (i * 251 + tag) % 256. Restated here from the
 * same definition oracles.py restates independently; neither reads the other. */
static void write_stream(int fd, long count, int tag)
{
    unsigned char chunk[CHUNK];
    long done = 0;
    while (done < count) {
        size_t n = (size_t)((count - done) < CHUNK ? (count - done) : CHUNK);
        for (size_t i = 0; i < n; i++) {
            chunk[i] = (unsigned char)(((done + (long)i) * 251 + tag) % 256);
        }
        size_t off = 0;
        while (off < n) {
            ssize_t w = write(fd, chunk + off, n - off);
            if (w < 0) {
                if (errno == EINTR) {
                    continue;
                }
                return; /* EPIPE and friends are the caller's fact to record */
            }
            off += (size_t)w;
        }
        done += (long)n;
    }
}

/* Descriptor enumeration by fcntl(F_GETFD), which OPENS NOTHING. Reading
 * /proc/self/fd would create the very descriptor being counted; that path is a
 * secondary cross-check in F1 and F4 only. */
static void emit_descriptors(void)
{
    struct rlimit rl;
    long max = 1024;
    if (getrlimit(RLIMIT_NOFILE, &rl) == 0 && rl.rlim_cur != RLIM_INFINITY) {
        max = (long)rl.rlim_cur;
    }
    if (max > 65536) {
        max = 65536;
    }
    emit_str("\"descriptors\":[");
    int first = 1;
    for (long fd = 0; fd < max; fd++) {
        int flags = fcntl((int)fd, F_GETFD);
        if (flags < 0) {
            continue;
        }
        emit_fmt("%s{\"fd\":%ld,\"cloexec\":%s}", first ? "" : ",", fd,
                 (flags & FD_CLOEXEC) ? "true" : "false");
        first = 0;
    }
    emit_str("]");
}

/* T3: exit inside the grace window on a frozen schedule, so "during the grace
 * window" is a schedule rather than a race. */
static volatile sig_atomic_t g_grace_delay_ms = -1;

static void grace_term_handler(int sig)
{
    (void)sig;
    long ms = (long)g_grace_delay_ms;
    if (ms > 0) {
        struct timespec ts;
        ts.tv_sec = ms / 1000;
        ts.tv_nsec = (ms % 1000) * 1000000L;
        nanosleep(&ts, NULL);
    }
    _exit(9);
}

static long arg_long(const char *s, long fallback)
{
    if (!s) {
        return fallback;
    }
    char *end = NULL;
    long v = strtol(s, &end, 10);
    return (end && *end == '\0') ? v : fallback;
}

int main(int argc, char **argv)
{
    const char *marker = "helper_report";
    long out_bytes = 0, err_bytes = 0, sleep_ms = 0;
    int exit_code = 0, want_report = 1, exit_immediately = -1;
    int raise_segv = 0, core_zero = 0, close_stdout_early = 0, ignore_sigterm = 0;
    long grace_exit_delay_ms = -1;

    for (int i = 1; i < argc; i++) {
        const char *a = argv[i];
        const char *v = (i + 1 < argc) ? argv[i + 1] : NULL;
        if (!strcmp(a, "--marker") && v) { marker = v; i++; }
        else if (!strcmp(a, "--stdout") && v) { out_bytes = arg_long(v, 0); i++; }
        else if (!strcmp(a, "--stderr") && v) { err_bytes = arg_long(v, 0); i++; }
        else if (!strcmp(a, "--exit") && v) { exit_code = (int)arg_long(v, 0); i++; }
        else if (!strcmp(a, "--sleep-ms") && v) { sleep_ms = arg_long(v, 0); i++; }
        else if (!strcmp(a, "--exit-immediately") && v) {
            exit_immediately = (int)arg_long(v, 0); i++;
        }
        else if (!strcmp(a, "--no-report")) { want_report = 0; }
        else if (!strcmp(a, "--raise-segv")) { raise_segv = 1; }
        else if (!strcmp(a, "--rlimit-core-zero")) { core_zero = 1; }
        else if (!strcmp(a, "--close-stdout-early")) { close_stdout_early = 1; }
        else if (!strcmp(a, "--ignore-sigterm")) { ignore_sigterm = 1; }
        else if (!strcmp(a, "--grace-exit-delay-ms") && v) {
            grace_exit_delay_ms = arg_long(v, -1); i++;
        }
    }

    /* R3/S4: the very first observable action, so no report, no output and no
     * setup can precede it. This is what makes the rapid-exit schedule forced
     * rather than hoped for. */
    if (exit_immediately >= 0) {
        _exit(exit_immediately);
    }

    if (core_zero) {
        struct rlimit zero = { 0, 0 };
        setrlimit(RLIMIT_CORE, &zero); /* R3 must not drop a core file */
    }
    if (ignore_sigterm) {
        signal(SIGTERM, SIG_IGN); /* T2/T6: force the grace window to elapse */
    }
    if (grace_exit_delay_ms >= 0) {
        g_grace_delay_ms = (sig_atomic_t)grace_exit_delay_ms;
        signal(SIGTERM, grace_term_handler);
    }

    /* Payload first, then the sentinel, then the report. */
    if (out_bytes > 0) {
        write_stream(1, out_bytes, 1);
    }
    if (err_bytes > 0) {
        write_stream(2, err_bytes, 2);
    }
    if (close_stdout_early) {
        close(1); /* O5: stderr keeps flowing after stdout reaches EOF */
    }

    if (want_report && !close_stdout_early) {
        char sig_blk[64] = "", sig_ign[64] = "", sig_cgt[64] = "";
        char sig_pnd[64] = "", nnp[64] = "";
        proc_status_field("SigBlk", sig_blk, sizeof(sig_blk));
        proc_status_field("SigIgn", sig_ign, sizeof(sig_ign));
        proc_status_field("SigCgt", sig_cgt, sizeof(sig_cgt));
        proc_status_field("SigPnd", sig_pnd, sizeof(sig_pnd));
        proc_status_field("NoNewPrivs", nnp, sizeof(nnp));

        struct stat cwd;
        int have_cwd = (stat(".", &cwd) == 0);

        emit_str("{");
        emit_str("\"marker\":");
        emit_json_string(marker, strlen(marker));

        emit_str(",\"argv\":[");
        for (int i = 0; i < argc; i++) {
            if (i) {
                emit_str(",");
            }
            emit_str("{\"len\":");
            emit_fmt("%zu", strlen(argv[i]));
            emit_str(",\"value\":");
            emit_json_string(argv[i], strlen(argv[i]));
            emit_str("}");
        }
        emit_str("]");

        /* The COMPLETE environment, so a leak is detectable. Under D-10 the
         * only correct observation is an empty array; the runner sanitises any
         * undeclared name before publication rather than reproducing it. */
        emit_str(",\"environ\":[");
        for (char **e = environ; e && *e; e++) {
            if (e != environ) {
                emit_str(",");
            }
            emit_json_string(*e, strlen(*e));
        }
        emit_str("]");

        emit_str(",");
        emit_descriptors();

        emit_fmt(",\"uid\":%u,\"euid\":%u,\"gid\":%u,\"egid\":%u",
                 (unsigned)getuid(), (unsigned)geteuid(),
                 (unsigned)getgid(), (unsigned)getegid());

        emit_str(",\"signals\":{\"SigBlk\":");
        emit_json_string(sig_blk, strlen(sig_blk));
        emit_str(",\"SigIgn\":");
        emit_json_string(sig_ign, strlen(sig_ign));
        emit_str(",\"SigCgt\":");
        emit_json_string(sig_cgt, strlen(sig_cgt));
        emit_str(",\"SigPnd\":");
        emit_json_string(sig_pnd, strlen(sig_pnd));
        emit_str("}");

        /* D-11 evidence: kernel-backed, not inferred. */
        emit_str(",\"no_new_privs\":");
        emit_json_string(nnp, strlen(nnp));

        if (have_cwd) {
            emit_fmt(",\"cwd\":{\"st_dev\":%llu,\"st_ino\":%llu}",
                     (unsigned long long)cwd.st_dev,
                     (unsigned long long)cwd.st_ino);
        } else {
            emit_str(",\"cwd\":null");
        }

        emit_fmt(",\"requested\":{\"stdout\":%ld,\"stderr\":%ld,\"exit\":%d}",
                 out_bytes, err_bytes, exit_code);
        emit_str("}\n");

        const char *sentinel = SENTINEL;
        size_t slen = strlen(sentinel);
        size_t off = 0;
        while (off < slen) {
            ssize_t w = write(1, sentinel + off, slen - off);
            if (w < 0) {
                if (errno == EINTR) {
                    continue;
                }
                break;
            }
            off += (size_t)w;
        }
        off = 0;
        while (off < report_len) {
            ssize_t w = write(1, report + off, report_len - off);
            if (w < 0) {
                if (errno == EINTR) {
                    continue;
                }
                break;
            }
            off += (size_t)w;
        }
    }

    if (raise_segv) {
        raise(SIGSEGV);
    }
    if (sleep_ms > 0) {
        struct timespec ts;
        ts.tv_sec = sleep_ms / 1000;
        ts.tv_nsec = (sleep_ms % 1000) * 1000000L;
        nanosleep(&ts, NULL);
    }
    return exit_code;
}
