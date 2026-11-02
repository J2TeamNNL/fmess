// fmess.app: the page's languages, and Google Analytics when it has an ID.

// Google Analytics 4: the fmess.app property (account fMess). Without an ID
// nothing is loaded or sent.
const GA_ID = 'G-9JEGSLHZXT';

// English is in the page itself. Each translation replaces the HTML of the
// element with that data-i18n key; a key missing here keeps the English.
const TRANSLATIONS = {
  vi: {
    title: "fMess - Ứng dụng Messenger cho máy tính",
    description: "Messenger.com đã chính thức bị khai tử. fMess mang trải nghiệm Messenger trở lại máy tính với ứng dụng gọn nhẹ cho macOS, Windows và Linux. Miễn phí, mã nguồn mở.",
    nav_features: "Tính năng",
    nav_notifications: "Thông báo",
    nav_shortcuts: "Phím tắt",
    nav_compare: "So sánh",
    nav_faq: "Hỏi đáp",
    nav_download: "Tải về",
    eyebrow: "Meta đã khai tử Messenger cho máy tính. <b>fMess hồi sinh nó.</b>",
    hero_title: "Messenger<br><span class=\"ink-text\">đã trở&nbsp;lại trên máy&nbsp;tính.</span>",
    hero_lead: "Ứng dụng native gọn nhẹ cho macOS, Windows và Linux: thông báo đầy đủ, gọi thoại/video, phím tắt quen thuộc.",
    download_hero: "Tải fMess",
    download_mac: "Tải cho Mac",
    download_windows: "Tải cho Windows",
    download_linux: "Tải cho Linux",
    star: "Star trên GitHub",
    meta_free: "Miễn phí, mã nguồn mở",
    meta_platforms: "macOS, Windows, Linux",
    meta_small: "Dung lượng chỉ vài MB",
    shot_messenger: "fMess trên Mac: danh sách chat và một đoạn chat, giao diện Messenger gốc",
    shot_zalo: "Cùng đoạn chat đó với theme Zalo của fMess",
    shot_whatsapp: "Cùng đoạn chat đó với theme WhatsApp của fMess",
    shot_teams: "Cùng đoạn chat đó với theme Teams của fMess",
    themes_label: "Giao diện",
    theme_toggle: "Chế độ tối",
    shot_title: "Dùng Zalo, WhatsApp, Teams để chat Messenger, tại sao không?",
    shot_note: "Giao diện Zalo, WhatsApp, Teams nhưng thực chất là Messenger. Tha hồ tám giữa giờ làm, hoặc đơn giản là muốn thử điều gì đó mới mẻ.",
    mock_chats: "Đoạn chat",
    mock_search: "Tìm kiếm trên Messenger",
    mock_dinner: "Tối nay đi ăn không? 🍜",
    mock_icons: "Leo: icon mới lên rồi nha!",
    mock_see_you: "Bạn: 8 giờ gặp nha",
    mock_photo: "Đã gửi một ảnh",
    mock_haha: "Haha y chang 😂",
    mock_active: "Đang hoạt động",
    mock_b1: "Vẫn xài laptop mới hả?",
    mock_b2: "Ừ, Messenger cũng về lại trên đó rồi nè 🎉",
    mock_b3: "Ủa, tưởng app bị khai tử rồi mà",
    mock_b4: "Đúng rồi. Đây là fMess. Thông báo, gọi điện, đủ hết.",
    time_now: "bây giờ",
    time_1m: "1 phút trước",
    time_2m: "2 phút trước",
    reply: "Trả lời…",
    reply_sure: "Ok, 7 giờ nha?",
    send: "Gửi",
    note_icons: "Leo: Icon mới lên rồi, xem thử nha 👀",
    note_way: "Đang tới nè!",
    features_title: "Đầy đủ như bản gốc. <span class=\"ink-text\">Thậm chí còn hơn.</span>",
    card1_title: "Thông báo native",
    card1_text: "Hiển thị ảnh đại diện người gửi, trả lời nhanh ngay trên banner.",
    card2_title: "Gọi thoại và video",
    card2_text: "Hỗ trợ cả cuộc trò chuyện mã hoá đầu cuối.",
    card3_title: "Giao diện tập trung",
    card3_text: "Lược bỏ News Feed, Reels, chỉ giữ lại đoạn chat.",
    card4_title: "Ẩn trạng thái đang nhập",
    card4_text: "Soạn tin thoải mái, đối phương không thấy “đang nhập…”.",
    card5_title: "Cài đặt tập trung",
    card5_text: "Âm thanh, Không làm phiền, Dark Mode, đồng bộ với điện thoại.",
    card6_title: "Tối ưu bàn phím",
    card6_text: "⌘K tìm kiếm, ⌘1–9 chuyển đoạn chat, ⌘+ phóng to.",
    card7_title: "Zoom ảnh mượt mà",
    card7_text: "Chụm hai ngón trên trackpad hoặc nhấp đúp để phóng to.",
    card8_title: "Luôn chạy nền",
    card8_text: "Badge tin chưa đọc trên Dock, biểu tượng trên thanh hệ thống.",
    card9_title: "Nhẹ và nhanh",
    card9_text: "Chỉ vài MB nhờ tận dụng webview có sẵn của hệ điều hành.",
    notifications_title: "Trả lời tin nhắn <span class=\"ink-text\">không cần chuyển app.</span>",
    notifications_lead: "Thông báo hiển thị ảnh đại diện, tên nhóm và ô trả lời nhanh, đúng chuẩn Messenger desktop trước đây (macOS).",
    privacy_title: "Quyền riêng tư <span class=\"ink-text\">là mặc định.</span>",
    point5: "<b>Không qua máy chủ trung gian.</b> Đăng nhập trực tiếp trên trang của Facebook.",
    point6: "<b>Không theo dõi.</b> Ứng dụng không thu thập hay gửi dữ liệu đi đâu.",
    point9: "<b>Mã nguồn mở.</b> Giấy phép MIT, công khai trên GitHub.",
    shortcuts_title: "Phím tắt <span class=\"ink-text\">quen thuộc.</span>",
    shortcuts_lead: "Trên Windows và Linux, thay ⌘ bằng Ctrl.",
    key1: "Tìm kiếm đoạn chat",
    key2: "Tin nhắn mới",
    key3: "Mở đoạn chat 1 … 9",
    key4: "Đoạn chat kế tiếp / trước đó",
    key5: "Về danh sách chat",
    key6: "Phóng to / thu nhỏ / mặc định",
    key7: "Cài đặt",
    download_title: "Tải fMess. <span class=\"ink-text\">Hoàn toàn miễn phí.</span>",
    mac_text: "macOS 12 trở lên",
    windows_text: "Windows 10 và 11",
    windows_button: "Tải cho Windows",
    linux_text: "Ubuntu, Debian và hầu hết các bản khác",
    mac_silicon: "Apple silicon",
    mac_intel: "Intel",
    mac_brew: "Hoặc dùng Homebrew: <code>brew install --cask toanbku/tap/fmess</code>",
    mac_which: "Không chắc? Menu Apple → About This Mac: dòng “Chip Apple M…” là Apple silicon.",
    intel_link: "Mac dùng chip Intel? Tải bản này.",
    all_files: "Tất cả file và ghi chú phát hành",
    latest_version: "Phiên bản mới nhất: {version}",
    windows_note: "Windows báo “Windows protected your PC”? Bấm <b>More info</b> rồi <b>Run anyway</b>. fMess chưa được ký số cho Windows.",
    mac_note: "macOS không cho mở fMess? Vào <b>System Settings → Privacy &amp; Security</b> rồi bấm <b>Open Anyway</b>.",
    cmp_title: "fMess <span class=\"ink-text\">so với app khác.</span>",
    cmp_lead: "Các cách dùng Messenger trên máy tính, tính đến tháng 11/2026.",
    cmp_tab: "Mở trong trình duyệt",
    cmp_for: "Làm cho",
    cmp_many: "Nhiều dịch vụ trong một app",
    cmp_all_fb: "Cả Facebook, có luôn feed",
    cmp_source: "Mã nguồn",
    cmp_open_mit: "Mở (MIT)",
    cmp_open_mit2: "Mở (MIT)",
    cmp_closed: "Đóng",
    cmp_open_apache: "Mở (Apache 2.0)",
    cmp_size: "Dung lượng tải về (Mac)",
    cmp_engine: "Nền tảng",
    cmp_webview: "Webview có sẵn của hệ thống",
    cmp_webview2: "Webview có sẵn của hệ thống",
    cmp_electron: "Electron (kèm nguyên một Chromium)",
    cmp_electron2: "Electron (kèm nguyên một Chromium)",
    cmp_browser: "Trình duyệt của bạn",
    cmp_signed: "Ký số trên Mac",
    cmp_notarized: "Có, Apple đã xác thực",
    cmp_notarized2: "Có, Apple đã xác thực",
    cmp_unsigned: "Không",
    cmp_yes: "Có",
    cmp_release: "Bản mới nhất",
    cmp_nov: "Tháng 11/2026",
    cmp_sep: "Tháng 9/2026",
    cmp_sep2: "Tháng 9/2026",
    cmp_jan: "Tháng 1/2026, trước khi messenger.com đóng cửa",
    cmp_why: "<b>Vì sao mã nguồn mở quan trọng ở đây.</b> App Messenger nào cũng giữ phiên đăng nhập Facebook của bạn và thấy mọi tin nhắn bạn gửi, nhận. Mã nguồn của fMess công khai, nên bạn hay bất kỳ ai cũng kiểm tra được app làm gì với chúng, và tự build lấy. Với app đóng mã nguồn, bạn chỉ có thể tin lời nhà phát triển.",
    cmp_also: "<b>fMess còn có:</b> thông báo kèm ảnh người gửi và trả lời nhanh, gọi điện trong chat mã hoá (macOS), giao diện Zalo, WhatsApp và Teams, ẩn ảnh đại diện, kéo thả file vào chat, kéo đổi độ rộng các khung, và phím tắt như app cũ.",
    cmp_note: "Lấy từ trang GitHub của từng dự án và bản tải về mới nhất cho Mac (Apple silicon), kiểm tra ngày 1/11/2026. Chữ ký theo kết quả kiểm tra của chính macOS (spctl).",
    faq_title: "Hỏi đáp",
    q1: "fMess có phải sản phẩm của Meta?",
    a1: "Không. fMess là dự án mã nguồn mở độc lập, hiển thị facebook.com/messages trong một cửa sổ riêng.",
    q2: "Sao không dùng Messenger.com?",
    a2: "Meta đã chính thức khai tử Messenger.com từ tháng 4/2026, toàn bộ đoạn chat được chuyển về facebook.com.",
    q3: "Đăng nhập có an toàn?",
    a3: "Bạn đăng nhập trực tiếp trên trang của Facebook. fMess không lưu mật khẩu và không có máy chủ riêng.",
    q5: "Windows báo không tìm thấy WebView2 Runtime?",
    a5: "fMess dùng WebView2, thành phần của Windows để hiện trang web trong app. Windows 10 và 11 có sẵn, nhưng vài bản Windows rút gọn đã gỡ mất. Bạn cài lại từ <a href=\"https://developer.microsoft.com/microsoft-edge/webview2\">trang WebView2 của Microsoft</a> (bản Evergreen) rồi mở lại fMess.",
    q4: "Có hỗ trợ chat mã hoá đầu cuối?",
    a4: "Có. Hỗ trợ đầy đủ tin nhắn, cuộc gọi (macOS), ảnh và tin nhắn thoại.",
    footer_copy: "© 2026 fMess contributors · <a href=\"https://github.com/toanbku/fmess\">GitHub</a> · Giấy phép MIT",
    footer_legal: "fMess không liên kết với, không được bảo trợ hay tài trợ bởi Meta Platforms, Inc. Messenger và Facebook là nhãn hiệu của Meta Platforms, Inc.",
  },
};
const ENGLISH = {
  download_mac: 'Download for Mac',
  download_windows: 'Download for Windows',
  download_linux: 'Download for Linux',
  latest_version: 'Latest version: {version}',
};

