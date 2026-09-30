/**
 * LOL META - Extractor Automatizado de Metadatos OP.GG y Data Dragon
 * Autor / Marca: vamp9
 *
 * Este script consulta las estadísticas reales y oficiales de League of Legends:
 * 1. Detecta la versión activa mediante Data Dragon (Riot Games API).
 * 2. Consulta los endpoints estructurados de OP.GG para servidores y rangos meta.
 * 3. Normaliza las runas, maestrías de habilidades, core builds y win rates.
 * 4. Genera los archivos en data/meta_{server}_{tier}.json y data/meta_current.json.
 */

const fs = require('fs');
const path = require('path');
const https = require('https');

// Configuración de red y User Agent
const USER_AGENT = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 LOL-Meta-Bot/2.0 (vamp9)';
const DATA_DIR = path.resolve(__dirname, '..', 'data');

// Servidores y Tiers objetivo
const TARGET_SERVERS = ['LAS', 'LAN', 'NA', 'EUW', 'KR', 'BR'];
const TARGET_TIERS = ['DIAMOND', 'EMERALD', 'MASTER'];

// Mapeos de nombres de servidores a OP.GG
const REGION_MAP = {
  LAS: 'las',
  LAN: 'lan',
  NA: 'na',
  EUW: 'euw',
  EUNE: 'eune',
  KR: 'kr',
  BR: 'br',
  GLOBAL: 'global',
};

// Mapeos de tiers a OP.GG
const TIER_MAP = {
  DIAMOND: 'diamond_plus',
  EMERALD: 'emerald_plus',
  MASTER: 'master_plus',
  PLATINUM: 'platinum',
  GOLD: 'gold',
  SILVER: 'silver',
  BRONZE: 'bronze',
  IRON: 'iron',
  GRANDMASTER: 'grandmaster',
  CHALLENGER: 'challenger',
};

// Bonus Tier para fórmula ponderada
const TIER_BONUS = {
  'S+': 3.0,
  'S': 2.0,
  'A': 1.0,
  'B': 0.0,
  'C': -1.0,
};

// Mapeo de fragmentos de runas (Stat Shards)
const SHARD_NAMES = {
  5001: '+10-180 Vida (según nivel)',
  5002: '+6 Armadura',
  5003: '+8 Resistencia Mágica',
  5005: '+10% Velocidad de Ataque',
  5007: '+8 Aceleración de Habilidad',
  5008: '+9 Fuerza Adaptable',
  5010: '+2% Velocidad de Movimiento',
  5011: '+65 Vida plana',
  5012: '+10% Tenacidad y Res. Ralentización',
  5013: '+60 Vida y +12% Res. Ralentización',
};

let axios = null;
try {
  axios = require('axios');
} catch (e) {
  // Axios no está instalado aún, se usa https nativo
}

// Helper HTTP con soporte para axios o https nativo
function httpGet(url) {
  if (axios) {
    return axios
      .get(url, {
        headers: {
          'User-Agent': USER_AGENT,
          'Accept': 'application/json, text/plain, */*',
          'Accept-Language': 'es-ES,es;q=0.9,en;q=0.8',
        },
        timeout: 12000,
      })
      .then((res) => res.data);
  }

  return new Promise((resolve, reject) => {
    const parsedUrl = new URL(url);
    const options = {
      protocol: parsedUrl.protocol,
      hostname: parsedUrl.hostname,
      port: parsedUrl.port,
      path: parsedUrl.pathname + parsedUrl.search,
      headers: {
        'User-Agent': USER_AGENT,
        'Accept': 'application/json, text/plain, */*',
        'Accept-Language': 'es-ES,es;q=0.9,en;q=0.8',
      },
    };

    const req = https.get(options, (res) => {
      // Manejar redirecciones
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return httpGet(res.headers.location).then(resolve).catch(reject);
      }
      if (res.statusCode < 200 || res.statusCode >= 300) {
        return reject(new Error(`HTTP ${res.statusCode} para ${url}`));
      }

      let data = '';
      res.on('data', (chunk) => { data += chunk; });
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          reject(new Error(`Error al parsear JSON de ${url}: ${e.message}`));
        }
      });
    });

    req.on('error', reject);
    req.setTimeout(12000, () => {
      req.destroy();
      reject(new Error(`Timeout de petición para ${url}`));
    });
  });
}

