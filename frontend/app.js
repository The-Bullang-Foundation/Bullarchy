'use strict';

// ── Star field ───────────────────────────────────────────────────────────────

(function initStars() {
  const canvas = document.getElementById('stars');
  const ctx    = canvas.getContext('2d');
  let stars    = [];

  function resize() {
    canvas.width  = window.innerWidth;
    canvas.height = window.innerHeight;
  }

  function makeStars(n) {
    stars = [];
    for (let i = 0; i < n; i++) {
      stars.push({
        x:    Math.random() * canvas.width,
        y:    Math.random() * canvas.height,
        r:    Math.random() * 1.2 + 0.2,
        a:    Math.random(),
        da:   (Math.random() - 0.5) * 0.003,
        blue: Math.random() > 0.7,
      });
    }
  }

  function draw() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    for (const s of stars) {
      s.a += s.da;
      if (s.a < 0) s.da =  Math.abs(s.da);
      if (s.a > 1) s.da = -Math.abs(s.da);
      ctx.beginPath();
      ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
      ctx.fillStyle = s.blue
        ? `rgba(109,184,255,${s.a * 0.8})`
        : `rgba(232,240,255,${s.a * 0.6})`;
      ctx.fill();
    }
    requestAnimationFrame(draw);
  }

  resize();
  makeStars(200);
  draw();
  window.addEventListener('resize', () => { resize(); makeStars(200); });
})();

// ── Constants ─────────────────────────────────────────────────────────────────

const RANKS = {
  1: 'skirmish',
  2: 'tactic → skirmish',
  3: 'strategy → tactic → skirmish',
  4: 'battle → strategy → tactic → skirmish',
  5: 'theater → battle → strategy → tactic → skirmish',
  6: 'war → theater → battle → strategy → tactic → skirmish',
};

const RANK_NAMES   = ['skirmish','tactic','strategy','battle','theater','war'];
const LANG_OPTIONS = [
  ['', 'auto (from #lang)'],
  ['rs', 'Rust'], ['py', 'Python'], ['c', 'C'], ['cpp', 'C++'], ['go', 'Go'],
];

const BLUEPRINT_PLACEHOLDER = `// Blueprint example — edit freely
// Rank keywords are optional (inferred from nesting depth)

war my_project {

    theater core {

        battle engine {
            strategy math {
                tactic vectors {
                    skirmish ops {
                        vec3.bu : cross, dot, normalize {
                            goal  : "Core 3D vector arithmetic"
                            owner : "alice"
                        }
                        vec4.bu : scale, lerp;
                    }
                }
            }
        }

    }

    python: theater tools {

        battle pipeline {
            strategy import {
                tactic mesh {
                    skirmish gltf {
                        loader.bu : load_scene, load_mesh {
                            goal  : "Imports glTF 2.0 scenes"
                            owner : "bob"
                        }
                    }
                }
            }
        }

    }

}
`;

// ── Card routing ──────────────────────────────────────────────────────────────

const PANELS = {
  init:      buildInitPanel,
  convert:   buildConvertPanel,
  blueprint: buildBlueprintPanel,
  control:   buildControlPanel,
  options:   buildOptionsPanel,
};

let activeCmd = null;

document.querySelectorAll('.cmd-card').forEach(card => {
  card.addEventListener('click', () => {
    const cmd = card.dataset.cmd;
    if (activeCmd === cmd) { closePanel(); return; }
    openPanel(cmd, card);
  });
});

