use serde_json::{Value, json};

use crate::domain::{NewParameter, NewTool, ParameterType, ValidationRules};

pub fn default_tools() -> Vec<DefaultToolSpec> {
    vec![
        ai_cli_doctor(),
        ai_cli_prompt(),
        ai_cli_debate(),
        gemini_image_generate(),
        gemini_video_generate(),
    ]
}

pub struct DefaultToolSpec {
    name: &'static str,
    description: &'static str,
    category: &'static str,
    script_body: fn() -> String,
    timeout_ms: i64,
    working_dir_expose: bool,
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
            enabled: true,
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
    Number(i64),
}

impl DefaultParameterValue {
    fn to_json(self) -> Option<Value> {
        match self {
            Self::None => None,
            Self::Text(value) => Some(Value::String(value.to_string())),
            Self::Number(value) => Some(json!(value)),
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

    const fn integer(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            regex: None,
            format: None,
            integer: true,
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

fn ai_cli_doctor() -> DefaultToolSpec {
    DefaultToolSpec {
        name: "ai_cli_doctor",
        description: "Checks which local AI CLIs are available to Conjure and prints paths, versions, and useful environment hints.",
        category: "AI",
        script_body: ai_cli_doctor_script,
        timeout_ms: 30_000,
        working_dir_expose: false,
        parameters: &[],
    }
}

fn ai_cli_prompt() -> DefaultToolSpec {
    DefaultToolSpec {
        name: "ai_cli_prompt",
        description: "Runs an installed AI CLI in non-interactive mode with an optional provider, model, context, and output format.",
        category: "AI",
        script_body: ai_cli_prompt_script,
        timeout_ms: 300_000,
        working_dir_expose: true,
        parameters: AI_CLI_PROMPT_PARAMETERS,
    }
}

fn ai_cli_debate() -> DefaultToolSpec {
    DefaultToolSpec {
        name: "ai_cli_debate",
        description: "Lets Codex debate a topic with Claude, Gemini, or Codex CLI by auto-selecting an available local provider.",
        category: "AI",
        script_body: ai_cli_debate_script,
        timeout_ms: 300_000,
        working_dir_expose: true,
        parameters: AI_CLI_DEBATE_PARAMETERS,
    }
}

fn gemini_image_generate() -> DefaultToolSpec {
    DefaultToolSpec {
        name: "gemini_image_generate",
        description: "Generates image files with Gemini native image models or Imagen through the Gemini API.",
        category: "AI",
        script_body: gemini_image_generate_script,
        timeout_ms: 300_000,
        working_dir_expose: false,
        parameters: GEMINI_IMAGE_PARAMETERS,
    }
}

fn gemini_video_generate() -> DefaultToolSpec {
    DefaultToolSpec {
        name: "gemini_video_generate",
        description: "Generates an MP4 with Veo through the Gemini API and saves it locally.",
        category: "AI",
        script_body: gemini_video_generate_script,
        timeout_ms: 900_000,
        working_dir_expose: false,
        parameters: GEMINI_VIDEO_PARAMETERS,
    }
}

const AI_CLI_PROVIDERS: &[&str] = &["auto", "claude", "gemini", "codex"];
const AI_CLI_LANGUAGES: &[&str] = &["tr", "en", "original"];
const AI_CLI_OUTPUT_FORMATS: &[&str] = &["prose", "markdown", "json"];
const AI_CLI_DEBATE_MODES: &[&str] = &["critic", "steelman", "judge", "red_team"];

const AI_CLI_PROMPT_PARAMETERS: &[DefaultParameterSpec] = &[
    DefaultParameterSpec {
        name: "prompt",
        parameter_type: ParameterType::String,
        description: "Prompt to send to the selected AI CLI.",
        required: true,
        default_value: DefaultParameterValue::None,
        validation: DefaultValidationSpec::length(Some(1.0), Some(24_000.0)),
    },
    DefaultParameterSpec {
        name: "context",
        parameter_type: ParameterType::String,
        description: "Optional background, constraints, or source material for the prompt.",
        required: false,
        default_value: DefaultParameterValue::Text(""),
        validation: DefaultValidationSpec::length(None, Some(48_000.0)),
    },
    DefaultParameterSpec {
        name: "provider",
        parameter_type: ParameterType::Enum,
        description: "Which local AI CLI to run. Auto tries Claude, then Gemini, then Codex.",
        required: false,
        default_value: DefaultParameterValue::Text("auto"),
        validation: DefaultValidationSpec::enumeration(AI_CLI_PROVIDERS),
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
        name: "output_format",
        parameter_type: ParameterType::Enum,
        description: "Preferred response shape.",
        required: false,
        default_value: DefaultParameterValue::Text("markdown"),
        validation: DefaultValidationSpec::enumeration(AI_CLI_OUTPUT_FORMATS),
    },
    DefaultParameterSpec {
        name: "model",
        parameter_type: ParameterType::String,
        description: "Optional provider model name. Leave empty to use the CLI default.",
        required: false,
        default_value: DefaultParameterValue::Text(""),
        validation: DefaultValidationSpec::length(None, Some(160.0)),
    },
];

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
        description: "Codex current argument or proposed answer for the other model to challenge.",
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
        description: "How the other model should participate in the debate.",
        required: false,
        default_value: DefaultParameterValue::Text("critic"),
        validation: DefaultValidationSpec::enumeration(AI_CLI_DEBATE_MODES),
    },
    DefaultParameterSpec {
        name: "provider",
        parameter_type: ParameterType::Enum,
        description: "Which local AI CLI to run. Auto tries Claude, then Gemini, then Codex.",
        required: false,
        default_value: DefaultParameterValue::Text("auto"),
        validation: DefaultValidationSpec::enumeration(AI_CLI_PROVIDERS),
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
        description: "Optional provider model name. Leave empty to use the CLI default.",
        required: false,
        default_value: DefaultParameterValue::Text(""),
        validation: DefaultValidationSpec::length(None, Some(160.0)),
    },
];

const GEMINI_IMAGE_MODELS: &[&str] = &[
    "gemini-3.1-flash-image",
    "gemini-3-pro-image",
    "gemini-2.5-flash-image",
    "imagen-4.0-fast-generate-001",
    "imagen-4.0-generate-001",
    "imagen-4.0-ultra-generate-001",
];

const GEMINI_IMAGE_ASPECT_RATIOS: &[&str] = &[
    "default", "1:1", "3:4", "4:3", "9:16", "16:9", "2:3", "3:2", "4:5", "5:4", "21:9",
];

const GEMINI_IMAGE_RESOLUTIONS: &[&str] = &["default", "512", "1K", "2K", "4K"];
const GEMINI_PERSON_GENERATION: &[&str] = &["default", "dont_allow", "allow_adult", "allow_all"];

const GEMINI_IMAGE_PARAMETERS: &[DefaultParameterSpec] = &[
    DefaultParameterSpec {
        name: "prompt",
        parameter_type: ParameterType::String,
        description: "Text prompt for the image.",
        required: true,
        default_value: DefaultParameterValue::None,
        validation: DefaultValidationSpec::length(Some(1.0), Some(12_000.0)),
    },
    DefaultParameterSpec {
        name: "model",
        parameter_type: ParameterType::Enum,
        description: "Gemini or Imagen image generation model.",
        required: false,
        default_value: DefaultParameterValue::Text("gemini-3.1-flash-image"),
        validation: DefaultValidationSpec::enumeration(GEMINI_IMAGE_MODELS),
    },
    DefaultParameterSpec {
        name: "number_of_images",
        parameter_type: ParameterType::Number,
        description: "Number of images to save, 1 to 4. Native Gemini image models may return one image.",
        required: false,
        default_value: DefaultParameterValue::Number(1),
        validation: DefaultValidationSpec::integer(1.0, 4.0),
    },
    DefaultParameterSpec {
        name: "aspect_ratio",
        parameter_type: ParameterType::Enum,
        description: "Output aspect ratio, when supported by the selected model.",
        required: false,
        default_value: DefaultParameterValue::Text("1:1"),
        validation: DefaultValidationSpec::enumeration(GEMINI_IMAGE_ASPECT_RATIOS),
    },
    DefaultParameterSpec {
        name: "resolution",
        parameter_type: ParameterType::Enum,
        description: "Output image size, when supported by the selected model.",
        required: false,
        default_value: DefaultParameterValue::Text("1K"),
        validation: DefaultValidationSpec::enumeration(GEMINI_IMAGE_RESOLUTIONS),
    },
    DefaultParameterSpec {
        name: "person_generation",
        parameter_type: ParameterType::Enum,
        description: "Imagen person generation policy. Ignored by native Gemini image models.",
        required: false,
        default_value: DefaultParameterValue::Text("allow_adult"),
        validation: DefaultValidationSpec::enumeration(GEMINI_PERSON_GENERATION),
    },
    DefaultParameterSpec {
        name: "output_dir",
        parameter_type: ParameterType::String,
        description: "Directory where generated image files are written.",
        required: false,
        default_value: DefaultParameterValue::Text("generated/gemini-images"),
        validation: DefaultValidationSpec::length(None, Some(1000.0)),
    },
    DefaultParameterSpec {
        name: "filename_prefix",
        parameter_type: ParameterType::String,
        description: "Filename prefix for generated image files.",
        required: false,
        default_value: DefaultParameterValue::Text("gemini-image"),
        validation: DefaultValidationSpec::length(None, Some(120.0)),
    },
];

const GEMINI_VIDEO_MODELS: &[&str] = &[
    "veo-3.1-generate-preview",
    "veo-3.1-fast-generate-preview",
    "veo-3.1-lite-generate-preview",
    "veo-3.0-generate-001",
    "veo-3.0-fast-generate-001",
];

const GEMINI_VIDEO_ASPECT_RATIOS: &[&str] = &["default", "16:9", "9:16"];
const GEMINI_VIDEO_RESOLUTIONS: &[&str] = &["default", "720p", "1080p", "4k"];

const GEMINI_VIDEO_PARAMETERS: &[DefaultParameterSpec] = &[
    DefaultParameterSpec {
        name: "prompt",
        parameter_type: ParameterType::String,
        description: "Text prompt for the video.",
        required: true,
        default_value: DefaultParameterValue::None,
        validation: DefaultValidationSpec::length(Some(1.0), Some(12_000.0)),
    },
    DefaultParameterSpec {
        name: "model",
        parameter_type: ParameterType::Enum,
        description: "Veo video generation model.",
        required: false,
        default_value: DefaultParameterValue::Text("veo-3.1-generate-preview"),
        validation: DefaultValidationSpec::enumeration(GEMINI_VIDEO_MODELS),
    },
    DefaultParameterSpec {
        name: "aspect_ratio",
        parameter_type: ParameterType::Enum,
        description: "Video aspect ratio.",
        required: false,
        default_value: DefaultParameterValue::Text("16:9"),
        validation: DefaultValidationSpec::enumeration(GEMINI_VIDEO_ASPECT_RATIOS),
    },
    DefaultParameterSpec {
        name: "resolution",
        parameter_type: ParameterType::Enum,
        description: "Video resolution when supported by the selected model.",
        required: false,
        default_value: DefaultParameterValue::Text("720p"),
        validation: DefaultValidationSpec::enumeration(GEMINI_VIDEO_RESOLUTIONS),
    },
    DefaultParameterSpec {
        name: "output_dir",
        parameter_type: ParameterType::String,
        description: "Directory where the generated MP4 is written.",
        required: false,
        default_value: DefaultParameterValue::Text("generated/gemini-videos"),
        validation: DefaultValidationSpec::length(None, Some(1000.0)),
    },
    DefaultParameterSpec {
        name: "filename_prefix",
        parameter_type: ParameterType::String,
        description: "Filename prefix for the generated MP4.",
        required: false,
        default_value: DefaultParameterValue::Text("gemini-video"),
        validation: DefaultValidationSpec::length(None, Some(120.0)),
    },
    DefaultParameterSpec {
        name: "poll_interval_seconds",
        parameter_type: ParameterType::Number,
        description: "Seconds between Veo operation polls.",
        required: false,
        default_value: DefaultParameterValue::Number(10),
        validation: DefaultValidationSpec::integer(5.0, 60.0),
    },
    DefaultParameterSpec {
        name: "max_wait_seconds",
        parameter_type: ParameterType::Number,
        description: "Maximum seconds to wait for the video operation.",
        required: false,
        default_value: DefaultParameterValue::Number(600),
        validation: DefaultValidationSpec::integer(30.0, 1200.0),
    },
];

fn ai_cli_doctor_script() -> String {
    AI_CLI_DOCTOR_SCRIPT.to_string()
}

fn ai_cli_prompt_script() -> String {
    format!("{AI_CLI_RUNNER_SCRIPT}\n{AI_CLI_PROMPT_MAIN_SCRIPT}")
}

fn ai_cli_debate_script() -> String {
    format!("{AI_CLI_RUNNER_SCRIPT}\n{AI_CLI_DEBATE_MAIN_SCRIPT}")
}

fn gemini_image_generate_script() -> String {
    GEMINI_IMAGE_GENERATE_SCRIPT.to_string()
}

fn gemini_video_generate_script() -> String {
    GEMINI_VIDEO_GENERATE_SCRIPT.to_string()
}

const AI_CLI_DOCTOR_SCRIPT: &str = r###"#!/usr/bin/env sh
set -eu

report_cli() {
  label="$1"
  override="$2"
  command_name="$3"
  fallback_path="${4:-}"

  printf '%s\n' "## ${label}"

  if [ -n "$override" ]; then
    printf 'override: %s\n' "$override"
    if [ -x "$override" ]; then
      "$override" --version 2>&1 || "$override" version 2>&1 || true
    else
      printf 'status: override is not executable\n'
    fi
    printf '\n'
    return
  fi

  if command -v "$command_name" >/dev/null 2>&1; then
    path="$(command -v "$command_name")"
    printf 'path: %s\n' "$path"
    "$path" --version 2>&1 || "$path" version 2>&1 || true
    printf '\n'
    return
  fi

  if [ -n "$fallback_path" ] && [ -x "$fallback_path" ]; then
    printf 'path: %s\n' "$fallback_path"
    "$fallback_path" --version 2>&1 || "$fallback_path" version 2>&1 || true
    printf '\n'
    return
  fi

  printf 'status: not found in PATH\n\n'
}

report_env() {
  name="$1"
  if [ -n "${2:-}" ]; then
    printf '%s: set\n' "$name"
  else
    printf '%s: not set\n' "$name"
  fi
}

report_cli "Claude CLI" "${CLAUDE_BIN:-}" "claude" ""
report_cli "Gemini CLI" "${GEMINI_BIN:-}" "gemini" ""
report_cli "Codex CLI" "${CODEX_BIN:-}" "codex" "/Applications/Codex.app/Contents/Resources/codex"

printf '%s\n' "## API environment"
report_env "GEMINI_API_KEY" "${GEMINI_API_KEY:-}"
report_env "GOOGLE_API_KEY" "${GOOGLE_API_KEY:-}"
"###;

const AI_CLI_RUNNER_SCRIPT: &str = r###"#!/usr/bin/env sh
set -eu

provider_available() {
  case "$1" in
    claude)
      [ -n "${CLAUDE_BIN:-}" ] && [ -x "${CLAUDE_BIN}" ] && return 0
      command -v claude >/dev/null 2>&1
      ;;
    gemini)
      [ -n "${GEMINI_BIN:-}" ] && [ -x "${GEMINI_BIN}" ] && return 0
      command -v gemini >/dev/null 2>&1
      ;;
    codex)
      [ -n "${CODEX_BIN:-}" ] && [ -x "${CODEX_BIN}" ] && return 0
      command -v codex >/dev/null 2>&1 && return 0
      [ -x "/Applications/Codex.app/Contents/Resources/codex" ]
      ;;
    *)
      return 1
      ;;
  esac
}

