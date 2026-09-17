//! Pure descriptor-layout planning (plan sections 8.2, 9.1; T18, T19).
//!
//! Descriptor numbers are **inert integers** here. Nothing in this module owns,
//! opens, duplicates, closes or inspects a descriptor, and nothing reads the
//! state of the host process. It computes, from numbers alone, the relocation
//! steps a later backend must perform, the final stdio mapping, and the ranges a
//! later backend must close so that exactly descriptors 0, 1 and 2 remain.
//!
//! In P1 no code consumes this plan; only tests do.

#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "P1 has no backend; the planner is exercised by tests only"
    )
)]

/// Every child-side descriptor is relocated to at least this number.
pub(crate) const FIRST_RELOCATED: u32 = 3;
/// The highest descriptor number a range may name (`~0U`).
pub(crate) const HIGHEST: u32 = u32::MAX;

/// The six descriptors the child needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Role {
    Executable,
    WorkingDirectory,
    StdinRead,
    StdoutWrite,
    StderrWrite,
    StatusWrite,
}

pub(crate) const ROLES: [Role; 6] = [
    Role::Executable,
    Role::WorkingDirectory,
    Role::StdinRead,
    Role::StdoutWrite,
    Role::StderrWrite,
    Role::StatusWrite,
];

/// Descriptor numbers by role, as inert integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Descriptors {
    pub(crate) executable: u32,
    pub(crate) working_directory: u32,
    pub(crate) stdin_read: u32,
    pub(crate) stdout_write: u32,
    pub(crate) stderr_write: u32,
    pub(crate) status_write: u32,
}

impl Descriptors {
    pub(crate) const fn get(&self, role: Role) -> u32 {
        match role {
            Role::Executable => self.executable,
            Role::WorkingDirectory => self.working_directory,
            Role::StdinRead => self.stdin_read,
            Role::StdoutWrite => self.stdout_write,
            Role::StderrWrite => self.stderr_write,
            Role::StatusWrite => self.status_write,
        }
    }
}

/// One unconditional relocation: duplicate `source` close-on-exec to the lowest
/// free number at or above `minimum`, then close `source`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Relocation {
    pub(crate) role: Role,
    pub(crate) source: u32,
    pub(crate) minimum: u32,
}

/// T19: every child-side descriptor is relocated, whatever its number and
/// close-on-exec state. There is no "already high enough" shortcut, because a
/// caller-supplied descriptor of unknown close-on-exec state must never reach
/// the executed image as itself.
pub(crate) fn relocations(original: &Descriptors) -> [Relocation; 6] {
    ROLES.map(|role| Relocation {
        role,
        source: original.get(role),
        minimum: FIRST_RELOCATED,
    })
}

/// A final stdio mapping `source -> target`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct StdioMapping {
    pub(crate) source: u32,
    pub(crate) target: u32,
}

/// An inclusive, never inverted, descriptor range to close.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CloseRange {
    pub(crate) first: u32,
    pub(crate) last: u32,
}

impl CloseRange {
    pub(crate) const fn contains(self, fd: u32) -> bool {
        self.first <= fd && fd <= self.last
    }
}

/// The child's descriptor plan after relocation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ChildLayout {
    /// stdin, stdout, stderr, in that order.
    pub(crate) stdio: [StdioMapping; 3],
    /// The two descriptors the range close must keep: the executable and the
    /// exec-status write end. Both are close-on-exec.
    pub(crate) preserved: [u32; 2],
    /// At most three ranges, ascending, covering every number from
    /// [`FIRST_RELOCATED`] to [`HIGHEST`] except the preserved two.
    pub(crate) close_ranges: Vec<CloseRange>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LayoutError {
    /// A descriptor was not relocated to at least [`FIRST_RELOCATED`].
    NotRelocated(Role),
    /// Two roles share one number.
    NotDistinct,
}

