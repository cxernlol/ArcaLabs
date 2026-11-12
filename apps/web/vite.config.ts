import { existsSync, readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig, type Connect, type PreviewServer, type ViteDevServer } from "vite";

const webRoot = fileURLToPath(new URL(".", import.meta.url));

function isPage(path: string): boolean {
  if (!existsSync(path)) return false;

  const stats = statSync(path);
  return stats.isFile() || (stats.isDirectory() && existsSync(resolve(path, "index.html")));
}

function customNotFoundMiddleware(root: string, transformHtml: (pathname: string, html: string) => Promise<string>): Connect.NextHandleFunction {
  const notFoundHtml = readFileSync(resolve(root, "404.html"), "utf8");

  return (request, response, next) => {
    const accept = request.headers.accept;
    if ((request.method !== "GET" && request.method !== "HEAD") || typeof accept !== "string" || !accept.includes("text/html")) {
      next();
      return;
    }

    const pathname = new URL(request.url ?? "/", "http://localhost").pathname;
    const relativePath = pathname.replace(/^\/+/, "");
    if (isPage(resolve(root, relativePath)) || isPage(resolve(root, "public", relativePath))) {
      next();
      return;
    }

    void transformHtml(pathname, notFoundHtml).then((html) => {
      response.statusCode = 404;
      response.setHeader("Content-Type", "text/html; charset=utf-8");
      if (request.method === "HEAD") {
        response.end();
      } else {
        response.end(html);
      }
    }).catch(next);
  };
}

const notFoundPlugin = {
  name: "custom-not-found",
  configureServer(server: ViteDevServer) {
    server.middlewares.use(customNotFoundMiddleware(webRoot, (pathname, html) => server.transformIndexHtml(pathname, html)));
  },
  configurePreviewServer(server: PreviewServer) {
    const distRoot = resolve(webRoot, "dist");
    server.middlewares.use(customNotFoundMiddleware(distRoot, (_pathname, html) => Promise.resolve(html)));
  },
};

export default defineConfig({
  plugins: [react(), notFoundPlugin],
  build: {
    rollupOptions: {
      input: {
        main: resolve(webRoot, "index.html"),
        privacy: resolve(webRoot, "privacy/index.html"),
        terms: resolve(webRoot, "terms/index.html"),
        notFound: resolve(webRoot, "404.html"),
      },
    },
  },
});
