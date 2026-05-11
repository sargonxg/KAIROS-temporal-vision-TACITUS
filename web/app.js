const DEMO = `January 15, 2024: Riverdale Water Authority director Sarah Chen announces emergency rationing after reservoir levels fall below 20 percent. Mayor Robert Hayes says the measure is temporary but refuses to commit to a repeal date.

February 2024: Hayes promises a $50M investment in new reservoir infrastructure. Chen warns that construction will not help the immediate shortage and asks residents to keep rationing through the spring.

March 12, 2024: Hayes replaces Chen with Marcus Liu, a former infrastructure adviser. Liu says the rationing program will continue until rainfall and consumption data improve.

April 2024: Heavy rain stabilizes the reservoir. Liu ends the strictest rationing rules but keeps outdoor water restrictions in place. The infrastructure investment commitment from February becomes the centerpiece of Hayes' re-election campaign launched on May 1.

June 2024: Construction begins on the new pipeline. Liu negotiates with the neighboring Greenfield district for joint use, signing an agreement on July 18, 2024.

September 2024: Election season heats up. Hayes faces challenger Patricia Wells, who campaigns on Chen's record. Wells argues the rationing program was prematurely abandoned.

November 5, 2024: Hayes loses re-election. Wells takes office on January 1, 2025. She announces a review of all water authority decisions made under Liu, raising the possibility of reinstating elements of Chen's policy.

January 15, 2025: Exactly one year after Chen's announcement, Wells reveals a new water policy. She commits to maintaining the Greenfield pipeline but freezes new construction pending environmental review.`;

const $ = (id) => document.getElementById(id);

$('btn-demo').addEventListener('click', () => {
  $('text-input').value = DEMO;
  $('signal-panel').textContent = 'demo loaded: Riverdale crisis chronology\nready for temporal extraction';
});

$('btn-analyze').addEventListener('click', analyze);

async function analyze() {
  const text = $('text-input').value.trim();
  if (!text) return;

  const btn = $('btn-analyze');
  btn.disabled = true;
  btn.classList.add('opacity-50');
  setStatus('Analyzing temporal structure...', 'text-amber-300');

  try {
    const res = await fetch('/api/analyze', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text }),
    });
    const payload = await res.json();
    if (!res.ok) throw new Error(payload.error || `HTTP ${res.status}`);
    render(payload, text);
    setStatus(`Done: ${payload.dates.length} dates, ${payload.events.length} events, ${payload.episodes.length} episodes, ${payload.relations.length} relations.`, 'text-emerald-300');
  } catch (err) {
    console.error(err);
    setStatus(`Error: ${err.message}`, 'text-red-300');
  } finally {
    btn.disabled = false;
    btn.classList.remove('opacity-50');
  }
}

function render(data, text) {
  $('metric-dates').textContent = data.dates.length;
  $('metric-episodes').textContent = data.episodes.length;
  $('metric-relations').textContent = data.relations.length;
  $('signal-panel').textContent = summarizeSignal(data);
  $('raw-json').textContent = JSON.stringify(data, null, 2);
  $('btn-download').classList.remove('hidden');
  $('btn-download').onclick = () => downloadJson(data);
  renderAnnotated(text, data);
  renderTimeline(data);
  renderRelations(data);
  renderAco(data);
}

function summarizeSignal(data) {
  const lines = [];
  for (const ep of data.episodes || []) {
    const start = shortDate(ep.interval.from);
    const end = ep.interval.to ? shortDate(ep.interval.to) : 'open';
    lines.push(`${ep.kind.padEnd(12)} ${start} -> ${end} :: ${ep.title}`);
  }
  return lines.join('\n') || 'no episodes detected';
}

function renderAnnotated(text, data) {
  const spans = [];
  for (const d of data.dates || []) {
    spans.push({
      start: d.char_start,
      end: d.char_end,
      cls: 'date-mark',
      title: `date: ${d.resolved || d.text}`,
    });
  }
  for (const e of data.events || []) {
    const needle = (e.mention || e.canonical_name || '').trim();
    const idx = needle ? text.toLowerCase().indexOf(needle.toLowerCase()) : -1;
    if (idx >= 0) {
      spans.push({
        start: idx,
        end: idx + needle.length,
        cls: 'event-mark',
        title: `event: ${e.canonical_name}`,
      });
    }
  }
  spans.sort((a, b) => a.start - b.start);

  const clean = [];
  let lastEnd = 0;
  for (const span of spans) {
    if (span.start < lastEnd) continue;
    clean.push(span);
    lastEnd = span.end;
  }

  let html = '';
  let cur = 0;
  for (const span of clean) {
    html += escapeHtml(text.slice(cur, span.start));
    html += `<span class="${span.cls}" title="${escapeHtml(span.title)}">${escapeHtml(text.slice(span.start, span.end))}</span>`;
    cur = span.end;
  }
  html += escapeHtml(text.slice(cur));
  $('annotated').innerHTML = html;
}

