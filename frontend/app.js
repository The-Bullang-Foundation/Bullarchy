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

// ── blueprint panel ───────────────────────────────────────────────────────────

function buildBlueprintPanel() {
  const frag = document.createDocumentFragment();

  const hintBanner = infoBanner(
    'Write your blueprint.bu in the editor on the left. The tree preview on the right updates as you type. Save to any path on disk when ready.'
  );
  frag.appendChild(hintBanner);

  // Editor / preview split
  const editor = document.createElement('div');
  editor.className = 'blueprint-editor';

  // ── Left pane: textarea ──────────────────────────────────────────────────
  const leftPane = document.createElement('div');
  leftPane.className = 'bp-pane bp-editor-pane';

  const leftHeader = document.createElement('div');
  leftHeader.className = 'bp-pane-header';
  leftHeader.innerHTML = '<span>blueprint.bu</span><span id="bp-parse-status"></span>';

  const textarea = document.createElement('textarea');
  textarea.className = 'bp-textarea';
  textarea.spellcheck = false;
  textarea.value = BLUEPRINT_PLACEHOLDER;

  leftPane.appendChild(leftHeader);
  leftPane.appendChild(textarea);

  // ── Right pane: live tree ────────────────────────────────────────────────
  const rightPane = document.createElement('div');
  rightPane.className = 'bp-pane';

  const rightHeader = document.createElement('div');
  rightHeader.className = 'bp-pane-header';
  rightHeader.innerHTML = '<span>Preview</span>';

  const preview = document.createElement('div');
  preview.className = 'bp-preview';

  rightPane.appendChild(rightHeader);
  rightPane.appendChild(preview);

  editor.appendChild(leftPane);
  editor.appendChild(rightPane);
  frag.appendChild(editor);

  // ── Save bar ──────────────────────────────────────────────────────────────
  const actionsBar = document.createElement('div');
  actionsBar.className = 'bp-actions';

  const pathIn = document.createElement('input');
  pathIn.className = 'bp-save-path';
  pathIn.type = 'text';
  pathIn.placeholder = '/home/user/projects/my_project/blueprint.bu';

  const saveBtn = document.createElement('button');
  saveBtn.className = 'btn-save-bp';
  saveBtn.textContent = 'Save blueprint';

  const statusEl = document.createElement('span');
  statusEl.className = 'bp-status';

  actionsBar.appendChild(pathIn);
  actionsBar.appendChild(saveBtn);
  actionsBar.appendChild(statusEl);
  frag.appendChild(actionsBar);

  // ── Live preview logic ───────────────────────────────────────────────────
  const parseStatusEl = leftHeader.querySelector('#bp-parse-status');

  function refreshPreview() {
    const src = textarea.value;
    try {
      const nodes = parseBlueprint(src);
      preview.innerHTML = '';
      if (nodes.length === 0) {
        const empty = document.createElement('div');
        empty.className = 'bp-tree-empty';
        empty.textContent = 'Nothing to show yet.';
        preview.appendChild(empty);
      } else {
        renderTree(nodes, preview, 0);
      }
      parseStatusEl.textContent = '✓ valid';
      parseStatusEl.style.color = 'var(--ok-green)';
    } catch (e) {
      preview.innerHTML = `<div class="bp-tree-error">${escHtml(e.message)}</div>`;
      parseStatusEl.textContent = '✗ error';
      parseStatusEl.style.color = 'var(--err-red)';
    }
  }

  textarea.addEventListener('input', refreshPreview);
  refreshPreview();

  // ── Save logic ───────────────────────────────────────────────────────────
  saveBtn.addEventListener('click', async () => {
    const savePath = pathIn.value.trim();
    if (!savePath) {
      statusEl.textContent = 'Enter a save path first.';
      statusEl.className = 'bp-status err';
      return;
    }
    saveBtn.disabled = true;
    statusEl.textContent = 'Saving…';
    statusEl.className = 'bp-status';

    try {
      const res  = await fetch('/api/blueprint/save', {
        method:  'POST',
        headers: { 'Content-Type': 'application/json' },
        body:    JSON.stringify({ path: savePath, content: textarea.value }),
      });
      const data = await res.json();
      if (data.ok) {
        statusEl.textContent = `Saved to ${savePath}`;
        statusEl.className = 'bp-status ok';
      } else {
        statusEl.textContent = data.error || 'Save failed.';
        statusEl.className = 'bp-status err';
      }
    } catch (e) {
      statusEl.textContent = `Network error: ${e.message}`;
      statusEl.className = 'bp-status err';
    } finally {
      saveBtn.disabled = false;
    }
  });

  return makePanel('blueprint — visual editor', '🗺️', frag);
}

// ── Blueprint client-side parser ──────────────────────────────────────────────
// Mirrors the Rust parser logic closely enough to give live feedback.

function parseBlueprint(src) {
  const lines = src.split('\n');
  const unit  = detectIndentUnit(lines);
  const [nodes] = parseBlock(lines, 0, 0, unit);
  return nodes;
}

function detectIndentUnit(lines) {
  for (const line of lines) {
    const t = line.trim();
    if (!t || t.startsWith('//')) continue;
    const spaces = line.length - line.trimStart().length;
    if (spaces > 0) return spaces;
  }
  return 4;
}

