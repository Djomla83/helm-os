/* OBS-FS-01 disposable syscall spike. NOT helm-observe product code.
 *
 * Mechanism under test (frozen Amendment 1, section 8 of the execution
 * authorisation):
 *   authorized root fd -> openat2(O_PATH|O_NOFOLLOW, frozen RESOLVE policy)
 *   -> statx classify on the pinned fd -> size/aggregate budget gate
 *   -> procfs-fd reopen for a permitted regular file only -> bounded read
 *   -> post-read consistency check -> result.
 *
 * No pathname fallback, no recursion, no comparison against expected values.
 * Never prints a path: diagnostics are fixed codes and logical target IDs.
 *
 * Build: gcc -O2 -Wall -Wextra -o spike spike.c
 */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/vfs.h>
#include <unistd.h>
#include <linux/stat.h>

#ifndef __NR_openat2
#define __NR_openat2 437
#endif
#ifndef RESOLVE_NO_XDEV
#define RESOLVE_NO_XDEV       0x01
#define RESOLVE_NO_MAGICLINKS 0x02
#define RESOLVE_NO_SYMLINKS   0x04
#define RESOLVE_BENEATH       0x08
#endif
#ifndef STATX_MNT_ID
#define STATX_MNT_ID 0x00001000U
#endif
#ifndef STATX_MNT_ID_UNIQUE
#define STATX_MNT_ID_UNIQUE 0x00004000U
#endif
#define PROC_SUPER_MAGIC 0x9fa0

struct open_how_l { uint64_t flags, mode, resolve; };

static long sys_openat2(int dfd, const char *p, struct open_how_l *h)
{
    return syscall(__NR_openat2, dfd, p, h, sizeof(*h));
}

#define FILE_CEILING   (512ULL << 20)   /* frozen per-file ceiling  */
#define TOTAL_CEILING  (1ULL << 30)     /* frozen aggregate ceiling */
#define READ_BUF       (64 * 1024)      /* frozen read buffer       */
#define MAX_TARGETS    64
#define MAX_PATH_BYTES 1024
#define MAX_COMPONENTS 32
#define MAX_ID         80

static const char *kind_of(unsigned int m)
{
    switch (m & S_IFMT) {
    case S_IFREG:  return "regular";
    case S_IFDIR:  return "directory";
    case S_IFLNK:  return "symlink";
    case S_IFIFO:  return "fifo";
    case S_IFSOCK: return "socket";
    case S_IFCHR:  return "chardev";
    case S_IFBLK:  return "blockdev";
    default:       return "unknown";
    }
}

static const char *resolve_outcome(int e)
{
    switch (e) {
    case ENOENT:  return "absent";
    case ENOTDIR: return "wrong_kind";
    case EACCES: case EPERM: return "permission_denied";
    case ELOOP:   return "symlink_forbidden";
    case EXDEV:   return "mount_crossing";   /* also covers a BENEATH escape */
    case EAGAIN:  return "resolution_race";
    default:      return "io_failure";
    }
}

static const char *errno_name(int e)
{
    switch (e) {
    case 0: return "OK";
    case ENOENT: return "ENOENT"; case ENOTDIR: return "ENOTDIR";
    case EACCES: return "EACCES"; case EPERM: return "EPERM";
    case ELOOP: return "ELOOP";   case EXDEV: return "EXDEV";
    case EAGAIN: return "EAGAIN"; case ENXIO: return "ENXIO";
    case EIO: return "EIO";       case EBADF: return "EBADF";
    case EISDIR: return "EISDIR"; case ENOMEM: return "ENOMEM";
    default: return "OTHER";
    }
}

struct target { char id[MAX_ID + 1]; char op[8]; char rel[MAX_PATH_BYTES + 1]; int has_rel; };
/* sscanf fields are wider than the stored fields on purpose: over-long input
 * must be rejected by validation, never silently truncated into storage. */
static struct target targets[MAX_TARGETS];
static int ntargets;

