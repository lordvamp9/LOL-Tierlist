use crate::models::ChampionRoleData;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tauri::{AppHandle, Manager};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 LoLClassicMeta/2.0.0 (vamp9)";
const CACHE_TTL_SECONDS: u64 = 12 * 3600; // 12 hours TTL
const RAW_GITHUB_BASE: &str = "https://raw.githubusercontent.com/lordvamp9/LOL-Tierlist/main/data";

// Fallback embebido para garantizar funcionamiento 100% offline en la primera ejecución
const EMBEDDED_DEFAULT_META: &str = include_str!("../../data/meta_current.json");

/// Obtiene la ruta al archivo de caché local en el directorio de datos de la aplicación
pub fn get_cache_file_path(app: &AppHandle, server: &str, tier: &str) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app_data_dir: {}", e))?;

    if !app_dir.exists() {
        let _ = fs::create_dir_all(&app_dir);
    }

    let clean_server = server.to_lowercase().replace(' ', "_");
    let clean_tier = tier.to_lowercase().replace(' ', "_");
    Ok(app_dir.join(format!("cache_{}_{}.json", clean_server, clean_tier)))
}

/// Consulta el parche actual desde Riot Games Data Dragon
pub async fn fetch_latest_patch() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://ddragon.leagueoflegends.com/api/versions.json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let versions: Vec<String> = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(first) = versions.first() {
        Ok(first.clone())
    } else {
        Err("No version returned from Data Dragon".into())
    }
}

/// Consume directamente los archivos JSON normalizados desde raw.githubusercontent.com
pub async fn fetch_from_github(server: &str, tier: &str) -> Result<Vec<ChampionRoleData>, String> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let clean_server = server.to_lowercase();
    let clean_tier = tier.to_lowercase();

    // 1. Intentar archivo específico para la combinación servidor + tier
    let specific_url = format!("{}/meta_{}_{}.json", RAW_GITHUB_BASE, clean_server, clean_tier);
    eprintln!("[INFO] Solicitando datos remotos de meta: {}", specific_url);

    match client.get(&specific_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let json_text = resp.text().await.map_err(|e| e.to_string())?;
            let champions: Vec<ChampionRoleData> = serde_json::from_str(&json_text)
                .map_err(|e| format!("Error parseando JSON de {}: {}", specific_url, e))?;
            eprintln!("[SUCCESS] {} campeones cargados desde {}", champions.len(), specific_url);
            return Ok(champions);
        }
        Ok(resp) => {
            eprintln!("[WARN] Servidor respondió status {} para {}. Intentando fallback consolidado...", resp.status(), specific_url);
        }
        Err(err) => {
            eprintln!("[WARN] Falló petición a {}: {}. Intentando fallback consolidado...", specific_url, err);
        }
    }

    // 2. Fallback a meta_current.json (snapshot consolidado)
    let fallback_url = format!("{}/meta_current.json", RAW_GITHUB_BASE);
    eprintln!("[INFO] Solicitando fallback consolidado: {}", fallback_url);

    let resp = client
        .get(&fallback_url)
        .send()
        .await
        .map_err(|e| format!("Error de conexión con GitHub ({})", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub retornó status {} al consultar {}", resp.status(), fallback_url));
    }

    let json_text = resp.text().await.map_err(|e| e.to_string())?;
    let champions: Vec<ChampionRoleData> = serde_json::from_str(&json_text)
        .map_err(|e| format!("Error parseando JSON consolidado: {}", e))?;

    eprintln!("[SUCCESS] {} campeones cargados desde fallback {}", champions.len(), fallback_url);
    Ok(champions)
}

