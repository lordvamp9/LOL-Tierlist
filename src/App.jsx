import React, { useState, useEffect, useMemo, useCallback } from 'react';
import TitleBar from './components/TitleBar';
import RoleTabs from './components/RoleTabs';
import FilterControls from './components/FilterControls';
import ChampionCard from './components/ChampionCard';
import ChampionDetail from './components/ChampionDetail';
import { AlertCircle, Filter, Trophy, Sparkles } from 'lucide-react';
import './styles/classic-client.css';

// Bonus Tier lookup for the weighted formula
const TIER_BONUS = {
  'S+': 3.0,
  'S': 2.0,
  'A': 1.0,
  'B': 0.0,
};

export default function App() {
  const [metaData, setMetaData] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [server, setServer] = useState('LAS'); // Default: LAS
  const [eloTier, setEloTier] = useState('DIAMOND'); // Default: DIAMOND
  const [activeRole, setActiveRole] = useState('TOP'); // TOP, JUNGLE, MID, CARRY, SUPPORT
  const [minPickRate, setMinPickRate] = useState(3.5); // Default 3.5% as requested
  const [selectedChampion, setSelectedChampion] = useState(null);
  const [errorMessage, setErrorMessage] = useState(null);

  // Invoke backend command get_meta_data with server, tier, and forceRefresh
  const fetchMetaData = useCallback(
    async (forceRefresh = false) => {
      if (forceRefresh) {
        setIsRefreshing(true);
      } else {
        setIsLoading(true);
      }
      setErrorMessage(null);

      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const data = await invoke('get_meta_data', {
          server,
          tier: eloTier,
          forceRefresh,
        });
        setMetaData(data);
      } catch (err) {
        console.warn('Native Tauri invoke error or running in web preview:', err);
        setErrorMessage(
          typeof err === 'string'
            ? err
            : 'Iniciando en modo cliente web o cargando datos...'
        );
      } finally {
        setIsLoading(false);
        setIsRefreshing(false);
      }
    },
    [server, eloTier]
  );

  // Re-fetch automatically when server or eloTier changes
  useEffect(() => {
    fetchMetaData(false);
  }, [fetchMetaData]);

  // Filter & Sort Logic: Real Top 5 Anti-Niche
  const { top5Champs, excludedCount } = useMemo(() => {
    if (!metaData || !metaData.champions) {
      return { top5Champs: [], excludedCount: 0 };
    }

    // 1. Filter by current active role (CARRY matches both CARRY and BOT)
    const roleChamps = metaData.champions.filter((c) => {
      const cRole = (c.role || '').toUpperCase();
      const aRole = activeRole.toUpperCase();
      if (aRole === 'CARRY') {
        return cRole === 'CARRY' || cRole === 'BOT';
      }
      return cRole === aRole;
    });

    // 2. Identify excluded niche picks
    const qualified = [];
    let excluded = 0;

    for (const champ of roleChamps) {
      if (champ.pickRate >= minPickRate) {
        // Weighted Formula Score
        const bonus = TIER_BONUS[champ.tier] ?? 0;
        const clampedPr = Math.min(champ.pickRate, 15);
        const calculatedScore = Number(
          ((champ.winRate * 0.6) + (clampedPr * 0.4) + bonus).toFixed(2)
        );

        qualified.push({
          ...champ,
          score: calculatedScore,
        });
      } else {
        excluded++;
      }
    }

    // 3. Sort descending by Score
    qualified.sort((a, b) => b.score - a.score);

    // 4. Extract Top 5 Real Meta
    return {
      top5Champs: qualified.slice(0, 5),
      excludedCount: excluded,
    };
  }, [metaData, activeRole, minPickRate]);

  return (
    <div className="classic-window-shell">
      {/* 1. Custom Titlebar */}
      <TitleBar
        onRefresh={() => fetchMetaData(true)}
        isRefreshing={isRefreshing}
        patch={metaData?.patch}
      />

      {/* 2. Subheader: Role Navigation & Functional Filters */}
      <div className="classic-subbar">
        <RoleTabs activeRole={activeRole} onSelectRole={setActiveRole} />

        <FilterControls
          server={server}
          setServer={setServer}
          eloTier={eloTier}
          setEloTier={setEloTier}
          minPickRate={minPickRate}
          setMinPickRate={setMinPickRate}
          patch={metaData?.patch}
          isCached={metaData?.isCached}
        />
      </div>

      {/* 3. Main Content Area */}
      <main className="classic-content-area">
        <div className="content-header">
          <div className="content-title-box">
            <h2 className="content-main-title">
              <Trophy size={20} color="#ffdc73" />
              TOP 5 META REAL — {activeRole}
            </h2>
            <p className="content-subtitle">
              Calculado para el servidor <strong>{server}</strong> en rango <strong>{eloTier}</strong>{' '}
              descartando picks inflados por bajo pick rate.
            </p>
          </div>
        </div>

        {/* Loading State */}
        {isLoading && (
          <div className="retro-loader-box">
            <div className="spin-icon" style={{ display: 'inline-block', marginBottom: 12 }}>
              <Sparkles size={28} />
            </div>
            <p className="retro-loader-text">
              SINCRONIZANDO META DE {server} ({eloTier})...
            </p>
          </div>
        )}

        {/* Error / Fallback Alert if applicable */}
        {errorMessage && !isLoading && (
          <div className="excluded-notice-card" style={{ marginBottom: 16 }}>
            <AlertCircle size={16} />
            <span>{errorMessage}</span>
          </div>
        )}

        {/* Top 5 Champion Cards (Redesigned) */}
        {!isLoading && (
          <div className="champion-cards-stack">
            {top5Champs.map((champ, index) => (
              <ChampionCard
                key={champ.id}
                champ={champ}
                rank={index}
                onClick={setSelectedChampion}
              />
            ))}

            {top5Champs.length === 0 && (
              <div style={{ textAlign: 'center', padding: '40px', color: '#a09b8c' }}>
                <p>Ningún campeón cumple con el Pick Rate mínimo seleccionado ({minPickRate}%).</p>
                <p style={{ fontSize: 12, marginTop: 4 }}>
                  Intenta reducir el umbral del filtro Anti-Niche Picks.
                </p>
              </div>
            )}

            {/* Anti-Niche Excluded Summary Banner */}
            {excludedCount > 0 && (
              <div className="excluded-notice-card">
                <Filter size={14} color="#ff8a94" />
                <span>
                  <strong>Filtro Anti-Niche Activo:</strong> Se descartaron{' '}
                  <strong>{excludedCount}</strong> selecciones infladas con menos de{' '}
                  <strong>{minPickRate.toFixed(1)}%</strong> de Pick Rate en {activeRole}.
                </span>
              </div>
            )}
          </div>
        )}
      </main>

      {/* 4. Champion Detail Modal (Runes, Core Build, Skill Order) */}
      {selectedChampion && (
        <ChampionDetail
          champ={selectedChampion}
          onClose={() => setSelectedChampion(null)}
        />
      )}
    </div>
  );
}