resolve_provider() {
  requested="${provider:-auto}"
  case "$requested" in
    claude|gemini|codex)
      if provider_available "$requested"; then
        printf '%s\n' "$requested"
        return
      fi
      printf 'Requested provider is not available: %s\n' "$requested" >&2
      exit 127
      ;;
    auto)
      for candidate in claude gemini codex; do
        if provider_available "$candidate"; then
          printf '%s\n' "$candidate"
          return
        fi
      done
      printf '%s\n' "No supported AI CLI found. Install claude, gemini, or codex, or set CLAUDE_BIN, GEMINI_BIN, or CODEX_BIN." >&2
      exit 127
      ;;
    *)
      printf 'Unsupported provider: %s\n' "$requested" >&2
      exit 2
      ;;
  esac
}

provider_label() {
  case "$1" in
    claude) printf '%s\n' "Claude" ;;
    gemini) printf '%s\n' "Gemini" ;;
    codex) printf '%s\n' "Codex" ;;
  esac
}

claude_bin() {
  if [ -n "${CLAUDE_BIN:-}" ]; then
    printf '%s\n' "$CLAUDE_BIN"
  else
    printf '%s\n' "claude"
  fi
}

gemini_bin() {
  if [ -n "${GEMINI_BIN:-}" ]; then
    printf '%s\n' "$GEMINI_BIN"
  else
    printf '%s\n' "gemini"
  fi
}

