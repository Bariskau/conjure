use serde_json::Value;

use crate::domain::{NewParameter, NewTool, ParameterType, ValidationRules};

pub fn default_tools() -> Vec<DefaultToolSpec> {
    vec![claude_debate(), codex_debate()]
}

pub struct DefaultToolSpec {
    name: &'static str,
    description: &'static str,
    category: &'static str,
    script_body: fn() -> String,
    timeout_ms: i64,
    working_dir_expose: bool,
    enabled: bool,
    parameters: &'static [DefaultParameterSpec],
}

impl DefaultToolSpec {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn parameters(&self) -> &[DefaultParameterSpec] {
        self.parameters
    }

    pub fn new_tool(&self) -> NewTool {
        NewTool {
            name: self.name.to_string(),
            description: self.description.to_string(),
            category: Some(self.category.to_string()),
            script_body: Some((self.script_body)()),
            script_path: None,
            working_dir: None,
            working_dir_expose: self.working_dir_expose,
            working_dir_required: false,
            timeout_ms: self.timeout_ms,
            enabled: self.enabled,
        }
    }
}

pub struct DefaultParameterSpec {
    name: &'static str,
    parameter_type: ParameterType,
    description: &'static str,
    required: bool,
    default_value: DefaultParameterValue,
    validation: DefaultValidationSpec,
}

impl DefaultParameterSpec {
    pub fn new_parameter(&self, position: i64) -> NewParameter {
        NewParameter {
            name: self.name.to_string(),
            parameter_type: self.parameter_type.clone(),
            description: Some(self.description.to_string()),
            required: self.required,
            default_value: self.default_value.to_json(),
            validation: self.validation.to_rules(),
            position,
        }
    }
}

#[derive(Clone, Copy)]
enum DefaultParameterValue {
    None,
    Text(&'static str),
}

impl DefaultParameterValue {
    fn to_json(self) -> Option<Value> {
        match self {
            Self::None => None,
            Self::Text(value) => Some(Value::String(value.to_string())),
        }
    }
}

#[derive(Clone, Copy)]
struct DefaultValidationSpec {
    min: Option<f64>,
    max: Option<f64>,
    regex: Option<&'static str>,
    format: Option<&'static str>,
    integer: bool,
    enum_values: &'static [&'static str],
}

impl DefaultValidationSpec {
    const fn length(min: Option<f64>, max: Option<f64>) -> Self {
        Self {
            min,
            max,
            regex: None,
            format: None,
            integer: false,
            enum_values: &[],
        }
    }

    const fn enumeration(enum_values: &'static [&'static str]) -> Self {
        Self {
            min: None,
            max: None,
            regex: None,
            format: None,
            integer: false,
            enum_values,
        }
    }

    fn to_rules(self) -> ValidationRules {
        ValidationRules {
            min: self.min,
            max: self.max,
            regex: self.regex.map(ToString::to_string),
            format: self.format.map(ToString::to_string),
            integer: self.integer,
            enum_values: self
                .enum_values
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        }
    }
}

fn claude_debate() -> DefaultToolSpec {
    DefaultToolSpec {
        name: "claude_debate",
        description: "Runs a local Claude CLI debate against Codex on a topic, decision, or proposal.",
        category: "AI",
        script_body: claude_debate_script,
        timeout_ms: 300_000,
        working_dir_expose: true,
        enabled: false,
        parameters: AI_CLI_DEBATE_PARAMETERS,
    }
}

fn codex_debate() -> DefaultToolSpec {
    DefaultToolSpec {
        name: "codex_debate",
        description: "Runs a local Codex CLI debate as a second opinion on a topic, decision, or proposal.",
        category: "AI",
        script_body: codex_debate_script,
        timeout_ms: 300_000,
        working_dir_expose: true,
        enabled: false,
        parameters: AI_CLI_DEBATE_PARAMETERS,
    }
}

const AI_CLI_LANGUAGES: &[&str] = &["tr", "en", "original"];
const AI_CLI_DEBATE_MODES: &[&str] = &["critic", "steelman", "judge", "red_team"];

const AI_CLI_DEBATE_PARAMETERS: &[DefaultParameterSpec] = &[
    DefaultParameterSpec {
        name: "topic",
        parameter_type: ParameterType::String,
        description: "Question, decision, proposal, or claim to debate.",
        required: true,
        default_value: DefaultParameterValue::None,
        validation: DefaultValidationSpec::length(Some(1.0), Some(24_000.0)),
    },
    DefaultParameterSpec {
        name: "codex_position",
        parameter_type: ParameterType::String,
        description: "Codex current argument or proposed answer for the other CLI to challenge.",
        required: false,
        default_value: DefaultParameterValue::Text(""),
        validation: DefaultValidationSpec::length(None, Some(48_000.0)),
    },
    DefaultParameterSpec {
        name: "context",
        parameter_type: ParameterType::String,
        description: "Relevant files, constraints, evidence, product context, or transcript.",
        required: false,
        default_value: DefaultParameterValue::Text(""),
        validation: DefaultValidationSpec::length(None, Some(48_000.0)),
    },
    DefaultParameterSpec {
        name: "mode",
        parameter_type: ParameterType::Enum,
        description: "How the other CLI should participate in the debate.",
        required: false,
        default_value: DefaultParameterValue::Text("critic"),
        validation: DefaultValidationSpec::enumeration(AI_CLI_DEBATE_MODES),
    },
    DefaultParameterSpec {
        name: "language",
        parameter_type: ParameterType::Enum,
        description: "Response language instruction.",
        required: false,
        default_value: DefaultParameterValue::Text("tr"),
        validation: DefaultValidationSpec::enumeration(AI_CLI_LANGUAGES),
    },
    DefaultParameterSpec {
        name: "model",
        parameter_type: ParameterType::String,
        description: "Optional CLI model name. Leave empty to use the CLI default.",
        required: false,
        default_value: DefaultParameterValue::Text(""),
        validation: DefaultValidationSpec::length(None, Some(160.0)),
    },
];

