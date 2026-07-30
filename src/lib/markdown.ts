import { marked } from "marked";
import DOMPurify from "dompurify";

// 子集白名单（design.md ADR-3）：仅放行排版类标签。
// 表格/图片/链接/脚本由 DOMPurify 白名单排除 → 降级为纯文本。
const ALLOWED_TAGS = [
  "b", "strong", "i", "em", "del", "s",
  "h1", "h2", "h3",
  "ul", "ol", "li",
  "code", "blockquote", "hr",
  "p", "br",
] as const;

/** 源码 → sanitize 后的 HTML。渲染管道单点出口（design.md LLD）。
 *  breaks:true —— 便签是短文本，单换行渲染为 <br> 符合直觉。 */
export function renderMd(src: string): string {
  const raw = marked.parse(src, { async: false, breaks: true }) as string;
  return DOMPurify.sanitize(raw, { ALLOWED_TAGS: [...ALLOWED_TAGS] });
}