/// Plan the child layout from relocated descriptor numbers.
///
/// Refuses numbers below 3: an unrelocated low number could be a `dup2` source
/// equal to its target, which would silently keep the close-on-exec flag.
pub(crate) fn plan_child_layout(relocated: &Descriptors) -> Result<ChildLayout, LayoutError> {
    for role in ROLES {
        if relocated.get(role) < FIRST_RELOCATED {
            return Err(LayoutError::NotRelocated(role));
        }
    }
    let mut numbers = ROLES.map(|r| relocated.get(r));
    numbers.sort_unstable();
    if numbers.windows(2).any(|w| matches!(w, [a, b] if a == b)) {
        return Err(LayoutError::NotDistinct);
    }

    let low = relocated.executable.min(relocated.status_write);
    let high = relocated.executable.max(relocated.status_write);
    let mut close_ranges = Vec::with_capacity(3);
    // Gaps around the two preserved numbers. An inverted gap is skipped, which
    // is exactly the adjacent (F7) and lowest-possible cases.
    let candidates = [
        (Some(FIRST_RELOCATED), low.checked_sub(1)),
        (low.checked_add(1), high.checked_sub(1)),
        (high.checked_add(1), Some(HIGHEST)),
    ];
    for (first, last) in candidates {
        if let (Some(first), Some(last)) = (first, last)
            && first <= last
        {
            close_ranges.push(CloseRange { first, last });
        }
    }

    Ok(ChildLayout {
        stdio: [
            StdioMapping {
                source: relocated.stdin_read,
                target: 0,
            },
            StdioMapping {
                source: relocated.stdout_write,
                target: 1,
            },
            StdioMapping {
                source: relocated.stderr_write,
                target: 2,
            },
        ],
        preserved: [low, high],
        close_ranges,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        CloseRange, Descriptors, FIRST_RELOCATED, HIGHEST, LayoutError, ROLES, Role,
        plan_child_layout, relocations,
    };

    /// Deterministic generator; no randomness source is read.
    struct SplitMix(u64);
    impl SplitMix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
        fn chance(&mut self, percent: u64) -> bool {
            self.below(100) < percent
        }
    }

    fn check_properties(input: &Descriptors) {
        let layout = plan_child_layout(input).unwrap();
        assert_eq!(
            plan_child_layout(input).unwrap(),
            layout,
            "not deterministic"
        );
        let preserved = [input.executable, input.status_write];
        assert!(layout.close_ranges.len() <= 3);
        for r in &layout.close_ranges {
            assert!(r.first <= r.last, "inverted range {r:?}");
            assert!(r.first >= FIRST_RELOCATED, "range reaches stdio: {r:?}");
            for p in preserved {
                assert!(!r.contains(p), "range {r:?} contains preserved {p}");
            }
        }
        assert!(
            layout
                .close_ranges
                .windows(2)
                .all(|w| w[0].last < w[1].first),
            "ranges overlap or are unordered"
        );
        // Coverage: every number >= 3 is in a range or preserved. Checked on the
        // interesting numbers and the range boundaries rather than 2^32 values.
        let mut probes = vec![FIRST_RELOCATED, HIGHEST, HIGHEST - 1];
        for role in ROLES {
            let n = input.get(role);
            probes.extend([n.saturating_sub(1), n, n.saturating_add(1)]);
        }
        for p in probes.into_iter().filter(|p| *p >= FIRST_RELOCATED) {
            let covered = layout.close_ranges.iter().any(|r| r.contains(p));
            assert_ne!(covered, preserved.contains(&p), "number {p} mishandled");
        }
        // Non-preserved child descriptors are always closed by a range.
        for role in [
            Role::WorkingDirectory,
            Role::StdinRead,
            Role::StdoutWrite,
            Role::StderrWrite,
        ] {
            let n = input.get(role);
            assert!(
                layout.close_ranges.iter().any(|r| r.contains(n)),
                "{role:?} kept"
            );
        }
        for (m, target) in layout.stdio.iter().zip(0u32..) {
            assert_eq!(m.target, target);
            assert_ne!(m.source, m.target, "dup2 source equals target");
        }
    }

    #[test]
    fn unrelocated_low_descriptors_are_refused() {
        let good = Descriptors {
            executable: 3,
            working_directory: 4,
            stdin_read: 5,
            stdout_write: 6,
            stderr_write: 7,
            status_write: 8,
        };
        check_properties(&good);
        for low in 0..3 {
            for role in ROLES {
                let mut d = good;
                match role {
                    Role::Executable => d.executable = low,
                    Role::WorkingDirectory => d.working_directory = low,
                    Role::StdinRead => d.stdin_read = low,
                    Role::StdoutWrite => d.stdout_write = low,
                    Role::StderrWrite => d.stderr_write = low,
                    Role::StatusWrite => d.status_write = low,
                }
                assert_eq!(plan_child_layout(&d), Err(LayoutError::NotRelocated(role)));
            }
        }
        let mut shared = good;
        shared.stdout_write = shared.stdin_read;
        assert_eq!(plan_child_layout(&shared), Err(LayoutError::NotDistinct));
    }

    #[test]
    fn adjacent_and_extreme_preserved_numbers_produce_no_inverted_range() {
        let cases = [
            // F7: executable and status adjacent at the lowest numbers.
            (
                3,
                4,
                vec![CloseRange {
                    first: 5,
                    last: HIGHEST,
                }],
            ),
            (
                4,
                3,
                vec![CloseRange {
                    first: 5,
                    last: HIGHEST,
                }],
            ),
            // Adjacent, higher up.
            (
                9,
                10,
                vec![
                    CloseRange { first: 3, last: 8 },
                    CloseRange {
                        first: 11,
                        last: HIGHEST,
                    },
                ],
            ),
            // One gap of exactly one number.
            (
                3,
                5,
                vec![
                    CloseRange { first: 4, last: 4 },
                    CloseRange {
                        first: 6,
                        last: HIGHEST,
                    },
                ],
            ),
            // The highest number preserved: no upper range.
            (
                20,
                HIGHEST,
                vec![
                    CloseRange { first: 3, last: 19 },
                    CloseRange {
                        first: 21,
                        last: HIGHEST - 1,
                    },
                ],
            ),
            (
                HIGHEST - 1,
                HIGHEST,
                vec![CloseRange {
                    first: 3,
                    last: HIGHEST - 2,
                }],
            ),
        ];
        for (executable, status_write, expected) in cases {
            let others: Vec<u32> = (3u32..)
                .filter(|n| *n != executable && *n != status_write)
                .take(4)
                .collect();
            let d = Descriptors {
                executable,
                working_directory: others[0],
                stdin_read: others[1],
                stdout_write: others[2],
                stderr_write: others[3],
                status_write,
            };
            let layout = plan_child_layout(&d).unwrap();
            assert_eq!(layout.close_ranges, expected, "{executable}/{status_write}");
            check_properties(&d);
        }
    }

    #[test]
    fn relocation_is_unconditional() {
        let d = Descriptors {
            executable: 100,
            working_directory: 0,
            stdin_read: 1,
            stdout_write: 2,
            stderr_write: 3,
            status_write: 4,
        };
        let steps = relocations(&d);
        assert_eq!(steps.len(), 6);
        for (step, role) in steps.iter().zip(ROLES) {
            assert_eq!(step.role, role);
            assert_eq!(step.source, d.get(role));
            assert_eq!(step.minimum, FIRST_RELOCATED);
        }
    }

    #[test]
    fn generated_distinct_layouts_satisfy_every_property() {
        let mut rng = SplitMix(0x5eed_1a70_0000_0001);
        for _ in 0..20_000 {
            let mut chosen: Vec<u32> = Vec::new();
            while chosen.len() < 6 {
                let n = match rng.below(4) {
                    0 => 3 + u32::try_from(rng.below(8)).unwrap(),
                    1 => 3 + u32::try_from(rng.below(1024)).unwrap(),
                    2 => HIGHEST - u32::try_from(rng.below(8)).unwrap(),
                    _ => u32::try_from(rng.below(u64::from(HIGHEST - 2))).unwrap() + 3,
                };
                if !chosen.contains(&n) {
                    chosen.push(n);
                }
            }
            let d = Descriptors {
                executable: chosen[0],
                working_directory: chosen[1],
                stdin_read: chosen[2],
                stdout_write: chosen[3],
                stderr_write: chosen[4],
                status_write: chosen[5],
            };
            check_properties(&d);
        }
    }

    // ----------------------------------------------- simulated descriptor table
    //
    // A pure in-memory model of a descriptor table, used only to generate
    // realistic shapes (host 0/1/2 closed, lowest-free allocation, caller
    // descriptors without close-on-exec) and to check the end state the plan
    // promises. It is a test model; no real descriptor is involved.

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    struct Entry {
        object: u32,
        cloexec: bool,
    }

    #[derive(Clone, Default)]
    struct Table {
        fds: BTreeMap<u32, Entry>,
        next_object: u32,
    }

    impl Table {
        fn lowest_free(&self, minimum: u32) -> u32 {
            (minimum..).find(|n| !self.fds.contains_key(n)).unwrap()
        }
        fn open(&mut self, cloexec: bool) -> u32 {
            let fd = self.lowest_free(0);
            self.next_object += 1;
            self.fds.insert(
                fd,
                Entry {
                    object: self.next_object,
                    cloexec,
                },
            );
            fd
        }
        fn relocate_cloexec(&mut self, source: u32, minimum: u32) -> u32 {
            let entry = self.fds[&source];
            let fd = self.lowest_free(minimum);
            self.fds.insert(
                fd,
                Entry {
                    object: entry.object,
                    cloexec: true,
                },
            );
            fd
        }
        fn close(&mut self, fd: u32) {
            self.fds.remove(&fd).unwrap();
        }
        fn map_onto(&mut self, source: u32, target: u32) {
            let entry = self.fds[&source];
            self.fds.insert(
                target,
                Entry {
                    object: entry.object,
                    cloexec: false,
                },
            );
        }
        fn close_span(&mut self, r: CloseRange) {
            self.fds.retain(|fd, _| !r.contains(*fd));
        }
        fn exec(&mut self) {
            self.fds.retain(|_, e| !e.cloexec);
        }
    }

    #[test]
    fn simulated_hosts_end_with_exactly_stdio_in_the_image() {
        let mut rng = SplitMix(0x0f0f_1a70_0000_0002);
        for _ in 0..5_000 {
            let mut host = Table::default();
            // Host 0, 1, 2 each open or closed; then unrelated host descriptors.
            for _ in 0..3 {
                host.open(false);
            }
            for fd in 0..3 {
                if rng.chance(40) {
                    host.close(fd);
                }
            }
            for _ in 0..rng.below(24) {
                let cloexec = rng.chance(50);
                host.open(cloexec);
            }
            for _ in 0..rng.below(6) {
                let open: Vec<u32> = host.fds.keys().copied().collect();
                if let Some(fd) =
                    open.get(usize::try_from(rng.below(open.len() as u64 + 1)).unwrap())
                {
                    host.close(*fd);
                }
            }
            // Caller-supplied executable and directory: any close-on-exec state,
            // possibly landing on 0, 1 or 2.
            let exe_cloexec = rng.chance(50);
            let exec_fd = host.open(exe_cloexec);
            let dir_fd = host.open(rng.chance(50));
            let exe_object = host.fds[&exec_fd].object;
            let host_objects: Vec<u32> = host.fds.values().map(|e| e.object).collect();

            // Four close-on-exec pipes, lowest-free allocation (F6 shapes).
            let (stdin_r, _stdin_w) = (host.open(true), host.open(true));
            let (_stdout_r, stdout_w) = (host.open(true), host.open(true));
            let (_stderr_r, stderr_w) = (host.open(true), host.open(true));
            let (_status_r, status_w) = (host.open(true), host.open(true));
            let objects = |t: &Table, fd: u32| t.fds[&fd].object;
            let stdin_obj = objects(&host, stdin_r);
            let stdout_obj = objects(&host, stdout_w);
            let stderr_obj = objects(&host, stderr_w);
            let status_obj = objects(&host, status_w);

            let original = Descriptors {
                executable: exec_fd,
                working_directory: dir_fd,
                stdin_read: stdin_r,
                stdout_write: stdout_w,
                stderr_write: stderr_w,
                status_write: status_w,
            };
            let mut relocated = original;
            for step in relocations(&original) {
                let new = host.relocate_cloexec(step.source, step.minimum);
                host.close(step.source);
                match step.role {
                    Role::Executable => relocated.executable = new,
                    Role::WorkingDirectory => relocated.working_directory = new,
                    Role::StdinRead => relocated.stdin_read = new,
                    Role::StdoutWrite => relocated.stdout_write = new,
                    Role::StderrWrite => relocated.stderr_write = new,
                    Role::StatusWrite => relocated.status_write = new,
                }
            }
            check_properties(&relocated);
            let layout = plan_child_layout(&relocated).unwrap();

            // The child's copy of the table after the clone.
            let mut child = host.clone();
            for m in layout.stdio {
                child.map_onto(m.source, m.target);
            }
            for fd in 0..3 {
                child.fds.get_mut(&fd).unwrap().cloexec = false;
            }
            assert!(
                child.fds.contains_key(&relocated.working_directory),
                "cwd closed before use"
            );
            for r in &layout.close_ranges {
                child.close_span(*r);
            }
            assert_eq!(child.fds[&relocated.executable].object, exe_object);
            assert_eq!(child.fds[&relocated.status_write].object, status_obj);
            child.exec();

            let image: Vec<(u32, u32)> = child.fds.iter().map(|(fd, e)| (*fd, e.object)).collect();
            assert_eq!(
                image,
                vec![(0, stdin_obj), (1, stdout_obj), (2, stderr_obj)]
            );
            // No host object and not the caller's executable survives.
            for (_, object) in image {
                assert!(!host_objects.contains(&object));
                assert_ne!(object, exe_object);
            }
        }
    }
}
