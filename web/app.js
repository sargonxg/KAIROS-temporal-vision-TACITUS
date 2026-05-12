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

December 16, 2024: Transcript excerpt, Haddad: "The freeze buys time, but it does not answer who can reverse a failed order." Reed answers that the ministry will publish a draft authority map by January 3, 2025.

January 6, 2025: The winter review concludes that the Meridian Compact prevented a full basin shutdown but failed to produce durable trust. Reed commits to a public implementation ledger, monthly satellite audits, and a February 2025 decision on whether to convert the compact into a permanent basin authority.

January 21, 2025: Dr. Mei Liao, newly appointed director of the verification cell, warns that payment records and satellite evidence still disagree in four districts. Vale says the discrepancy proves bad faith; Reed says it proves data fragmentation.

January 29, 2025: A leaked note from Ibarra's campaign claims the February decision should be delayed until after the party convention. Soren calls the note procedural obstruction and says port crews will not accept another indefinite review.

February 14, 2025: The successor framework is published as the Meridian Authority Bill. It preserves the compensation ledger, converts the verification cell into a standing temporal-monitoring unit, and requires every future emergency order to specify valid time, review time, and reversal conditions.

February 18, 2025: The Administrative Court schedules a March 7, 2025 hearing on whether old injunctions still constrain the new bill. The court clerk notes that no party has filed a clean timeline of valid orders, reversals, and expired sanctions.

March 7, 2025: The hearing exposes a contradiction: Okoye's April 2024 suspension order was partly blocked, but the September 2024 sanctions expansion relied on the same authority. Reed asks for a narrow ruling; broker counsel asks the court to void the entire chain.

March 20, 2025: Jonas Silva is named interim basin authority chair. Silva promises a 30-day implementation reset, but Liao warns that resetting deadlines without preserving evidence will erase the audit trail.

April 4, 2025: Public statement, Vale: "We will cooperate with Silva if payments and canal audits are dated, signed, and reversible." Ibarra replies that Selene will not keep a terminal open under a moving deadline.

April 17, 2025: Internal memo from the verification cell finds that three districts reported completion before field teams arrived. Liao labels the gap a source-integrity warning rather than proof of fraud.

May 1, 2025: The authority releases its first temporal ledger. It lists every order, commitment, review window, and reversal condition, but flags two unresolved contradictions for ministerial review.

May 19, 2025: Reed tells parliament that the ledger has reduced rumor-driven escalation, while Soren testifies that workers still lack a reliable compensation trigger when audits slip.

June 2, 2025: The basin authority votes to keep emergency powers active through July 31, 2025. Vale supports the extension only if the authority publishes missed-deadline reasons within 48 hours.

