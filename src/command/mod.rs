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
    pub(crate) fn from_counts(counts: AHashMap<String, usize>) -> Self {
        Self { counts }
    }

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

    pub(crate) fn into_ranked(
        self,
        more_than: usize,
    ) -> Vec<RankedCommand> {
        let mut commands: Vec<RankedCommand> = self
            .counts
            .into_iter()
            .filter(|(_, count)| *count > more_than)
            .map(|(name, count)| RankedCommand { name, count })
            .collect();

        commands.sort_unstable_by(|a, b| {
            b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name))
        });
        commands
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

#[derive(Debug, Clone)]
pub(crate) struct RankedCommand {
    pub(crate) name: String,
    pub(crate) count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_and_sort_commands() {
        let mut counts = AHashMap::default();
        counts.insert("ls".to_string(), 4);
        counts.insert("git".to_string(), 2);
        counts.insert("cd".to_string(), 1);

        let commands =
            FrequencyTable::from_counts(counts).into_ranked(1);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].name, "ls");
        assert_eq!(commands[0].count, 4);
        assert_eq!(commands[1].name, "git");
        assert_eq!(commands[1].count, 2);
    }

    #[test]
    fn test_filter_and_sort_commands_deterministic_tie_break() {
        let mut counts = AHashMap::default();
        counts.insert("zsh".to_string(), 2);
        counts.insert("bash".to_string(), 2);

        let commands =
            FrequencyTable::from_counts(counts).into_ranked(0);
        assert_eq!(commands[0].name, "bash");
        assert_eq!(commands[1].name, "zsh");
    }

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
                line: "sudo grep foo",
                ignored: &["grep"],
                mode: CommandMode::History,
                expected: &[],
            },
            Case {
                line: "",
                ignored: &[],
                mode: CommandMode::History,
                expected: &[],
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
