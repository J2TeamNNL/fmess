// Checks how inject-instagram.js reads the unread chat count from Instagram's
// page title. `node tests/instagram-unread.mjs`
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const src = readFileSync(new URL('../src-tauri/src/inject-instagram.js', import.meta.url), 'utf8').replace(/\r\n/g, '\n');
const unreadFromTitle = new Function(`${src.match(/  const unreadFromTitle = .*\n/)[0]}\nreturn unreadFromTitle;`)();

assert.equal(unreadFromTitle('(3) Instagram'), 3);
assert.equal(unreadFromTitle('(12) Inbox • Chats'), 12);
assert.equal(unreadFromTitle(' (1) Instagram'), 1);
assert.equal(unreadFromTitle('(9+) Instagram'), 9);
assert.equal(unreadFromTitle('Instagram'), 0);
assert.equal(unreadFromTitle('Inbox (2) • Chats'), 0);
assert.equal(unreadFromTitle(''), 0);
assert.equal(unreadFromTitle(undefined), 0);
console.log('instagram unread: ok');