fn claude_debate_script() -> String {
    format!("{AI_CLI_DEBATE_CLAUDE_SCRIPT}\n{AI_CLI_DEBATE_PROMPT_SCRIPT}")
}

fn codex_debate_script() -> String {
    format!("{AI_CLI_DEBATE_CODEX_SCRIPT}\n{AI_CLI_DEBATE_PROMPT_SCRIPT}")
}

const AI_CLI_DEBATE_CLAUDE_SCRIPT: &str = r###"#!/usr/bin/env sh
set -eu

selected_label="Claude"

claude_bin() {
  if [ -n "${CLAUDE_BIN:-}" ]; then
    [ -x "${CLAUDE_BIN}" ] || return 1
    printf '%s\n' "$CLAUDE_BIN"
  elif command -v claude >/dev/null 2>&1; then
    command -v claude
  elif [ -n "${HOME:-}" ]; then
    for candidate in "$HOME"/.npm/_npx/*/node_modules/.bin/claude; do
      [ -x "$candidate" ] || continue
      printf '%s\n' "$candidate"
      return 0
    done
    return 1
  else
    return 1
  fi
}

run_debate_prompt() {
  prompt_text="$1"
  if ! bin="$(claude_bin)"; then
    printf '%s\n' "Claude CLI is not available." >&2
    exit 127
  fi

  if [ -n "${model:-}" ]; then
    "$bin" -p "$prompt_text" --permission-mode bypassPermissions --model "$model" </dev/null
  else
    "$bin" -p "$prompt_text" --permission-mode bypassPermissions </dev/null
  fi
}
"###;

const AI_CLI_DEBATE_CODEX_SCRIPT: &str = r###"#!/usr/bin/env sh
set -eu

selected_label="Codex CLI"

codex_bin() {
  if [ -n "${CODEX_BIN:-}" ]; then
    [ -x "${CODEX_BIN}" ] || return 1
    printf '%s\n' "$CODEX_BIN"
  elif command -v codex >/dev/null 2>&1; then
    command -v codex
  elif [ -x "/Applications/Codex.app/Contents/Resources/codex" ]; then
    printf '%s\n' "/Applications/Codex.app/Contents/Resources/codex"
  else
    return 1
  fi
}

run_debate_prompt() {
  prompt_text="$1"
  if ! bin="$(codex_bin)"; then
    printf '%s\n' "Codex CLI is not available." >&2
    exit 127
  fi

  if [ -n "${model:-}" ]; then
    "$bin" -a never exec --skip-git-repo-check --sandbox read-only --color never -m "$model" "$prompt_text" </dev/null
  else
    "$bin" -a never exec --skip-git-repo-check --sandbox read-only --color never "$prompt_text" </dev/null
  fi
}
"###;

const AI_CLI_DEBATE_PROMPT_SCRIPT: &str = r###"
language_instruction() {
  case "${language:-tr}" in
    tr) printf '%s\n' "Respond in Turkish." ;;
    en) printf '%s\n' "Respond in English." ;;
    original) printf '%s\n' "Use the most natural language for the request." ;;
    *) printf '%s\n' "Respond in Turkish." ;;
  esac
}

topic_text="${topic:-}"
if [ -z "$topic_text" ]; then
  printf '%s\n' "topic is required" >&2
  exit 2
fi

mode_instruction="Challenge Codex's current position, identify weak assumptions, and propose stronger alternatives."
case "${mode:-critic}" in
  steelman)
    mode_instruction="First steelman Codex's position, then explain the best opposing argument."
    ;;
  judge)
    mode_instruction="Act as a neutral judge. Compare Codex's position with the strongest counter-position and decide what would change your mind."
    ;;
  red_team)
    mode_instruction="Red-team Codex's position professionally. Focus on failure modes, hidden risks, and missing evidence."
    ;;
esac

PROMPT="$(printf '%s\n' \
  "You are ${selected_label} in a structured debate with the primary Codex session." \
  "$(language_instruction)" \
  "" \
  "Debate mode:" \
  "${mode:-critic}" \
  "" \
  "Mode instruction:" \
  "$mode_instruction" \
  "" \
  "Topic or decision under debate:" \
  "$topic_text" \
  "" \
  "Primary Codex current position:" \
  "${codex_position:-No Codex position was provided. Infer a reasonable target from the topic and clearly state your assumption.}" \
  "" \
  "Additional context:" \
  "${context:-No additional context was provided.}" \
  "" \
  "Rules:" \
  "- Be direct and specific, not performatively agreeable." \
  "- Separate evidence from inference." \
  "- If a claim needs current facts or external verification, flag it explicitly." \
  "- Keep the answer useful for the primary Codex session to respond to in the next turn." \
  "" \
  "Return exactly these sections:" \
  "## ${selected_label} Stance" \
  "## Strongest Agreement" \
  "## Main Disagreement" \
  "## Risks / Missing Evidence" \
  "## Questions For Codex" \
  "## Suggested Next Move")"

run_debate_prompt "$PROMPT"
"###;
