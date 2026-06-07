import type { Page } from "playwright";
import type {
  NavigateRequest,
  NavigateResponse,
  ClickRequest,
  TypeRequest,
  FillRequest,
  SelectRequest,
  ScrollRequest,
  ScreenshotOptions,
  ScreenshotResponse,
  EvaluateRequest,
  WaitForRequest,
  GetContentRequest,
  ContentResponse,
  CookieRequest,
  SetCookiesRequest,
  ElementInfo,
  AccessibilityNode,
} from "./types.js";

export async function navigate(page: Page, req: NavigateRequest): Promise<NavigateResponse> {
  const response = await page.goto(req.url, {
    timeout: req.timeout ?? 30000,
    waitUntil: req.waitUntil ?? "domcontentloaded",
  });

  return {
    url: page.url(),
    title: await page.title(),
    status: response?.status() ?? null,
  };
}

export async function click(page: Page, req: ClickRequest): Promise<void> {
  if (req.selector) {
    await page.click(req.selector, {
      button: req.button ?? "left",
      clickCount: req.clickCount ?? 1,
      timeout: req.timeout ?? 10000,
    });
  } else if (req.position) {
    await page.mouse.click(req.position.x, req.position.y, {
      button: req.button ?? "left",
      clickCount: req.clickCount ?? 1,
    });
  } else {
    throw new Error("Either selector or position must be provided");
  }
}

export async function type(page: Page, req: TypeRequest): Promise<void> {
  await page.fill(req.selector, "", { timeout: req.timeout ?? 10000 });
  await page.type(req.selector, req.text, { delay: req.delay ?? 0 });
}

export async function fill(page: Page, req: FillRequest): Promise<void> {
  await page.fill(req.selector, req.value, { timeout: req.timeout ?? 10000 });
}

export async function select(page: Page, req: SelectRequest): Promise<string[]> {
  return page.selectOption(req.selector, req.values, { timeout: req.timeout ?? 10000 });
}

export async function scroll(page: Page, req: ScrollRequest): Promise<void> {
  const amount = req.amount ?? 500;
  const deltaX = req.direction === "left" ? -amount : req.direction === "right" ? amount : 0;
  const deltaY = req.direction === "up" ? -amount : req.direction === "down" ? amount : 0;

  if (req.selector) {
    await page.evaluate(
      ({ sel, dx, dy }) => {
        const el = document.querySelector(sel);
        if (el) el.scrollBy(dx, dy);
      },
      { sel: req.selector, dx: deltaX, dy: deltaY },
    );
  } else {
    await page.mouse.wheel(deltaX, deltaY);
  }
}

export async function screenshot(page: Page, options: ScreenshotOptions = {}): Promise<ScreenshotResponse> {
  const type = options.type ?? "png";
  const screenshotOptions: Record<string, unknown> = {
    type,
    fullPage: options.fullPage ?? false,
  };

  if (type === "jpeg" && options.quality !== undefined) {
    screenshotOptions.quality = options.quality;
  }

  let buffer: Buffer;
  if (options.selector) {
    const element = page.locator(options.selector);
    const { fullPage: _, ...elementOptions } = screenshotOptions;
    buffer = await element.screenshot(elementOptions);
  } else {
    buffer = await page.screenshot(screenshotOptions);
  }

  return {
    data: buffer.toString("base64"),
    mimeType: type === "jpeg" ? "image/jpeg" : "image/png",
  };
}

export async function evaluate(page: Page, req: EvaluateRequest): Promise<unknown> {
  return page.evaluate(req.expression);
}

export async function waitFor(page: Page, req: WaitForRequest): Promise<void> {
  if (req.selector) {
    await page.waitForSelector(req.selector, {
      timeout: req.timeout ?? 30000,
      state: req.state ?? "visible",
    });
  } else if (req.url) {
    await page.waitForURL(req.url, { timeout: req.timeout ?? 30000 });
  } else {
    throw new Error("Either selector or url must be provided");
  }
}

