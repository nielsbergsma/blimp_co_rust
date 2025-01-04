import http from "http"
import fs from "fs/promises"

const assetPath = "../frontend/backoffice";
const prefix = "/backoffice";
const host = '127.0.0.1';
const port = 5001;

const pagePaths = [
    "",
    "/",
    "/flight-scheduling",
    "/reservations"
];

const mimeTypesPerExtension = {
    ".js": "text/javascript",
    ".html": "text/html",
    ".css": "text/css",
    ".webp": "image/webp",
    ".svg": "image/svg",
}

function getContentType(path) {
    for (const [key, value] of Object.entries(mimeTypesPerExtension)) {
        if (path.endsWith(key)) {
            return value;
        }
    }
    return "application/octet-stream"
}

async function handler(request, response) {
    let path = decodeURIComponent(request.url);
    if (path.startsWith(prefix)) {
        path = path.substring(prefix.length);
    }

    if (pagePaths.includes(path)) {
        path = "/index.html"
    }

    try {
        const content = await fs.readFile(assetPath + path);
        response.setHeader("Content-Type", getContentType(path));
        response.end(content);
    } catch (exception) {
        console.error(exception);
        response.writeHead(404, { "Content-Type": "text/plain" });
        response.end("not found");
    }
}

const server = http.createServer(handler);
server.listen(port, host, () => {
    console.log(`server is running on http://${host}:${port}`);
});