function openPanel(cmd, card) {
  document.querySelectorAll('.cmd-card').forEach(c => c.classList.remove('active'));
  card.classList.add('active');
  activeCmd = cmd;

  const wrap = document.getElementById('panel-wrap');
  wrap.innerHTML = '';
  const panel = PANELS[cmd]();
  wrap.appendChild(panel);
  panel.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

function closePanel() {
  document.querySelectorAll('.cmd-card').forEach(c => c.classList.remove('active'));
  activeCmd = null;
  document.getElementById('panel-wrap').innerHTML = '';
}

// ── Shared helpers ────────────────────────────────────────────────────────────

function makePanel(title, icon, bodyEl) {
  const panel = document.createElement('div');
  panel.className = 'panel';

  const header = document.createElement('div');
  header.className = 'panel-header';
  header.innerHTML = `<span class="panel-title">${icon} ${title}</span>
    <button class="panel-close" title="Close">×</button>`;
  header.querySelector('.panel-close').addEventListener('click', closePanel);

  const body = document.createElement('div');
  body.className = 'panel-body';
  body.appendChild(bodyEl);

  panel.appendChild(header);
  panel.appendChild(body);
  return panel;
}

function field(labelText, inputEl, hint) {
  const wrap = document.createElement('div');
  wrap.className = 'field-group';
  const lbl = document.createElement('label');
  lbl.textContent = labelText;
  wrap.appendChild(lbl);
  wrap.appendChild(inputEl);
  if (hint) {
    const h = document.createElement('div');
    h.style.cssText = 'font-size:11px;color:var(--text-dim);margin-top:2px;';
    h.textContent = hint;
    wrap.appendChild(h);
  }
  return wrap;
}

function textInput(placeholder, val) {
  const el = document.createElement('input');
  el.type = 'text';
  el.placeholder = placeholder || '';
  if (val) el.value = val;
  return el;
}

function selectEl(options, val) {
  const el = document.createElement('select');
  options.forEach(([v, t]) => {
    const o = document.createElement('option');
    o.value = v; o.textContent = t;
    if (v === val) o.selected = true;
    el.appendChild(o);
  });
  return el;
}

function runButton(label) {
  const btn = document.createElement('button');
  btn.className = 'btn-run';
  btn.textContent = label || 'Run';
  return btn;
}

function consoleEl() {
  const wrap = document.createElement('div');
  wrap.className = 'console-wrap';
  const lbl = document.createElement('div');
  lbl.className = 'console-label';
  lbl.textContent = 'Output';
  const pre = document.createElement('pre');
  pre.className = 'console';
  wrap.appendChild(lbl);
  wrap.appendChild(pre);
  return { wrap, pre };
}

async function runCmd(endpoint, payload, btn, pre) {
  btn.disabled = true;
  btn.classList.add('loading');
  pre.textContent = '';
  pre.className = 'console';

  try {
    const res  = await fetch(endpoint, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify(payload),
    });
    const data = await res.json();
    pre.textContent = data.output || '(no output)';
    pre.classList.add(data.ok ? 'ok' : 'err');
  } catch (e) {
    pre.textContent = `Network error: ${e.message}`;
    pre.classList.add('err');
  } finally {
    btn.disabled = false;
    btn.classList.remove('loading');
  }
}

function infoBanner(text) {
  const el = document.createElement('div');
  el.className = 'info-banner';
  el.textContent = text;
  return el;
}

// ── init panel ────────────────────────────────────────────────────────────────

function buildInitPanel() {
  const body = document.createDocumentFragment();

  const nameIn = textInput('my_project');
  const pathIn = textInput('/home/user/projects  (optional)');
  const bpIn   = textInput('/path/to/blueprint.bu  (optional)');

  const depthSlider = document.createElement('input');
  depthSlider.type = 'range';
  depthSlider.min = '1'; depthSlider.max = '6'; depthSlider.value = '2';

  const depthLbl = document.createElement('span');
  depthLbl.className = 'depth-label';
  depthLbl.textContent = `2 — tactic`;
  depthSlider.addEventListener('input', () => {
    const v = parseInt(depthSlider.value);
    depthLbl.textContent = `${v} — ${RANK_NAMES[v - 1]}`;
  });

  const depthRow = document.createElement('div');
  depthRow.className = 'depth-row';
  depthRow.appendChild(depthSlider);
  depthRow.appendChild(depthLbl);

  const depthField = document.createElement('div');
  depthField.className = 'field-group';
  const depthLblEl = document.createElement('label');
  depthLblEl.textContent = 'Depth';
  depthField.appendChild(depthLblEl);
  depthField.appendChild(depthRow);

  const langSel = selectEl(LANG_OPTIONS);

  const libsList = document.createElement('div');
  libsList.className = 'libs-list';

  const addLibBtn = document.createElement('button');
  addLibBtn.className = 'btn-add-lib';
  addLibBtn.textContent = '+ add library';

  function addLibRow() {
    const row = document.createElement('div');
    row.className = 'lib-row';
    const inp = textInput('header_name.h');
    const rm  = document.createElement('button');
    rm.className = 'btn-remove';
    rm.textContent = '−';
    rm.addEventListener('click', () => row.remove());
    row.appendChild(inp);
    row.appendChild(rm);
    libsList.appendChild(row);
  }
  addLibBtn.addEventListener('click', addLibRow);

  const libsWrap = document.createElement('div');
  libsWrap.className = 'field-group';
  const libsLbl = document.createElement('label');
  libsLbl.textContent = 'External Libraries';
  libsWrap.appendChild(libsLbl);
  libsWrap.appendChild(libsList);
  libsWrap.appendChild(addLibBtn);

  const btn = runButton('Run init');
  const { wrap: cWrap, pre } = consoleEl();

  const row1 = document.createElement('div');
  row1.className = 'field-row';
  row1.appendChild(field('Project Name', nameIn));
  row1.appendChild(field('Output Path', pathIn));

  const row2 = document.createElement('div');
  row2.className = 'field-row';
  row2.appendChild(depthField);
  row2.appendChild(field('Target Language', langSel));

  [row1, row2, field('Blueprint File', bpIn, 'Overrides depth when provided'), libsWrap, btn, cWrap]
    .forEach(el => body.appendChild(el));

  btn.addEventListener('click', () => {
    const libs = Array.from(libsList.querySelectorAll('.lib-row input'))
      .map(i => i.value.trim()).filter(Boolean);
    runCmd('/api/init', {
      name:      nameIn.value.trim() || 'my_project',
      depth:     parseInt(depthSlider.value),
      lang:      langSel.value || null,
      libs,
      blueprint: bpIn.value.trim() || null,
      path:      pathIn.value.trim() || null,
    }, btn, pre);
  });

  return makePanel('init — scaffold project', '🏗️', body);
}

