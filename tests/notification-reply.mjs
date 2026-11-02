// Runs the page script's reply from a notification against a fake Messenger: chats
// with message boxes, an editor that shows changes a moment later, and a user
// who can open another chat at any time. `node tests/notification-reply.mjs`
import { readdirSync, readFileSync } from 'node:fs';

// The page script's parts, joined in file-name order as build.rs does.
// Windows checkouts may have CRLF line ends.
const parts = new URL('../src-tauri/src/inject/', import.meta.url);
const src = readdirSync(parts)
  .filter((name) => name.endsWith('.js'))
  .sort()
  .map((name) => readFileSync(new URL(name, parts), 'utf8'))
  .join('')
  .replace(/\r\n/g, '\n');
const waitFor = src.match(/  const waitFor = async[\s\S]*?\n  };\n/)[0];
const reply = src.match(/  let replying = Promise\.resolve\(\);[\s\S]*?\n    return true;\n  };\n/)[0];

function world() {
  const w = { sent: [], notices: [], backs: 0, boxes: {}, focused: null, path: '/messages/t/X/' };
  const box = (chat) => (w.boxes[chat] ??= {
    chat,
    textContent: '',
    isConnected: true,
    focus() { w.focused = this; },
    dispatchEvent(event) {
      if (event.key !== 'Enter' || !this.textContent.trim()) return;
      w.sent.push([this.chat, this.textContent]);
      setTimeout(() => (this.textContent = ''), 40);
    },
  });
  w.open = (chat) => {
    if (w.current) w.current.isConnected = false;
    w.path = `/messages/t/${chat}/`;
    w.current = box(chat);
    w.current.isConnected = true;
    // Opening a chat puts the cursor in its message box (fMess does too).
    w.focused = w.current;
  };
  w.open('X');
  // Messenger changes the address first, the message box a moment later.
  w.openSlowly = (chat, ms) => {
    w.path = `/messages/t/${chat}/`;
    setTimeout(() => w.open(chat), ms);
  };
  const env = {
    location: { get pathname() { return w.path; } },
    history: { back: () => (w.backs += 1) },
    composer: () => w.current,
    openChat: (id, thread) => setTimeout(() => w.open(thread), 150),
    document: {
      execCommand(cmd, _, value) {
        const target = w.focused;
        if (cmd === 'delete') setTimeout(() => (target.textContent = ''), 120);
        // The editor shows typed text a moment later.
        if (cmd === 'insertText') setTimeout(() => (target.textContent += value), 120);
      },
    },
    FMessNotification: class { constructor(title, { body }) { w.notices.push(`${title}: ${body}`); } },
    KeyboardEvent: class { constructor(type, init) { Object.assign(this, init); } },
    window: {},
  };
  const run = new Function(...Object.keys(env), `${waitFor}${reply}return window;`);
  const page = run(...Object.values(env));
  w.reply = page.__fmessReply;
  w.returnDraft = page.__fmessReturnDraft;
  return w;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (name, ok, detail) => results.push(`${ok ? 'PASS' : 'FAIL'} ${name}${ok ? '' : `: ${JSON.stringify(detail)}`}`);

{
  const w = world();
  const ok = await w.reply(1, 'A', 'hi A');
  check('replies in its chat, then goes back', ok && w.sent.length === 1 && w.sent[0][0] === 'A' && w.backs === 1, w);
}
{
  const w = world();
  const done = w.reply(1, 'A', 'for A');
  // A opens at 150 ms; the typed text shows at about 270 ms. Open B at 220 ms.
  await sleep(220);
  w.open('B');
  const ok = await done;
  check('another chat opened meanwhile: nothing sent anywhere', !ok && w.sent.length === 0 && w.boxes.B.textContent === '' && w.notices.length === 1 && w.backs === 0, { ok, sent: w.sent, B: w.boxes.B.textContent, notices: w.notices });
}
{
  const w = world();
  const [a, b] = await Promise.all([w.reply(1, 'A', 'for A'), w.reply(2, 'B', 'for B')]);
  check('two replies at once: each in its own chat', a && b && JSON.stringify(w.sent) === JSON.stringify([['A', 'for A'], ['B', 'for B']]), w.sent);
}
{
  const w = world();
  w.open('A');
  w.boxes.A.textContent = 'my draft';
  const ok = await w.reply(1, 'A', 'quick');
  await sleep(200);
  check('a draft comes back after the reply', ok && w.sent[0]?.[1] === 'quick' && w.boxes.A.textContent === 'my draft', { sent: w.sent, A: w.boxes.A.textContent });
}
{
  // Codex's case: the chat has a draft, and another chat opens while the
  // editor is taking the draft out.
  const w = world();
  w.open('A');
  w.boxes.A.textContent = 'my draft';
  w.open('X');
  const done = w.reply(1, 'A', 'for A');
  await sleep(250);
  w.open('B');
  const ok = await done;
  await sleep(200);
  check('another chat opened while the draft goes out: nothing sent, draft kept in the notice', !ok && w.sent.length === 0 && w.boxes.B.textContent === '' && /my draft/.test(w.notices[0] ?? ''), { ok, sent: w.sent, B: w.boxes.B.textContent, notices: w.notices });
}
{
  // Codex's second case: the reply goes out, then another chat opens before
  // the draft is back in the first one.
  const w = world();
  w.open('A');
  w.boxes.A.textContent = 'my draft';
  w.open('X');
  const done = w.reply(1, 'A', 'quick');
  while (!w.sent.length) await sleep(5);
  w.open('B');
  const ok = await done;
  const kept = w.notices.some((notice) => /my draft/.test(notice));
  w.open('A');
  await w.returnDraft();
  await sleep(200);
  check('another chat opened right after sending: the draft comes back to its chat', ok && w.sent.length === 1 && w.sent[0][0] === 'A' && w.boxes.B.textContent === '' && kept && w.boxes.A.textContent === 'my draft', { ok, sent: w.sent, A: w.boxes.A.textContent, B: w.boxes.B.textContent, notices: w.notices });
}
{
  // Codex's third case: back to the chat with a kept draft, its address
  // before its message box. The draft mustn't go into the box still there.
  const w = world();
  w.open('A');
  w.boxes.A.textContent = 'my draft';
  w.open('X');
  const done = w.reply(1, 'A', 'quick');
  while (!w.sent.length) await sleep(5);
  w.open('B');
  await done;
  w.boxes.B.textContent = '';
  const previousBox = w.current;
  w.openSlowly('A', 300);
  const returning = w.returnDraft(previousBox);
  await sleep(150);
  const inBWhileSlow = w.boxes.B.textContent;
  await returning;
  await sleep(200);
  check('address before message box: the draft waits for its own chat', inBWhileSlow === '' && w.boxes.B.textContent === '' && w.boxes.A.textContent === 'my draft', { inBWhileSlow, B: w.boxes.B.textContent, A: w.boxes.A.textContent });
}
{
  // Codex's fourth case: A, then C, then A again before A's box came. The
  // first visit's wait must not take C's box for A's.
  const w = world();
  w.open('A');
  w.boxes.A.textContent = 'my draft';
  w.open('X');
  const done = w.reply(1, 'A', 'quick');
  while (!w.sent.length) await sleep(5);
  w.open('B');
  await done;
  let visit = 0;
  const go = (chat, ms) => {
    const previousBox = w.current;
    const mine = ++visit;
    w.openSlowly(chat, ms);
    return w.returnDraft(previousBox, () => mine === visit);
  };
  const first = go('A', 400);
  await sleep(100);
  go('C', 50);
  await sleep(150);
  const second = go('A', 300);
  await Promise.all([first, second]);
  await sleep(300);
  check('A, C, then A again: the draft goes back to A once, never to C', (w.boxes.C?.textContent ?? '') === '' && w.boxes.A.textContent === 'my draft', { A: w.boxes.A.textContent, C: w.boxes.C?.textContent });
}
console.log(results.join('\n'));
if (results.some((line) => line.startsWith('FAIL'))) process.exit(1);
