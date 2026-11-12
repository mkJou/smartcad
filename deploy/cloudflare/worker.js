// SmartCAD: sirve el bundle web (wasm de ~97 MB, mayor al límite de Pages)
// desde R2, con las cabeceras COOP/COEP que habilitan SharedArrayBuffer.
const TYPES = {
  html: 'text/html; charset=utf-8',
  js: 'text/javascript; charset=utf-8',
  mjs: 'text/javascript; charset=utf-8',
  wasm: 'application/wasm',
  json: 'application/json; charset=utf-8',
  webmanifest: 'application/manifest+json',
  svg: 'image/svg+xml',
  png: 'image/png',
  ico: 'image/x-icon',
  css: 'text/css; charset=utf-8',
  ttf: 'font/ttf',
  otf: 'font/otf',
  woff: 'font/woff',
  woff2: 'font/woff2',
  txt: 'text/plain; charset=utf-8',
};

// Archivos con hash en el nombre (trunk): se pueden cachear para siempre.
const HASHED = /-[0-9a-f]{16}(_bg)?\.(wasm|js|css|svg)$/;

export default {
  async fetch(request, env) {
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      return new Response('Method Not Allowed', { status: 405 });
    }
    const url = new URL(request.url);
    let key = decodeURIComponent(url.pathname.replace(/^\/+/, ''));
    if (!key || key.endsWith('/')) key += 'index.html';
    if (key.includes('..')) return new Response('Bad Request', { status: 400 });

    const wantsRange = request.headers.has('Range');
    let object = await env.BUNDLE.get(key, {
      onlyIf: request.headers,
      ...(wantsRange ? { range: request.headers } : {}),
    });
    if (!object && !key.includes('.')) {
      key = 'index.html';
      object = await env.BUNDLE.get(key);
    }
    if (!object) return new Response('Not Found', { status: 404 });

    const ext = key.split('.').pop().toLowerCase();
    const headers = new Headers();
    object.writeHttpMetadata(headers);
    headers.set('Content-Type', TYPES[ext] || 'application/octet-stream');
    headers.set('ETag', object.httpEtag);
    headers.set('Cross-Origin-Opener-Policy', 'same-origin');
    headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
    headers.set('Cross-Origin-Resource-Policy', 'same-origin');
    headers.set('X-Content-Type-Options', 'nosniff');
    headers.set(
      'Cache-Control',
      HASHED.test(key)
        ? 'public, max-age=31536000, immutable'
        : 'no-cache',
    );

    // Sin cuerpo: 304 por If-None-Match.
    if (!('body' in object) || !object.body) {
      return new Response(null, { status: 304, headers });
    }
    return new Response(request.method === 'HEAD' ? null : object.body, {
      status: wantsRange && object.range ? 206 : 200,
      headers,
    });
  },
};