// ── convert panel ─────────────────────────────────────────────────────────────

function buildConvertPanel() {
  const body = document.createDocumentFragment();

  const targetIn = textInput('./my_project  or  ./file.bu');
  const secondIn = textInput('rs / py / c / cpp / go  or  out.rs  (optional)');

  const btn = runButton('Run convert');
  const { wrap: cWrap, pre } = consoleEl();

  const row = document.createElement('div');
  row.className = 'field-row';
  row.appendChild(field('Source Path', targetIn));
  row.appendChild(field('Language / Output', secondIn, 'Short ext = language override; filename = explicit output path'));

  [row, btn, cWrap].forEach(el => body.appendChild(el));

  btn.addEventListener('click', () => {
    runCmd('/api/convert', {
      target: targetIn.value.trim() || null,
      second: secondIn.value.trim() || null,
    }, btn, pre);
  });

  return makePanel('convert — transpile to target language', '⚡', body);
}

// ── control panel (check + fmt) ───────────────────────────────────────────────

function buildControlPanel() {
  const body = document.createDocumentFragment();

  // Sub-card chooser
  const subRow = document.createElement('div');
  subRow.className = 'sub-choice-row';

  const checkCard = makeSubCard('🔍', 'check',
    'Validate structure, type-check, and verify formatting from the current directory.');
  const fmtCard   = makeSubCard('✨', 'fmt',
    'Reformat all .bu files to canonical style, with optional dry-run preview.');

  subRow.appendChild(checkCard);
  subRow.appendChild(fmtCard);
  body.appendChild(subRow);

  // Sub-panel container
  const subPanelWrap = document.createElement('div');
  body.appendChild(subPanelWrap);

  checkCard.addEventListener('click', () => {
    toggleSubCard(checkCard, fmtCard);
    subPanelWrap.innerHTML = '';
    if (checkCard.classList.contains('active'))
      subPanelWrap.appendChild(buildCheckSubPanel());
  });

  fmtCard.addEventListener('click', () => {
    toggleSubCard(fmtCard, checkCard);
    subPanelWrap.innerHTML = '';
    if (fmtCard.classList.contains('active'))
      subPanelWrap.appendChild(buildFmtSubPanel());
  });

  return makePanel('control — check & fmt', '🔧', body);
}

function toggleSubCard(active, other) {
  if (active.classList.contains('active')) {
    active.classList.remove('active');
  } else {
    active.classList.add('active');
    other.classList.remove('active');
  }
}

function makeSubCard(icon, title, desc) {
  const card = document.createElement('div');
  card.className = 'sub-card';
  card.innerHTML = `
    <span class="sub-card-icon">${icon}</span>
    <div class="sub-card-title">${title}</div>
    <div class="sub-card-desc">${desc}</div>
  `;
  return card;
}