/* Inert plan validation. Runs before any filesystem access. */
static const char *validate_rel(const char *p)
{
    size_t n = strlen(p);
    if (n == 0 || n > MAX_PATH_BYTES) return "path_length";
    if (p[0] == '/') return "absolute_path";
    if (strchr(p, '\\')) return "backslash";
    int comps = 0;
    size_t i = 0;
    while (i < n) {
        size_t j = i;
        while (j < n && p[j] != '/') j++;
        size_t len = j - i;
        if (len == 0) return "empty_component";
        if (len > 255) return "component_length";
        if (len == 1 && p[i] == '.') return "dot_component";
        if (len == 2 && p[i] == '.' && p[i + 1] == '.') return "parent_component";
        if (p[i] == ' ') return "leading_space";
        if (p[j - 1] == ' ' || p[j - 1] == '.') return "trailing_space_or_dot";
        for (size_t k = i; k < j; k++) {
            unsigned char c = (unsigned char)p[k];
            if (c < 0x20 || c > 0x7e) return "non_portable_byte";
        }
        if (++comps > MAX_COMPONENTS) return "component_count";
        i = (j < n) ? j + 1 : j;
    }
    return NULL;
}

static const char *load_plan(const char *path)
{
    FILE *f = fopen(path, "r");
    if (!f) return "plan_unreadable";
    char line[2048];
    while (fgets(line, sizeof line, f)) {
        char *nl = strchr(line, '\n');
        if (nl) *nl = 0;
        if (!line[0] || line[0] == '#') continue;
        char directive[64], id[256], op[64], rel[2048];
        int got = sscanf(line, "%63s %255s %63s %2047s", directive, id, op, rel);
        if (got < 3) { fclose(f); return "plan_syntax"; }
        /* Closed vocabulary: any other directive, including a descriptor-shaped
         * field such as "fd" or "root_fd", is rejected without any I/O. */
        if (strcmp(directive, "target") != 0) { fclose(f); return "unknown_directive"; }
        if (ntargets >= MAX_TARGETS) { fclose(f); return "target_count"; }
        if (strlen(id) > MAX_ID) { fclose(f); return "id_length"; }
        for (const char *c = id; *c; c++)
            if (!((*c >= 'a' && *c <= 'z') || (*c >= '0' && *c <= '9') ||
                  *c == '.' || *c == '_' || *c == '-')) { fclose(f); return "id_grammar"; }
        for (int k = 0; k < ntargets; k++)
            if (!strcmp(targets[k].id, id)) { fclose(f); return "duplicate_id"; }
        if (strcmp(op, "file") && strcmp(op, "dir")) { fclose(f); return "unknown_operation"; }
        struct target *t = &targets[ntargets];
        memcpy(t->id, id, strlen(id) + 1);
        memcpy(t->op, op, strlen(op) + 1);
        if (got >= 4) {
            if (strlen(rel) > MAX_PATH_BYTES) { fclose(f); return "path_length"; }
            const char *bad = validate_rel(rel);
            if (bad) { fclose(f); return bad; }
            memcpy(t->rel, rel, strlen(rel) + 1);
            t->has_rel = 1;
        } else {
            t->has_rel = 0;   /* root-directory metadata operation */
        }
        ntargets++;
    }
    fclose(f);
    if (ntargets == 0) return "empty_plan";
    return NULL;
}

struct out {
    const char *id, *stage, *outcome, *kind;
    int err;
    long long size, nlink, bytes;
    unsigned long long ino, mnt_id;
    unsigned int dev_major, dev_minor;
    int reopened, identity_match, change_detected, mnt_unique;
};

static void emit(const struct out *o)
{
    printf("{\"target\":\"%s\",\"stage\":\"%s\",\"outcome\":\"%s\",\"errno\":%d,"
           "\"errno_name\":\"%s\",\"kind\":\"%s\",\"size\":%lld,\"nlink\":%lld,"
           "\"ino\":%llu,\"dev_major\":%u,\"dev_minor\":%u,\"mnt_id\":%llu,"
           "\"mnt_id_unique_supported\":%s,\"bytes_read\":%lld,\"reopened\":%s,"
           "\"identity_match\":%s,\"change_detected\":%s}\n",
           o->id ? o->id : "-", o->stage, o->outcome, o->err, errno_name(o->err),
           o->kind ? o->kind : "none", o->size, o->nlink, o->ino,
           o->dev_major, o->dev_minor, o->mnt_id,
           o->mnt_unique ? "true" : "false", o->bytes,
           o->reopened ? "true" : "false", o->identity_match ? "true" : "false",
           o->change_detected ? "true" : "false");
    fflush(stdout);
}