/// Carga los datos del meta aplicando arquitectura de caché offline + sincronización GitHub
pub async fn load_meta_data(
    app: &AppHandle,
    server: &str,
    tier: &str,
    force_refresh: bool,
) -> Result<Vec<ChampionRoleData>, String> {
    let cache_path = get_cache_file_path(app, server, tier)?;

    // 1. Verificar caché offline en disco (TTL de 12 horas)
    if !force_refresh && cache_path.exists() {
        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed.as_secs() < CACHE_TTL_SECONDS {
                        if let Ok(content) = fs::read_to_string(&cache_path) {
                            if let Ok(mut cached_data) = serde_json::from_str::<Vec<ChampionRoleData>>(&content) {
                                if !cached_data.is_empty() {
                                    for item in &mut cached_data {
                                        item.is_cached = Some(true);
                                    }
                                    return Ok(cached_data);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Descargar datos 100% verídicos desde raw.githubusercontent.com
    match fetch_from_github(server, tier).await {
        Ok(mut fresh_data) => {
            // Guardar en caché local
            if let Ok(json_str) = serde_json::to_string_pretty(&fresh_data) {
                let _ = fs::write(&cache_path, json_str);
            }

            for item in &mut fresh_data {
                item.is_cached = Some(false);
            }
            Ok(fresh_data)
        }
        Err(network_err) => {
            eprintln!("[WARN] No se pudo obtener datos remotos de GitHub: {}. Activando modo offline / fallback...", network_err);

            // 3. Fallback a caché local existente aunque esté expirado
            if cache_path.exists() {
                if let Ok(content) = fs::read_to_string(&cache_path) {
                    if let Ok(mut cached_data) = serde_json::from_str::<Vec<ChampionRoleData>>(&content) {
                        if !cached_data.is_empty() {
                            for item in &mut cached_data {
                                item.is_cached = Some(true);
                            }
                            return Ok(cached_data);
                        }
                    }
                }
            }

            // 4. Fallback final garantizado: snapshot compilado internamente en la aplicación
            if let Ok(mut embedded_data) = serde_json::from_str::<Vec<ChampionRoleData>>(EMBEDDED_DEFAULT_META) {
                for item in &mut embedded_data {
                    item.is_cached = Some(true);
                }
                return Ok(embedded_data);
            }

            Err(format!("Error crítico cargando datos del meta: {}", network_err))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_data_validity() {
        let champs: Vec<ChampionRoleData> = serde_json::from_str(EMBEDDED_DEFAULT_META)
            .expect("El JSON embebido meta_current.json debe ser deserializable");
        assert!(!champs.is_empty(), "La lista de campeones embebida no debe estar vacía");

        for role in &["TOP", "JUNGLE", "MID", "CARRY", "SUPPORT"] {
            let role_champs: Vec<_> = champs.iter().filter(|c| c.role == *role).collect();
            assert!(!role_champs.is_empty(), "El rol {} debe tener campeones", role);

            let s_plus_count = role_champs.iter().filter(|c| c.tier == "S+").count();
            let s_count = role_champs.iter().filter(|c| c.tier == "S").count();
            assert!(s_plus_count >= 2, "El rol {} debe tener al menos 2 picks S+, encontrados: {}", role, s_plus_count);
            assert!(s_count >= 4, "El rol {} debe tener al menos 4 picks S, encontrados: {}", role, s_count);

            for c in role_champs {
                assert!(!c.build.starting_items.is_empty(), "El campeón {} debe tener starting items", c.name);
                assert!(!c.build.core_items.is_empty(), "El campeón {} debe tener core items", c.name);
                assert!(!c.build.situational_items.is_empty(), "El campeón {} debe tener situational items", c.name);
                assert!(!c.runes.keystone_name.is_empty(), "El campeón {} debe tener keystone", c.name);
                assert!(c.skill_order.len() >= 3, "El campeón {} debe tener al menos 3 habilidades de orden", c.name);
            }
        }
    }

    #[test]
    fn test_camel_case_serialization() {
        let champs: Vec<ChampionRoleData> = serde_json::from_str(EMBEDDED_DEFAULT_META)
            .expect("Debe deserializar");
        let json_str = serde_json::to_string(&champs[0]).expect("Debe serializar");
        assert!(json_str.contains("\"startingItems\":"), "Debe serializar como camelCase startingItems");
        assert!(json_str.contains("\"coreItems\":"), "Debe serializar como camelCase coreItems");
        assert!(json_str.contains("\"metaScore\":"), "Debe serializar como camelCase metaScore");
    }

    #[tokio::test]
    async fn test_live_fetch_from_github() {
        let champs = fetch_from_github("LAS", "DIAMOND").await
            .expect("fetch_from_github debe obtener y parsear exitosamente los datos desde raw.githubusercontent.com");
        assert!(!champs.is_empty(), "La respuesta remota de GitHub no debe estar vacia");
        assert_eq!(champs[0].patch.as_deref(), Some("16.19.1"), "Debe contener el parche real 16.19.1");
        assert!(champs.iter().any(|c| c.name == "Tryndamere" || c.name == "Garen"), "Debe contener campeones reales");
    }
}
