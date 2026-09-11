/* LAUNCH-EXEC-01 launcher spike: the mechanism under test.
 *
 * Disposable pre-implementation spike code. NOT crates/helm-launch, not a Cargo
 * workspace member, defines no product API, and must not be promoted into one.
 * Its only purpose is to falsify the mechanism described in the architecture.
 *
 * It implements ONLY what the frozen cases need:
 *   pin -> measure -> admission -> clone3(CLONE_PIDFD) -> child setup -> execveat
 *
 * There is no convenience pathname launcher, no shell, no PATH lookup, no
 * command-string parser and no Wine. The one pathname open is the TRUSTED
 * CALLER's act of handing over a descriptor; after that no name is resolved.
 *
 * Build: cc -O2 -Wall -Wextra -static -o launcher_spike launcher_spike.c
 *
 * NOT_RUN: no preregistered trial has been executed with this program.
 */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <pthread.h>
#include <signal.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

/* ------------------------------------------------------------ syscall numbers
 * Taken as raw syscalls, never through the glibc wrappers: the wrappers for
 * execveat and close_range are declared for linux-gnu only and close_range's
 * arrived in glibc 2.34, so using them would add an invisible libc floor to a
 * cohort whose only stated floor is the kernel. */
#ifndef __NR_execveat
#define __NR_execveat 322
#endif
#ifndef __NR_close_range
#define __NR_close_range 436
#endif
#ifndef __NR_clone3
#define __NR_clone3 435
#endif
#ifndef __NR_pidfd_send_signal
#define __NR_pidfd_send_signal 424
#endif
/* Used ONLY by the M5 rejected-acquisition arm, never by the mechanism. */
#ifndef __NR_pidfd_open
#define __NR_pidfd_open 434
#endif
#ifndef AT_EMPTY_PATH
#define AT_EMPTY_PATH 0x1000
#endif
#ifndef CLONE_PIDFD
#define CLONE_PIDFD 0x00001000ULL
#endif
#ifndef P_PIDFD
#define P_PIDFD 3
#endif

struct helm_clone_args {
    uint64_t flags, pidfd, child_tid, parent_tid, exit_signal;
    uint64_t stack, stack_size, tls, set_tid, set_tid_size, cgroup;
};

/* ------------------------------------------------------------- frozen stages
 * Every post-fork operation maps to exactly one of these. "The correct stage"
 * is unverifiable without a closed set, so the set is frozen here and in
 * frozen_cases.py, and case M4 matches the implemented order against it. */
enum stage {
    ST_RELOCATE = 1, ST_DUP2, ST_CLEAR_CLOEXEC, ST_CHDIR, ST_CLOSE_RANGE,
    ST_SETPGID, ST_SIGMASK, ST_SIGACTION, ST_NO_NEW_PRIVS, ST_EXEC
};
static const char *stage_name(int s)
{
    switch (s) {
    case ST_RELOCATE: return "RELOCATE";
    case ST_DUP2: return "DUP2";
    case ST_CLEAR_CLOEXEC: return "CLEAR_CLOEXEC";
    case ST_CHDIR: return "CHDIR";
    case ST_CLOSE_RANGE: return "CLOSE_RANGE";
    case ST_SETPGID: return "SETPGID";
    case ST_SIGMASK: return "SIGMASK";
    case ST_SIGACTION: return "SIGACTION";
    case ST_NO_NEW_PRIVS: return "NO_NEW_PRIVS";
    case ST_EXEC: return "EXEC";
    default: return "UNKNOWN";
    }
}

struct exec_status { uint8_t stage; int32_t err; };

/* =============================================================== SHA-256
 * Needed because the receipt binds a measurement the independent Python oracle
 * must be able to contradict. Standard FIPS 180-4; no library dependency. */
typedef struct { uint32_t h[8]; uint64_t len; uint8_t buf[64]; size_t n; } sha256;

static const uint32_t K256[64] = {
0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2};

#define ROR(x,n) (((x) >> (n)) | ((x) << (32 - (n))))

static void sha256_block(sha256 *c, const uint8_t *p)
{
    uint32_t w[64], a, b, cc, d, e, f, g, h, t1, t2;
    for (int i = 0; i < 16; i++) {
        w[i] = ((uint32_t)p[i*4] << 24) | ((uint32_t)p[i*4+1] << 16) |
               ((uint32_t)p[i*4+2] << 8) | (uint32_t)p[i*4+3];
    }
    for (int i = 16; i < 64; i++) {
        uint32_t s0 = ROR(w[i-15],7) ^ ROR(w[i-15],18) ^ (w[i-15] >> 3);
        uint32_t s1 = ROR(w[i-2],17) ^ ROR(w[i-2],19) ^ (w[i-2] >> 10);
        w[i] = w[i-16] + s0 + w[i-7] + s1;
    }
    a=c->h[0]; b=c->h[1]; cc=c->h[2]; d=c->h[3];
    e=c->h[4]; f=c->h[5]; g=c->h[6]; h=c->h[7];
    for (int i = 0; i < 64; i++) {
        uint32_t S1 = ROR(e,6) ^ ROR(e,11) ^ ROR(e,25);
        uint32_t ch = (e & f) ^ ((~e) & g);
        t1 = h + S1 + ch + K256[i] + w[i];
        uint32_t S0 = ROR(a,2) ^ ROR(a,13) ^ ROR(a,22);
        uint32_t maj = (a & b) ^ (a & cc) ^ (b & cc);
        t2 = S0 + maj;
        h=g; g=f; f=e; e=d+t1; d=cc; cc=b; b=a; a=t1+t2;
    }
    c->h[0]+=a; c->h[1]+=b; c->h[2]+=cc; c->h[3]+=d;
    c->h[4]+=e; c->h[5]+=f; c->h[6]+=g; c->h[7]+=h;
}

static void sha256_init(sha256 *c)
{
    c->h[0]=0x6a09e667; c->h[1]=0xbb67ae85; c->h[2]=0x3c6ef372; c->h[3]=0xa54ff53a;
    c->h[4]=0x510e527f; c->h[5]=0x9b05688c; c->h[6]=0x1f83d9ab; c->h[7]=0x5be0cd19;
    c->len = 0; c->n = 0;
}

static void sha256_update(sha256 *c, const uint8_t *p, size_t n)
{
    c->len += n;
    while (n) {
        size_t take = 64 - c->n;
        if (take > n) { take = n; }
        memcpy(c->buf + c->n, p, take);
        c->n += take; p += take; n -= take;
        if (c->n == 64) { sha256_block(c, c->buf); c->n = 0; }
    }
}