static void admission_fail(const char *code, int err)
{
    printf("{\"admission\":\"%s\",\"errno\":%d,\"errno_name\":\"%s\"}\n",
           code, err, errno_name(err));
    fflush(stdout);
}

/* Frozen procfs admission: filesystem-type validation plus a self-fd identity
 * probe against an object this process already controls. Metadata only, and it
 * never probes a plan-controlled path. */
static int admit_procfs(int procfd, int knownfd)
{
    struct statfs sfs;
    if (fstatfs(procfd, &sfs) < 0) return 0;
    if ((long)sfs.f_type != PROC_SUPER_MAGIC) return 0;
    struct stat a;
    if (fstat(knownfd, &a) < 0) return 0;
    char num[32];
    snprintf(num, sizeof num, "%d", knownfd);
    /* Deliberately WITHOUT O_NOFOLLOW: this magic link must be traversed so
     * the probe observes the pinned object, not the link. O_NOFOLLOW here
     * would return the magic link itself and the identity check would
     * always fail. */
    int probe = openat(procfd, num, O_PATH | O_CLOEXEC);
    if (probe < 0) return 0;
    struct stat b;
    int ok = (fstat(probe, &b) == 0 && a.st_dev == b.st_dev && a.st_ino == b.st_ino);
    close(probe);
    return ok;
}

static void usage(void)
{
    fprintf(stderr,
        "spike --root DIR (--plan FILE | --rel PATH [--op file|dir] [--id ID])\n"
        "      [--proc DIR] [--dump-prefix P] [--budget BYTES] [--allow-xdev]\n"
        "      [--direct] [--nonblock] [--skip-proc-admission]\n");
    exit(2);
}

