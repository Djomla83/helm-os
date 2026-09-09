/* LAUNCH-EXEC-01 set-user-ID admission fixture (owner decision D-9).
 *
 * Disposable spike code. This helper exists to be REFUSED, not to run.
 *
 * D-9 refuses any object whose admission metadata carries S_ISUID or S_ISGID,
 * with the bounded typed refusal SetIdBitsPresent. Case X7 is therefore a
 * deterministic admission-refusal test: execveat must never be reached, and the
 * case rejects the mechanism outright if it is.
 *
 * The fixture is built by chmod u+s on a file the RUNNING USER ALREADY OWNS.
 * No privileged fixture is created, no cross-UID elevation is manufactured, and
 * no sudo is used. Setting the set-user-ID bit on your own file grants nothing:
 * the owner is the caller, so even if it did execute, no privilege transition
 * would occur. That is exactly why the real privilege-transition suppression of
 * D-11 is a separate, deliberately BLOCKED case (N3) and is never claimed to
 * have been demonstrated here.
 *
 * If this program ever runs, the experiment has already failed X7.
 *
 * Build: cc -O2 -Wall -Wextra -static -o helper_setid helper_setid.c
 *        chmod u+s helper_setid
 */
#define _GNU_SOURCE
#include <string.h>
#include <unistd.h>

int main(void)
{
    static const char msg[] =
        "{\"marker\":\"helper_setid\",\"note\":\"X7 must refuse this at "
        "admission; reaching exec is a mechanism rejection\"}\n";
    ssize_t ignored = write(1, msg, strlen(msg));
    (void)ignored;
    return 0;
}
