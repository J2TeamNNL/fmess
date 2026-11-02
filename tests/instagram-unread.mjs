// Checks how inject-instagram.js counts unread Instagram chats (the title,
// the badge on Instagram's Messages icon and the chat rows), splits a row's
// texts and tells which side a bubble is on.
// `node tests/instagram-unread.mjs`
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const src = readFileSync(new URL('../src-tauri/src/inject-instagram.js', import.meta.url), 'utf8').replace(/\r\n/g, '\n');
const block = src.match(/\/\/ unread-pure:start\n([\s\S]*?)\/\/ unread-pure:end/)[1];
const { unreadFromTitle, badgeCount, isUnreadRow, unreadTotal, rowParts, rowTexts, bubbleDir } = new Function(
  `${block}\nreturn { unreadFromTitle, badgeCount, isUnreadRow, unreadTotal, rowParts, rowTexts, bubbleDir };`,
)();

assert.equal(unreadFromTitle('(3) Instagram'), 3);
assert.equal(unreadFromTitle('(12) Inbox • Chats'), 12);
assert.equal(unreadFromTitle(' (1) Instagram'), 1);
assert.equal(unreadFromTitle('(9+) Instagram'), 9);
assert.equal(unreadFromTitle('Instagram • Tin nhắn'), 0);
assert.equal(unreadFromTitle('Inbox (2) • Chats'), 0);
assert.equal(unreadFromTitle(''), 0);
assert.equal(unreadFromTitle(undefined), 0);

assert.equal(badgeCount('3'), 3);
assert.equal(badgeCount(' 12 '), 12);
assert.equal(badgeCount('9+'), 9);
assert.equal(badgeCount(''), 0);
assert.equal(badgeCount('Tin nhắn'), 0);
assert.equal(badgeCount(undefined), 0);

assert.equal(isUnreadRow({ weights: [400, 400, 400] }), false);
assert.equal(isUnreadRow({ weights: [600, 400] }), true);
assert.equal(isUnreadRow({ weights: [400, 700] }), true);
assert.equal(isUnreadRow({ weights: [400], dot: true }), true);
assert.equal(isUnreadRow({}), false);

assert.equal(unreadTotal({}), 0);
assert.equal(unreadTotal({ title: 2, badge: 0, rows: 0 }), 2);
assert.equal(unreadTotal({ title: 0, badge: 4, rows: 2 }), 4);
assert.equal(unreadTotal({ title: 0, badge: 0, rows: 3 }), 3);

assert.deepEqual(rowTexts(['Hâm', 'Em đã gửi một file đính kèm.', '·', '21 giờ']), { name: 'Hâm', preview: 'Em đã gửi một file đính kèm.' });
assert.deepEqual(rowTexts(['Jocy', 'Đã thích một tin nhắn', '·', '4 giờ']), { name: 'Jocy', preview: 'Đã thích một tin nhắn' });
assert.deepEqual(rowTexts(['Phương', 'Hoạt động 2 giờ trước']), { name: 'Phương', preview: 'Hoạt động 2 giờ trước' });
assert.deepEqual(rowTexts([]), { name: '', preview: '' });

assert.deepEqual(rowParts(['Hâm', 'Bạn: ', 'ok', '·', '21 giờ']), { name: 0, preview: [1, 2], time: 4 });
assert.deepEqual(rowParts(['Phương', 'Hoạt động 2 giờ trước']), { name: 0, preview: [1], time: -1 });
assert.deepEqual(rowParts(['Jocy']), { name: 0, preview: [], time: -1 });
assert.deepEqual(rowParts(['Jocy', 'Hi', '·']), { name: 0, preview: [1], time: -1 });
assert.deepEqual(rowParts([]), { name: -1, preview: [], time: -1 });

const area = { left: 400, width: 600 };
assert.equal(bubbleDir({ left: 750, width: 230 }, area), 'out');
assert.equal(bubbleDir({ left: 440, width: 200 }, area), 'in');
assert.equal(bubbleDir({ left: 420, width: 560 }, area), 'in');
console.log('instagram unread: ok');
