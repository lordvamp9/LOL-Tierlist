import React from 'react';
import ClassicDropdown from './ClassicDropdown';

const SERVERS = ['LAS', 'LAN', 'NA', 'EUW', 'EUNE', 'KR', 'BR'];

const ELO_TIERS = [
  { id: 'IRON', label: 'Hierro', iconName: 'iron' },
  { id: 'BRONZE', label: 'Bronce', iconName: 'bronze' },
  { id: 'SILVER', label: 'Plata', iconName: 'silver' },
  { id: 'GOLD', label: 'Oro', iconName: 'gold' },
  { id: 'PLATINUM', label: 'Platino', iconName: 'platinum' },
  { id: 'EMERALD', label: 'Esmeralda', iconName: 'emerald' },
  { id: 'DIAMOND', label: 'Diamante', iconName: 'diamond' },
  { id: 'MASTER', label: 'Maestro', iconName: 'master' },
  { id: 'GRANDMASTER', label: 'Gran Maestro', iconName: 'grandmaster' },
  { id: 'CHALLENGER', label: 'Retador', iconName: 'challenger' },
];

export default function FilterControls({
  server,
  setServer,
  eloTier,
  setEloTier,
  minPickRate,
  setMinPickRate,
  patch,
  isCached,
}) {
  const getRankIconUrl = (iconName) =>
    `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/images/ranked-mini-crests/${iconName}.png`;

  return (
    <div className="filter-controls-bar">
      {/* 1. Server / Region Selector */}
      <ClassicDropdown
        label="SERVIDOR:"
        options={SERVERS}
        value={server}
        onChange={setServer}
        width={95}
        renderSelected={(s) => <span className="server-tag-selected">{s}</span>}
        renderOption={(s) => <span className="server-option-text">{s}</span>}
      />

      {/* 2. Elo / Tier Selector with Official Emblems */}
      <ClassicDropdown
        label="RANGO / ELO:"
        options={ELO_TIERS}
        value={eloTier}
        onChange={setEloTier}
        width={160}
        renderSelected={(t) => (
          <div className="elo-selected-item">
            <img
              src={getRankIconUrl(t.iconName)}
              alt={t.label}
              className="elo-mini-emblem"
              onError={(e) => {
                e.target.src = `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/images/ranked-emblem/emblem-${t.iconName}.png`;
              }}
            />
            <span className="elo-label-text">{t.label.toUpperCase()}</span>
          </div>
        )}
        renderOption={(t) => (
          <div className="elo-dropdown-item">
            <img
              src={getRankIconUrl(t.iconName)}
              alt={t.label}
              className="elo-mini-emblem"
              onError={(e) => {
                e.target.src = `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/images/ranked-emblem/emblem-${t.iconName}.png`;
              }}
            />
            <span className="elo-option-title">{t.label}</span>
          </div>
        )}
      />

      {/* 3. Pick Rate Slider (Anti-Niche Picks) */}
      <div
        className="slider-group"
        title="Filtro Anti-Niche Picks: Descarta campeones inflados con pocas partidas pero alto WinRate"
      >
        <div className="slider-label-row">
          <span className="slider-title">PICK RATE MÍNIMO:</span>
          <span className="slider-value">{minPickRate.toFixed(1)}%</span>
        </div>
        <input
          type="range"
          min="1.0"
          max="10.0"
          step="0.1"
          value={minPickRate}
          onChange={(e) => setMinPickRate(parseFloat(e.target.value))}
          className="classic-slider"
        />
      </div>

      {/* 4. Meta Status Badges */}
      <div className="meta-status-badges">
        <div className="badge-patch" title="Parche oficial sincronizado desde Riot Data Dragon">
          PARCHE {patch || '14.24.1'}
        </div>
        <div
          className={`badge-cache ${isCached ? 'cached' : 'live'}`}
          title={
            isCached
              ? `Caché Local Offline específica (cache_${server.toLowerCase()}_${eloTier.toLowerCase()}.json, TTL: 12 Horas)`
              : 'Datos descargados y procesados para esta región y elo'
          }
        >
          {isCached ? 'CACHÉ (12h)' : 'EN VIVO'}
        </div>
      </div>
    </div>
  );
}