int main(int argc, char **argv)
{
    const char *root = NULL, *plan = NULL, *rel = NULL, *op = "file", *id = "t1";
    const char *procdir = "/proc/self/fd", *dumpp = NULL;
    unsigned long long budget = TOTAL_CEILING;
    int allow_xdev = 0, direct = 0, nonblock = 0, skip_admit = 0;

    for (int i = 1; i < argc; i++) {
        if (!strcmp(argv[i], "--root") && i + 1 < argc) root = argv[++i];
        else if (!strcmp(argv[i], "--plan") && i + 1 < argc) plan = argv[++i];
        else if (!strcmp(argv[i], "--rel") && i + 1 < argc) rel = argv[++i];
        else if (!strcmp(argv[i], "--op") && i + 1 < argc) op = argv[++i];
        else if (!strcmp(argv[i], "--id") && i + 1 < argc) id = argv[++i];
        else if (!strcmp(argv[i], "--proc") && i + 1 < argc) procdir = argv[++i];
        else if (!strcmp(argv[i], "--dump-prefix") && i + 1 < argc) dumpp = argv[++i];
        else if (!strcmp(argv[i], "--budget") && i + 1 < argc) budget = strtoull(argv[++i], NULL, 10);
        else if (!strcmp(argv[i], "--allow-xdev")) allow_xdev = 1;
        else if (!strcmp(argv[i], "--direct")) direct = 1;
        else if (!strcmp(argv[i], "--nonblock")) nonblock = 1;
        else if (!strcmp(argv[i], "--skip-proc-admission")) skip_admit = 1;
        else usage();
    }
    if (!root || (!plan && !rel)) usage();

    /* Stage 1: inert plan validation, before any filesystem access. */
    if (plan) {
        const char *bad = load_plan(plan);
        if (bad) { admission_fail(bad, 0); return 2; }
    } else {
        if (strlen(rel) > MAX_PATH_BYTES || strlen(id) > MAX_ID || strlen(op) > 4) {
            admission_fail("argument_length", 0); return 2; }
        const char *bad = validate_rel(rel);
        if (bad) { admission_fail(bad, 0); return 2; }
        memcpy(targets[0].id, id, strlen(id) + 1);
        memcpy(targets[0].op, op, strlen(op) + 1);
        memcpy(targets[0].rel, rel, strlen(rel) + 1);
        targets[0].has_rel = 1;
        ntargets = 1;
    }

    int rootfd = open(root, O_PATH | O_DIRECTORY | O_CLOEXEC);
    if (rootfd < 0) { admission_fail("root_unavailable", errno); return 3; }

    uint64_t resolve = RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS;
    if (!allow_xdev) resolve |= RESOLVE_NO_XDEV;

    /* Stage 2: procfs capability admission, before any target read. */
    int procfd = -1;
    if (!direct) {
        procfd = open(procdir, O_PATH | O_DIRECTORY | O_CLOEXEC);
        if (procfd < 0) { admission_fail("procfs_unavailable", errno); return 6; }
        if (!skip_admit && !admit_procfs(procfd, rootfd)) {
            admission_fail("procfs_rejected", 0);
            return 6;
        }
    }

    unsigned long long spent = 0;
    int budget_exhausted = 0;

    for (int ti = 0; ti < ntargets; ti++) {
        struct target *t = &targets[ti];
        struct out o;
        memset(&o, 0, sizeof o);
        o.id = t->id; o.kind = "none";

        if (budget_exhausted) {
            o.stage = "budget"; o.outcome = "not_attempted_total_limit";
            emit(&o); continue;
        }

        /* D1/D2 comparison arm: deliberately the rejected strategy B. */
        if (direct) {
            struct open_how_l dh = {
                .flags = (uint64_t)(O_RDONLY | O_CLOEXEC | O_NOCTTY | (nonblock ? O_NONBLOCK : 0)),
                .mode = 0, .resolve = resolve };
            o.stage = "direct_open";
            long dfd = sys_openat2(rootfd, t->rel, &dh);
            if (dfd < 0) { o.err = errno; o.outcome = resolve_outcome(errno); emit(&o); continue; }
            struct statx dx;
            if (statx((int)dfd, "", AT_EMPTY_PATH, STATX_BASIC_STATS, &dx) == 0) {
                o.kind = kind_of(dx.stx_mode);
                o.size = (long long)dx.stx_size;
            }
            o.outcome = "direct_data_open_succeeded";
            o.reopened = 1;
            emit(&o);
            close((int)dfd);
            continue;
        }

        struct open_how_l h = { .flags = (uint64_t)(O_PATH | O_NOFOLLOW | O_CLOEXEC),
                                .mode = 0, .resolve = resolve };
        o.stage = "resolve";
        long pin = t->has_rel ? sys_openat2(rootfd, t->rel, &h) : dup(rootfd);
        if (pin < 0) { o.err = errno; o.outcome = resolve_outcome(errno); emit(&o); continue; }

        struct statx sx;
        o.stage = "classify";
        if (statx((int)pin, "", AT_EMPTY_PATH,
                  STATX_BASIC_STATS | STATX_MNT_ID | STATX_MNT_ID_UNIQUE, &sx) < 0) {
            o.err = errno; o.outcome = "io_failure"; emit(&o); close((int)pin); continue;
        }
        o.kind = kind_of(sx.stx_mode);
        o.size = (long long)sx.stx_size;
        o.nlink = (long long)sx.stx_nlink;
        o.ino = (unsigned long long)sx.stx_ino;
        o.dev_major = sx.stx_dev_major;
        o.dev_minor = sx.stx_dev_minor;
        o.mnt_id = (unsigned long long)sx.stx_mnt_id;
        o.mnt_unique = (sx.stx_mask & STATX_MNT_ID_UNIQUE) ? 1 : 0;

        if (!strcmp(t->op, "dir")) {
            o.outcome = S_ISDIR(sx.stx_mode) ? "observed_directory" : "wrong_kind";
            emit(&o); close((int)pin); continue;
        }
        if (!S_ISREG(sx.stx_mode)) {
            /* Metadata-only rejection: no data-open, no read, no wait, no connect. */
            o.outcome = S_ISDIR(sx.stx_mode) ? "wrong_kind"
                      : S_ISLNK(sx.stx_mode) ? "symlink_forbidden" : "special_file";
            emit(&o); close((int)pin); continue;
        }

        /* Stage 3: bound BEFORE any data-open. */
        o.stage = "budget";
        if ((unsigned long long)sx.stx_size > FILE_CEILING) {
            o.outcome = "file_limit"; emit(&o); close((int)pin); continue;
        }
        unsigned long long need = (unsigned long long)sx.stx_size + 1ULL;
        if (spent + need > budget) {
            o.outcome = "total_limit"; budget_exhausted = 1;
            emit(&o); close((int)pin); continue;
        }

        /* Stage 4: reopen exactly the pinned object through the procfs
         * capability. Deliberate magic-link traversal: the frozen RESOLVE
         * policy applies to target resolution, not to this reopen. */
        o.stage = "reopen";
        char num[32];
        snprintf(num, sizeof num, "%d", (int)pin);
        int rfd = openat(procfd, num, O_RDONLY | O_CLOEXEC | O_NONBLOCK | O_NOCTTY);
        if (rfd < 0) {
            o.err = errno;
            o.outcome = (errno == EACCES || errno == EPERM) ? "permission_denied"
                                                            : "reopen_unavailable";
            emit(&o); close((int)pin); continue;
        }
        o.reopened = 1;

        struct statx rx;
        if (statx(rfd, "", AT_EMPTY_PATH, STATX_BASIC_STATS, &rx) < 0) {
            o.err = errno; o.outcome = "io_failure";
            emit(&o); close(rfd); close((int)pin); continue;
        }
        o.identity_match = (rx.stx_ino == sx.stx_ino &&
                            rx.stx_dev_major == sx.stx_dev_major &&
                            rx.stx_dev_minor == sx.stx_dev_minor &&
                            S_ISREG(rx.stx_mode));
        if (!o.identity_match) {
            o.outcome = "io_failure";
            emit(&o); close(rfd); close((int)pin); continue;
        }

        /* Stage 5: bounded stream from offset zero through that one handle. */
        o.stage = "read";
        int df = -1;
        if (dumpp) {
            char dp[512];
            snprintf(dp, sizeof dp, "%s%s.bin", dumpp, t->id);
            df = open(dp, O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC, 0600);
        }
        static unsigned char buf[READ_BUF];
        long long total = 0;
        unsigned long long limit = (unsigned long long)sx.stx_size;
        int failed = 0;
        for (;;) {
            if ((unsigned long long)total >= limit + 1ULL) break;
            size_t want = READ_BUF;
            unsigned long long remain = limit + 1ULL - (unsigned long long)total;
            if (remain < want) want = (size_t)remain;
            ssize_t n = read(rfd, buf, want);
            if (n < 0) { o.err = errno; failed = 1; break; }
            if (n == 0) break;
            if (df >= 0 && (unsigned long long)total < limit) {
                size_t w = (size_t)n;
                if ((unsigned long long)total + w > limit)
                    w = (size_t)(limit - (unsigned long long)total);
                if (write(df, buf, w) < 0) { o.err = errno; failed = 1; break; }
            }
            total += n;
        }
        if (df >= 0) close(df);
        o.bytes = total;
        spent += (unsigned long long)total;   /* charge actual bytes, including failures */

        struct statx px;
        o.stage = "post_check";
        if (statx(rfd, "", AT_EMPTY_PATH, STATX_BASIC_STATS, &px) < 0) {
            o.err = errno; o.outcome = "io_failure";
            emit(&o); close(rfd); close((int)pin); continue;
        }
        o.change_detected = (px.stx_size != sx.stx_size ||
                             px.stx_nlink != sx.stx_nlink ||
                             px.stx_mtime.tv_sec != sx.stx_mtime.tv_sec ||
                             px.stx_mtime.tv_nsec != sx.stx_mtime.tv_nsec ||
                             px.stx_ctime.tv_sec != sx.stx_ctime.tv_sec ||
                             px.stx_ctime.tv_nsec != sx.stx_ctime.tv_nsec);

        if (failed) o.outcome = "io_failure";
        else if (o.change_detected || (unsigned long long)total != limit)
            o.outcome = "changed_during_read";
        else o.outcome = "observed_file";
        emit(&o);
        close(rfd);
        close((int)pin);
    }
    close(rootfd);
    if (procfd >= 0) close(procfd);
    return 0;
}