codex_bin() {
  if [ -n "${CODEX_BIN:-}" ]; then
    printf '%s\n' "$CODEX_BIN"
  elif command -v codex >/dev/null 2>&1; then
    command -v codex
  else
    printf '%s\n' "/Applications/Codex.app/Contents/Resources/codex"
  fi
}

run_claude_prompt() {
  prompt_text="$1"
  bin="$(claude_bin)"
  if [ -n "${model:-}" ]; then
    "$bin" -p "$prompt_text" --model "$model"
  else
    "$bin" -p "$prompt_text"
  fi
}

run_gemini_prompt() {
  prompt_text="$1"
  bin="$(gemini_bin)"
  if [ -n "${model:-}" ]; then
    "$bin" -m "$model" -p "$prompt_text" --skip-trust
  else
    "$bin" -p "$prompt_text" --skip-trust
  fi
}

run_codex_prompt() {
  prompt_text="$1"
  bin="$(codex_bin)"
  if [ -n "${model:-}" ]; then
    "$bin" exec --skip-git-repo-check --sandbox read-only --ask-for-approval never --color never -m "$model" "$prompt_text"
  else
    "$bin" exec --skip-git-repo-check --sandbox read-only --ask-for-approval never --color never "$prompt_text"
  fi
}

run_provider_prompt() {
  selected_provider="$1"
  prompt_text="$2"
  case "$selected_provider" in
    claude) run_claude_prompt "$prompt_text" ;;
    gemini) run_gemini_prompt "$prompt_text" ;;
    codex) run_codex_prompt "$prompt_text" ;;
  esac
}