// Installers under stable names (the release workflow attaches them), so these
// links always fetch the newest version.
const LATEST = 'https://github.com/toanbku/fmess/releases/latest/download/';
const HERO_DOWNLOAD = {
  mac: 'fMess-mac-apple-silicon.dmg',
  windows: 'fMess-windows-setup.exe',
  linux: 'fMess-linux.AppImage',
};

const LANGUAGE_KEY = 'fmess.language';
const system = (() => {
  const ua = navigator.userAgent;
  if (/Mac/.test(ua)) return 'mac';
  if (/Win/.test(ua)) return 'windows';
  if (/Linux/.test(ua) && !/Android/.test(ua)) return 'linux';
  return null;
})();

// Light or dark: the visitor's pick, kept in fmess.theme (index.html applies
// it before the page paints), or else the system's.
const THEME_KEY = 'fmess.theme';
function isDark() {
  return (document.documentElement.dataset.theme ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')) === 'dark';
}
// The hero's screenshots come in each language, light and dark.
function shotSrc(img) {
  return `assets/shot-${img.dataset.shot}${document.documentElement.lang === 'vi' ? '-vi' : ''}${isDark() ? '-dark' : ''}.webp?v=2`;
}

// The language asked for in the link, then the one chosen here before, then
// the browser's.
const chosenLanguage = () => {
  const asked = new URLSearchParams(location.search).get('lang');
  if (asked === 'en' || Object.hasOwn(TRANSLATIONS, asked)) return asked;
  try {
    const saved = localStorage.getItem(LANGUAGE_KEY);
    if (saved === 'en' || Object.hasOwn(TRANSLATIONS, saved)) return saved;
  } catch {}
  const browser = (navigator.languages ?? [navigator.language]).map((tag) => String(tag).slice(0, 2).toLowerCase());
  return browser.find((code) => Object.hasOwn(TRANSLATIONS, code)) ?? 'en';
};

const english = new Map();
const setLanguage = (language) => {
  const words = TRANSLATIONS[language] ?? {};
  const word = (key) => words[key] ?? ENGLISH[key];
  document.documentElement.lang = language;
  for (const el of document.querySelectorAll('[data-i18n]')) {
    if (!english.has(el)) english.set(el, el.innerHTML);
    el.innerHTML = words[el.dataset.i18n] ?? english.get(el);
  }
  for (const [key, attribute] of [['i18nAlt', 'alt'], ['i18nLabel', 'aria-label']]) {
    for (const el of document.querySelectorAll(`[data-${key === 'i18nAlt' ? 'i18n-alt' : 'i18n-label'}]`)) {
      const id = `${attribute}:${el.dataset[key]}`;
      if (!english.has(el)) english.set(el, {});
      english.get(el)[id] ??= el.getAttribute(attribute);
      el.setAttribute(attribute, words[el.dataset[key]] ?? english.get(el)[id]);
    }
  }
  for (const img of document.querySelectorAll('img[data-shot]')) img.src = shotSrc(img);
  // A page names its own title and description keys (data-title, data-description on body).
  const page = document.body.dataset;
  if (!english.has(document)) english.set(document, document.title);
  document.title = words[page.title ?? 'title'] ?? english.get(document);
  const description = document.querySelector('meta[name="description"]');
  if (!english.has(description)) english.set(description, description.content);
  description.content = words[page.description ?? 'description'] ?? english.get(description);
  // The main button names the visitor's system.
  const hero = document.querySelector('[data-i18n="download_hero"]');
  if (hero && system) hero.innerHTML = word(`download_${system}`);
  if (latestVersion) {
    const line = document.getElementById('latest-version');
    line.textContent = word('latest_version').replace('{version}', latestVersion);
    line.hidden = false;
  }
  for (const button of document.querySelectorAll('.lang button')) {
    button.setAttribute('aria-pressed', String(button.dataset.lang === language));
  }
};

// The main button downloads the file for the visitor's system right away;
// on a Mac it's the Apple silicon build, with a link to the Intel one below.
let latestVersion = null;
const heroLink = document.getElementById('download-hero');
if (heroLink && HERO_DOWNLOAD[system]) {
  heroLink.href = LATEST + HERO_DOWNLOAD[system];
  if (system === 'mac') document.getElementById('download-intel').hidden = false;
}
setLanguage(chosenLanguage());
// The newest version's number, from GitHub; the links work without it.
fetch('https://api.github.com/repos/toanbku/fmess/releases/latest')
  .then((response) => (response.ok ? response.json() : null))
  .then((release) => {
    latestVersion = release?.tag_name?.replace(/^v/, '') ?? null;
    if (latestVersion) setLanguage(document.documentElement.lang);
  })
  .catch(() => {});
for (const button of document.querySelectorAll('.lang button')) {
  button.addEventListener('click', () => {
    setLanguage(button.dataset.lang);
    try {
      localStorage.setItem(LANGUAGE_KEY, button.dataset.lang);
    } catch {}
    track('language_change', { language: button.dataset.lang });
  });
}

const themeButton = document.querySelector('.theme-toggle');
const showTheme = () => {
  themeButton?.setAttribute('aria-pressed', String(isDark()));
  for (const img of document.querySelectorAll('img[data-shot]')) img.src = shotSrc(img);
};
themeButton?.addEventListener('click', () => {
  const next = isDark() ? 'light' : 'dark';
  document.documentElement.dataset.theme = next;
  try {
    localStorage.setItem(THEME_KEY, next);
  } catch {}
  showTheme();
  track('theme_change', { theme: next });
});
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', showTheme);
showTheme();

// Analytics: page views, and which download and GitHub links get clicked.
function track(name, params) {
  if (GA_ID && typeof window.gtag === 'function') window.gtag('event', name, params);
}
if (GA_ID) {
  window.dataLayer = window.dataLayer || [];
  window.gtag = function gtag() {
    window.dataLayer.push(arguments);
  };
  window.gtag('js', new Date());
  window.gtag('config', GA_ID);
  const script = document.createElement('script');
  script.async = true;
  script.src = `https://www.googletagmanager.com/gtag/js?id=${encodeURIComponent(GA_ID)}`;
  document.head.append(script);
}
document.addEventListener('click', (event) => {
  const link = event.target.closest('a[href]');
  if (!link) return;
  const platform = link.closest('.platform')?.querySelector('h3')?.textContent ?? system ?? 'unknown';
  if (link.href.includes('/releases/')) {
    const file = link.href.includes('/download/') ? link.href.split('/').pop() : 'release page';
    track('download_click', { platform, file, language: document.documentElement.lang });
  }
  else if (link.hostname === 'github.com') track('github_click', { language: document.documentElement.lang });
});


// The hero's screenshots, a carousel: Messenger's look, then the themes, one
// sliding in after another (not when the visitor's system asks for less
// motion). A look's button shows it; the pointer over the pictures holds the
// turn; a swipe goes forwards or back.
const shot = document.querySelector('.shot');
if (shot) {
  const DWELL = 4000;
  const track = shot.querySelector('.track');
  const looks = track.children.length;
  // A copy of the first look after the last, so the turn back to it slides on.
  track.append(track.children[0].cloneNode(true));
  const buttons = [...shot.querySelectorAll('.looks button')];
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
  shot.style.setProperty('--dwell', `${DWELL}ms`);
  let index = 0;
  let timer = null;
  const show = (to) => {
    index = to;
    track.style.transform = `translateX(${-100 * to}%)`;
    [...track.children].forEach((img, i) => img.classList.toggle('on', i === to));
    buttons.forEach((button, i) => {
      button.setAttribute('aria-pressed', String(i === to % looks));
      button.classList.remove('run');
    });
    if (still || shot.classList.contains('paused')) return;
    // Restart the bar under the look shown.
    void buttons[to % looks].offsetWidth;
    buttons[to % looks].classList.add('run');
    clearTimeout(timer);
    timer = setTimeout(() => show(index + 1), DWELL);
  };
  track.addEventListener('transitionend', (event) => {
    if (event.target !== track || index !== looks) return;
    track.classList.add('snap');
    show(0);
    void track.offsetWidth;
    track.classList.remove('snap');
  });
  buttons.forEach((button, i) => button.addEventListener('click', () => show(i)));
  const viewport = shot.querySelector('.viewport');
  viewport.addEventListener('pointerenter', () => {
    shot.classList.add('paused');
    clearTimeout(timer);
  });
  viewport.addEventListener('pointerleave', () => {
    shot.classList.remove('paused');
    show(index % looks);
  });
  let from = null;
  viewport.addEventListener('pointerdown', (event) => (from = event.clientX));
  viewport.addEventListener('pointerup', (event) => {
    const moved = from === null ? 0 : event.clientX - from;
    from = null;
    if (Math.abs(moved) > 40) show(((index % looks) + (moved < 0 ? 1 : looks - 1)) % looks);
  });
  show(0);
}

// Motion (motion.dev, assets/vendor): the hero plays in, sections rise as
// they scroll into view, cards answer the pointer.
if (document.documentElement.classList.contains('motion')) {
  if (window.Motion) playMotion(window.Motion);
  else document.documentElement.classList.remove('motion');
}

function playMotion({ animate, inView, hover, press, stagger, spring }) {
  const ease = [0.2, 0.7, 0.2, 1];
  const rise = { opacity: [0, 1], y: [24, 0] };
  const bounce = (amount, duration) => ({ type: spring, bounce: amount, duration });
  const forever = (target, keyframes, duration) => animate(target, keyframes, { duration, ease: 'easeInOut', repeat: Infinity });

  // The hero: the words, then the app.
  animate('.hero .eyebrow, .hero h1, .hero .lead, .hero .cta, .hero .meta', rise, { duration: 0.9, ease, delay: stagger(0.08) });
  animate('.shot', { opacity: [0, 1], y: [36, 0], scale: [0.97, 1] }, { duration: 1.1, ease, delay: 0.25 }).then(() =>
    forever('.shot .viewport', { y: [0, -8, 0] }, 8),
  );

  // Sections rise as they come into view; the items of a list follow one another.
  const reveal = (selector, { items = false, then } = {}) => {
    for (const el of document.querySelectorAll(selector)) {
      const targets = items ? [...el.children] : [el];
      for (const target of targets) target.style.opacity = '0';
      inView(
        el,
        () => {
          animate(targets, rise, { duration: 0.8, ease, delay: stagger(0.07) });
          then?.(el, targets);
        },
        { margin: '0px 0px -5% 0px' },
      );
    }
  };
  reveal('section h2');
  reveal('section .section-lead');
  reveal('.grid', { items: true });
  reveal('.checks', { items: true });
  reveal('.keys', { items: true });
  reveal('.platforms', { items: true });
  reveal('.note-mac');
  reveal('#faq details');
  reveal('.stack', {
    items: true,
    then: (stack, notes) => {
      const field = stack.querySelector('[data-i18n="reply_sure"]');
      if (field) setTimeout(() => typeReply(field), notes.length * 70 + 800);
    },
  });

  // The reply in the notification list types itself, then Send is pressed.
  function typeReply(field) {
    const text = field.textContent;
    field.textContent = '';
    field.classList.add('typing-cursor');
    let shown = 0;
    const timer = setInterval(() => {
      field.textContent = text.slice(0, ++shown);
      if (shown < text.length) return;
      clearInterval(timer);
      setTimeout(() => {
        field.classList.remove('typing-cursor');
        const send = field.parentElement.querySelector('em');
        if (send) animate(send, { scale: [1, 0.86, 1] }, bounce(0.5, 0.5));
      }, 450);
    }, 70);
  }

  // Cards lift towards the pointer and their icon tilts; buttons give under a press.
  hover('.card', (card) => {
    const icon = card.querySelector('.icon');
    animate(card, { y: -6 }, bounce(0.3, 0.4));
    if (icon) animate(icon, { rotate: -8, scale: 1.12 }, bounce(0.5, 0.5));
    return () => {
      animate(card, { y: 0 }, bounce(0.3, 0.4));
      if (icon) animate(icon, { rotate: 0, scale: 1 }, bounce(0.5, 0.5));
    };
  });
  press('.btn', (button) => {
    animate(button, { scale: 0.95 }, bounce(0.4, 0.25));
    return () => animate(button, { scale: 1 }, bounce(0.5, 0.35));
  });
}