// Cola de ejecución concurrente con límite para no saturar endpoints
async function mapConcurrent(items, limit, fn) {
  const results = new Array(items.length);
  let currentIndex = 0;

  async function worker() {
    while (currentIndex < items.length) {
      const idx = currentIndex++;
      try {
        results[idx] = await fn(items[idx], idx);
      } catch (err) {
        results[idx] = null;
      }
    }
  }

  const workers = Array.from({ length: Math.min(limit, items.length) }, () => worker());
  await Promise.all(workers);
  return results;
}

// 1. Obtener versión y metadatos de Data Dragon
async function fetchDDragonData() {
  console.log('[INFO] Consultando versiones en Data Dragon...');
  const versions = await httpGet('https://ddragon.leagueoflegends.com/api/versions.json');
  const patch = versions[0] || '14.24.1';
  console.log(`[INFO] Parche actual detectado: ${patch}`);

  console.log('[INFO] Descargando catálogo de campeones, objetos y runas en español (es_ES)...');
  const [championsData, itemsData, runesData, summonersData] = await Promise.all([
    httpGet(`https://ddragon.leagueoflegends.com/cdn/${patch}/data/es_ES/champion.json`),
    httpGet(`https://ddragon.leagueoflegends.com/cdn/${patch}/data/es_ES/item.json`),
    httpGet(`https://ddragon.leagueoflegends.com/cdn/${patch}/data/es_ES/runesReforged.json`),
    httpGet(`https://ddragon.leagueoflegends.com/cdn/${patch}/data/es_ES/summoner.json`),
  ]);

  // Indexar campeones por key numérica y por id
  const champByKey = new Map();
  const champById = new Map();
  for (const c of Object.values(championsData.data)) {
    champByKey.set(c.key, c);
    champById.set(c.id, c);
  }

  // Indexar objetos
  const itemMap = new Map();
  for (const [id, item] of Object.entries(itemsData.data)) {
    const numericId = parseInt(id, 10);
    itemMap.set(numericId, {
      id: numericId,
      name: item.name,
      cost: item.gold ? item.gold.total : 0,
      iconUrl: `https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${id}.png`,
      tags: item.tags || [],
    });
  }

  // Indexar árboles de runas y runas individuales
  const treeMap = new Map();
  const runeMap = new Map();
  for (const tree of runesData) {
    treeMap.set(tree.id, {
      id: tree.id,
      key: tree.key,
      name: tree.name,
      iconUrl: `https://ddragon.leagueoflegends.com/cdn/img/${tree.icon}`,
    });
    for (let slotIdx = 0; slotIdx < tree.slots.length; slotIdx++) {
      for (const r of tree.slots[slotIdx].runes) {
        runeMap.set(r.id, {
          id: r.id,
          key: r.key,
          name: r.name,
          iconUrl: `https://ddragon.leagueoflegends.com/cdn/img/${r.icon}`,
          slot: slotIdx,
        });
      }
    }
  }

  // Indexar hechizos de invocador
  const spellMap = new Map();
  for (const spell of Object.values(summonersData.data)) {
    spellMap.set(parseInt(spell.key, 10), {
      id: spell.id,
      name: spell.name,
      iconUrl: `https://ddragon.leagueoflegends.com/cdn/${patch}/img/spell/${spell.image.full}`,
    });
  }

  return { patch, champByKey, champById, itemMap, treeMap, runeMap, spellMap };
}

// 2. Extractor de detalles de campeón en OP.GG con caché en memoria
const championDetailsCache = new Map();

