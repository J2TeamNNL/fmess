  // The panes can be resized, as in the old app: drag the edge between the
  // chat list and the chat, or between the chat and its details (the pane
  // with Search), and double-click it to go back to Facebook's widths. fMess
  // keeps the widths in its settings (localStorage didn't last: it came back
  // empty after a restart).
  //
  // The list is the role="navigation" beside role="main". The details are the
  // second child of the row that holds the chat (CHAT_ROW, found from the
  // messages); Facebook adds and drops them in that row as they're opened
  // and closed, and the row stays, so its widths apply as soon as they open.
  const LIST_PANE = 'data-fmess-list';
  const CHAT_ROW = 'data-fmess-chat-row';
  const RESIZING = 'data-fmess-resizing';
  // The chat keeps at least this much room.
  const MIN_CHAT_WIDTH = 360;
  const paneWidths = { ...settings.paneWidths };
  const paneSelectors = { list: `html[${CHATS}] [${LIST_PANE}]`, details: `html[${CHATS}] [${CHAT_ROW}] > :nth-child(2)` };
  const edgeStyle = document.createElement('style');
  edgeStyle.textContent = `
    [${LIST_PANE}], [${CHAT_ROW}] > :nth-child(2) { position: relative; }
    .fmess-pane-edge { position: absolute; top: 0; bottom: 0; width: 9px; z-index: 5; cursor: col-resize; touch-action: none; }
    /* Half over the chat: the list's own scroll bar is at its edge. */
    .fmess-pane-edge[data-pane="list"] { right: -6px; }
    /* Inside the details: the chat's scroll bar is at their edge. */
    .fmess-pane-edge[data-pane="details"] { left: 0; }
    .fmess-pane-edge::after {
      content: ''; position: absolute; top: 0; bottom: 0; width: 2px;
      background: var(--primary-button-background, #0866ff);
      opacity: 0; transition: opacity 0.15s;
    }
    .fmess-pane-edge[data-pane="list"]::after { left: 5px; }
    .fmess-pane-edge[data-pane="details"]::after { left: 0; }
    .fmess-pane-edge:hover::after { opacity: 0.6; transition-delay: 0.15s; }
    .fmess-pane-edge.dragging::after { opacity: 0.6; transition-delay: 0s; }
    html[${RESIZING}], html[${RESIZING}] * { cursor: col-resize !important; -webkit-user-select: none !important; user-select: none !important; }
  `;
  withRoot((root) => root.append(edgeStyle));
  const widthStyle = document.createElement('style');
  // A set width doesn't shrink with the window (it would stop following the
  // pointer near the limit); instead it is capped so the chat keeps its room,
  // and the details theirs (Facebook's minimum, 300).
  const paneCaps = { list: `calc(100% - ${MIN_CHAT_WIDTH + 300}px)`, details: `calc(100% - ${MIN_CHAT_WIDTH}px)` };
  const paneWidthRule = (pane, width) =>
    `${paneSelectors[pane]} { flex: 0 0 ${width}px !important; width: ${width}px !important; max-width: ${paneCaps[pane]} !important; }`;
  const showPaneWidths = () => {
    widthStyle.textContent = Object.entries(paneWidths)
      .filter(([pane, width]) => paneSelectors[pane] && width > 0)
      .map(([pane, width]) => paneWidthRule(pane, width))
      .join('\n');
  };
  showPaneWidths();
  withRoot((root) => root.append(widthStyle));
  const savePaneWidths = () => {
    showPaneWidths();
    window.__TAURI_INTERNALS__?.invoke('set_pane_widths', { widths: paneWidths }).catch(() => {});
  };

  const chatRow = () => document.querySelector(`[${CHAT_ROW}]`);
  const startPaneDrag = (event, edge) => {
    if (event.button !== 0) return;
    const pane = edge.dataset.pane;
    const el = edge.parentElement;
    if (!el) return;
    event.preventDefault();
    event.stopPropagation();
    try {
      edge.setPointerCapture(event.pointerId);
    } catch {}
    const startX = event.clientX;
    const startWidth = el.getBoundingClientRect().width;
    // The chat is the row's first child, or all of role="main" without details.
    const chat = chatRow()?.firstElementChild ?? document.querySelector('[role="main"]');
    const room = Math.max(0, (chat?.getBoundingClientRect().width ?? 0) - MIN_CHAT_WIDTH);
    const min = parseFloat(getComputedStyle(el).minWidth) || 0;
    const max = Math.max(min, startWidth + room);
    const direction = pane === 'details' ? -1 : 1;
    let width = startWidth;
    let frame = 0;
    document.documentElement.setAttribute(RESIZING, '');
    edge.classList.add('dragging');
    // While dragging, only the pane's own style changes, not a style sheet
    // (which would have the whole page's style worked out again each frame).
    const move = (moveEvent) => {
      width = Math.round(Math.min(max, Math.max(min, startWidth + direction * (moveEvent.clientX - startX))));
      frame ||= requestAnimationFrame(() => {
        frame = 0;
        el.style.setProperty('flex', `0 0 ${width}px`, 'important');
        el.style.setProperty('width', `${width}px`, 'important');
        el.style.setProperty('max-width', paneCaps[pane], 'important');
      });
    };
    const end = () => {
      edge.removeEventListener('pointermove', move);
      edge.removeEventListener('lostpointercapture', end);
      cancelAnimationFrame(frame);
      document.documentElement.removeAttribute(RESIZING);
      edge.classList.remove('dragging');
      if (width !== startWidth) {
        paneWidths[pane] = width;
        savePaneWidths();
      }
      for (const prop of ['flex', 'width', 'max-width']) el.style.removeProperty(prop);
    };
    edge.addEventListener('pointermove', move);
    edge.addEventListener('lostpointercapture', end);
  };
  const paneEdge = (pane) => {
    const edge = document.createElement('div');
    edge.className = 'fmess-pane-edge';
    edge.dataset.pane = pane;
    edge.addEventListener('pointerdown', (event) => startPaneDrag(event, edge));
    edge.addEventListener('dblclick', (event) => {
      event.stopPropagation();
      paneWidths[pane] = null;
      savePaneWidths();
    });
    return edge;
  };
  const listEdge = paneEdge('list');
  const detailsEdge = paneEdge('details');
  // The row whose first child holds the messages and whose second is a pane
  // of its own (Facebook gives the details a minimum width).
  const findChatRow = (log, main) => {
    for (let el = log; el && el !== main; el = el.parentElement) {
      const row = el.parentElement;
      if (!row || row.children.length < 2 || row.firstElementChild !== el) continue;
      const details = el.nextElementSibling;
      const style = getComputedStyle(row);
      if (style.display === 'flex' && style.flexDirection === 'row' && parseFloat(getComputedStyle(details).minWidth) >= 200) return row;
    }
    return null;
  };
  let searchedLog = null;
  const syncPanes = () => {
    if (window.opener || !onChatsPage()) return;
    const main = document.querySelector('[role="main"]');
    const list = main?.parentElement?.querySelector(':scope > [role="navigation"]');
    if (list && !list.hasAttribute(LIST_PANE)) {
      document.querySelector(`[${LIST_PANE}]`)?.removeAttribute(LIST_PANE);
      list.setAttribute(LIST_PANE, '');
    }
    if (list && listEdge.parentElement !== list) list.append(listEdge);
    const log = main?.querySelector('[role="log"]');
    let row = chatRow();
    // Looked for until the details have been open once; finding the row
    // reads styles, so not every second: once per chat, then after a click
    // or a resize (either can open the details).
    if (log && !row?.contains(log) && log !== searchedLog) {
      searchedLog = log;
      const found = findChatRow(log, main);
      if (found) {
        row?.removeAttribute(CHAT_ROW);
        found.setAttribute(CHAT_ROW, '');
        row = found;
      }
    }
    const details = row?.contains(log) && row.children.length > 1 ? row.children[1] : null;
    if (details && detailsEdge.parentElement !== details) details.append(detailsEdge);
    else if (!details) detailsEdge.remove();
    markUi(main, list, log, details);
  };
  setInterval(syncPanes, 1000);
  // Opening a chat or its details is a click; look again once it has drawn.
  document.addEventListener(
    'click',
    () => {
      searchedLog = null;
      setTimeout(syncPanes, 50);
      setTimeout(syncPanes, 400);
    },
    { capture: true, passive: true },
  );
  window.addEventListener('resize', () => (searchedLog = null), { passive: true });