export async function getContent(page: Page, req: GetContentRequest): Promise<ContentResponse> {
  const url = page.url();
  const title = await page.title();
  let content: string;

  const target = req.selector ? await page.locator(req.selector).first() : null;

  switch (req.format) {
    case "html":
      content = target ? await target.innerHTML() : await page.content();
      break;
    case "text":
      content = target ? (await target.textContent()) ?? "" : await page.innerText("body");
      break;
    case "markdown":
      content = await page.evaluate(() => {
        const body = document.body;
        if (!body) return "";

        function nodeToMarkdown(node: Node, depth: number = 0): string {
          if (node.nodeType === Node.TEXT_NODE) {
            return node.textContent ?? "";
          }

          if (node.nodeType !== Node.ELEMENT_NODE) return "";

          const el = node as Element;
          const tag = el.tagName.toLowerCase();

          if (["script", "style", "noscript"].includes(tag)) return "";

          const children = Array.from(node.childNodes)
            .map((child) => nodeToMarkdown(child, depth + 1))
            .join("");

          switch (tag) {
            case "h1": return `\n# ${children.trim()}\n`;
            case "h2": return `\n## ${children.trim()}\n`;
            case "h3": return `\n### ${children.trim()}\n`;
            case "h4": return `\n#### ${children.trim()}\n`;
            case "h5": return `\n##### ${children.trim()}\n`;
            case "h6": return `\n###### ${children.trim()}\n`;
            case "p": return `\n${children.trim()}\n`;
            case "br": return "\n";
            case "hr": return "\n---\n";
            case "strong":
            case "b": return `**${children.trim()}**`;
            case "em":
            case "i": return `*${children.trim()}*`;
            case "code": return `\`${children.trim()}\``;
            case "pre": return `\n\`\`\`\n${children.trim()}\n\`\`\`\n`;
            case "a": {
              const href = el.getAttribute("href");
              return href ? `[${children.trim()}](${href})` : children.trim();
            }
            case "img": {
              const alt = el.getAttribute("alt") ?? "";
              const src = el.getAttribute("src") ?? "";
              return `![${alt}](${src})`;
            }
            case "li": return `\n- ${children.trim()}`;
            case "ul":
            case "ol": return `\n${children}\n`;
            case "blockquote": return `\n> ${children.trim()}\n`;
            case "table": return `\n${children}\n`;
            case "tr": return `| ${children.trim()} |\n`;
            case "th":
            case "td": return ` ${children.trim()} |`;
            default: return children;
          }
        }

        return nodeToMarkdown(body).replace(/\n{3,}/g, "\n\n").trim();
      });
      break;
    default:
      content = await page.content();
  }

  return { content, format: req.format, url, title };
}

export async function getCookies(page: Page, req?: CookieRequest) {
  const context = page.context();
  return req?.urls ? context.cookies(req.urls) : context.cookies();
}

export async function setCookies(page: Page, req: SetCookiesRequest): Promise<void> {
  const context = page.context();
  await context.addCookies(req.cookies);
}

export async function getElements(page: Page, selector: string): Promise<ElementInfo[]> {
  return page.evaluate((sel: string) => {
    const elements = document.querySelectorAll(sel);
    return Array.from(elements).map((el) => {
      const rect = el.getBoundingClientRect();
      const attributes: Record<string, string> = {};
      for (const attr of Array.from(el.attributes)) {
        attributes[attr.name] = attr.value;
      }
      return {
        tagName: el.tagName.toLowerCase(),
        text: (el.textContent ?? "").trim().slice(0, 200),
        attributes,
        boundingBox: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
      };
    });
  }, selector);
}

export async function getAccessibilityTree(page: Page): Promise<AccessibilityNode | null> {
  // TODO: accessibility API may not be available in all Playwright versions
  console.warn("getAccessibilityTree is not implemented yet");
  return null;
}

export async function pressKey(page: Page, key: string): Promise<void> {
  await page.keyboard.press(key);
}

export async function hover(page: Page, selector: string): Promise<void> {
  await page.hover(selector);
}

export async function dragAndDrop(page: Page, source: string, target: string): Promise<void> {
  await page.locator(source).dragTo(page.locator(target));
}

export async function uploadFile(page: Page, selector: string, filePaths: string[]): Promise<void> {
  await page.locator(selector).setInputFiles(filePaths);
}

export async function goBack(page: Page): Promise<NavigateResponse> {
  await page.goBack({ waitUntil: "domcontentloaded" });
  return { url: page.url(), title: await page.title(), status: null };
}

export async function goForward(page: Page): Promise<NavigateResponse> {
  await page.goForward({ waitUntil: "domcontentloaded" });
  return { url: page.url(), title: await page.title(), status: null };
}

export async function reload(page: Page): Promise<NavigateResponse> {
  await page.reload({ waitUntil: "domcontentloaded" });
  return { url: page.url(), title: await page.title(), status: null };
}
