  // Scroll bars like macOS overlay scroll bars. On macOS fMess asks AppKit for
  // them (AppleShowScrollBars, see main.rs); WebKit then draws them itself and
  // moves them on its scrolling thread, in step with the content, and the page
  // only makes them thin (which also sets aside Facebook's own
  // ::-webkit-scrollbar styling). Where the page gets classic scroll bars
  // instead (Windows), they would be always-on grey tracks that take room from
  // the content (selected chats stopped short of the edge), so the page hides
  // them and floats a thin indicator over the scrolling pane while it moves.
  // Call windows keep WebKit's. Wheel and trackpad scrolling are unchanged.
  const MAIN = 'data-fmess-main';
  const OVERLAY = 'data-fmess-overlay-scrollbars';
  let overlayScrollbars = false;
  // Measured in a shadow root, where the page's styles don't reach.
  const measureScrollbars = (root) => {
    const host = document.createElement('div');
    host.style.cssText = 'position: absolute; top: -200px; left: 0; width: 0; height: 0; overflow: hidden;';
    const box = document.createElement('div');
    box.style.cssText = 'width: 100px; height: 100px; overflow: scroll;';
    host.attachShadow({ mode: 'closed' }).append(box);
    root.append(host);
    overlayScrollbars = box.offsetWidth === box.clientWidth;
    host.remove();
    root.toggleAttribute(OVERLAY, overlayScrollbars);
  };
  if (!window.opener)
    withRoot((root) => {
      root.setAttribute(MAIN, '');
      measureScrollbars(root);
    });
  const scrollStyle = document.createElement('style');
  scrollStyle.textContent = `
    html[${MAIN}][${OVERLAY}], html[${MAIN}][${OVERLAY}] * { scrollbar-width: thin !important; }
    html[${MAIN}]:not([${OVERLAY}]), html[${MAIN}]:not([${OVERLAY}]) * { scrollbar-width: none !important; }
    html[${MAIN}]:not([${OVERLAY}]) ::-webkit-scrollbar, html[${MAIN}]:not([${OVERLAY}])::-webkit-scrollbar { width: 0; height: 0; }
    #fmess-scroll-indicator {
      position: fixed; z-index: 2147483646; top: 0; left: 0; width: 6px; border-radius: 3px;
      background: rgba(128, 128, 128, 0.55); pointer-events: none;
      opacity: 0; transition: opacity 0.3s;
      /* A layer of its own: moving it neither lays out nor repaints the page. */
      will-change: transform, opacity;
    }
  `;
  withRoot((root) => root.append(scrollStyle));
  const indicator = document.createElement('div');
  indicator.id = 'fmess-scroll-indicator';
  withRoot((root) => root.append(indicator));
  // When something last scrolled (the unread count waits for it to stop).
  let lastScrollAt = -Infinity;
  let hideIndicator;
  let scrolled = null;
  let indicatorFrame = 0;
  let indicatorHeight = 0;
  let coverScroller = null;
  let coverCheckedAt = -Infinity;
  let covered = false;
  // Once per frame (scroll events can come more often), when the page lays
  // itself out anyway.
  const drawIndicator = (now) => {
    indicatorFrame = 0;
    const scroller = scrolled;
    scrolled = null;
    if (!scroller?.isConnected) return;
    const range = scroller.scrollHeight - scroller.clientHeight;
    if (range <= 1) return;
    const box = scroller.getBoundingClientRect();
    const height = Math.max(24, (box.height * scroller.clientHeight) / scroller.scrollHeight);
    const top = box.top + ((box.height - height) * scroller.scrollTop) / range;
    // The indicator floats above the whole page, so it stays hidden where
    // a menu or dialog covers the pane (it lets points through to them).
    // Finding what is under a point is costly: a few times a second.
    if (scroller !== coverScroller || now - coverCheckedAt > 250) {
      coverScroller = scroller;
      coverCheckedAt = now;
      covered = [top + 2, top + height / 2, top + height - 2].some((y) => {
        const el = document.elementFromPoint(box.right - 6, y);
        return el && !scroller.contains(el);
      });
    }
    clearTimeout(hideIndicator);
    if (covered) {
      indicator.style.opacity = '0';
      return;
    }
    if (height !== indicatorHeight) {
      indicatorHeight = height;
      indicator.style.height = `${height}px`;
    }
    indicator.style.transform = `translate(${box.right - 9}px, ${top}px)`;
    indicator.style.opacity = '1';
    hideIndicator = setTimeout(() => {
      indicator.style.opacity = '0';
      coverScroller = null;
    }, 800);
  };
  document.addEventListener(
    'scroll',
    (event) => {
      lastScrollAt = performance.now();
      if (overlayScrollbars || window.opener || !(event.target instanceof Element)) return;
      scrolled = event.target;
      indicatorFrame ||= requestAnimationFrame(drawIndicator);
    },
    { capture: true, passive: true },
  );