function buildCheckSubPanel() {
  const wrap = document.createElement('div');
  wrap.className = 'sub-panel';

  const banner = infoBanner(
    'Runs structural validation, type-checking, and a format drift check on the Bullang project rooted at the server\'s current working directory.'
  );
  const btn = runButton('Run check');
  const { wrap: cWrap, pre } = consoleEl();

  [banner, btn, cWrap].forEach(el => wrap.appendChild(el));
  btn.addEventListener('click', () => runCmd('/api/check', {}, btn, pre));
  return wrap;
}

function buildFmtSubPanel() {
  const wrap = document.createElement('div');
  wrap.className = 'sub-panel';

  const folderIn = textInput('./my_project  (leave empty for current dir)');

  const toggleRow = document.createElement('div');
  toggleRow.className = 'field-group';
  const optLbl = document.createElement('label');
  optLbl.textContent = 'Options';
  const tRow = document.createElement('div');
  tRow.className = 'toggle-row';
  const label = document.createElement('label');
  label.className = 'toggle';
  const cb    = document.createElement('input');
  cb.type = 'checkbox';
  const track = document.createElement('span');
  track.className = 'toggle-track';
  label.appendChild(cb);
  label.appendChild(track);
  const toggleTxt = document.createElement('span');
  toggleTxt.style.cssText = 'font-size:12px;color:var(--text-muted);';
  toggleTxt.textContent = 'Dry run (preview without writing)';
  tRow.appendChild(label);
  tRow.appendChild(toggleTxt);
  toggleRow.appendChild(optLbl);
  toggleRow.appendChild(tRow);

  const btn = runButton('Run fmt');
  const { wrap: cWrap, pre } = consoleEl();

  [field('Project Folder', folderIn), toggleRow, btn, cWrap]
    .forEach(el => wrap.appendChild(el));

  btn.addEventListener('click', () => {
    runCmd('/api/fmt', {
      folder:  folderIn.value.trim() || null,
      dry_run: cb.checked,
    }, btn, pre);
  });

  return wrap;
}

// ── options panel (editor-setup + update) ─────────────────────────────────────

function buildOptionsPanel() {
  const body = document.createDocumentFragment();

  const subRow = document.createElement('div');
  subRow.className = 'sub-choice-row';

  const editorCard = makeSubCard('🛠️', 'editor-setup',
    'Write LSP configs for Neovim, Vim, Helix, and Emacs automatically.');
  const updateCard = makeSubCard('🚀', 'update',
    'Reinstall Bullarchy from the latest commit on the main branch.');

  subRow.appendChild(editorCard);
  subRow.appendChild(updateCard);
  body.appendChild(subRow);

  const subPanelWrap = document.createElement('div');
  body.appendChild(subPanelWrap);

  editorCard.addEventListener('click', () => {
    toggleSubCard(editorCard, updateCard);
    subPanelWrap.innerHTML = '';
    if (editorCard.classList.contains('active'))
      subPanelWrap.appendChild(buildEditorSetupSubPanel());
  });

  updateCard.addEventListener('click', () => {
    toggleSubCard(updateCard, editorCard);
    subPanelWrap.innerHTML = '';
    if (updateCard.classList.contains('active'))
      subPanelWrap.appendChild(buildUpdateSubPanel());
  });

  return makePanel('options — editor & update', '⚙️', body);
}

function buildEditorSetupSubPanel() {
  const wrap = document.createElement('div');
  wrap.className = 'sub-panel';

  const banner = infoBanner(
    'Detects installed editors (Neovim, Vim, Helix, Emacs) and writes LSP configuration files so they can use the Bullang language server automatically.'
  );
  const btn = runButton('Run editor-setup');
  const { wrap: cWrap, pre } = consoleEl();

  [banner, btn, cWrap].forEach(el => wrap.appendChild(el));
  btn.addEventListener('click', () => runCmd('/api/editor-setup', {}, btn, pre));
  return wrap;
}

function buildUpdateSubPanel() {
  const wrap = document.createElement('div');
  wrap.className = 'sub-panel';

  const banner = infoBanner(
    'Reinstalls Bullarchy from the latest commit on the main branch via `cargo install --git`. Checks the installed hash first and skips if already up to date.'
  );
  const btn = runButton('Run update');
  const { wrap: cWrap, pre } = consoleEl();

  [banner, btn, cWrap].forEach(el => wrap.appendChild(el));
  btn.addEventListener('click', () => runCmd('/api/update', {}, btn, pre));
  return wrap;
}

// ── Blueprint state ──────────────────────────────────────────────────────────

