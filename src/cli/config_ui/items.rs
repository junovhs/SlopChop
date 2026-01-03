use crate::config::Config;

/// Configuration items that can be edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigItem {
    MaxTokens,
    MaxComplexity,
    MaxNesting,
    MaxArgs,
    AutoCopy,
    WriteFixPacket,
    RequirePlan,
    AutoPromote,
    LocalityMode,
    LocalityMaxDistance,
}

impl ConfigItem {
    #[must_use]
    pub const fn all() -> [Self; 10] {
        [
            Self::MaxTokens,
            Self::MaxComplexity,
            Self::MaxNesting,
            Self::MaxArgs,
            Self::AutoCopy,
            Self::WriteFixPacket,
            Self::RequirePlan,
            Self::AutoPromote,
            Self::LocalityMode,
            Self::LocalityMaxDistance,
        ]
    }
    
    #[must_use]
    pub const fn label(self) -> &'static str {
        const LABELS: [&str; 10] = [
            "Max file tokens",
            "Max complexity",
            "Max nesting",
            "Max args",
            "Auto-copy to clipboard",
            "Write fix packet to file",
            "Require PLAN block",
            "Auto-promote on green",
            "Locality mode",
            "Locality max distance",
        ];
        LABELS[self as usize]
    }

    #[must_use]
    pub const fn is_boolean(self) -> bool {
        matches!(
            self,
            Self::AutoCopy | Self::WriteFixPacket | Self::RequirePlan | Self::AutoPromote
        )
    }

    #[must_use]
    pub const fn is_enum(self) -> bool {
        matches!(self, Self::LocalityMode)
    }

    #[must_use]
    pub fn get_value(self, config: &Config) -> String {
        if self.is_boolean() {
            return self.get_boolean_value(config);
        }
        
        if self.is_enum() {
             return format!("[{}]", config.rules.locality.mode);
        }

        self.get_numeric_value(config)
    }

    fn get_boolean_value(self, config: &Config) -> String {
         let checked = match self {
            Self::AutoCopy => config.preferences.auto_copy,
            Self::WriteFixPacket => config.preferences.write_fix_packet,
            Self::RequirePlan => config.preferences.require_plan,
            Self::AutoPromote => config.preferences.auto_promote,
            _ => false,
        };
        checkbox(checked)
    }

    fn get_numeric_value(self, config: &Config) -> String {
        let val = match self {
            Self::MaxTokens => config.rules.max_file_tokens,
            Self::MaxComplexity => config.rules.max_cyclomatic_complexity,
            Self::MaxNesting => config.rules.max_nesting_depth,
            Self::MaxArgs => config.rules.max_function_args,
            Self::LocalityMaxDistance => config.rules.locality.max_distance,
            _ => 0,
        };
        format!("[{val}]")
    }
}

fn checkbox(checked: bool) -> String {
    if checked { "[x]".to_string() } else { "[ ]".to_string() }
}