static void sha256_final(sha256 *c, char out[65])
{
    uint64_t bits = c->len * 8;
    uint8_t pad = 0x80;
    sha256_update(c, &pad, 1);
    uint8_t zero = 0;
    while (c->n != 56) { sha256_update(c, &zero, 1); }
    uint8_t lenb[8];
    for (int i = 0; i < 8; i++) { lenb[i] = (uint8_t)(bits >> (56 - 8*i)); }
    memcpy(c->buf + c->n, lenb, 8);
    sha256_block(c, c->buf);
    for (int i = 0; i < 8; i++) {
        snprintf(out + i*8, 9, "%08x", c->h[i]);
    }
    out[64] = '\0';
}

/* ============================================================ parent helpers */
static long now_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000L + ts.tv_nsec / 1000000L;
}

/* Relocate a descriptor above 2 so the dup2 step and the close_range gap
 * arithmetic have no special cases. `keep_cloexec` must be 0 for a descriptor
 * whose whole purpose is to be non-CLOEXEC (case X2c): relocating with
 * F_DUPFD_CLOEXEC unconditionally would silently restore the flag the case
 * exists to remove, whenever the descriptor happened to land at 0-2. */
static int move_above_2(int fd, int keep_cloexec)
{
    if (fd > 2) { return fd; }
    int moved = fcntl(fd, keep_cloexec ? F_DUPFD_CLOEXEC : F_DUPFD, 3);
    if (moved < 0) { return fd; }
    close(fd);
    return moved;
}

/* ======================================================= child setup (ST_*)
 * EVERYTHING BELOW RUNS AFTER clone3 AND BEFORE execveat.
 *
 * Only async-signal-safe raw syscalls appear here. No malloc, no printf, no
 * stdio, no locale, no pthread, no destructor-dependent cleanup and no libc
 * state mutation. Every buffer it touches -- argv, envp, the exec-status record
 * -- is fully materialised by the parent BEFORE the clone. Its only exits are
 * execveat and _exit.
 *
 * Per-syscall justification for being on this path:
 *   dup3/dup2      pure descriptor-table edit, no userspace state
 *   fcntl F_SETFD  pure descriptor-flag edit
 *   fchdir         pure process-attribute edit on an ALREADY-OPEN descriptor,
 *                  so no pathname is resolved; on signal-safety(7)'s POSIX list
 *   close_range    pure descriptor-table edit; Linux-specific, so absent from
 *                  POSIX's list -- this is an argument from the syscall
 *                  contract, not a citation, and it is recorded as such
 *   setpgid        pure process-attribute edit; on the POSIX list
 *   rt_sigprocmask pure signal-state edit; on the POSIX list
 *   rt_sigaction   pure signal-state edit; on the POSIX list
 *   prctl          pure process-attribute edit; sets no_new_privs (D-11)
 *   write          the exec-status record only, on the POSIX list
 *   execveat       the target of the whole path
 *   _exit          the only non-exec exit
 */
struct child_plan {
    int exec_fd, status_w, dir_fd, in_fd, out_fd, err_fd;
    char **argv, **envp;
    int skip_no_new_privs;   /* N2 control arm only */
    int die_before_exec;     /* S5 only */
    long stall_pre_exec_ms;  /* S6 only */
};

static void child_fail(int status_w, int stage, int err)
{
    struct exec_status rec;
    rec.stage = (uint8_t)stage;
    rec.err = (int32_t)err;
    size_t off = 0;
    const uint8_t *p = (const uint8_t *)&rec;
    while (off < sizeof(rec)) {
        ssize_t w = write(status_w, p + off, sizeof(rec) - off);
        if (w < 0) {
            if (errno == EINTR) { continue; }
            break;
        }
        off += (size_t)w;
    }
    _exit(127);
}