const BP_LANGS = ['', 'rs', 'py', 'c', 'cpp', 'go'];
const BP_RANKS = ['skirmish','tactic','strategy','battle','theater','war'];

let bpTree      = null;
let bpNextId    = 1;
let bpPopover   = null;
let bpActiveId  = null;
let bpRenderRoot = null;
let bpCanvasEl   = null;

function bpNewFolder(name = 'folder', lang = '', owner = '') {
  return { id: bpNextId++, kind: 'folder', name, lang, owner, children: [] };
}

function bpNewFile(name = 'file', fns = [], owner = '') {
  return { id: bpNextId++, kind: 'file', name, fns, owner };
}

function bpRemoveNode(parent, id) {
  if (parent.kind !== 'folder') return false;
  const idx = parent.children.findIndex(c => c.id === id);
  if (idx !== -1) { parent.children.splice(idx, 1); return true; }
  for (const c of parent.children) { if (bpRemoveNode(c, id)) return true; }
  return false;
}

function bpDepthOf(node, target, d = 0) {
  if (node.id === target) return d;
  if (node.kind === 'folder') {
    for (const c of node.children) {
      const r = bpDepthOf(c, target, d + 1);
      if (r !== -1) return r;
    }
  }
  return -1;
}

function bpSerialize(node, indent = 0) {
  const pad  = '    '.repeat(indent);
  const pad1 = '    '.repeat(indent + 1);
  if (node.kind === 'file') {
    const fns  = node.fns.length ? node.fns.join(', ') : '';
    const stub = fns ? `: ${fns}` : ': _';
    if (node.owner)
      return `${pad}${node.name}.bu ${stub} {\n${pad1}owner : "${node.owner}"\n${pad}}\n`;
    return `${pad}${node.name}.bu ${stub};\n`;
  }
  const langPrefix = node.lang ? `${node.lang}: ` : '';
  const depth      = bpTree ? bpDepthOf(bpTree, node.id) : 0;
  const rank       = BP_RANKS[Math.min(depth, BP_RANKS.length - 1)];
  const header     = `${pad}${langPrefix}${rank} ${node.name}`;
  if (!node.children.length) return `${header} {}\n`;
  const inner     = node.children.map(c => bpSerialize(c, indent + 1)).join('');
  const ownerLine = node.owner ? `${pad1}// owner: ${node.owner}\n` : '';
  return `${header} {\n${ownerLine}${inner}${pad}}\n`;
}

function bpGenerateBu() {
  if (!bpTree) return '';
  return bpSerialize(bpTree, 0);
}

function bpSetStatus(el, msg, ok) {
  el.textContent = msg;
  el.className   = `bp-status ${ok ? 'ok' : 'err'}`;
}

function bpReset() {
  bpTree       = null;
  bpNextId     = 1;
  bpPopover    = null;
  bpActiveId   = null;
  bpRenderRoot = null;
  bpCanvasEl   = null;
}

function bpMountInto(container) {
  bpReset();
  container.innerHTML = '';

  const toolbar    = document.createElement('div');
  toolbar.className = 'bp-toolbar';
  const savePathIn  = document.createElement('input');
  savePathIn.className = 'bp-save-path';
  savePathIn.type  = 'text';
  savePathIn.placeholder = '/home/user/project/blueprint.bu';
  const saveBtn    = document.createElement('button');
  saveBtn.className = 'btn-save-bp';
  saveBtn.textContent = 'Save blueprint';
  const statusEl   = document.createElement('span');
  statusEl.className = 'bp-status';
  toolbar.appendChild(savePathIn);
  toolbar.appendChild(saveBtn);
  toolbar.appendChild(statusEl);
  container.appendChild(toolbar);

  const stage      = document.createElement('div');
  stage.className  = 'bp-stage';
  bpCanvasEl       = document.createElement('canvas');
  bpCanvasEl.className = 'bp-canvas';
  stage.appendChild(bpCanvasEl);
  bpRenderRoot     = document.createElement('div');
  bpRenderRoot.className = 'bp-pyramid';
  stage.appendChild(bpRenderRoot);
  container.appendChild(stage);

  bpRenderEmpty();

  document.addEventListener('pointerdown', bpOnOutsideClick, { capture: true });

  saveBtn.addEventListener('click', async () => {
    const path = savePathIn.value.trim();
    if (!path)       { bpSetStatus(statusEl, 'Enter a save path first.', false); return; }
    const content = bpGenerateBu();
    if (!content)    { bpSetStatus(statusEl, 'Nothing to save.', false); return; }
    saveBtn.disabled = true;
    try {
      const res  = await fetch('/api/blueprint/save', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path, content }),
      });
      const data = await res.json();
      bpSetStatus(statusEl, data.ok ? `Saved to ${path}` : (data.error || 'Save failed.'), data.ok);
    } catch (e) {
      bpSetStatus(statusEl, `Network error: ${e.message}`, false);
    } finally { saveBtn.disabled = false; }
  });
}

