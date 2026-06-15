<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{
  modelValue?: string | null;
}>();

defineEmits<{
  "update:modelValue": [value: string];
}>();

const textareaRef = ref<HTMLTextAreaElement | null>(null);
const previewRef = ref<HTMLPreElement | null>(null);
const gutterRef = ref<HTMLPreElement | null>(null);

const code = computed(() => props.modelValue ?? "");
const lineNumbers = computed(() => Array.from({ length: code.value.split("\n").length }, (_, index) => index + 1).join("\n"));
const highlightedHtml = computed(() => `${highlightShell(code.value)}\n`);

function syncScroll(): void {
  if (!textareaRef.value) {
    return;
  }

  if (previewRef.value) {
    previewRef.value.scrollTop = textareaRef.value.scrollTop;
    previewRef.value.scrollLeft = textareaRef.value.scrollLeft;
  }

  if (gutterRef.value) {
    gutterRef.value.scrollTop = textareaRef.value.scrollTop;
  }
}

function highlightShell(source: string): string {
  return source.split("\n").map(highlightLine).join("\n");
}

function highlightLine(line: string): string {
  let index = 0;
  let output = "";

  while (index < line.length) {
    const chunk = nextHighlightedChunk(line, index);
    output += chunk.html;
    index = chunk.nextIndex;
  }

  return output || "&nbsp;";
}

function nextHighlightedChunk(line: string, index: number): { html: string; nextIndex: number } {
  const character = line[index];

  if (character === "#" && (index === 0 || /\s/.test(line[index - 1]))) {
    return { html: shellSpan("sh-com", line.slice(index)), nextIndex: line.length };
  }

  if (character === '"' || character === "'") {
    return stringChunk(line, index, character);
  }

  if (character === "$") {
    return regexChunk(line, index, /^(\$\{[^}]*\}|\$\([^)]*\)|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?!])/, "sh-var");
  }

  if (/[0-9]/.test(character) && (index === 0 || /[^A-Za-z0-9_]/.test(line[index - 1]))) {
    return regexChunk(line, index, /^[0-9]+(\.[0-9]+)*/, "sh-num");
  }

  if (/[A-Za-z_-]/.test(character)) {
    return wordChunk(line, index);
  }

  if ("|&;<>(){}=".includes(character)) {
    let end = index + 1;
    while (end < line.length && "|&;<>".includes(line[end])) {
      end += 1;
    }

    return { html: shellSpan("sh-op", line.slice(index, end)), nextIndex: end };
  }

  return { html: escapeHtml(character), nextIndex: index + 1 };
}

function stringChunk(line: string, index: number, quote: string): { html: string; nextIndex: number } {
  let end = index + 1;
  while (end < line.length && !(line[end] === quote && line[end - 1] !== "\\")) {
    end += 1;
  }

  end = Math.min(end + 1, line.length);
  return { html: highlightString(quote, line.slice(index, end)), nextIndex: end };
}

function regexChunk(line: string, index: number, pattern: RegExp, className: string): { html: string; nextIndex: number } {
  const match = pattern.exec(line.slice(index));
  if (!match) {
    return { html: escapeHtml(line[index]), nextIndex: index + 1 };
  }

  return { html: shellSpan(className, match[0]), nextIndex: index + match[0].length };
}

function wordChunk(line: string, index: number): { html: string; nextIndex: number } {
  const match = /^(-{1,2}[A-Za-z0-9][A-Za-z0-9-]*|[A-Za-z0-9_][A-Za-z0-9_-]*)/.exec(line.slice(index));
  if (!match) {
    return { html: escapeHtml(line[index]), nextIndex: index + 1 };
  }

  const word = match[0];
  if (word.startsWith("-")) {
    return { html: shellSpan("sh-flag", word), nextIndex: index + word.length };
  }

  if (SHELL_KEYWORDS.has(word)) {
    return { html: shellSpan("sh-kw", word), nextIndex: index + word.length };
  }

  if (SHELL_FUNCTIONS.has(word)) {
    return { html: shellSpan("sh-fn", word), nextIndex: index + word.length };
  }

  return { html: escapeHtml(word), nextIndex: index + word.length };
}