language_instruction() {
  case "${language:-tr}" in
    tr) printf '%s\n' "Respond in Turkish." ;;
    en) printf '%s\n' "Respond in English." ;;
    original) printf '%s\n' "Use the most natural language for the request." ;;
    *) printf '%s\n' "Respond in Turkish." ;;
  esac
}
"###;

const AI_CLI_PROMPT_MAIN_SCRIPT: &str = r###"prompt_text="${prompt:-}"
if [ -z "$prompt_text" ]; then
  printf '%s\n' "prompt is required" >&2
  exit 2
fi

format_instruction="Use concise prose."
case "${output_format:-markdown}" in
  markdown) format_instruction="Use clean Markdown with short sections when useful." ;;
  json) format_instruction="Return valid JSON only. Do not wrap it in Markdown." ;;
  prose) format_instruction="Use concise prose." ;;
esac

selected_provider="$(resolve_provider)"
selected_label="$(provider_label "$selected_provider")"

PROMPT="$(printf '%s\n' \
  "You are ${selected_label}, called from Conjure as a local AI CLI tool." \
  "$(language_instruction)" \
  "$format_instruction" \
  "" \
  "User request:" \
  "$prompt_text" \
  "" \
  "Context:" \
  "${context:-No additional context was provided.}" \
  "" \
  "Rules:" \
  "- Be direct and specific." \
  "- Separate facts from inference." \
  "- Flag anything that needs current external verification.")"