function bpRenderEmpty() {
  bpRenderRoot.innerHTML = '';
  const wrap = document.createElement('div');
  wrap.className = 'bp-empty';
  const txt = document.createElement('div');
  txt.className = 'bp-empty-text';
  txt.textContent = 'Start your blueprint by creating the root folder.';
  const addBtn = document.createElement('button');
  addBtn.className = 'bp-add-root';
  addBtn.textContent = '+ Create root folder';
  addBtn.addEventListener('click', () => { bpTree = bpNewFolder('root'); bpRenderTree(); });
  wrap.appendChild(txt);
  wrap.appendChild(addBtn);
  bpRenderRoot.appendChild(wrap);
}

function bpRenderTree() {
  if (!bpRenderRoot) return;
  bpClosePopover();
  bpRenderRoot.innerHTML = '';

  const levels = [];
  let current  = [{ node: bpTree, parentId: null }];
  while (current.length) {
    levels.push(current);
    const next = [];
    for (const { node } of current) {
      if (node.kind === 'folder') {
        for (const c of node.children) next.push({ node: c, parentId: node.id });
      }
    }
    current = next;
  }

  const nodeEls = {};
  for (const level of levels) {
    const row = document.createElement('div');
    row.className = 'bp-row';
    for (const { node } of level) {
      const el = bpRenderNode(node);
      nodeEls[node.id] = el;
      row.appendChild(el);
    }
    bpRenderRoot.appendChild(row);
  }

  requestAnimationFrame(() => bpDrawConnectors(levels, nodeEls));
}

function bpRenderNode(node) {
  const wrap = document.createElement('div');
  wrap.className = `bp-node bp-node-${node.kind}`;
  wrap.dataset.id = node.id;

  const card = document.createElement('div');
  card.className = 'bp-card';
  card.addEventListener('click', (e) => { e.stopPropagation(); bpOpenPopover(node, card); });

  const icon = document.createElement('span');
  icon.className = 'bp-node-icon';
  icon.textContent = node.kind === 'folder' ? '📁' : '📄';

  const nameEl = document.createElement('span');
  nameEl.className = 'bp-node-name';
  nameEl.textContent = node.kind === 'folder'
    ? node.name + (node.lang ? `  [${node.lang}]` : '')
    : node.name + '.bu';

  card.appendChild(icon);
  card.appendChild(nameEl);

  if (node.kind === 'file' && node.fns.length) {
    const fnsEl = document.createElement('div');
    fnsEl.className = 'bp-node-fns';
    fnsEl.textContent = node.fns.join(', ');
    card.appendChild(fnsEl);
  }

  if (node.owner) {
    const ownerEl = document.createElement('div');
    ownerEl.className = 'bp-node-owner';
    ownerEl.textContent = `@${node.owner}`;
    card.appendChild(ownerEl);
  }

  wrap.appendChild(card);

  if (node.kind === 'folder') {
    const addBtn = document.createElement('button');
    addBtn.className = 'bp-add-child';
    addBtn.textContent = '+';
    addBtn.title = 'Add child';
    addBtn.addEventListener('click', (e) => { e.stopPropagation(); bpOpenAddChild(node, addBtn); });
    wrap.appendChild(addBtn);
  }

  return wrap;
}