function parseBlock(lines, start, baseIndent, unit) {
  const nodes = [];
  let i = start;

  while (i < lines.length) {
    const line    = lines[i];
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('//')) { i++; continue; }

    const indent = line.length - line.trimStart().length;
    if (indent < baseIndent) break;
    if (trimmed === '}') { i++; continue; }

    if (indent > baseIndent) {
      throw new Error(`Line ${i+1}: unexpected indentation (expected ${baseIndent}, got ${indent})`);
    }

    if (isEntryLine(trimmed)) {
      const { stem, fns, hasMeta } = parseEntryHeader(trimmed, i + 1);
      i++;
      let goal = null, owner = null;
      if (hasMeta) {
        const meta = parseMetaBlock(lines, i, baseIndent, unit, i);
        goal = meta.goal; owner = meta.owner; i = meta.next;
      }
      nodes.push({ type: 'entry', name: stem, fns, goal, owner });
    } else if (trimmed.endsWith('{') || trimmed.endsWith(':')) {
      const { lang, name } = parseFolderHeader(trimmed);
      if (!name) throw new Error(`Line ${i+1}: empty folder name`);
      i++;
      const [children, next] = parseBlock(lines, i, baseIndent + unit, unit);
      nodes.push({ type: 'folder', name, lang, children });
      i = next;
    } else {
      throw new Error(`Line ${i+1}: expected folder or file entry — got: "${trimmed}"`);
    }
  }

  return [nodes, i];
}

function isEntryLine(t) {
  const pos = t.indexOf('.bu');
  return pos !== -1 && t.slice(pos + 3).trimStart().startsWith(':');
}

function parseEntryHeader(t, lineNo) {
  const pos  = t.indexOf('.bu');
  const stem = t.slice(0, pos).trim();
  if (!stem) throw new Error(`Line ${lineNo}: empty file name`);
  const after = t.slice(pos + 3).trimStart();
  if (!after.startsWith(':')) throw new Error(`Line ${lineNo}: expected ':' after filename`);
  const rest    = after.slice(1).trim();
  const hasMeta = rest.endsWith('{');
  const fnsRaw  = hasMeta ? rest.slice(0, -1).trim() : rest.replace(/;$/, '').trim();
  const fns     = fnsRaw.split(',').map(s => s.trim()).filter(Boolean);
  return { stem, fns, hasMeta };
}

function parseMetaBlock(lines, start, entryIndent, unit, entryLine) {
  let goal = null, owner = null, i = start;
  const fieldIndent = entryIndent + unit;

  while (i < lines.length) {
    const line    = lines[i];
    const trimmed = line.trim();
    if (!trimmed) { i++; continue; }
    const indent = line.length - line.trimStart().length;

    if (trimmed === '}' && indent === entryIndent) return { goal, owner, next: i + 1 };
    if (indent === fieldIndent) {
      const gv = extractQuotedField(trimmed, 'goal');
      const ov = extractQuotedField(trimmed, 'owner');
      if (gv !== null) goal  = gv;
      if (ov !== null) owner = ov;
    }
    i++;
  }
  throw new Error(`Line ${entryLine}: metadata block was never closed`);
}

function extractQuotedField(t, key) {
  if (!t.toLowerCase().startsWith(key)) return null;
  const rest = t.slice(key.length).trimStart();
  if (!rest.startsWith(':')) return null;
  const val = rest.slice(1).trim().replace(/^"|"$/g, '');
  return val || null;
}

const LANG_MAP = {
  rust:'rs', rs:'rs', python:'py', py:'py',
  c:'c', cpp:'cpp', 'c++':'cpp', go:'go', golang:'go',
};
const RANK_WORDS = new Set(['war','theater','battle','strategy','tactic','skirmish']);

function parseFolderHeader(t) {
  const stripped = t.replace(/\{$/, '').replace(/:$/, '').trim();
  const colon    = stripped.indexOf(':');
  if (colon !== -1) {
    const prefix = stripped.slice(0, colon).trim().toLowerCase();
    const rest   = stripped.slice(colon + 1).trim();
    if (rest && LANG_MAP[prefix]) {
      return { lang: LANG_MAP[prefix], name: stripRank(rest) };
    }
  }
  return { lang: null, name: stripRank(stripped) };
}

function stripRank(s) {
  const parts = s.trim().split(/\s+/);
  if (parts.length > 1 && RANK_WORDS.has(parts[0].toLowerCase())) {
    return parts.slice(1).join(' ');
  }
  return s.trim();
}

// ── Blueprint tree renderer ───────────────────────────────────────────────────

function renderTree(nodes, container, depth) {
  for (const node of nodes) {
    if (node.type === 'folder') {
      const el = document.createElement('div');
      el.className = 'bp-tree-node';
      el.style.paddingLeft = `${depth * 16}px`;
      const folderEl = document.createElement('div');
      folderEl.className = 'bp-tree-folder';
      folderEl.textContent = node.name + (node.lang ? `  [${node.lang}]` : '');
      el.appendChild(folderEl);
      container.appendChild(el);
      renderTree(node.children, container, depth + 1);

    } else if (node.type === 'entry') {
      const el = document.createElement('div');
      el.className = 'bp-tree-node';
      el.style.paddingLeft = `${depth * 16}px`;

      const fileEl = document.createElement('div');
      fileEl.className = 'bp-tree-file';
      fileEl.textContent = `${node.name}.bu`;
      el.appendChild(fileEl);

      for (const fn of node.fns) {
        const fnEl = document.createElement('div');
        fnEl.className = 'bp-tree-fn';
        fnEl.style.paddingLeft = `${depth * 16}px`;
        fnEl.textContent = fn;
        el.appendChild(fnEl);
      }

      if (node.goal) {
        const goalEl = document.createElement('div');
        goalEl.style.cssText = `padding-left:${depth * 16 + 16}px;font-size:10px;color:var(--text-dim);font-style:italic;`;
        goalEl.textContent = `goal: "${node.goal}"`;
        el.appendChild(goalEl);
      }

      container.appendChild(el);
    }
  }
}

function escHtml(s) {
  return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}