static void child_main(struct child_plan *p)
{
    /* ST_DUP2 -- bind the three prepared endpoints. The parent already
     * relocated every preserved descriptor above 2 (ST_RELOCATE), so no dup2
     * target can collide with a source and the no-op case is unreachable. */
    if (dup2(p->in_fd, 0) < 0)  { child_fail(p->status_w, ST_DUP2, errno); }
    if (dup2(p->out_fd, 1) < 0) { child_fail(p->status_w, ST_DUP2, errno); }
    if (dup2(p->err_fd, 2) < 0) { child_fail(p->status_w, ST_DUP2, errno); }

    /* ST_CLEAR_CLOEXEC -- dup2 clears FD_CLOEXEC only when oldfd != newfd
     * ("if newfd has the same value as oldfd, then dup2() does nothing"), so
     * the clear is required, not defensive. Without it a host with stdio closed
     * yields a child whose stdio the kernel closes at exec, and a launcher that
     * then records bytes_drained: 0 with a digest of the empty string. */
    if (fcntl(0, F_SETFD, 0) < 0) { child_fail(p->status_w, ST_CLEAR_CLOEXEC, errno); }
    if (fcntl(1, F_SETFD, 0) < 0) { child_fail(p->status_w, ST_CLEAR_CLOEXEC, errno); }
    if (fcntl(2, F_SETFD, 0) < 0) { child_fail(p->status_w, ST_CLEAR_CLOEXEC, errno); }

    /* ST_CHDIR -- BEFORE the range close, because dir_fd is not one of the two
     * descriptors the range preserves. Closing it first was the defect that
     * made an earlier form of this sequence unable to complete any launch. */
    if (fchdir(p->dir_fd) < 0) { child_fail(p->status_w, ST_CHDIR, errno); }

    if (p->stall_pre_exec_ms > 0) {
        /* S6 only: force the pre-exec phase past SPAWN_CONFIRM_TIMEOUT_MS so
         * the launcher must terminate and reap rather than leak a child. */
        struct timespec ts;
        ts.tv_sec = p->stall_pre_exec_ms / 1000;
        ts.tv_nsec = (p->stall_pre_exec_ms % 1000) * 1000000L;
        nanosleep(&ts, NULL);
    }

    /* ST_CLOSE_RANGE -- close everything above 2 except the exec descriptor and
     * the exec-status write end, skipping any inverted gap: close_range returns
     * EINVAL when first > last, and this region's only exits are execveat and
     * _exit. Closing by RANGE is why descriptor-number reuse is not a hazard. */
    int a = p->exec_fd < p->status_w ? p->exec_fd : p->status_w;
    int b = p->exec_fd < p->status_w ? p->status_w : p->exec_fd;
    if (a > 3) {
        if (syscall(__NR_close_range, 3u, (unsigned)(a - 1), 0u) < 0) {
            child_fail(p->status_w, ST_CLOSE_RANGE, errno);
        }
    }
    if (b > a + 1) {
        if (syscall(__NR_close_range, (unsigned)(a + 1), (unsigned)(b - 1), 0u) < 0) {
            child_fail(p->status_w, ST_CLOSE_RANGE, errno);
        }
    }
    if (syscall(__NR_close_range, (unsigned)(b + 1), ~0u, 0u) < 0) {
        child_fail(p->status_w, ST_CLOSE_RANGE, errno);
    }

    /* ST_SETPGID -- the parent calls setpgid(child, child) too, so the group
     * exists before any sweep can be issued. */
    if (setpgid(0, 0) < 0) { child_fail(p->status_w, ST_SETPGID, errno); }

    /* ST_SIGMASK -- the mask is inherited across clone and PRESERVED across
     * execve, so without this the host's blocked set silently becomes part of
     * the executed program's contract. */
    sigset_t empty;
    sigemptyset(&empty);
    if (sigprocmask(SIG_SETMASK, &empty, NULL) < 0) {
        child_fail(p->status_w, ST_SIGMASK, errno);
    }

    /* ST_SIGACTION -- execve resets HANDLED dispositions but leaves IGNORED
     * ones unchanged, so an inherited SIG_IGN survives into the executed image.
     * A Rust host sets SIGPIPE to SIG_IGN before main, which would otherwise
     * silently change the behaviour case O5 tests. */
    struct sigaction dfl;
    memset(&dfl, 0, sizeof(dfl));
    dfl.sa_handler = SIG_DFL;
    for (int sig = 1; sig < NSIG; sig++) {
        if (sig == SIGKILL || sig == SIGSTOP) { continue; }
        sigaction(sig, &dfl, NULL); /* best effort: unknown signals are benign */
    }

    /* ST_NO_NEW_PRIVS (D-11) -- exec may not grant new privilege through
     * set-user-ID, set-group-ID or file capabilities. Defense in depth: D-9
     * already refuses set-id objects at admission, and file capabilities are
     * not covered by that refusal at all. It does NOT drop the caller's
     * existing privileges, isolate anything, or sandbox the program. */
    if (!p->skip_no_new_privs) {
        if (prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) < 0) {
            child_fail(p->status_w, ST_NO_NEW_PRIVS, errno);
        }
    }

    if (p->die_before_exec) {
        /* S5 only: die between the last setup stage and execveat, with NO
         * record written, so the parent sees a clean EOF byte-identical to a
         * successful exec. This is what establishes that clean EOF alone does
         * not prove exec. */
        kill(getpid(), SIGKILL);
        _exit(126);
    }

    /* ST_EXEC */
    syscall(__NR_execveat, p->exec_fd, "", p->argv, p->envp, AT_EMPTY_PATH);
    child_fail(p->status_w, ST_EXEC, errno);
}

/* ============================================ capture prefix, binary-safe out
 * PRE-D7-B1. The bounded capture prefix was retained in memory and then freed
 * without ever being emitted, so the helper's own report -- the FIRST item in
 * the definition's evidence preference order -- never reached the harness. The
 * prefix is now written out base64-encoded, so arbitrary bytes survive a JSON
 * receipt without a length-changing escape and without a second descriptor.
 *
 * Encoding only. It changes no count, no digest and no completeness value, and
 * it adds nothing to the child's inherited descriptor set: the report still
 * arrives on descriptor 1 and the executed image still sees exactly {0,1,2}.
 *
 * The encoded bytes are INTERNAL observation input. They may contain whatever
 * the executed image wrote, so the driver decodes them, derives tokens, and
 * hands the result to the P-14 sanitiser; the published evidence never carries
 * this field. A secret in base64 is still a secret. */
