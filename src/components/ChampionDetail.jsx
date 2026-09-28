import React from 'react';
import { X, Zap, Shield, Sparkles, Sword, Crosshair } from 'lucide-react';

export default function ChampionDetail({ champ, onClose }) {
  if (!champ) return null;

  const tierClass = champ.tier === 'S+' ? 'tier-S-plus' : `tier-${champ.tier}`;

  return (
    <div className="champion-detail-overlay" onClick={onClose}>
      <div className="champion-detail-modal" onClick={(e) => e.stopPropagation()}>
        {/* Splash Art Header with Viñette */}
        <div className="detail-splash-header">
          <img
            src={champ.splashUrl}
            alt={champ.name}
            className="detail-splash-img"
          />
          <div className="detail-splash-vignette" />

          <div className="detail-header-content">
            <div className="detail-header-left">
              <img
                src={champ.iconUrl}
                alt={champ.name}
                className="detail-avatar-large"
              />
              <div className="detail-title-group">
                <h2>{champ.name}</h2>
                <p>{champ.title}</p>
              </div>
              <span className={`tier-badge ${tierClass}`} style={{ position: 'static', padding: '4px 10px', fontSize: 13 }}>
                Tier {champ.tier}
              </span>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
              <div className="score-badge-box" style={{ padding: '6px 16px' }}>
                <span className="score-num" style={{ fontSize: 20 }}>{champ.score.toFixed(1)}</span>
                <span className="score-tag">META SCORE</span>
              </div>
              <button className="btn-close-detail" onClick={onClose}>
                VOLVER «
              </button>
            </div>
          </div>
        </div>

        {/* Modal Content Grid */}
        <div className="detail-body-grid">
          {/* LEFT COLUMN: RUNES & SPELLS */}
          <div className="detail-section-card">
            <h3 className="section-box-title">
              <Zap size={15} /> ÁRBOL DE RUNAS ÓPTIMO
            </h3>

            <div className="runes-display-wrapper">
              {/* Primary Path */}
              <div className="rune-tree-row">
                <div className="keystone-gem-slot">
                  <img
                    src={champ.runes?.keystoneIcon}
                    alt={champ.runes?.keystoneName}
                    className="keystone-gem-img"
                  />
                  <span className="keystone-name">{champ.runes?.keystoneName}</span>
                </div>

                <div className="minor-runes-group">
                  {champ.runes?.primaryRunes?.map((r) => (
                    <div key={r.id} className="minor-rune-item" title={r.name}>
                      <img src={r.iconUrl} alt={r.name} className="minor-rune-img" />
                      <span className="minor-rune-name">{r.name}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Secondary Path */}
              <div className="rune-tree-row">
                <div style={{ minWidth: 64, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4 }}>
                  <img
                    src={champ.runes?.secondaryStyleIcon}
                    alt={champ.runes?.secondaryStyleName}
                    style={{ width: 34, height: 34 }}
                  />
                  <span style={{ fontSize: 10, color: '#c8aa6e', fontWeight: 700 }}>
                    {champ.runes?.secondaryStyleName}
                  </span>
                </div>

                <div className="minor-runes-group">
                  {champ.runes?.secondaryRunes?.map((r) => (
                    <div key={r.id} className="minor-rune-item" title={r.name}>
                      <img src={r.iconUrl} alt={r.name} className="minor-rune-img" />
                      <span className="minor-rune-name">{r.name}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Adaptive Shards */}
              {champ.runes?.shards && (
                <div className="shards-row">
                  <Sparkles size={14} />
                  <span>Fragmentos: {champ.runes.shards.join('  •  ')}</span>
                </div>
              )}

              {/* Summoner Spells */}
              <div style={{ marginTop: 12 }}>
                <span className="item-tier-label" style={{ display: 'block', marginBottom: 8 }}>
                  HECHIZOS DE INVOCADOR
                </span>
                <div className="spells-row">
                  {champ.summonerSpells?.map((spell) => (
                    <div key={spell.id} className="spell-mini-box">
                      <img src={spell.iconUrl} alt={spell.name} className="spell-img" />
                      <span style={{ fontSize: 11, color: '#f0e6d2' }}>{spell.name}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* RIGHT COLUMN: CORE BUILD & ABILITY ORDER */}
          <div className="detail-section-card">
            <h3 className="section-box-title">
              <Sword size={15} /> CORE BUILD & SECUENCIA
            </h3>

            <div className="items-display-wrapper">
              {/* Starting items */}
              <div className="item-tier-group">
                <span className="item-tier-label">OBJETOS INICIALES</span>
                <div className="item-icons-row">
                  {champ.build?.startingItems?.map((it) => (
                    <div key={it.id} className="item-card-mini" title={`${it.name} (${it.cost}g)`}>
                      <img src={it.iconUrl} alt={it.name} className="item-img" />
                      <div className="item-meta">
                        <span className="item-name">{it.name}</span>
                        <span className="item-cost">{it.cost}g</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Priority Boots */}
              {champ.build?.boots && (
                <div className="item-tier-group">
                  <span className="item-tier-label">BOTAS PRIORITARIAS</span>
                  <div className="item-icons-row">
                    <div className="item-card-mini" title={`${champ.build.boots.name} (${champ.build.boots.cost}g)`}>
                      <img src={champ.build.boots.iconUrl} alt={champ.build.boots.name} className="item-img" />
                      <div className="item-meta">
                        <span className="item-name">{champ.build.boots.name}</span>
                        <span className="item-cost">{champ.build.boots.cost}g</span>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* 3 Core Items sequence */}
              <div className="item-tier-group">
                <span className="item-tier-label">SECUENCIA CORE BUILD (3 ÍTEMS)</span>
                <div className="item-icons-row">
                  {champ.build?.coreItems?.map((it, idx) => (
                    <div key={it.id} className="item-card-mini" title={`Ítem ${idx + 1}: ${it.name} (${it.cost}g)`}>
                      <img src={it.iconUrl} alt={it.name} className="item-img" />
                      <div className="item-meta">
                        <span className="item-name">#{idx + 1} {it.name}</span>
                        <span className="item-cost">{it.cost}g</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Ability Max Order */}
              <div className="item-tier-group" style={{ marginTop: 8 }}>
                <span className="item-tier-label">MAXIMIZACIÓN DE HABILIDADES</span>
                <div className="skills-order-row">
                  {champ.skillOrder?.map((skill, idx) => (
                    <React.Fragment key={idx}>
                      <div className="skill-badge">{skill}</div>
                      {idx < champ.skillOrder.length - 1 && <span className="skill-arrow">›</span>}
                    </React.Fragment>
                  ))}
                  <span style={{ fontSize: 11, color: '#a09b8c', marginLeft: 10 }}>
                    (Prioridad principal de maxeo)
                  </span>
                </div>
              </div>

              {/* Performance stats summary */}
              <div style={{ marginTop: 10, display: 'flex', gap: 14, background: 'rgba(5, 12, 18, 0.6)', padding: 10, border: '1px solid #463714' }}>
                <div style={{ flex: 1, textAlign: 'center' }}>
                  <div style={{ fontSize: 10, color: '#785a28', fontWeight: 'bold' }}>WIN RATE</div>
                  <div style={{ fontSize: 15, color: '#4ade80', fontWeight: 'bold', fontFamily: 'Cinzel, serif' }}>{champ.winRate.toFixed(2)}%</div>
                </div>
                <div style={{ flex: 1, textAlign: 'center' }}>
                  <div style={{ fontSize: 10, color: '#785a28', fontWeight: 'bold' }}>PICK RATE</div>
                  <div style={{ fontSize: 15, color: '#38bdf8', fontWeight: 'bold', fontFamily: 'Cinzel, serif' }}>{champ.pickRate.toFixed(1)}%</div>
                </div>
                <div style={{ flex: 1, textAlign: 'center' }}>
                  <div style={{ fontSize: 10, color: '#785a28', fontWeight: 'bold' }}>BAN RATE</div>
                  <div style={{ fontSize: 15, color: '#f87171', fontWeight: 'bold', fontFamily: 'Cinzel, serif' }}>{champ.banRate.toFixed(1)}%</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
