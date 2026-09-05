//! Internal command interpretation and frequency analysis.

use ahash::{AHashMap, AHashSet};
use bstr::ByteSlice;

use crate::shared::command_parse::{SplitCommands, get_first_word};

/// Select the command interpretation policy used for an input line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandMode {
    History,
    Raw,
}

/// The command interpretation facts shared by all history adapters.
pub(crate) struct CommandPolicy<'a> {
    ignored: AHashSet<&'a str>,
    wrappers: AHashSet<&'a str>,
    mode: CommandMode,
}

impl<'a> CommandPolicy<'a> {
    pub(crate) fn new(
        ignored: &'a [String],
        mode: CommandMode,
    ) -> Self {
        let ignored = ignored.iter().map(String::as_str).collect();
        let wrappers = match mode {
            CommandMode::History =>
                AHashSet::from_iter(["sudo", "doas"]),
            CommandMode::Raw => AHashSet::new(),
        };

        Self { ignored, wrappers, mode }
    }
}

/// Opaque command frequency accumulator shared by history adapters.
#[derive(Debug, Default)]
pub(crate) struct FrequencyTable {
    counts: AHashMap<String, usize>,
}

impl FrequencyTable {
    /// Record every command represented by one decoded history line.
    pub(crate) fn record_line(
        &mut self,
        line: &str,
        policy: &CommandPolicy<'_>,
    ) {
        match policy.mode {
            CommandMode::History => {
                if line.as_bytes().find_byte(b'|').is_some() {
                    for segment in SplitCommands::new(line) {
                        self.record_history_segment(segment, policy);
                    }
                } else {
                    self.record_history_segment(line, policy);
                }
            }
            CommandMode::Raw => {
                self.record_raw_line(line, policy);
            }
        }
    }

    /// Convert the accumulator at compatibility boundaries that still expose
    /// the historic map representation.
    pub(crate) fn into_counts(self) -> AHashMap<String, usize> {
        self.counts
    }

    fn record_history_segment(
        &mut self,
        segment: &str,
        policy: &CommandPolicy<'_>,
    ) {
        let Some(command) = get_first_word(segment, &policy.wrappers) else {
            return;
        };

        // An ignored canonical command discards its whole segment. In
        // particular, arguments must never become a new command.
        if policy.ignored.contains(command) {
            return;
        }

        self.increment(command);
    }

    fn record_raw_line(
        &mut self,
        line: &str,
        policy: &CommandPolicy<'_>,
    ) {
        let Some(command) = line.split_whitespace().next() else {
            return;
        };

        if !policy.ignored.contains(command) {
            self.increment(command);
        }
    }

    #[inline]
    fn increment(&mut self, command: &str) {
        *self.counts.entry(command.to_string()).or_default() += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_line_contract_table() {
        struct Case {
            line: &'static str,
            ignored: &'static [&'static str],
            mode: CommandMode,
            expected: &'static [(&'static str, usize)],
        }

        let cases = [
            Case {
                line: "ls | grep foo",
                ignored: &["grep"],
                mode: CommandMode::History,
                expected: &[("ls", 1)],
            },
            Case {
                line: "sudo apt update",
                ignored: &[],
                mode: CommandMode::History,
                expected: &[("apt", 1)],
            },
            Case {
                line: "FOO=bar /bin/ls -la",
                ignored: &[],
                mode: CommandMode::History,
                expected: &[("ls", 1)],
            },
            Case {
                line: "sudo apt | grep foo",
                ignored: &["grep"],
                mode: CommandMode::Raw,
                expected: &[("sudo", 1)],
            },
        ];

        for case in cases {
            let ignored: Vec<String> =
                case.ignored.iter().map(|value| (*value).to_string()).collect();
            let policy = CommandPolicy::new(&ignored, case.mode);
            let mut table = FrequencyTable::default();
            table.record_line(case.line, &policy);

            let expected = case
                .expected
                .iter()
                .map(|(name, count)| ((*name).to_string(), *count))
                .collect::<AHashMap<_, _>>();
            assert_eq!(table.into_counts(), expected, "case: {}", case.line);
        }
    }
}