function bpOpenPopover(node, anchor) {
  if (bpActiveId === node.id) { bpClosePopover(); return; }
  bpClosePopover();
  bpActiveId = node.id;
  anchor.classList.add('bp-card-active');

  const pop = document.createElement('div');
  pop.className = 'bp-popover';

  const title = document.createElement('div');
  title.className = 'bp-pop-title';
  title.textContent = node.kind === 'folder' ? 'Edit folder' : 'Edit file';
  pop.appendChild(title);

  pop.appendChild(bpPopField('Name', node.name, (v) => { node.name = v; bpRefreshNode(node); }));

  if (node.kind === 'folder') {
    const lg  = document.createElement('div');
    lg.className = 'bp-pop-field';
    const ll  = document.createElement('label');
    ll.textContent = 'Language';
    const sel = document.createElement('select');
    sel.className = 'bp-pop-select';
    BP_LANGS.forEach(l => {
      const o = document.createElement('option');
      o.value = l; o.textContent = l || 'auto';
      if (l === node.lang) o.selected = true;
      sel.appendChild(o);
    });
    sel.addEventListener('change', () => { node.lang = sel.value; bpRefreshNode(node); });
    lg.appendChild(ll); lg.appendChild(sel);
    pop.appendChild(lg);
  }

  if (node.kind === 'file') {
    const fw  = document.createElement('div');
    fw.className = 'bp-pop-field';
    const fl  = document.createElement('label');
    fl.textContent = 'Functions';
    const fi  = document.createElement('input');
    fi.className = 'bp-pop-input'; fi.type = 'text';
    fi.placeholder = 'fn1, fn2, fn3';
    fi.value = node.fns.join(', ');
    fi.addEventListener('input', () => {
      node.fns = fi.value.split(',').map(s => s.trim()).filter(Boolean);
      bpRefreshNode(node);
    });
    fw.appendChild(fl); fw.appendChild(fi);
    pop.appendChild(fw);
  }

  pop.appendChild(bpPopField('Owner (optional)', node.owner || '', (v) => { node.owner = v; bpRefreshNode(node); }));

  if (node !== bpTree) {
    const del = document.createElement('button');
    del.className = 'bp-pop-delete';
    del.textContent = 'Delete node';
    del.addEventListener('click', () => {
      bpRemoveNode(bpTree, node.id);
      bpClosePopover();
      bpRenderTree();
    });
    pop.appendChild(del);
  }

  bpPositionPopover(pop, anchor);
  bpPopover = pop;
}

function bpPopField(label, value, onChange) {
  const wrap = document.createElement('div');
  wrap.className = 'bp-pop-field';
  const lbl = document.createElement('label');
  lbl.textContent = label;
  const inp = document.createElement('input');
  inp.className = 'bp-pop-input'; inp.type = 'text'; inp.value = value;
  inp.addEventListener('input', () => onChange(inp.value.trim()));
  wrap.appendChild(lbl); wrap.appendChild(inp);
  return wrap;
}

function bpPositionPopover(pop, anchor) {
  pop.style.cssText = 'position:fixed;visibility:hidden;';
  document.body.appendChild(pop);
  const rect = anchor.getBoundingClientRect();
  const pw   = pop.offsetWidth  || 220;
  const ph   = pop.offsetHeight || 200;
  const gap  = 10;
  let left   = rect.right + gap;
  let top    = rect.top;
  if (left + pw > window.innerWidth  - gap) left = rect.left - pw - gap;
  if (top  + ph > window.innerHeight - gap) top  = window.innerHeight - ph - gap;
  if (top < gap) top = gap;
  pop.style.left = `${left}px`;
  pop.style.top  = `${top}px`;
  pop.style.visibility = 'visible';
}

function bpOpenAddChild(parentNode, anchor) {
  bpClosePopover();
  bpActiveId = `add-${parentNode.id}`;

  const pop = document.createElement('div');
  pop.className = 'bp-popover';

  const title = document.createElement('div');
  title.className = 'bp-pop-title';
  title.textContent = 'Add child';
  pop.appendChild(title);

  const choices = document.createElement('div');
  choices.className = 'bp-pop-choices';

  const folderBtn = document.createElement('button');
  folderBtn.className = 'bp-pop-choice';
  folderBtn.innerHTML = '📁<span>Folder</span>';
  folderBtn.addEventListener('click', () => {
    parentNode.children.push(bpNewFolder());
    bpClosePopover();
    bpRenderTree();
  });

  const fileBtn = document.createElement('button');
  fileBtn.className = 'bp-pop-choice';
  fileBtn.innerHTML = '📄<span>File</span>';
  fileBtn.addEventListener('click', () => {
    parentNode.children.push(bpNewFile());
    bpClosePopover();
    bpRenderTree();
  });

  choices.appendChild(folderBtn);
  choices.appendChild(fileBtn);
  pop.appendChild(choices);
  bpPositionPopover(pop, anchor);
  bpPopover = pop;
}

