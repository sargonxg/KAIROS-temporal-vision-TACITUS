const DEMO = `January 8, 2024: The Meridian River Basin Authority reports that upstream reservoirs have fallen below 31 percent after three failed rainy seasons. Interior Minister Amara Okoye orders a temporary export pause on industrial water permits while insisting that food shipments to the coastal cities will not be interrupted.

January 22, 2024: Port unions in Selene City begin rolling strikes after the export pause cuts hours for refinery crews. Governor Rafael Ibarra frames the strikes as an energy-security emergency and asks the national cabinet for authority to reopen priority terminals.

February 5, 2024: Okoye convenes the first Meridian Compact round with Governor Ibarra, upstream cooperative leader Niko Vale, Selene port-union chair Marta Soren, and Envoy Leila Haddad from the Northbridge Mediation Office. The parties agree to a 45-day monitoring period, but no one accepts responsibility for compensating workers or farmers.

February 19, 2024: The cabinet authorizes a $1.2B resilience package for desalination, canal repairs, and emergency crop insurance. Ibarra publicly commits to keeping two export terminals open, while Vale says the package is meaningless unless upstream farmers receive cash before planting season.

March 14, 2024: Satellite imagery shows illegal night pumping from three private canals. Okoye announces targeted sanctions on two water-broker firms and suspends their licenses. Soren backs the sanctions but warns that the port unions will expand the strike if refinery crews are treated as collateral damage.

April 2, 2024: Haddad opens a backchannel in Northbridge with Vale and Soren. The backchannel produces a draft sequencing plan: sanctions first, compensation second, terminal reopening third. Ibarra refuses to attend but sends technical staff to observe.

April 26, 2024: A court blocks part of Okoye's license suspension order, creating a legal gap between the sanctions episode and the compensation negotiations. The cabinet narrows the export pause to high-water industrial users and asks Haddad to merge the backchannel draft with the official compact.

May 10, 2024: The parties sign the Meridian Compact. The agreement creates a three-stage implementation clock: farmer payments by June 15, 2024, canal audits by July 31, 2024, and conditional terminal reopening by August 20, 2024. Ibarra promises that Selene will comply if the first two milestones are met.

June 17, 2024: Payment data shows that only 42 percent of eligible farmers have received compensation. Vale declares the compact in breach and threatens to restart canal blockades. Okoye says the delay is administrative, not political, and asks for a two-week cure period.

July 9, 2024: A flash flood damages the eastern canal inspection route. The flood temporarily eases reservoir pressure but destroys audit equipment and pushes the July 31 canal-audit milestone into uncertainty. Haddad warns that the parties are mistaking rainfall relief for institutional compliance.

August 22, 2024: Selene reopens one export terminal under a provisional safety protocol. Soren suspends the strike but keeps crews on a 72-hour recall posture. Ibarra claims victory; Vale calls the reopening premature because the canal audit remains incomplete.

September 18, 2024: A leaked memo shows that two sanctioned water brokers continued advising private farms through shell companies. Okoye expands the sanctions list, and the cabinet creates a verification cell to reconcile satellite evidence, payment records, and port throughput.

October 7, 2024: The verification cell finds that compensation improved after the cure period, but implementation remains uneven across districts. Haddad proposes a winter review window from November 1, 2024 through December 15, 2024 to separate crisis relief, legal enforcement, and long-term basin reform.

November 12, 2024: Ibarra enters the national leadership race and recasts the compact as proof that hard infrastructure can beat scarcity politics. Okoye resigns from the cabinet two days later after accusing campaign officials of turning the compact into an election asset.

December 3, 2024: Deputy Minister Tomas Reed takes over the water portfolio and freezes new sanctions pending the winter review. Vale returns to the table, Soren keeps the port open, and Haddad begins drafting a successor framework for 2025.

January 6, 2025: The winter review concludes that the Meridian Compact prevented a full basin shutdown but failed to produce durable trust. Reed commits to a public implementation ledger, monthly satellite audits, and a February 2025 decision on whether to convert the compact into a permanent basin authority.

February 14, 2025: The successor framework is published as the Meridian Authority Bill. It preserves the compensation ledger, converts the verification cell into a standing temporal-monitoring unit, and requires every future emergency order to specify valid time, review time, and reversal conditions.`;

const $ = (id) => document.getElementById(id);

$('btn-demo').addEventListener('click', () => {
  $('text-input').value = DEMO;
  $('signal-panel').textContent = 'demo loaded: Meridian Compact crisis\nready for temporal extraction';
});

$('btn-analyze').addEventListener('click', analyze);
$('btn-clear-key').addEventListener('click', () => {
  $('gemini-key').value = '';
  $('gemini-key').focus();
});

async function analyze() {
  const text = $('text-input').value.trim();
  if (!text) return;

  const btn = $('btn-analyze');
  btn.disabled = true;
  btn.classList.add('opacity-50');
  setStatus('Analyzing temporal structure...', 'text-amber-300');

  try {
    const geminiKey = $('gemini-key').value.trim();
    const geminiModel = $('gemini-model').value.trim();
    const body = { text };
    if (geminiKey) {
      body.gemini_api_key = geminiKey;
      if (geminiModel) body.gemini_model = geminiModel;
    }
    const res = await fetch('/api/analyze', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
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