function renderTimeline(data) {
  const palette = {
    regime: '#ef4444',
    leadership: '#38bdf8',
    agreement: '#22c55e',
    sanction: '#a78bfa',
    escalation: '#f97316',
    de_escalation: '#14b8a6',
    pivot: '#facc15',
    crisis: '#f43f5e',
    custom: '#94a3b8',
  };
  const items = (data.episodes || []).map((ep) => ({
    id: ep.id,
    content: `<strong>${escapeHtml(ep.title)}</strong>`,
    start: ep.interval.from,
    end: ep.interval.to || new Date().toISOString(),
    style: `background:${palette[ep.kind] || '#94a3b8'};color:#09090b;border:0;border-radius:0;padding:5px 8px;font-weight:700;`,
    title: ep.narrative || '',
  }));
  const container = $('timeline');
  container.innerHTML = '';
  if (!items.length || !window.vis) {
    container.textContent = items.length ? 'Timeline library failed to load.' : 'No episodes detected.';
    return;
  }
  new vis.Timeline(container, new vis.DataSet(items), {
    stack: true,
    margin: { item: 12 },
    orientation: 'top',
    zoomable: true,
    height: 320,
  });
}

function renderRelations(data) {
  const epById = Object.fromEntries((data.episodes || []).map((ep) => [ep.id, ep.title]));
  const tbody = $('relations-tbody');
  const interesting = (data.relations || []).filter((r) => !['Before', 'After'].includes(r.relation));
  tbody.innerHTML = interesting.map((r) => `
    <tr>
      <td class="py-2 pr-3 font-semibold text-zinc-100">${escapeHtml(epById[r.from_episode] || r.from_episode)}</td>
      <td class="py-2 pr-3 font-mono text-emerald-300">${escapeHtml(humanRel(r.relation))}</td>
      <td class="py-2 text-zinc-100">${escapeHtml(epById[r.to_episode] || r.to_episode)}</td>
    </tr>
  `).join('') || '<tr><td class="py-2 text-zinc-400">Only before/after relations detected.</td></tr>';
}

function renderAco(data) {
  const actors = (data.actors || []).map((a) => `<li><strong>${escapeHtml(a.name)}</strong> <span class="text-zinc-400">${escapeHtml(a.role)}</span></li>`).join('');
  const commitments = (data.commitments || []).map((c) => `<li><strong>${escapeHtml(c.summary)}</strong><br><span class="text-zinc-400">${escapeHtml(c.committer)} -> ${escapeHtml(c.committee)} :: ${escapeHtml(c.state)}</span></li>`).join('');
  $('aco-section').innerHTML = `
    <div class="grid gap-4 sm:grid-cols-2">
      <div><h3 class="mb-2 font-bold text-zinc-100">Actors</h3><ul class="space-y-2">${actors || '<li class="text-zinc-500">None</li>'}</ul></div>
      <div><h3 class="mb-2 font-bold text-zinc-100">Commitments</h3><ul class="space-y-2">${commitments || '<li class="text-zinc-500">None</li>'}</ul></div>
    </div>
  `;
}

function downloadJson(data) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `kairos-${data.session_id}.json`;
  a.click();
  URL.revokeObjectURL(a.href);
}

function humanRel(relation) {
  return ({
    Before: 'before',
    After: 'after',
    Meets: 'meets',
    MetBy: 'met by',
    Overlaps: 'overlaps',
    OverlappedBy: 'overlapped by',
    Starts: 'starts',
    StartedBy: 'started by',
    During: 'during',
    Contains: 'contains',
    Finishes: 'finishes',
    FinishedBy: 'finished by',
    Equals: 'coincides with',
  })[relation] || relation;
}

function shortDate(s) {
  return String(s || '').slice(0, 10);
}

function setStatus(text, cls) {
  $('status').className = `mt-2 min-h-6 text-sm ${cls}`;
  $('status').textContent = text;
}

function escapeHtml(s) {
  return String(s == null ? '' : s).replace(/[&<>"']/g, (c) => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  })[c]);
}
