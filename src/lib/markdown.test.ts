import { describe, it, expect } from "vitest";
import { renderMd } from "./markdown";

describe("renderMd — 子集渲染", () => {
  it("加粗 **x** → <strong>", () => {
    expect(renderMd("**重点**")).toContain("<strong>重点</strong>");
  });
  it("斜体 *x* → <em>", () => {
    expect(renderMd("*斜*")).toContain("<em>斜</em>");
  });
  it("删除线 ~~x~~ → <del>", () => {
    expect(renderMd("~~旧~~")).toContain("<del>旧</del>");
  });
  it("行内 code `x` → <code>", () => {
    expect(renderMd("`code`")).toContain("<code>code</code>");
  });
  it("标题 # → <h1>", () => {
    expect(renderMd("# 标题")).toContain("<h1>");
  });
  it("引用 > → <blockquote>", () => {
    expect(renderMd("> 引用")).toContain("<blockquote>");
  });
  it("无序列表 - → <ul><li>", () => {
    const html = renderMd("- 项\n- 项二");
    expect(html).toContain("<ul>");
    expect(html).toContain("<li>项</li>");
  });
  it("有序列表 1. → <ol>", () => {
    expect(renderMd("1. 一\n2. 二")).toContain("<ol>");
  });
  it("分隔线 --- → <hr>", () => {
    expect(renderMd("---")).toContain("<hr");
  });
});

describe("renderMd — XSS 防护", () => {
  it("剥离 <script>", () => {
    const html = renderMd("<script>alert(1)</script>");
    expect(html).not.toContain("<script");
    expect(html).not.toContain("alert(1)");
  });
  it("剥离 on* 事件属性", () => {
    const html = renderMd(`<b onclick="evil()">x</b>`);
    expect(html).not.toContain("onclick");
  });
  it("剥离 javascript: 伪协议", () => {
    const html = renderMd(`<a href="javascript:evil()">x</a>`);
    expect(html).not.toContain("javascript:");
  });
});

describe("renderMd — 排除项降级", () => {
  it("图片 ![](url) 不渲染 <img>", () => {
    const html = renderMd("![alt](http://x/a.png)");
    expect(html).not.toContain("<img");
  });
  it("链接 [t](url) 不渲染为可点击 <a>", () => {
    const html = renderMd("[文本](http://x)");
    expect(html).not.toContain("<a ");
  });
});