function highlightString(quote: string, raw: string): string {
  if (quote === "'") {
    return shellSpan("sh-str", raw);
  }

  let output = "";
  let lastIndex = 0;
  const pattern = /(\$\{[^}]*\}|\$\([^)]*\)|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?!])/g;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(raw))) {
    if (match.index > lastIndex) {
      output += shellSpan("sh-str", raw.slice(lastIndex, match.index));
    }
    output += shellSpan("sh-var", match[0]);
    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < raw.length) {
    output += shellSpan("sh-str", raw.slice(lastIndex));
  }

  return output;
}

function shellSpan(className: string, text: string): string {
  return `<span class="${className}">${escapeHtml(text)}</span>`;
}

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

const SHELL_KEYWORDS = new Set([
  "set",
  "if",
  "then",
  "elif",
  "else",
  "fi",
  "for",
  "while",
  "until",
  "do",
  "done",
  "case",
  "esac",
  "in",
  "function",
  "return",
  "local",
  "export",
  "declare",
  "readonly",
  "break",
  "continue",
  "exit",
  "source",
  "time",
]);

const SHELL_FUNCTIONS = new Set([
  "echo",
  "cd",
  "pwd",
  "read",
  "printf",
  "test",
  "eval",
  "exec",
  "trap",
  "shift",
  "kill",
  "wait",
  "sleep",
  "true",
  "false",
  "cat",
  "grep",
  "sed",
  "awk",
  "cut",
  "sort",
  "uniq",
  "head",
  "tail",
  "find",
  "xargs",
  "date",
  "seq",
  "ping",
  "journalctl",
  "cwebp",
  "pg_dump",
  "gzip",
  "gunzip",
  "tar",
  "curl",
  "wget",
  "ssh",
  "scp",
  "rsync",
  "mkdir",
  "rm",
  "cp",
  "mv",
  "chmod",
  "chown",
  "ln",
  "touch",
  "kubectl",
  "docker",
  "git",
  "make",
  "npm",
  "node",
  "python",
  "go",
]);
</script>

<template>
  <div class="glass code-surface code-editor">
    <pre ref="gutterRef" aria-hidden="true" class="t-mono code-editor__gutter">{{ lineNumbers }}</pre>
    <div class="code-editor__stage">
      <pre ref="previewRef" aria-hidden="true" class="scroll t-mono code-editor__preview" v-html="highlightedHtml" />
      <textarea
        ref="textareaRef"
        class="scroll t-mono code-editor__textarea"
        spellcheck="false"
        :value="modelValue ?? ''"
        @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
        @scroll="syncScroll"
      />
    </div>
  </div>
</template>

<style scoped>
.code-editor {
  display: flex;
  overflow: hidden;
  border-radius: 14px;
  background: var(--console-bg);
}

.code-editor__gutter {
  overflow: hidden;
  min-width: 46px;
  max-height: 380px;
  margin: 0;
  border-right: 1px solid var(--divider);
  color: var(--text-tertiary);
  font-size: 13px;
  line-height: 21px;
  padding: 14px 12px 14px 16px;
  text-align: right;
  user-select: none;
  white-space: pre;
}

.code-editor__stage {
  position: relative;
  flex: 1;
  height: 380px;
}

.code-editor__preview,
.code-editor__textarea {
  position: absolute;
  inset: 0;
  overflow: auto;
  width: 100%;
  height: 100%;
  margin: 0;
  border: none;
  outline: none;
  font-size: 13px;
  line-height: 21px;
  padding: 14px 16px;
  white-space: pre;
}

.code-editor__preview {
  color: var(--text-primary);
  pointer-events: none;
}

.code-editor__textarea {
  resize: none;
  background: transparent;
  color: transparent;
  -webkit-text-fill-color: transparent;
}
</style>