function bpClosePopover() {
  if (bpPopover) { bpPopover.remove(); bpPopover = null; }
  bpActiveId = null;
  document.querySelectorAll('.bp-card-active').forEach(el => el.classList.remove('bp-card-active'));
}

function bpOnOutsideClick(e) {
  if (!bpPopover) return;
  if (!bpPopover.contains(e.target) && !e.target.closest('.bp-card') && !e.target.closest('.bp-add-child'))
    bpClosePopover();
}

function bpRefreshNode(node) {
  const el      = document.querySelector(`.bp-node[data-id="${node.id}"] .bp-card`);
  if (!el) return;
  const nameEl  = el.querySelector('.bp-node-name');
  const fnsEl   = el.querySelector('.bp-node-fns');
  const ownerEl = el.querySelector('.bp-node-owner');
  if (nameEl) nameEl.textContent = node.kind === 'folder'
    ? node.name + (node.lang ? `  [${node.lang}]` : '')
    : node.name + '.bu';
  if (node.kind === 'file') {
    if (node.fns.length) {
      if (fnsEl) { fnsEl.textContent = node.fns.join(', '); }
      else { const f = document.createElement('div'); f.className = 'bp-node-fns'; f.textContent = node.fns.join(', '); el.appendChild(f); }
    } else if (fnsEl) fnsEl.remove();
  }
  if (node.owner) {
    if (ownerEl) { ownerEl.textContent = `@${node.owner}`; }
    else { const o = document.createElement('div'); o.className = 'bp-node-owner'; o.textContent = `@${node.owner}`; el.appendChild(o); }
  } else if (ownerEl) ownerEl.remove();
}

function bpDrawConnectors(levels, nodeEls) {
  if (!bpCanvasEl || !bpRenderRoot) return;
  const stageRect    = bpRenderRoot.closest('.bp-stage').getBoundingClientRect();
  bpCanvasEl.width   = stageRect.width;
  bpCanvasEl.height  = stageRect.height;
  const ctx          = bpCanvasEl.getContext('2d');
  ctx.clearRect(0, 0, bpCanvasEl.width, bpCanvasEl.height);
  ctx.strokeStyle    = 'rgba(74, 158, 255, 0.35)';
  ctx.lineWidth      = 1.5;
  ctx.setLineDash([4, 4]);

  for (let li = 0; li < levels.length - 1; li++) {
    for (const { node } of levels[li]) {
      if (node.kind !== 'folder') continue;
      const pEl   = nodeEls[node.id];
      if (!pEl) continue;
      const pRect = pEl.getBoundingClientRect();
      const px    = pRect.left + pRect.width / 2 - stageRect.left;
      const py    = pRect.bottom - stageRect.top;
      for (const child of node.children) {
        const cEl   = nodeEls[child.id];
        if (!cEl) continue;
        const cRect = cEl.getBoundingClientRect();
        const cx    = cRect.left + cRect.width / 2 - stageRect.left;
        const cy    = cRect.top - stageRect.top;
        ctx.beginPath();
        ctx.moveTo(px, py);
        ctx.bezierCurveTo(px, py + (cy - py) * 0.5, cx, py + (cy - py) * 0.5, cx, cy);
        ctx.stroke();
      }
    }
  }
}

window.addEventListener('resize', () => {
  if (!bpTree || !bpRenderRoot) return;
  const levels  = [];
  let current   = [{ node: bpTree }];
  const nodeEls = {};
  while (current.length) {
    levels.push(current);
    const next = [];
    for (const { node } of current) {
      const el = document.querySelector(`.bp-node[data-id="${node.id}"]`);
      if (el) nodeEls[node.id] = el;
      if (node.kind === 'folder') for (const c of node.children) next.push({ node: c });
    }
    current = next;
  }
  requestAnimationFrame(() => bpDrawConnectors(levels, nodeEls));
});

// ── blueprint panel ───────────────────────────────────────────────────────────

function buildBlueprintPanel() {
  const frag = document.createDocumentFragment();

  const hint = infoBanner(
    'Build your project blueprint visually. Click any node to edit it. Use + to add child folders or files. Save to disk when ready.'
  );
  frag.appendChild(hint);

  const mount = document.createElement('div');
  mount.className = 'bp-editor-mount';
  frag.appendChild(mount);

  const panel = makePanel('blueprint — visual editor', '🗺️', frag);

  // Mount after panel is inserted into the DOM
  requestAnimationFrame(() => bpMountInto(mount));

  return panel;
}