static const char B64[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

static void emit_base64(const unsigned char *data, long n)
{
    long i = 0;
    while (i + 2 < n) {
        unsigned v = ((unsigned)data[i] << 16) | ((unsigned)data[i + 1] << 8) |
                     (unsigned)data[i + 2];
        putchar(B64[(v >> 18) & 63]); putchar(B64[(v >> 12) & 63]);
        putchar(B64[(v >> 6) & 63]);  putchar(B64[v & 63]);
        i += 3;
    }
    if (n - i == 1) {
        unsigned v = (unsigned)data[i] << 16;
        putchar(B64[(v >> 18) & 63]); putchar(B64[(v >> 12) & 63]);
        putchar('='); putchar('=');
    } else if (n - i == 2) {
        unsigned v = ((unsigned)data[i] << 16) | ((unsigned)data[i + 1] << 8);
        putchar(B64[(v >> 18) & 63]); putchar(B64[(v >> 12) & 63]);
        putchar(B64[(v >> 6) & 63]);  putchar('=');
    }
}

/* ====================================== M2: extra live threads in the PARENT
 * TEST/CONTROL ONLY. Never part of the candidate mechanism, and never enabled
 * without --extra-threads.
 *
 * M2's frozen note fixes this arm exactly: ">=3 extra live threads, one with an
 * allocation in flight and one with a registered pthread_atfork handler".
 * Nothing here runs after the clone: the threads exist so that the CHILD window
 * is observed from a multi-threaded parent, which is the only parent shape a
 * real HELM caller has. The child sequence must come out identical to the
 * single-threaded arm, and M2 fails if it does not.
 *
 * These threads live in the parent only. clone3 without CLONE_THREAD gives the
 * child a single thread of execution, so the post-clone window this experiment
 * traces is unchanged by their existence -- which is precisely the claim M2
 * exists to test rather than assume. */
static volatile int g_threads_stop;
static volatile int g_atfork_prepare_calls;

static void atfork_prepare(void)   { g_atfork_prepare_calls++; }
static void atfork_parent(void)    { }
static void atfork_child(void)     { }

static void *thread_allocating(void *arg)
{
    (void)arg;
    /* An allocation genuinely in flight: this thread holds and releases a
     * malloc arena continuously, so the clone lands while the allocator lock is
     * contended rather than quiescent. */
    while (!g_threads_stop) {
        void *p = malloc(4096);
        if (p) { memset(p, 0x5a, 4096); free(p); }
    }
    return NULL;
}

static void *thread_idle(void *arg)
{
    (void)arg;
    while (!g_threads_stop) {
        struct timespec ts = { 0, 1000000L };
        nanosleep(&ts, NULL);
    }
    return NULL;
}

static int start_extra_threads(int count, pthread_t *out)
{
    if (pthread_atfork(atfork_prepare, atfork_parent, atfork_child) != 0) {
        return -1;
    }
    for (int i = 0; i < count; i++) {
        void *(*fn)(void *) = (i == 0) ? thread_allocating : thread_idle;
        if (pthread_create(&out[i], NULL, fn, NULL) != 0) { return -1; }
    }
    return 0;
}

/* ============================ M5: the REJECTED fork + pidfd_open acquisition
 * TEST/CONTROL ONLY, and explicitly NOT a candidate mechanism. It runs before
 * the real mechanism, reports what it OBSERVED, and touches nothing the
 * mechanism uses.
 *
 * Its whole purpose is to evidence why clone3(CLONE_PIDFD) is primary rather
 * than asserting it: under SIGCHLD = SIG_IGN the child can be auto-reaped
 * between fork() and pidfd_open(), which is exactly the race clone3 closes by
 * returning the pidfd in the same syscall that creates the process.
 *
 * This code reports RAW observations only -- the fd, the two errno values, and
 * whether any exit status was actually observed. It maps nothing to a token:
 * that is observations.py's job, so the outcome vocabulary stays in one place. */
struct rejected_arm {
    int attempted;
    int pidfd;
    int pidfd_open_errno;
    int waitid_rc;
    int waitid_errno;
    int exit_status_observed;
    int exit_status;
};

static void run_rejected_acquisition_arm(struct rejected_arm *arm)
{
    memset(arm, 0, sizeof(*arm));
    arm->attempted = 1;
    arm->pidfd = -1;
    arm->exit_status = -1;

    pid_t pid = fork();
    if (pid < 0) { arm->pidfd_open_errno = errno; return; }
    if (pid == 0) { _exit(0); }

    /* The window clone3 removes. Under SIGCHLD=SIG_IGN the kernel may reap the
     * child before this line runs, and pidfd_open then returns ESRCH. */
    errno = 0;
    long fd = syscall(__NR_pidfd_open, (long)pid, 0L);
    arm->pidfd_open_errno = (fd < 0) ? errno : 0;
    arm->pidfd = (fd < 0) ? -1 : (int)fd;

    siginfo_t info;
    memset(&info, 0, sizeof(info));
    errno = 0;
    arm->waitid_rc = waitid(P_PID, (id_t)pid, &info, WEXITED);
    arm->waitid_errno = (arm->waitid_rc < 0) ? errno : 0;
    if (arm->waitid_rc == 0 && info.si_pid == pid && info.si_code == CLD_EXITED) {
        /* Only a status the launcher actually observed may ever be reported.
         * That is M5's gate, and it is enforced here rather than downstream. */
        arm->exit_status_observed = 1;
        arm->exit_status = info.si_status;
    }
    if (arm->pidfd >= 0) { close(arm->pidfd); }
}

/* ============================================== TEST/CONTROL ONLY: post-pin
 * A barrier the HARNESS uses to land a preregistered adversarial change on the
 * external object AFTER this process has pinned, measured and admitted it, and
 * BEFORE anything is cloned or executed.
 *
 * It exists because several frozen cases -- E2, E3, E4, E5, E6, E6b, E6c, E6d,
 * X1, S2 and S7 -- are defined by a change that must happen in exactly that
 * window, and because the caller-state cases (V1, F2, F3, F5, F6, F7, R4, T6,
 * M5) need their state proven present in this process before clone3. Trial
 * #1's driver declared those conditions and never created them, so those cases
 * ran against an unmodified object or an ordinary parent and tested nothing.
 *
 * This is NOT part of the candidate mechanism. It is inactive unless
 * --post-pin-control-fd is passed, it lives only in the parent, it never
 * touches child setup, it adds no helper descriptor, and it is CLOSED before
 * clone3 so no control descriptor can reach the child or the executed image.
 * The mutation itself is never performed here: C only synchronises, and the
 * Python harness is the actor that changes the object.
 */
#define POST_PIN_CONTROL_TIMEOUT_MS 60000

static int post_pin_barrier(int fd)
{
    /* READY: pinned, measured, admitted, nothing cloned. One byte. */
    const char ready = 'R';
    ssize_t w;
    do { w = write(fd, &ready, 1); } while (w < 0 && errno == EINTR);
    if (w != 1) { return -1; }

    /* Bounded, never indefinite: a harness that dies must not hang a trial. */
    struct pollfd p;
    p.fd = fd;
    p.events = POLLIN;
    for (;;) {
        int r = poll(&p, 1, POST_PIN_CONTROL_TIMEOUT_MS);
        if (r < 0) { if (errno == EINTR) { continue; } return -1; }
        if (r == 0) { errno = ETIMEDOUT; return -1; }
        break;
    }
    char go = 0;
    ssize_t n;
    do { n = read(fd, &go, 1); } while (n < 0 && errno == EINTR);
    if (n != 1 || go != 'C') { if (n >= 0) { errno = EPROTO; } return -1; }
    return 0;
}

/* ============================== TEST/CONTROL ONLY: preregistered caller state
 * The launcher process IS the caller whose state F3, F6 and F7 describe, so
 * those states have to exist in this process when it pins, clones and execs.
 * The harness cannot create them from outside: it can pass a descriptor, but
 * it cannot mark that descriptor CLOEXEC here, and it cannot close this
 * process's stdio without also destroying the receipt channel.
 *
 *   --parent-fd-set-cloexec N   F3: an inherited, unrelated descriptor N is made
 *                               FD_CLOEXEC in the caller before anything else.
 *   --parent-close-low-fds K    F6 (K=3) and F7 (K=1): descriptors 0..K-1 are
 *                               closed in the caller before the pin, so the
 *                               launcher's own exec fd and working-directory
 *                               capability land on them. With K >= 2 the
 *                               receipt channel is first saved above 2,
 *                               CLOEXEC, and restored only when the receipt is
 *                               written.
 *
 * Both act ONCE, before the M5 arm and before the pin, and never again. Neither
 * touches the executable object, child setup or the {0,1,2} contract, and the
 * harness proves at the post-pin barrier that each state is really present in
 * this process -- the flag is the construction, never the evidence. */
static int g_receipt_fd = -1;

static void receipt_channel(void)
{
    if (g_receipt_fd < 0) { return; }
    fflush(stdout);
    if (dup2(g_receipt_fd, 1) < 0) { _exit(2); }
    close(g_receipt_fd);
    g_receipt_fd = -1;
}

/* ==================================================================== main */
static void die(const char *msg)
{
    fprintf(stderr, "launcher_spike: %s: %s\n", msg, strerror(errno));
    exit(2);
}

int main(int argc, char **argv)
{
    const char *exec_path = NULL, *work_dir = ".";
    long timeout_ms = 5000, grace_ms = 2000, spawn_confirm_ms = 5000;
    long post_exit_drain_ms = 2000, post_fork_delay_ms = 0, stall_pre_exec_ms = 0;
    int bypass_admission = 0, exec_fd_no_cloexec = 0, skip_nnp = 0;
    int die_before_exec = 0, exec_fd_o_path = 0;
    int extra_threads = 0, rejected_arm_requested = 0;
    int post_pin_control_fd = -1;
    int parent_cloexec_fd = -1, parent_close_low_fds = 0;
    long max_capture_bytes = 64 * 1024;   /* MAX_CAPTURE_BYTES */
    char *child_argv[64];
    int child_argc = 0;

    for (int i = 1; i < argc; i++) {
        const char *a = argv[i];
        const char *v = (i + 1 < argc) ? argv[i + 1] : NULL;
        if (!strcmp(a, "--exec-path") && v) { exec_path = v; i++; }
        else if (!strcmp(a, "--work-dir") && v) { work_dir = v; i++; }
        else if (!strcmp(a, "--timeout-ms") && v) { timeout_ms = atol(v); i++; }
        else if (!strcmp(a, "--grace-ms") && v) { grace_ms = atol(v); i++; }
        else if (!strcmp(a, "--spawn-confirm-ms") && v) { spawn_confirm_ms = atol(v); i++; }
        else if (!strcmp(a, "--post-exit-drain-ms") && v) { post_exit_drain_ms = atol(v); i++; }
        else if (!strcmp(a, "--post-fork-delay-ms") && v) { post_fork_delay_ms = atol(v); i++; }
        else if (!strcmp(a, "--stall-pre-exec-ms") && v) { stall_pre_exec_ms = atol(v); i++; }
        else if (!strcmp(a, "--max-capture-bytes") && v) { max_capture_bytes = atol(v); i++; }
        else if (!strcmp(a, "--bypass-admission")) { bypass_admission = 1; }
        else if (!strcmp(a, "--exec-fd-no-cloexec")) { exec_fd_no_cloexec = 1; }
        else if (!strcmp(a, "--exec-fd-o-path")) { exec_fd_o_path = 1; }
        else if (!strcmp(a, "--skip-no-new-privs")) { skip_nnp = 1; }
        else if (!strcmp(a, "--die-before-exec")) { die_before_exec = 1; }
        /* Both TEST/CONTROL ONLY. Neither is part of the candidate mechanism,
         * and neither runs unless its flag is passed. */
        else if (!strcmp(a, "--extra-threads") && v) { extra_threads = atoi(v); i++; }
        else if (!strcmp(a, "--rejected-acquisition-arm")) { rejected_arm_requested = 1; }
        /* TEST/CONTROL ONLY, and inactive without it: an inherited descriptor
         * the harness uses to land a preregistered change after the pin. */
        else if (!strcmp(a, "--post-pin-control-fd") && v) {
            post_pin_control_fd = atoi(v); i++;
        }
        /* TEST/CONTROL ONLY, and inactive without them: preregistered caller
         * state for F3, F6 and F7. See receipt_channel() above. */
        else if (!strcmp(a, "--parent-fd-set-cloexec") && v) {
            parent_cloexec_fd = atoi(v); i++;
        }
        else if (!strcmp(a, "--parent-close-low-fds") && v) {
            parent_close_low_fds = atoi(v); i++;
        }
        else if (!strcmp(a, "--arg") && v) {
            if (child_argc < 63) { child_argv[child_argc++] = (char *)v; }
            i++;
        }
    }
    child_argv[child_argc] = NULL;
    if (!exec_path) {
        fprintf(stderr, "launcher_spike: --exec-path is required\n");
        return 2;
    }

    /* TEST/CONTROL ONLY: the preregistered caller state, established once and
     * before anything else, so the M5 arm, the pin, the clone and the exec all
     * happen from inside it. */
    if (parent_cloexec_fd >= 0) {
        if (parent_cloexec_fd < 3) { errno = EINVAL; die("parent-fd-set-cloexec"); }
        if (fcntl(parent_cloexec_fd, F_SETFD, FD_CLOEXEC) < 0) {
            die("parent-fd-set-cloexec");
        }
    }
    if (parent_close_low_fds > 0) {
        if (parent_close_low_fds > 3) { errno = EINVAL; die("parent-close-low-fds"); }
        if (parent_close_low_fds >= 2) {
            g_receipt_fd = fcntl(1, F_DUPFD_CLOEXEC, 3);
            if (g_receipt_fd < 0) { die("save receipt channel"); }
        }
        for (int fd = 0; fd < parent_close_low_fds; fd++) { close(fd); }
    }

    /* M5, TEST/CONTROL ONLY. Runs BEFORE the mechanism and shares nothing with
     * it: its own fork, its own child, its own reap. The mechanism below is
     * unchanged whether or not this ran, which is what keeps the rejected arm
     * a control rather than a variant of the thing under test. */
    struct rejected_arm rejected;
    memset(&rejected, 0, sizeof(rejected));
    if (rejected_arm_requested) { run_rejected_acquisition_arm(&rejected); }

    /* ---- the trusted caller's ONE pathname act ------------------------- */
    int open_flags = O_RDONLY | (exec_fd_no_cloexec ? 0 : O_CLOEXEC);
    if (exec_fd_o_path) {
        /* X6 only. The kernel would ACCEPT an O_PATH descriptor for execveat;
         * HELM refuses it because the body cannot be measured through one. The
         * mode exists so the refusal is testable rather than asserted. */
        open_flags = O_PATH | O_CLOEXEC;
    }
    int exec_fd = open(exec_path, open_flags);
    if (exec_fd < 0) { die("open exec_path"); }
    int dir_fd = open(work_dir, O_RDONLY | O_DIRECTORY | O_CLOEXEC);
    if (dir_fd < 0) { die("open work_dir"); }
    /* From here on NO pathname is resolved. */

    struct stat st;
    if (fstat(exec_fd, &st) < 0) { die("fstat exec_fd"); }

    char refusal[64] = "";
    if (!bypass_admission) {
        if (exec_fd_o_path) {
            /* The body is unreadable through O_PATH, and an unmeasurable
             * executable defeats the identity the receipt exists to carry. */
            snprintf(refusal, sizeof(refusal), "DescriptorModeUnsuitable");
        } else if (!S_ISREG(st.st_mode)) {
            snprintf(refusal, sizeof(refusal), "NotRegularFile");
        } else if (st.st_mode & (S_ISUID | S_ISGID)) {
            /* D-9: refuse, never record-and-permit. */
            snprintf(refusal, sizeof(refusal), "SetIdBitsPresent");
        } else {
            unsigned char hdr[64];
            ssize_t n = pread(exec_fd, hdr, sizeof(hdr), 0);
            int ok = (n >= 20) && hdr[0] == 0x7f && hdr[1] == 'E' &&
                     hdr[2] == 'L' && hdr[3] == 'F' && hdr[4] == 2 && hdr[5] == 1;
            if (ok) {
                unsigned e_type = (unsigned)hdr[16] | ((unsigned)hdr[17] << 8);
                unsigned e_machine = (unsigned)hdr[18] | ((unsigned)hdr[19] << 8);
                ok = (e_machine == 62) && (e_type == 2 || e_type == 3);
            }
            if (!ok) { snprintf(refusal, sizeof(refusal), "ElfNotInCohort"); }
        }
    }

    /* ---- measurement: pre-execution, through the SAME descriptor -------- */
    char digest[65] = "";
    if (!refusal[0]) {
        sha256 ctx;
        sha256_init(&ctx);
        unsigned char buf[65536];
        off_t off = 0;
        for (;;) {
            ssize_t n = pread(exec_fd, buf, sizeof(buf), off);
            if (n < 0) { if (errno == EINTR) { continue; } die("pread"); }
            if (n == 0) { break; }
            sha256_update(&ctx, buf, (size_t)n);
            off += n;
        }
        sha256_final(&ctx, digest);
    }

    if (refusal[0]) {
        /* An authorization refusal produces NO receipt: nothing was executed,
         * so manufacturing an execution artifact would be a lie. */
        receipt_channel();
        printf("{\"admission\":\"refused\",\"refusal\":\"%s\","
               "\"exec_reached\":false}\n", refusal);
        return 0;
    }

    /* ---- TEST/CONTROL ONLY: the post-pin barrier ------------------------
     * EXACTLY here, and the location is frozen. Above this line the executable
     * has been opened and pinned, fstat'ed and classified, measured where
     * applicable, and admitted. Below it nothing has yet been cloned, no child
     * descriptor has been set up and no image has been executed. That is the
     * only window in which the post-pin E and X cases mean what they claim, in
     * which S2/S7's directory change lands after the capability is open, and in
     * which the harness can observe this process's own descriptor table,
     * signal state and environment to prove a preregistered caller state is
     * really present. The driver's barrier set is derived from its plans.
     *
     * The control descriptor is closed immediately, well before clone3, so it
     * cannot be inherited by the child or survive into the executed image. */
    if (post_pin_control_fd >= 0) {
        if (post_pin_barrier(post_pin_control_fd) != 0) {
            die("post-pin control barrier");
        }
        close(post_pin_control_fd);
        post_pin_control_fd = -1;
    }

    /* ---- pipes, all CLOEXEC ------------------------------------------- */
    int inp[2], outp[2], errp[2], stat_p[2];
    if (pipe2(inp, O_CLOEXEC) || pipe2(outp, O_CLOEXEC) ||
        pipe2(errp, O_CLOEXEC) || pipe2(stat_p, O_CLOEXEC)) {
        die("pipe2");
    }

    /* ST_RELOCATE, in the parent before the clone: move every preserved
     * descriptor above 2 so the dup2 step and the close_range gap arithmetic
     * have no special cases. */
    exec_fd = move_above_2(exec_fd, !exec_fd_no_cloexec);
    dir_fd = move_above_2(dir_fd, 1);
    stat_p[1] = move_above_2(stat_p[1], 1);
    inp[0] = move_above_2(inp[0], 1);
    outp[1] = move_above_2(outp[1], 1);
    errp[1] = move_above_2(errp[1], 1);

    char *envp_empty[1] = { NULL };   /* D-10: the environment is exactly empty */

    struct child_plan plan;
    plan.exec_fd = exec_fd; plan.status_w = stat_p[1]; plan.dir_fd = dir_fd;
    plan.in_fd = inp[0]; plan.out_fd = outp[1]; plan.err_fd = errp[1];
    plan.argv = child_argv; plan.envp = envp_empty;
    plan.skip_no_new_privs = skip_nnp;
    plan.die_before_exec = die_before_exec;
    plan.stall_pre_exec_ms = stall_pre_exec_ms;

    /* ---- clone3(CLONE_PIDFD): atomic pidfd acquisition -----------------
     * Primary, not a refinement. pidfd_open-after-fork is sound only under
     * three conditions pidfd_open(2) states -- no SIGCHLD=SIG_IGN, no
     * SA_NOCLDWAIT, no other reaper -- and a LIBRARY cannot establish any of
     * them. clone3 also avoids the pthread_atfork handlers glibc's fork() runs
     * in the child before any launcher code. */
    /* M2, TEST/CONTROL ONLY. The extra threads exist in the PARENT so the
     * child window is observed from a multi-threaded caller. They are started
     * as late as possible -- immediately before the clone -- so nothing else in
     * the mechanism runs in a different process shape than it does normally. */
    pthread_t extra[8];
    int extra_started = 0;
    if (extra_threads > 0) {
        if (extra_threads > 8) { extra_threads = 8; }
        if (start_extra_threads(extra_threads, extra) != 0) {
            die("pthread_create");
        }
        extra_started = extra_threads;
    }

    int pidfd = -1;
    struct helm_clone_args cargs;
    memset(&cargs, 0, sizeof(cargs));
    cargs.flags = CLONE_PIDFD;
    cargs.pidfd = (uint64_t)(uintptr_t)&pidfd;
    cargs.exit_signal = SIGCHLD;

    long child = syscall(__NR_clone3, &cargs, sizeof(cargs));
    if (child < 0) { die("clone3"); }
    if (child == 0) {
        child_main(&plan);
        _exit(127); /* unreachable */
    }

    setpgid((pid_t)child, (pid_t)child); /* ignore EACCES: the child may have won */

    if (post_fork_delay_ms > 0) {
        /* S4 only: a FORCED schedule, so the child has certainly execed and
         * exited before the parent observes anything. Not a timing hope. */
        struct timespec ts;
        ts.tv_sec = post_fork_delay_ms / 1000;
        ts.tv_nsec = (post_fork_delay_ms % 1000) * 1000000L;
        nanosleep(&ts, NULL);
    }

    /* Parent closes its OWN copies immediately. The child's CLOEXEC closures
     * are necessary and NOT sufficient: a pipe reaches EOF only when every
     * descriptor referring to the write end is closed, and the clone gave the
     * parent its own copy of each. */
    close(stat_p[1]); close(outp[1]); close(errp[1]); close(inp[0]);
    close(inp[1]);    /* stdin: immediate EOF for the child */

    long start = now_ms();
    long deadline = start + spawn_confirm_ms;   /* phase 1 */
    /* O5 instrumentation: every return of the drain poll() below, counted
     * without judgement. A spinning launcher also satisfies "no hang", so the
     * frozen bound of 10000 returns is checked by the harness from this count.
     * It is an instrumentation fact, not a receipt field, and it stays outside
     * the receipt exactly as the elapsed time does. */
    long poll_returns = 0;
    int exec_confirmed = 0, exec_failed = 0, status_short = 0, timed_out = 0;
    struct exec_status rec; memset(&rec, 0, sizeof(rec));
    size_t rec_have = 0;

    long out_bytes = 0, err_bytes = 0;
    sha256 out_ctx, err_ctx;
    sha256_init(&out_ctx); sha256_init(&err_ctx);

    /* capture_prefix: the retained prefix lives IN MEMORY ONLY and is never
     * serialised into the receipt. Draining continues past the bound, so a
     * child that writes more than the bound is never blocked by a full pipe --
     * the excess is discarded, not withheld. `truncated` describes THIS BUFFER,
     * not the stream, which is why it never appears in the receipt. */
    unsigned char *out_prefix = malloc((size_t)max_capture_bytes);
    unsigned char *err_prefix = malloc((size_t)max_capture_bytes);
    if (!out_prefix || !err_prefix) { die("malloc capture buffers"); }
    long out_kept = 0, err_kept = 0;
    int out_truncated = 0, err_truncated = 0;
    int out_open = 1, err_open = 1, status_open = 1, pid_ready = 0;
    long child_end_ms = -1;

    unsigned char buf[65536];
    for (;;) {
        struct pollfd fds[4];
        int idx[4], n = 0;
        if (status_open) { fds[n].fd = stat_p[0]; fds[n].events = POLLIN; idx[n] = 0; n++; }
        if (out_open)    { fds[n].fd = outp[0];   fds[n].events = POLLIN; idx[n] = 1; n++; }
        if (err_open)    { fds[n].fd = errp[0];   fds[n].events = POLLIN; idx[n] = 2; n++; }
        if (!pid_ready)  { fds[n].fd = pidfd;     fds[n].events = POLLIN; idx[n] = 3; n++; }
        if (n == 0) { break; }

        long remaining = deadline - now_ms();
        if (remaining < 0) { remaining = 0; }
        int pr = poll(fds, (nfds_t)n, (int)remaining);
        poll_returns++;
        if (pr < 0) { if (errno == EINTR) { continue; } die("poll"); }

        if (pr == 0) {
            if (!exec_confirmed && !exec_failed) { break; }      /* pre-exec bound */
            if (!pid_ready && !timed_out) {                      /* timeout path */
                syscall(__NR_pidfd_send_signal, pidfd, SIGTERM, NULL, 0u);
                deadline = now_ms() + grace_ms;
                timed_out = 1;
                continue;
            }
            break;
        }

        for (int i = 0; i < n; i++) {
            if (!fds[i].revents) { continue; }
            if (idx[i] == 3) { pid_ready = 1; child_end_ms = now_ms(); continue; }

            int fd = fds[i].fd;
            ssize_t got = read(fd, buf, sizeof(buf));
            if (got < 0) { if (errno == EINTR) { continue; } got = 0; }

            if (idx[i] == 0) {
                if (got > 0) {
                    size_t take = sizeof(rec) - rec_have;
                    if ((size_t)got < take) { take = (size_t)got; }
                    memcpy((uint8_t *)&rec + rec_have, buf, take);
                    rec_have += take;
                } else {
                    /* POLLHUP is never acted on before read() returns 0: a
                     * child that writes the record and _exits sets POLLIN and
                     * POLLHUP in the same return, and reading hangup first
                     * would report exec success for a child that never execed. */
                    status_open = 0;
                    if (rec_have == sizeof(rec)) { exec_failed = 1; }
                    else if (rec_have > 0) { status_short = 1; }
                    else { exec_confirmed = 1; deadline = now_ms() + timeout_ms; }
                }
            } else if (idx[i] == 1) {
                if (got > 0) {
                    out_bytes += got;
                    sha256_update(&out_ctx, buf, (size_t)got);
                    long room = max_capture_bytes - out_kept;
                    if (room > 0) {
                        long take = got < room ? got : room;
                        memcpy(out_prefix + out_kept, buf, (size_t)take);
                        out_kept += take;
                    }
                    if (out_kept >= max_capture_bytes && got > 0) {
                        out_truncated = (out_bytes > max_capture_bytes);
                    }
                } else { out_open = 0; }
            } else {
                if (got > 0) {
                    err_bytes += got;
                    sha256_update(&err_ctx, buf, (size_t)got);
                    long room = max_capture_bytes - err_kept;
                    if (room > 0) {
                        long take = got < room ? got : room;
                        memcpy(err_prefix + err_kept, buf, (size_t)take);
                        err_kept += take;
                    }
                    if (err_kept >= max_capture_bytes && got > 0) {
                        err_truncated = (err_bytes > max_capture_bytes);
                    }
                } else { err_open = 0; }
            }
        }

        if (exec_failed || status_short) { break; }
        if (pid_ready && !out_open && !err_open) { break; }
        if (pid_ready && child_end_ms >= 0) {
            /* Bounded post-exit drain: a descendant may hold a write end, in
             * which case the pipes never reach EOF and the launcher's own
             * liveness would otherwise depend on a process it disclaims. */
            long drain_deadline = child_end_ms + post_exit_drain_ms;
            if (deadline > drain_deadline) { deadline = drain_deadline; }
            if (now_ms() >= drain_deadline) { break; }
        }
    }

    /* Sweep STRICTLY BEFORE the reap: a zombie keeps its process group alive,
     * so the pgid cannot have been recycled yet. After the reap it may have
     * been, and signalling it would reach processes never created here. */
    int sweep_issued = 0;
    if (!pid_ready) {
        syscall(__NR_pidfd_send_signal, pidfd, SIGKILL, NULL, 0u);
    }
    kill(-(pid_t)child, SIGKILL);
    sweep_issued = 1;

    siginfo_t info;
    memset(&info, 0, sizeof(info));
    int reaped = waitid(P_PIDFD, (id_t)pidfd, &info, WEXITED);
    int wait_errno = reaped < 0 ? errno : 0;

    char out_hex[65], err_hex[65];
    sha256_final(&out_ctx, out_hex);
    sha256_final(&err_ctx, err_hex);

    const char *out_completeness = out_open ? "WriterRetainedAfterChildExit"
                                            : "CompleteAtEof";
    const char *err_completeness = err_open ? "WriterRetainedAfterChildExit"
                                            : "CompleteAtEof";

    /* Classification is from what was OBSERVED, never from what the launcher
     * did. TimedOut is emitted only when the deadline expired while the pidfd
     * was still not readable; a child whose pidfd became readable first is
     * Exited or Signaled even if a signal was later issued at a corpse, because
     * waitid reports a signal number and not a sender. */
    const char *disposition;
    const char *timeout_disposition = "";
    if (exec_failed) { disposition = "ExecFailed"; }
    else if (status_short) { disposition = "ExecStatusIndeterminate"; }
    else if (!exec_confirmed) { disposition = "ExecStatusIndeterminate"; }
    else if (reaped < 0 && wait_errno == ECHILD) { disposition = "ExitStatusUnobservable"; }
    else if (timed_out) {
        disposition = "TimedOut";
        if (reaped < 0) { timeout_disposition = "TerminationFailed"; }
        else if (info.si_code == CLD_EXITED) { timeout_disposition = "ExitedDuringGrace"; }
        else { timeout_disposition = "KilledByLauncher"; }
    }
    else if (info.si_code == CLD_EXITED) { disposition = "Exited"; }
    else { disposition = "Signaled"; }

    receipt_channel();
    printf("{\"admission\":\"accepted\","
           "\"pre_exec_body_sha256\":\"%s\","
           "\"pre_exec_body_size\":%lld,"
           "\"pre_exec_mode_bits\":%u,"
           "\"process_disposition\":\"%s\","
           "\"timeout_disposition\":\"%s\","
           "\"exec_failed_stage\":\"%s\",\"exec_failed_errno\":%d,"
           "\"exit_code\":%d,\"term_signal\":%d,"
           "\"launcher_signal_issued\":%s,"
           "\"group_sweep_issued\":%s,"
           "\"wait_errno\":%d,",
           digest, (long long)st.st_size, (unsigned)(st.st_mode & 07777),
           disposition, timeout_disposition,
           exec_failed ? stage_name(rec.stage) : "", exec_failed ? rec.err : 0,
           info.si_code == CLD_EXITED ? info.si_status : -1,
           info.si_code == CLD_KILLED ? info.si_status : -1,
           timed_out ? "true" : "false",
           sweep_issued ? "true" : "false",
           wait_errno);

    /* PRE-D7-B1: each stream carries its counts, its digest, its completeness
     * and the RETAINED PREFIX, base64-encoded. The counts and the digest are
     * over every drained byte and are unchanged; the prefix is bounded by
     * max_capture_bytes and is a strict prefix of the same stream. Emitting it
     * adds no descriptor to the child and alters no measurement -- it only
     * stops the launcher from discarding the evidence it already collected. */
    printf("\"stdout\":{\"bytes_drained\":%ld,\"drained_sha256\":\"%s\","
           "\"completeness\":\"%s\",\"capture_prefix_length\":%ld,"
           "\"capture_prefix_truncated\":%s,\"capture_prefix_base64\":\"",
           out_bytes, out_hex, out_completeness, out_kept,
           out_truncated ? "true" : "false");
    emit_base64(out_prefix, out_kept);
    printf("\"},");

    printf("\"stderr\":{\"bytes_drained\":%ld,\"drained_sha256\":\"%s\","
           "\"completeness\":\"%s\",\"capture_prefix_length\":%ld,"
           "\"capture_prefix_truncated\":%s,\"capture_prefix_base64\":\"",
           err_bytes, err_hex, err_completeness, err_kept,
           err_truncated ? "true" : "false");
    emit_base64(err_prefix, err_kept);
    printf("\"},");

    /* M5's RAW observations. No token is derived here: the outcome vocabulary
     * lives in observations.py so that one file owns the whole mapping. The
     * exit status appears only when the launcher actually observed it, which is
     * M5's gate rather than a convention. */
    printf("\"rejected_acquisition_arm\":{\"attempted\":%s,\"pidfd\":%d,"
           "\"pidfd_open_errno\":%d,\"waitid_rc\":%d,\"waitid_errno\":%d,"
           "\"exit_status_observed\":%s,\"exit_status\":%d},",
           rejected.attempted ? "true" : "false", rejected.pidfd,
           rejected.pidfd_open_errno, rejected.waitid_rc, rejected.waitid_errno,
           rejected.exit_status_observed ? "true" : "false",
           rejected.exit_status_observed ? rejected.exit_status : -1);

    /* M2's parent shape, recorded as a fact about THIS run so a comparison
     * between the two arms can never be made against an unknown shape. */
    printf("\"parent_shape\":{\"extra_threads\":%d,"
           "\"atfork_handler_registered\":%s,\"atfork_prepare_calls\":%d},",
           extra_started, extra_started > 0 ? "true" : "false",
           g_atfork_prepare_calls);

    printf("\"environment_mode\":\"empty\","
           "\"retained_prefix_not_in_receipt\":{"
           "\"stdout_kept\":%ld,\"stdout_truncated\":%s,"
           "\"stderr_kept\":%ld,\"stderr_truncated\":%s,\"bound\":%ld},"
           "\"poll_returns_not_in_receipt\":%ld,"
           "\"elapsed_ms_not_in_receipt\":%ld}\n",
           out_kept, out_truncated ? "true" : "false",
           err_kept, err_truncated ? "true" : "false", max_capture_bytes,
           poll_returns, now_ms() - start);

    g_threads_stop = 1;
    for (int t = 0; t < extra_started; t++) { pthread_join(extra[t], NULL); }
    free(out_prefix);
    free(err_prefix);
    return 0;
}
