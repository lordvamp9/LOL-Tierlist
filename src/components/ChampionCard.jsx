import React from 'react';
import { ChevronRight } from 'lucide-react';

export default function ChampionCard({ champ, rank, onClick }) {
  const rankNumber = rank + 1;
  const isGoldBronzeAccent = rankNumber === 1 || rankNumber === 3;
  const tierClass = champ.tier === 'S+' ? 'tier-S-plus' : `tier-${champ.tier}`;

  return (
    <div
      className="champion-card-row"
      onClick={() => onClick(champ)}
      title="Haz clic para ver Runas completas, Core Build y Orden de Habilidades"
    >
      {/* 1. Rectangular Beveled Metal Rank Box (#1 to #5) */}
      <div className={`rank-box-metal ${isGoldBronzeAccent ? 'accent-gold-bronze' : 'accent-silver-dark'}`}>
        <span>#{rankNumber}</span>
      </div>

      {/* 2. Square Avatar with Dark Beveled Border & Tier Badge */}
      <div className="avatar-square-container">
        <img
          src={champ.iconUrl}
          alt={champ.name}
          className="avatar-square-img"
          loading="lazy"
        />
        <span className={`avatar-tier-badge ${tierClass}`}>{champ.tier}</span>
      </div>

      {/* 3. Champion Identity: Name & Italicized Subtitle */}
      <div className="identity-block">
        <h3 className="champ-title-name">{champ.name}</h3>
        <p className="champ-subtitle-italic">{champ.title}</p>
      </div>

      {/* 4. Vertical Metrics on the Right (WR & PR) */}
      <div className="metrics-column-right">
        <div className="metric-line-wr">
          <span className="metric-val-cyan">{champ.winRate.toFixed(2)}%</span>
          <span className="metric-label-tag">WR</span>
        </div>
        <div className="metric-line-pr">
          <span className="metric-val-gray">{champ.pickRate.toFixed(1)}%</span>
          <span className="metric-label-tag">PR</span>
        </div>
      </div>

      {/* 5. Discreet Chevron Arrow */}
      <div className="expand-arrow-slot">
        <ChevronRight size={18} className="chevron-discreet" />
      </div>
    </div>
  );
}