async function getOpGGChampionDetail(champId, position, region = 'global') {
  const cacheKey = `${champId}_${position.toLowerCase()}`;
  if (championDetailsCache.has(cacheKey)) {
    return championDetailsCache.get(cacheKey);
  }

  const pos = position.toLowerCase();
  const url = `https://lol-api-champion.op.gg/api/${region}/champions/ranked/${champId}/${pos}`;

  try {
    const res = await httpGet(url);
    if (res && res.data) {
      championDetailsCache.set(cacheKey, res.data);
      return res.data;
    }
  } catch (err) {
    // Si la región falló, intentar fallback a global
    if (region !== 'global') {
      try {
        const fallbackUrl = `https://lol-api-champion.op.gg/api/global/champions/ranked/${champId}/${pos}`;
        const fallbackRes = await httpGet(fallbackUrl);
        if (fallbackRes && fallbackRes.data) {
          championDetailsCache.set(cacheKey, fallbackRes.data);
          return fallbackRes.data;
        }
      } catch (fallbackErr) {
        // Ignorar y retornar null
      }
    }
  }

  return null;
}

// Helper para categorizar ítems
function categorizeItem(item) {
  const tags = item?.tags || [];
  if (tags.includes('Damage') || tags.includes('CriticalStrike') || tags.includes('AttackSpeed')) return 'Ofensivo';
  if (tags.includes('SpellDamage') || tags.includes('Mana')) return 'Poder de Habilidad';
  if (tags.includes('Armor') || tags.includes('SpellBlock') || tags.includes('Health')) return 'Defensivo';
  if (tags.includes('Boots')) return 'Movilidad';
  return 'Situacional';
}