July 31, 2025: The emergency powers expire on schedule, but Silva asks for a successor vote after new reservoir data shows another dry quarter. Haddad warns that scarcity pressure is returning faster than institutional trust.`;

const $ = (id) => document.getElementById(id);
let lastAnalysis = null;
let relationFilter = 'interesting';
let corrections = {};

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
  lastAnalysis = data;
  corrections = {};
  $('metric-dates').textContent = data.dates.length;
  $('metric-episodes').textContent = data.episodes.length;
  $('metric-relations').textContent = data.relations.length;
  $('signal-panel').textContent = summarizeSignal(data);
  $('raw-json').textContent = JSON.stringify(data, null, 2);
  $('btn-download').classList.remove('hidden');
  $('btn-download').onclick = () => downloadJson(enrichedExport());
  renderMode(data);
  renderDiagnostics(data);
  renderBrief(data);
  renderAnnotated(text, data);
  renderTimeline(data);
  renderActorLanes(data);
  renderFrictionMap(data);
  renderRelations(data, relationFilter);
  renderAco(data);
  renderCorrections(data);
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

function renderMode(data) {
  const meta = data.metadata || {};
  $('mode-panel').textContent = `${meta.provider || 'unknown'} / ${meta.model || 'unknown'} / ${meta.elapsed_ms || 0}ms / schema ${meta.schema_version || 'legacy'}`;
}

function renderDiagnostics(data) {
  const d = data.diagnostics || {};
  const counts = d.relation_counts || {};
  const countText = Object.entries(counts)
    .sort((a, b) => b[1] - a[1])
    .map(([k, v]) => `${k}:${v}`)
    .join('  ');
  const warnings = d.warnings || [];
  $('diagnostics-panel').innerHTML = `
    ${chip('non-trivial relations', d.non_trivial_relations || 0)}
    ${chip('overlap pairs', (d.dense_overlap_pairs || []).length)}
    ${chip('deadline commitments', d.deadline_commitments || 0)}
    ${chip('frictions', d.friction_count || 0)}
    ${chip('escalating friction', d.escalating_friction_count || 0)}
    ${chip('open-ended episodes', d.open_ended_episodes || 0)}
    ${chip('unresolved dates', d.unresolved_dates || 0)}
    <div class="diag-line"><strong>relations</strong><span>${escapeHtml(countText || 'none')}</span></div>
    <div class="diag-line"><strong>warnings</strong><span>${warnings.length ? escapeHtml(warnings.join(' | ')) : 'none'}</span></div>
  `;
}

function renderBrief(data) {
  const episodes = data.episodes || [];
  const commitments = data.commitments || [];
  const frictions = data.frictions || [];
  const dates = data.dates || [];
  const keyEpisodes = episodes.slice(0, 4).map((ep) => `${shortDate(ep.interval.from)}: ${ep.title}`);
  const unresolved = commitments.filter((c) => /proposed|announced|authorized|ordered|signed/i.test(c.state || '')).slice(0, 4);
  $('brief-panel').innerHTML = `
    <p><strong>${episodes.length}</strong> episodes convert the source into a computable chronology across <strong>${dates.length}</strong> detected temporal anchors.</p>
    <p class="text-zinc-400">${escapeHtml(keyEpisodes.join(' -> ') || 'No episode bands detected yet.')}</p>
    <p><strong>Commitment watch:</strong> ${escapeHtml(unresolved.map((c) => c.summary).join(' | ') || 'No active commitments detected.')}</p>
    <p><strong>Friction watch:</strong> ${escapeHtml(frictions.slice(0, 3).map((f) => f.summary).join(' | ') || 'No friction objects detected.')}</p>
  `;
}

function renderActorLanes(data) {
  const actors = data.actors || [];
  const events = data.events || [];
  const fallbackActors = actors.length ? actors : [{ id: 'source', name: 'Source chronology', role: 'aggregate' }];
  $('actor-lanes').innerHTML = fallbackActors.map((actor, idx) => {
    const related = events.filter((event) => (event.actor_ids || []).includes(actor.id));
    const laneEvents = related.length ? related : events.filter((_, i) => i % fallbackActors.length === idx).slice(0, 4);
    return `
      <div class="lane">
        <div class="lane-head">
          <strong>${escapeHtml(actor.name)}</strong>
          <span>${escapeHtml(actor.role || 'actor')}</span>
        </div>
        <div class="lane-events">
          ${laneEvents.map((event) => `<span title="${escapeHtml(event.canonical_name)}">${shortDate(event.at)}</span>`).join('') || '<em>no linked events</em>'}
        </div>
      </div>
    `;
  }).join('');
}

function renderFrictionMap(data) {
  const actorsById = Object.fromEntries((data.actors || []).map((actor) => [actor.id, actor.name]));
  const frictions = data.frictions || [];
  $('friction-map').innerHTML = frictions.map((friction) => {
    const actors = (friction.actors_involved || []).map((id) => actorsById[id] || id).join(' + ');
    const evidence = (friction.evidence_spans || []).map((span) => span.text).filter(Boolean).join(' | ');
    return `
      <div class="friction-card">
        <div class="friction-top">
          <strong>${escapeHtml(kindLabel(friction.kind))}</strong>
          <span>${escapeHtml(friction.trajectory || 'mutating')} / ${Math.round((friction.intensity || 0) * 100)}%</span>
        </div>
        <p>${escapeHtml(friction.summary)}</p>
        <small>${escapeHtml(actors || 'unlinked actors')}</small>
        ${evidence ? `<blockquote>${escapeHtml(evidence)}</blockquote>` : ''}
      </div>
    `;
  }).join('') || '<div class="text-sm text-zinc-500">No friction detected yet.</div>';
}

function renderRelations(data, filter = 'interesting') {
  const epById = Object.fromEntries((data.episodes || []).map((ep) => [ep.id, ep.title]));
  const tbody = $('relations-tbody');
  const filtered = filterRelations(data.relations || [], filter);
  tbody.innerHTML = filtered.map((r) => `
    <tr>
      <td class="py-2 pr-3 font-semibold text-zinc-100">${escapeHtml(epById[r.from_episode] || r.from_episode)}</td>
      <td class="py-2 pr-3 font-mono text-emerald-300">${escapeHtml(humanRel(r.relation))}</td>
      <td class="py-2 text-zinc-100">${escapeHtml(epById[r.to_episode] || r.to_episode)}</td>
    </tr>
  `).join('') || '<tr><td class="py-2 text-zinc-400">No relations in this filter.</td></tr>';
}

function kindLabel(kind) {
  return String(kind || 'custom').replaceAll('_', ' ');
}

function filterRelations(relations, filter) {
  if (filter === 'all') return relations;
  if (filter === 'overlap') return relations.filter((r) => ['Overlaps', 'OverlappedBy', 'Contains', 'During', 'Equals'].includes(r.relation));
  if (filter === 'boundary') return relations.filter((r) => ['Meets', 'MetBy', 'Starts', 'StartedBy', 'Finishes', 'FinishedBy'].includes(r.relation));
  return relations.filter((r) => !['Before', 'After'].includes(r.relation));
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

function renderCorrections(data) {
  const episodes = (data.episodes || []).slice(0, 8);
  $('corrections-panel').innerHTML = episodes.map((ep) => {
    const state = corrections[ep.id] || ep.review_state || 'proposed';
    return `
      <div class="correction-row">
        <span>${escapeHtml(ep.title)}</span>
        <div class="correction-actions">
          ${['approved', 'modified', 'rejected'].map((choice) => `<button class="${state === choice ? 'selected' : ''}" data-correction="${escapeHtml(ep.id)}" data-state="${choice}">${choice}</button>`).join('')}
        </div>
      </div>
    `;
  }).join('') || '<div class="text-zinc-500">Run analysis to review episodes.</div>';
  document.querySelectorAll('[data-correction]').forEach((btn) => {
    btn.addEventListener('click', () => {
      corrections[btn.dataset.correction] = btn.dataset.state;
      renderCorrections(lastAnalysis || data);
      $('raw-json').textContent = JSON.stringify(enrichedExport(), null, 2);
    });
  });
}

function enrichedExport() {
  return {
    ...lastAnalysis,
    analyst_corrections: corrections,
  };
}

function downloadJson(data) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `kairos-${data.session_id}.json`;
  a.click();
  URL.revokeObjectURL(a.href);
}

function chip(label, value) {
  return `<div class="diag-chip"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong></div>`;
}

document.querySelectorAll('.relation-filter').forEach((btn) => {
  btn.addEventListener('click', () => {
    relationFilter = btn.dataset.filter;
    document.querySelectorAll('.relation-filter').forEach((item) => item.classList.remove('active'));
    btn.classList.add('active');
    if (lastAnalysis) renderRelations(lastAnalysis, relationFilter);
  });
});

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
