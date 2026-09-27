// Captures the current Touch Bar frame from a react-drm preview instance
// (start one with: cd ~/opencode/react-drm/linux-touchbar-control-center &&
//  REACT_DRM_BACKEND=preview REACT_DRM_PREVIEW_PORT=8799 npx tsx index.tsx)
import WebSocket from '/home/Astra/opencode/react-drm/node_modules/ws/index.js';
import fs from 'fs';
let frame = null;
const ws = new WebSocket('ws://127.0.0.1:8799/ws');
ws.on('message', (d, isBinary) => { if (isBinary) frame = d.subarray(16); });
ws.on('open', async () => {
  await new Promise(r => setTimeout(r, 800));
  if (!frame) { console.log('no frame'); process.exit(1); }
  const w = 2008, h = 60;
  const rgb = [];
  for (let y = 0; y < h; y++) {
    const row = frame.subarray(y * w * 4, (y + 1) * w * 4);
    for (let x = 0; x < w; x++) rgb.push(row[x*4+2], row[x*4+1], row[x*4]);
  }
  const zlib = await import('zlib');
  const chunk = (t, d) => { const c = Buffer.concat([t, d]); return Buffer.concat([Buffer.from([(d.length>>>24)&255,(d.length>>>16)&255,(d.length>>>8)&255,d.length&255]), c, Buffer.from([zlib.crc32 ? 0 : 0])]); };
  // simpler: write raw then let the python helper make the PNG
  fs.writeFileSync('/home/Astra/opencode/zcode/mtmr-linux/scripts/previews/current-bar.rgba', Buffer.from(rgb));
  console.log('saved current-bar.rgba');
  process.exit(0);
});
setTimeout(() => { console.log('timeout'); process.exit(1); }, 15000);