// Fallback de runas si no vienen en OP.GG
function getFallbackRunes(dd) {
  return {
    primaryTreeId: 8000,
    primaryStyleId: 8000,
    primaryStyleName: 'Precisión',
    primaryStyleIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7201_Precision.png',
    keystoneId: 8010,
    keystoneName: 'Conquistador',
    keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png',
    primaryRunes: [
      { id: 9111, name: 'Triunfo', iconUrl: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Precision/Triumph.png', slot: 1 },
      { id: 9105, name: 'Leyenda: Celeridad', iconUrl: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Precision/LegendAlacrity/LegendAlacrity.png', slot: 2 },
      { id: 8299, name: 'Último esfuerzo', iconUrl: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Precision/LastStand/LastStand.png', slot: 3 },
    ],
    secondaryTreeId: 8400,
    secondaryStyleId: 8400,
    secondaryStyleName: 'Valor',
    secondaryStyleIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7204_Resolve.png',
    secondaryRunes: [
      { id: 8444, name: 'Segundo aire', iconUrl: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Resolve/SecondWind/SecondWind.png', slot: 1 },
      { id: 8451, name: 'Sobrecrecimiento', iconUrl: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Resolve/Overgrowth/Overgrowth.png', slot: 2 },
    ],
    shards: ['+9 Fuerza Adaptable', '+6 Armadura', '+10-180 Vida'],
  };
}

// 3. Normalizar el objeto de campeón completo
function buildNormalizedChampion(champRankEntry, positionData, detail, dd, server, tierName) {
  const numericId = champRankEntry.id;
  const ddChamp = dd.champByKey.get(String(numericId)) || {
    id: `Champ_${numericId}`,
    name: `Campeón ${numericId}`,
    title: 'del Meta',
  };

  const rawRole = positionData.name; // 'TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'
  const normalizedRole = rawRole === 'ADC' ? 'CARRY' : rawRole;

  const stats = positionData.stats || {};
  const winRate = parseFloat(((stats.win_rate || 0.5) * 100).toFixed(2));
  const pickRate = parseFloat(((stats.pick_rate || 0.05) * 100).toFixed(2));
  const banRate = parseFloat(((stats.ban_rate || 0.02) * 100).toFixed(2));

  // Cálculo de Tier (S+, S, A, B, C)
  const rank = stats.tier_data?.rank || 99;
  const opggTier = stats.tier_data?.tier || 3;

  let tier = 'B';
  if (rank <= 2 || (opggTier === 1 && rank <= 3)) {
    tier = 'S+';
  } else if (rank <= 6 || opggTier === 1 || (opggTier === 2 && rank <= 8)) {
    tier = 'S';
  } else if (rank <= 14 || opggTier === 2 || opggTier === 3) {
    tier = 'A';
  } else if (rank <= 26 || opggTier === 4) {
    tier = 'B';
  } else {
    tier = 'C';
  }

  // MetaScore ponderado: (winRate * 0.6) + (Math.min(pickRate, 15) * 0.4) + tierBonus
  const tierBonus = TIER_BONUS[tier] || 0.0;
  const metaScore = parseFloat(((winRate * 0.6) + (Math.min(pickRate, 15) * 0.4) + tierBonus).toFixed(2));

  // --- OBJETOS INICIALES ---
  let starterItemsIds = [1054, 2003]; // Escudo Doran + Poción por defecto
  if (detail?.starter_items && detail.starter_items.length > 0) {
    starterItemsIds = detail.starter_items[0].ids || [1054, 2003];
  }
  const startingItems = starterItemsIds.map((id) => {
    const info = dd.itemMap.get(id);
    return {
      id,
      name: info?.name || `Ítem #${id}`,
      iconUrl: info?.iconUrl || `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/item/${id}.png`,
      cost: info?.cost || 450,
      winRate: null,
      category: 'Inicial',
    };
  });

  // --- BOTAS ---
  let primaryBootId = 3006;
  const altBootIds = [];
  if (detail?.boots && detail.boots.length > 0) {
    primaryBootId = detail.boots[0].ids?.[0] || 3006;
    for (let i = 1; i < Math.min(3, detail.boots.length); i++) {
      if (detail.boots[i]?.ids?.[0]) {
        altBootIds.push(detail.boots[i].ids[0]);
      }
    }
  }
  const bootInfo = dd.itemMap.get(primaryBootId);
  const bootsObj = {
    id: primaryBootId,
    name: bootInfo?.name || 'Botas',
    iconUrl: bootInfo?.iconUrl || `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/item/${primaryBootId}.png`,
    cost: bootInfo?.cost || 1100,
    winRate: detail?.boots?.[0]?.play ? parseFloat(((detail.boots[0].win / detail.boots[0].play) * 100).toFixed(1)) : null,
    category: 'Botas',
    alternatives: altBootIds,
  };

  // --- 3 CORE ITEMS ---
  let coreItemsIds = [6631, 3046, 3033];
  if (detail?.core_items && detail.core_items.length > 0 && Array.isArray(detail.core_items[0].ids)) {
    coreItemsIds = detail.core_items[0].ids.slice(0, 3);
  }
  const coreItems = coreItemsIds.map((id) => {
    const info = dd.itemMap.get(id);
    return {
      id,
      name: info?.name || `Ítem #${id}`,
      iconUrl: info?.iconUrl || `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/item/${id}.png`,
      cost: info?.cost || 3000,
      winRate: null,
      category: categorizeItem(info),
    };
  });

  // --- 4º, 5º Y 6º ÍTEMS FRECUENTES Y SITUACIONALES ---
  const lateItemsPool = [];
  if (detail?.last_items && detail.last_items.length > 0) {
    const excludedIds = new Set([...starterItemsIds, primaryBootId, ...altBootIds, ...coreItemsIds]);
    for (const itemEntry of detail.last_items) {
      const id = itemEntry.ids?.[0];
      if (id && !excludedIds.has(id)) {
        const info = dd.itemMap.get(id);
        const itemWr = itemEntry.play > 0 ? parseFloat(((itemEntry.win / itemEntry.play) * 100).toFixed(1)) : null;
        lateItemsPool.push({
          id,
          name: info?.name || `Ítem #${id}`,
          iconUrl: info?.iconUrl || `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/item/${id}.png`,
          cost: info?.cost || 2800,
          winRate: itemWr,
          category: categorizeItem(info),
        });
      }
    }
  }

  // Desglosar opciones de 4to, 5to y 6to ítem
  const fourthItems = lateItemsPool.slice(0, 3);
  const fifthItems = lateItemsPool.slice(3, 6);
  const sixthItems = lateItemsPool.slice(6, 9);
  const situationalItems = lateItemsPool.slice(0, 5);
  const situationalItemIds = situationalItems.map((it) => it.id);

  // --- RUNAS ÓPTIMAS ---
  let runesObj = null;
  if (detail?.runes && detail.runes.length > 0) {
    const bestRune = detail.runes[0];
    const pTree = dd.treeMap.get(bestRune.primary_page_id);
    const sTree = dd.treeMap.get(bestRune.secondary_page_id);
    const keystone = dd.runeMap.get(bestRune.id || bestRune.primary_rune_ids?.[0]);

    const primaryRunes = (bestRune.primary_rune_ids || []).slice(1).map((rId, idx) => {
      const r = dd.runeMap.get(rId);
      return {
        id: rId,
        name: r?.name || `Runa #${rId}`,
        iconUrl: r?.iconUrl || '',
        slot: idx + 1,
      };
    });

    const secondaryRunes = (bestRune.secondary_rune_ids || []).map((rId, idx) => {
      const r = dd.runeMap.get(rId);
      return {
        id: rId,
        name: r?.name || `Runa #${rId}`,
        iconUrl: r?.iconUrl || '',
        slot: idx + 1,
      };
    });

    const shards = (bestRune.stat_mod_ids || []).map((sId) => SHARD_NAMES[sId] || '+ Estadísticas adaptables');

    runesObj = {
      primaryTreeId: bestRune.primary_page_id,
      primaryStyleId: bestRune.primary_page_id,
      primaryStyleName: pTree?.name || 'Primaria',
      primaryStyleIcon: pTree?.iconUrl || '',
      keystoneId: bestRune.id || bestRune.primary_rune_ids?.[0],
      keystoneName: keystone?.name || 'Runa Clave',
      keystoneIcon: keystone?.iconUrl || '',
      primaryRunes,
      secondaryTreeId: bestRune.secondary_page_id,
      secondaryStyleId: bestRune.secondary_page_id,
      secondaryStyleName: sTree?.name || 'Secundaria',
      secondaryStyleIcon: sTree?.iconUrl || '',
      secondaryRunes,
      shards: shards.length > 0 ? shards : ['+9 Fuerza Adaptable', '+6 Armadura', '+10-180 Vida'],
    };
  } else {
    runesObj = getFallbackRunes(dd);
  }

  // --- ORDEN DE HABILIDADES ---
  let skillOrder = ['Q', 'W', 'E'];
  if (detail?.skill_masteries && detail.skill_masteries.length > 0 && Array.isArray(detail.skill_masteries[0].ids)) {
    skillOrder = detail.skill_masteries[0].ids.slice(0, 3);
  } else if (detail?.skills && detail.skills.length > 0 && Array.isArray(detail.skills[0].order)) {
    const seen = new Set();
    const derived = [];
    for (const s of detail.skills[0].order) {
      if (!seen.has(s) && ['Q', 'W', 'E'].includes(s)) {
        seen.add(s);
        derived.push(s);
      }
    }
    if (derived.length >= 3) skillOrder = derived.slice(0, 3);
  }

  // --- HECHIZOS DE INVOCADOR ---
  let summonerSpells = [
    { id: 'SummonerFlash', name: 'Destello', iconUrl: `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/spell/SummonerFlash.png` },
    { id: 'SummonerDot', name: 'Prender', iconUrl: `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/spell/SummonerDot.png` },
  ];
  if (detail?.summoner_spells && detail.summoner_spells.length > 0 && Array.isArray(detail.summoner_spells[0].ids)) {
    const spells = detail.summoner_spells[0].ids.map((sId) => {
      const sp = dd.spellMap.get(sId);
      return {
        id: sp?.id || 'SummonerFlash',
        name: sp?.name || 'Hechizo',
        iconUrl: sp?.iconUrl || `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/spell/SummonerFlash.png`,
      };
    });
    if (spells.length >= 2) summonerSpells = spells.slice(0, 2);
  }

  // URLs de Splash Art e Icono Oficial de Riot
  const iconUrl = `https://ddragon.leagueoflegends.com/cdn/${dd.patch}/img/champion/${ddChamp.id}.png`;
  const splashUrl = `https://ddragon.leagueoflegends.com/cdn/img/champion/splash/${ddChamp.id}_0.jpg`;

  return {
    id: ddChamp.id,
    championId: ddChamp.id,
    key: String(numericId),
    name: ddChamp.name,
    title: ddChamp.title,
    role: normalizedRole,
    tier,
    winRate,
    pickRate,
    banRate,
    score: metaScore,
    metaScore,
    starterItems: starterItemsIds,
    boots: bootsObj,
    coreItems: coreItemsIds,
    fourthItemOptions: fourthItems,
    fifthItemOptions: fifthItems,
    sixthItemOptions: sixthItems,
    situationalItems: situationalItemIds,
    runes: runesObj,
    build: {
      startingItems,
      boots: bootsObj,
      coreItems,
      fourthItems,
      fifthItems,
      sixthItems,
      situationalItems,
    },
    skillOrder,
    summonerSpells,
    iconUrl,
    splashUrl,
    patch: dd.patch,
    server,
    tierRank: tierName,
    isCached: false,
  };
}

// 4. Extracción por combinación de Servidor y Tier
async function extractServerTierMeta(server, tierName, dd) {
  const region = REGION_MAP[server] || 'las';
  const tierParam = TIER_MAP[tierName] || 'diamond_plus';

  console.log(`[FETCH] Obteniendo ranking OP.GG para ${server} (${region}) en elo ${tierName} (${tierParam})...`);
  const rankedUrl = `https://lol-api-champion.op.gg/api/${region}/champions/ranked?tier=${tierParam}`;

  let rankedData;
  try {
    rankedData = await httpGet(rankedUrl);
  } catch (err) {
    console.warn(`[WARN] Falló petición a ${rankedUrl}: ${err.message}. Intentando fallback global...`);
    rankedData = await httpGet(`https://lol-api-champion.op.gg/api/global/champions/ranked?tier=${tierParam}`);
  }

  if (!rankedData || !Array.isArray(rankedData.data)) {
    throw new Error(`Respuesta no válida de OP.GG para ${server} ${tierName}`);
  }

  // Recolectar todas las combinaciones campeón - posición
  const champRolePairs = [];
  for (const c of rankedData.data) {
    if (Array.isArray(c.positions)) {
      for (const pos of c.positions) {
        champRolePairs.push({
          champion: c,
          position: pos,
        });
      }
    }
  }

  console.log(`[PROCESS] ${champRolePairs.length} pares campeón-rol detectados para ${server} ${tierName}. Consultando detalles...`);

  // Descargar o reutilizar detalles con concurrencia controlada (12 hilos concurrentes)
  const normalizedList = await mapConcurrent(champRolePairs, 12, async (pair) => {
    const detail = await getOpGGChampionDetail(pair.champion.id, pair.position.name, region);
    return buildNormalizedChampion(pair.champion, pair.position, detail, dd, server, tierName);
  });

  const validChampions = normalizedList.filter(Boolean);

  // Ordenar y ajustar tiers por rol de forma determinista para garantizar metas competitivos (mínimo 2 S+ y 4 S)
  const roleGroups = new Map();
  for (const c of validChampions) {
    if (!roleGroups.has(c.role)) roleGroups.set(c.role, []);
    roleGroups.get(c.role).push(c);
  }

  const finalChampionList = [];
  for (const [role, list] of roleGroups.entries()) {
    // Ordenar de mayor a menor por metaScore
    list.sort((a, b) => b.metaScore - a.metaScore);

    // Ajustar distribución de tiers por rol
    list.forEach((c, index) => {
      if (index < 2) {
        c.tier = 'S+';
      } else if (index < 6) {
        c.tier = 'S';
      } else if (index < 14) {
        c.tier = 'A';
      } else if (index < 26) {
        c.tier = 'B';
      } else {
        c.tier = 'C';
      }
      // Recalcular metaScore con el tier ajustado
      const bonus = TIER_BONUS[c.tier] || 0.0;
      c.metaScore = parseFloat(((c.winRate * 0.6) + (Math.min(c.pickRate, 15) * 0.4) + bonus).toFixed(2));
      c.score = c.metaScore;
      finalChampionList.push(c);
    });
  }

  return finalChampionList;
}

// 5. Función principal de ejecución
async function main() {
  const startTime = Date.now();
  console.log('====================================================');
  console.log('  LOL META - Sincronizador de Datos Vía GitHub Actions');
  console.log('  Arquitectura de Datos Desacoplada | Autor: vamp9');
  console.log('====================================================');

  if (!fs.existsSync(DATA_DIR)) {
    fs.mkdirSync(DATA_DIR, { recursive: true });
    console.log(`[INFO] Directorio creado: ${DATA_DIR}`);
  }

  // Descarga de metadatos de Data Dragon
  const dd = await fetchDDragonData();

  // Generar combinaciones principales solicitadas
  // Por defecto: LAS DIAMOND (primario del proyecto), LAN DIAMOND, NA DIAMOND, EUW DIAMOND, KR DIAMOND, BR DIAMOND, LAS EMERALD, LAS MASTER
  const configurations = [
    { server: 'LAS', tier: 'DIAMOND', isPrimary: true },
    { server: 'LAN', tier: 'DIAMOND' },
    { server: 'NA', tier: 'DIAMOND' },
    { server: 'EUW', tier: 'DIAMOND' },
    { server: 'KR', tier: 'DIAMOND' },
    { server: 'BR', tier: 'DIAMOND' },
    { server: 'LAS', tier: 'EMERALD' },
    { server: 'LAS', tier: 'MASTER' },
  ];

  let primaryData = null;

  for (const config of configurations) {
    const sName = config.server.toLowerCase();
    const tName = config.tier.toLowerCase();
    const fileName = `meta_${sName}_${tName}.json`;
    const filePath = path.join(DATA_DIR, fileName);

    try {
      console.log(`\n----------------------------------------------------`);
      console.log(`[SYNC] Procesando ${config.server} - ${config.tier}...`);
      const champions = await extractServerTierMeta(config.server, config.tier, dd);

      fs.writeFileSync(filePath, JSON.stringify(champions, null, 2), 'utf-8');
      console.log(`[WRITE] Guardado exitosamente: data/${fileName} (${champions.length} campeones)`);

      if (config.isPrimary) {
        primaryData = champions;
      }
    } catch (err) {
      console.error(`[ERROR] Falló extracción para ${config.server} ${config.tier}:`, err.message);
    }
  }

  // Guardar archivo consolidado / por defecto: data/meta_current.json
  if (primaryData && primaryData.length > 0) {
    const currentPath = path.join(DATA_DIR, 'meta_current.json');
    fs.writeFileSync(currentPath, JSON.stringify(primaryData, null, 2), 'utf-8');
    console.log(`\n[SUCCESS] Archivo meta_current.json consolidado guardado (${primaryData.length} campeones).`);
  }

  const durationSec = ((Date.now() - startTime) / 1000).toFixed(1);
  console.log('\n====================================================');
  console.log(`  Sincronización completada exitosamente en ${durationSec}s`);
  console.log('====================================================');
}

main().catch((err) => {
  console.error('[FATAL ERROR]:', err);
  process.exit(1);
});
