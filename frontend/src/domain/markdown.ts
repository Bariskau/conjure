import MarkdownIt from "markdown-it";

const renderer = new MarkdownIt({
  breaks: true,
  html: false,
  linkify: true,
  typographer: false,
});

renderer.renderer.rules.link_open = (tokens, index, options, env, self) => {
  const token = tokens[index];
  token.attrSet("target", "_blank");
  token.attrSet("rel", "noopener noreferrer");
  return self.renderToken(tokens, index, options);
};

export function renderMarkdown(source: string | null | undefined): string {
  const markdown = source?.trim() || "(empty)";
  return renderer.render(markdown);
}