run_provider_prompt "$selected_provider" "$PROMPT"
"###;

const AI_CLI_DEBATE_MAIN_SCRIPT: &str = r###"topic_text="${topic:-}"
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

selected_provider="$(resolve_provider)"
selected_label="$(provider_label "$selected_provider")"

PROMPT="$(printf '%s\n' \
  "You are ${selected_label} in a structured debate with Codex." \
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
  "Codex current position:" \
  "${codex_position:-No Codex position was provided. Infer a reasonable target from the topic and clearly state your assumption.}" \
  "" \
  "Additional context:" \
  "${context:-No additional context was provided.}" \
  "" \
  "Rules:" \
  "- Be direct and specific, not performatively agreeable." \
  "- Separate evidence from inference." \
  "- If a claim needs current facts or external verification, flag it explicitly." \
  "- Keep the answer useful for Codex to respond to in the next turn." \
  "" \
  "Return exactly these sections:" \
  "## ${selected_label} Stance" \
  "## Strongest Agreement" \
  "## Main Disagreement" \
  "## Risks / Missing Evidence" \
  "## Questions For Codex" \
  "## Suggested Next Move")"

run_provider_prompt "$selected_provider" "$PROMPT"
"###;

const GEMINI_IMAGE_GENERATE_SCRIPT: &str = r###"#!/usr/bin/env sh
set -eu
node <<'NODE'
const fs = require("fs");
const path = require("path");

