  // The viewer's download and forward buttons sit at the photo's top right
  // corner, which moves with each photo, so keep its close button beside
  // them, styled like them: Facebook's close button is the light one made for
  // its white top bar.
  const setStyle = (el, prop, value) => {
    if (el.style.getPropertyValue(prop) !== value) el.style.setProperty(prop, value, 'important');
  };
  function placeViewerClose() {
    const area = (el) => el.getBoundingClientRect().width * el.getBoundingClientRect().height;
    const dialog = [...document.querySelectorAll('[role="dialog"]')].sort((a, b) => area(b) - area(a))[0];
    const closeBox = dialog?.querySelector(':scope > div:has(> a[href="/"])');
    const close = closeBox?.querySelector('[role="button"]');
    if (!close) return;
    // Small buttons near the top, not the tall previous/next ones or the thumbnails.
    const neighbour = [...dialog.querySelectorAll('a[role="link"], [role="button"]')]
      .filter((el) => !closeBox.contains(el))
      .map((el) => [el, el.getBoundingClientRect()])
      .filter(([, box]) => box.width > 0 && box.height <= 48 && box.top < 120)
      .sort((a, b) => b[1].right - a[1].right)[0];
    if (!neighbour) return;
    const [like, box] = neighbour;

    const looks = getComputedStyle(like);
    for (const prop of ['width', 'height', 'background-color', 'color', 'opacity']) setStyle(close, prop, looks.getPropertyValue(prop));

    setStyle(closeBox, 'left', `${Math.round(box.right + 8)}px`);
    setStyle(closeBox, 'right', 'auto');
    setStyle(closeBox, 'top', `${Math.round(box.top)}px`);
  }

  // Zooming the photo in the viewer, as in Preview: pinch on the trackpad
  // (Ctrl + scroll too), double-click to zoom in or back out, and scroll or
  // drag to move around the zoomed photo. Facebook's viewer has no zoom.
  // WebKit hands a pinch to the page as gesture events (the page itself
  // doesn't magnify); Chromium-based webviews send it as Ctrl + wheel.
  const ZOOM_MAX = 8;
  const ZOOM_DOUBLE_CLICK = 2.5;
  let zoom = null;
  // Dragging the zoomed photo; a drag isn't a click on whatever is under it.
  let drag = null;
  let dragged = false;
  // The photo: the largest image in the viewer (the blurred backdrop behind it
  // is a small thumbnail, and the thumbnails at the bottom are small).
  const viewerPhoto = () => {
    const pixels = (img) => img.naturalWidth * img.naturalHeight;
    return (
      [...document.querySelectorAll('[role="dialog"] img')]
        .filter((img) => img.getBoundingClientRect().width > 100)
        .sort((a, b) => pixels(b) - pixels(a))[0] ?? null
    );
  };
  const isOn = (el, x, y) => {
    const box = el.getBoundingClientRect();
    return x >= box.left && x <= box.right && y >= box.top && y <= box.bottom;
  };
  function resetZoom(animate) {
    if (!zoom) return;
    Object.assign(zoom.img.style, { transition: animate ? 'transform 0.2s' : '', transform: '', cursor: '' });
    zoom = null;
  }
  // Another photo (the viewer's URL names it) or a closed viewer starts unzoomed.
  function dropStaleZoom() {
    if (zoom && (!zoom.img.isConnected || zoom.page !== location.href)) resetZoom();
  }
  const startZoom = () => {
    if (zoom) return zoom;
    const img = viewerPhoto();
    if (!img) return null;
    // The zoomed photo may fill what shows of the viewer, not just its own
    // box: the window, cut down by whatever clips the photo (Facebook's own
    // clipping box is wider than the window).
    const stage = { left: 0, top: 0, right: innerWidth, bottom: innerHeight };
    for (let el = img.parentElement; el; el = el.parentElement) {
      if (getComputedStyle(el).overflow === 'visible') continue;
      const box = el.getBoundingClientRect();
      Object.assign(stage, {
        left: Math.max(stage.left, box.left),
        top: Math.max(stage.top, box.top),
        right: Math.min(stage.right, box.right),
        bottom: Math.min(stage.bottom, box.bottom),
      });
    }
    img.style.transformOrigin = '0 0';
    zoom = { img, page: location.href, base: img.getBoundingClientRect(), stage, scale: 1, x: 0, y: 0 };
    return zoom;
  };
  // A photo smaller than the stage stays inside it; a larger one covers it.
  const clampOffset = (offset, start, size, stageStart, stageEnd) => {
    const a = stageStart - start;
    const b = stageEnd - size - start;
    return Math.min(Math.max(offset, Math.min(a, b)), Math.max(a, b));
  };
  const applyZoom = (scale, x, y, animate) => {
    zoom.scale = Math.min(Math.max(scale, 1), ZOOM_MAX);
    if (zoom.scale === 1) return resetZoom(animate);
    const { base, stage } = zoom;
    zoom.x = clampOffset(x, base.left, base.width * zoom.scale, stage.left, stage.right);
    zoom.y = clampOffset(y, base.top, base.height * zoom.scale, stage.top, stage.bottom);
    Object.assign(zoom.img.style, {
      transition: animate ? 'transform 0.2s' : '',
      transform: `translate(${zoom.x}px, ${zoom.y}px) scale(${zoom.scale})`,
      cursor: drag ? 'grabbing' : 'grab',
    });
  };
  // Zooms to scale keeping the point of the photo under (x, y) where it is.
  const zoomAt = (scale, x, y, animate) => {
    const pointX = (x - zoom.base.left - zoom.x) / zoom.scale;
    const pointY = (y - zoom.base.top - zoom.y) / zoom.scale;
    const next = Math.min(Math.max(scale, 1), ZOOM_MAX);
    applyZoom(next, x - zoom.base.left - next * pointX, y - zoom.base.top - next * pointY, animate);
  };

  let pinchFrom = 1;
  document.addEventListener(
    'gesturestart',
    (event) => {
      if (!viewerOpen || !startZoom()) return;
      event.preventDefault();
      pinchFrom = zoom.scale;
    },
    true,
  );
  document.addEventListener(
    'gesturechange',
    (event) => {
      // Pinching back to fit ends the zoom; pinching out again starts anew.
      if (!viewerOpen || !startZoom()) return;
      event.preventDefault();
      zoomAt(pinchFrom * event.scale, event.clientX, event.clientY);
    },
    true,
  );
  document.addEventListener('gestureend', (event) => viewerOpen && zoom && event.preventDefault(), true);
  // Only while the viewer is open: a wheel listener that may cancel scrolling
  // makes WebKit hold the first event of every scroll gesture, anywhere in
  // the page, until the page's main thread has run it (up to 50 ms; WebKit's
  // ScrollingTree), which made each scroll of a chat start late.
  function listenForViewerWheel(on) {
    if (on) window.addEventListener('wheel', onViewerWheel, { capture: true, passive: false });
    else window.removeEventListener('wheel', onViewerWheel, { capture: true });
  }
  function onViewerWheel(event) {
    if (!viewerOpen) return;
    if (event.ctrlKey) {
      if (!startZoom()) return;
      event.preventDefault();
      zoomAt(zoom.scale * Math.exp(-event.deltaY / 100), event.clientX, event.clientY);
    } else if (zoom) {
      event.preventDefault();
      applyZoom(zoom.scale, zoom.x - event.deltaX, zoom.y - event.deltaY);
    } else {
      return;
    }
    event.stopPropagation();
  }
  document.addEventListener(
    'dblclick',
    (event) => {
      const photo = viewerOpen && viewerPhoto();
      if (!photo || !isOn(photo, event.clientX, event.clientY)) return;
      event.preventDefault();
      if (zoom) return resetZoom(true);
      startZoom();
      zoomAt(ZOOM_DOUBLE_CLICK, event.clientX, event.clientY, true);
    },
    true,
  );
  document.addEventListener(
    'pointerdown',
    (event) => {
      if (!zoom || event.button !== 0 || !isOn(zoom.img, event.clientX, event.clientY)) return;
      if (!event.target.contains(zoom.img)) return;
      event.preventDefault();
      drag = { x: event.clientX - zoom.x, y: event.clientY - zoom.y, fromX: event.clientX, fromY: event.clientY };
      dragged = false;
      zoom.img.style.cursor = 'grabbing';
    },
    true,
  );
  document.addEventListener(
    'pointermove',
    (event) => {
      if (!drag || !zoom) return;
      dragged ||= Math.hypot(event.clientX - drag.fromX, event.clientY - drag.fromY) > 3;
      applyZoom(zoom.scale, event.clientX - drag.x, event.clientY - drag.y);
    },
    true,
  );
  document.addEventListener(
    'pointerup',
    () => {
      if (!drag) return;
      drag = null;
      if (zoom) zoom.img.style.cursor = 'grab';
      // The click, if any, comes before this runs.
      setTimeout(() => (dragged = false));
    },
    true,
  );
  document.addEventListener(
    'click',
    (event) => {
      if (!dragged) return;
      dragged = false;
      event.preventDefault();
      event.stopPropagation();
    },
    true,
  );
  document.addEventListener('dragstart', (event) => zoom && event.preventDefault(), true);
  window.addEventListener('resize', () => resetZoom());