const apiKey = process.env.GEMINI_API_KEY || process.env.GOOGLE_API_KEY;
if (!apiKey) {
  console.error("GEMINI_API_KEY or GOOGLE_API_KEY is required in the backend environment for Gemini image generation.");
  process.exit(2);
}

const prompt = (process.env.prompt || "").trim();
if (!prompt) {
  console.error("prompt is required");
  process.exit(2);
}

const model = (process.env.model || "gemini-3.1-flash-image").trim();
const outputDir = path.resolve(process.env.output_dir || "generated/gemini-images");
const filenamePrefix = sanitize(process.env.filename_prefix || "gemini-image");
const numberOfImages = clampInteger(process.env.number_of_images, 1, 4, 1);
const aspectRatio = normalizeOptional(process.env.aspect_ratio);
const resolution = normalizeOptional(process.env.resolution);
const personGeneration = normalizeOptional(process.env.person_generation);

fs.mkdirSync(outputDir, { recursive: true });

function normalizeOptional(value) {
  const normalized = (value || "").trim();
  return normalized && normalized !== "default" ? normalized : "";
}

function clampInteger(value, min, max, fallback) {
  const parsed = Number.parseInt(value || "", 10);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function sanitize(value) {
  const sanitized = String(value).toLowerCase().replace(/[^a-z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
  return sanitized || "gemini-image";
}

function extensionFromMime(mimeType) {
  if ((mimeType || "").includes("jpeg")) return "jpg";
  if ((mimeType || "").includes("webp")) return "webp";
  return "png";
}

async function requestJson(url, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "x-goog-api-key": apiKey,
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
  const text = await response.text();
  let json = null;
  if (text) {
    try { json = JSON.parse(text); } catch { json = { raw: text }; }
  }
  if (!response.ok) {
    throw new Error("Gemini API request failed (" + response.status + "): " + text);
  }
  return json || {};
}

function writeImage(base64, mimeType, index) {
  const extension = extensionFromMime(mimeType);
  const filePath = path.join(outputDir, filenamePrefix + "-" + String(index).padStart(2, "0") + "." + extension);
  fs.writeFileSync(filePath, Buffer.from(base64, "base64"));
  return filePath;
}

async function generateWithImagen() {
  const parameters = { sampleCount: numberOfImages };
  if (aspectRatio) parameters.aspectRatio = aspectRatio;
  if (resolution) parameters.imageSize = resolution;
  if (personGeneration) parameters.personGeneration = personGeneration;

  const response = await requestJson(
    "https://generativelanguage.googleapis.com/v1beta/models/" + encodeURIComponent(model) + ":predict",
    { instances: [{ prompt }], parameters }
  );

  const predictions = Array.isArray(response.predictions) ? response.predictions : [];
  const files = [];
  predictions.forEach((prediction, index) => {
    const image = prediction.image || prediction;
    const base64 = image.bytesBase64Encoded || image.imageBytes || prediction.bytesBase64Encoded;
    if (base64) files.push(writeImage(base64, image.mimeType || prediction.mimeType || "image/png", index + 1));
  });
  return files;
}

async function generateWithNativeGemini() {
  const generationConfig = { responseModalities: ["TEXT", "IMAGE"] };
  const imageConfig = {};
  if (aspectRatio) imageConfig.aspectRatio = aspectRatio;
  if (resolution) imageConfig.imageSize = resolution;
  if (Object.keys(imageConfig).length > 0) {
    generationConfig.responseFormat = { image: imageConfig };
  }

  const response = await requestJson(
    "https://generativelanguage.googleapis.com/v1/models/" + encodeURIComponent(model) + ":generateContent",
    {
      contents: [{ parts: [{ text: prompt }] }],
      generationConfig,
    }
  );

  const parts = response.candidates?.flatMap((candidate) => candidate.content?.parts || []) || [];
  const textParts = [];
  const files = [];
  parts.forEach((part) => {
    if (part.text) textParts.push(part.text);
    const inlineData = part.inlineData || part.inline_data;
    if (inlineData?.data) files.push(writeImage(inlineData.data, inlineData.mimeType || inlineData.mime_type || "image/png", files.length + 1));
  });

  if (textParts.length) {
    console.log(textParts.join("\n"));
  }
  return files;
}

(async () => {
  const files = model.startsWith("imagen-") ? await generateWithImagen() : await generateWithNativeGemini();
  if (!files.length) {
    throw new Error("Gemini returned no image data. Check model access, prompt safety filters, and billing/quota.");
  }
  console.log(JSON.stringify({ ok: true, model, output_dir: outputDir, files }, null, 2));
})().catch((error) => {
  console.error(error.message || String(error));
  process.exit(1);
});
NODE
"###;

const GEMINI_VIDEO_GENERATE_SCRIPT: &str = r###"#!/usr/bin/env sh
set -eu
node <<'NODE'
const fs = require("fs");
const path = require("path");

const apiKey = process.env.GEMINI_API_KEY || process.env.GOOGLE_API_KEY;
if (!apiKey) {
  console.error("GEMINI_API_KEY or GOOGLE_API_KEY is required in the backend environment for Gemini video generation.");
  process.exit(2);
}

const prompt = (process.env.prompt || "").trim();
if (!prompt) {
  console.error("prompt is required");
  process.exit(2);
}

const model = (process.env.model || "veo-3.1-generate-preview").trim();
const outputDir = path.resolve(process.env.output_dir || "generated/gemini-videos");
const filenamePrefix = sanitize(process.env.filename_prefix || "gemini-video");
const aspectRatio = normalizeOptional(process.env.aspect_ratio);
const resolution = normalizeOptional(process.env.resolution);
const pollIntervalMs = clampInteger(process.env.poll_interval_seconds, 5, 60, 10) * 1000;
const maxWaitMs = clampInteger(process.env.max_wait_seconds, 30, 1200, 600) * 1000;
const baseUrl = "https://generativelanguage.googleapis.com/v1beta";

fs.mkdirSync(outputDir, { recursive: true });

function normalizeOptional(value) {
  const normalized = (value || "").trim();
  return normalized && normalized !== "default" ? normalized : "";
}

function clampInteger(value, min, max, fallback) {
  const parsed = Number.parseInt(value || "", 10);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function sanitize(value) {
  const sanitized = String(value).toLowerCase().replace(/[^a-z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
  return sanitized || "gemini-video";
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function readJson(response) {
  const text = await response.text();
  if (!text) return {};
  try { return JSON.parse(text); } catch { return { raw: text }; }
}

async function postJson(url, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: { "x-goog-api-key": apiKey, "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  const json = await readJson(response);
  if (!response.ok) throw new Error("Gemini API request failed (" + response.status + "): " + JSON.stringify(json));
  return json;
}

async function getJson(url) {
  const response = await fetch(url, { headers: { "x-goog-api-key": apiKey } });
  const json = await readJson(response);
  if (!response.ok) throw new Error("Gemini operation poll failed (" + response.status + "): " + JSON.stringify(json));
  return json;
}

function extractVideoUri(operation) {
  return operation.response?.generateVideoResponse?.generatedSamples?.[0]?.video?.uri
    || operation.response?.generatedVideos?.[0]?.video?.uri
    || operation.response?.generated_videos?.[0]?.video?.uri;
}

(async () => {
  const parameters = {};
  if (aspectRatio) parameters.aspectRatio = aspectRatio;
  if (resolution) parameters.resolution = resolution;

  const operation = await postJson(
    baseUrl + "/models/" + encodeURIComponent(model) + ":predictLongRunning",
    { instances: [{ prompt }], parameters }
  );
  if (!operation.name) throw new Error("Gemini did not return an operation name: " + JSON.stringify(operation));
  console.log("operation: " + operation.name);

  const deadline = Date.now() + maxWaitMs;
  let status = operation;
  while (!status.done) {
    if (Date.now() > deadline) {
      throw new Error("Timed out waiting for Gemini video operation " + operation.name);
    }
    await sleep(pollIntervalMs);
    status = await getJson(baseUrl + "/" + operation.name);
    console.log("waiting: " + operation.name);
  }

  if (status.error) throw new Error("Gemini video operation failed: " + JSON.stringify(status.error));
  const videoUri = extractVideoUri(status);
  if (!videoUri) throw new Error("Gemini operation completed without a downloadable video URI: " + JSON.stringify(status));

  const download = await fetch(videoUri, { headers: { "x-goog-api-key": apiKey } });
  if (!download.ok) {
    const text = await download.text();
    throw new Error("Video download failed (" + download.status + "): " + text);
  }

  const filePath = path.join(outputDir, filenamePrefix + ".mp4");
  fs.writeFileSync(filePath, Buffer.from(await download.arrayBuffer()));
  console.log(JSON.stringify({ ok: true, model, operation: operation.name, output_dir: outputDir, file: filePath }, null, 2));
})().catch((error) => {
  console.error(error.message || String(error));
  process.exit(1);
});
NODE
"###;
