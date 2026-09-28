use crate::models::{
    BuildRecommendation, ChampionRoleData, ItemInfo, RuneItem, RuneTree, SummonerSpell,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tauri::{AppHandle, Manager};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 LoLClassicMeta/1.0.1 (vamp9)";
const CACHE_TTL_SECONDS: u64 = 12 * 3600; // 12 hours TTL

#[derive(Clone, Copy)]
pub enum Archetype {
    AdBruiser,
    ApMage,
    AdCarry,
    Tank,
    AdAssassin,
    ApAssassin,
    Enchanter,
    SupportTank,
}

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

fn item(patch: &str, id: u32, name: &str, cost: u32, win_rate: Option<f64>, category: Option<&str>) -> ItemInfo {
    ItemInfo {
        id,
        name: name.to_string(),
        icon_url: format!("https://ddragon.leagueoflegends.com/cdn/{}/img/item/{}.png", patch, id),
        cost,
        win_rate,
        category: category.map(|s| s.to_string()),
    }
}

fn rune(id: u32, name: &str, path: &str, slot: u8) -> RuneItem {
    RuneItem {
        id,
        name: name.to_string(),
        icon_url: format!("https://ddragon.leagueoflegends.com/cdn/img/{}", path),
        slot,
    }
}

fn spell(id: &str, name: &str) -> SummonerSpell {
    SummonerSpell {
        id: id.to_string(),
        name: name.to_string(),
        icon_url: format!("https://ddragon.leagueoflegends.com/cdn/14.24.1/img/spell/{}.png", id),
    }
}

fn build_for_champion(patch: &str, champ_id: &str, archetype: Archetype) -> BuildRecommendation {
    match champ_id {
        "Jinx" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3006, "Grebas de Berserker", 1100, None, None),
            core_items: vec![
                item(patch, 6672, "Verdugo de Krakens", 3100, None, None),
                item(patch, 3085, "Huracan de Runaan", 2600, None, None),
                item(patch, 3031, "Filo del Infinito", 3600, None, None),
            ],
            fourth_items: vec![
                item(patch, 3036, "Recuerdos de Lord Dominik", 3000, Some(57.2), Some("Penetracion Armor")),
                item(patch, 6676, "El Coleccionista", 3200, Some(56.5), Some("Letalidad Critica")),
            ],
            fifth_items: vec![
                item(patch, 3072, "La Sanguinaria", 3400, Some(60.8), Some("Robo de Vida")),
                item(patch, 3094, "Cañon de Fuego Rapido", 3000, Some(59.4), Some("Alcance Extra")),
            ],
            sixth_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, Some(63.4), Some("Segunda Vida")),
                item(patch, 6673, "Arcoescudo Inmortal", 3000, Some(61.8), Some("Supervivencia")),
            ],
            situational_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Armadura y Revivir")),
                item(patch, 3072, "La Sanguinaria", 3400, None, Some("Sustento Extremo")),
                item(patch, 3139, "Cimitarra Mercurial", 3300, None, Some("Limpieza de CC")),
                item(patch, 3033, "Recordatorio Mortal", 3000, None, Some("Heridas Graves")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Escudo Magico")),
            ],
        },
        "Jhin" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3009, "Botas de Rapidez", 900, None, None),
            core_items: vec![
                item(patch, 6676, "El Coleccionista", 3200, None, None),
                item(patch, 3031, "Filo del Infinito", 3600, None, None),
                item(patch, 3094, "Cañon de Fuego Rapido", 3000, None, None),
            ],
            fourth_items: vec![
                item(patch, 3036, "Recuerdos de Lord Dominik", 3000, Some(58.1), Some("Anti-Tanques")),
                item(patch, 3087, "Daga de Statikk", 2900, Some(56.9), Some("Onda Electrica")),
            ],
            fifth_items: vec![
                item(patch, 3072, "La Sanguinaria", 3400, Some(61.4), Some("Daño y Escudo")),
                item(patch, 6697, "Oportunidad", 2700, Some(59.8), Some("Movilidad y Burst")),
            ],
            sixth_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, Some(64.0), Some("Revivir")),
                item(patch, 3814, "Filo de la Noche", 2800, Some(62.1), Some("Anti-Hechizos")),
            ],
            situational_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Revivir")),
                item(patch, 3814, "Filo de la Noche", 2800, None, Some("Inmunidad a CC")),
                item(patch, 3033, "Recordatorio Mortal", 3000, None, Some("Anti-Curacion")),
                item(patch, 3072, "La Sanguinaria", 3400, None, Some("Robo de Vida")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Defensa AP")),
            ],
        },
        "Samira" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3047, "Punteras de Acero", 1100, None, None),
            core_items: vec![
                item(patch, 6676, "El Coleccionista", 3200, None, None),
                item(patch, 3031, "Filo del Infinito", 3600, None, None),
                item(patch, 3072, "La Sanguinaria", 3400, None, None),
            ],
            fourth_items: vec![
                item(patch, 3036, "Recuerdos de Lord Dominik", 3000, Some(58.6), Some("Penetracion")),
                item(patch, 6673, "Arcoescudo Inmortal", 3000, Some(57.8), Some("Escudo Salvavidas")),
            ],
            fifth_items: vec![
                item(patch, 6333, "Danza de la Muerte", 3300, Some(61.2), Some("Anti-Burst AD")),
                item(patch, 3026, "Angel Guardian", 3200, Some(62.5), Some("Segunda Vida")),
            ],
            sixth_items: vec![
                item(patch, 3156, "Fauces de Malmortius", 3100, Some(63.8), Some("Escudo Magico")),
                item(patch, 3814, "Filo de la Noche", 2800, Some(61.9), Some("Escudo Antihechizos")),
            ],
            situational_items: vec![
                item(patch, 6673, "Arcoescudo Inmortal", 3000, None, Some("Supervivencia")),
                item(patch, 6333, "Danza de la Muerte", 3300, None, Some("Defensa en Peleas")),
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Revivir")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Anti-Burst AP")),
                item(patch, 3033, "Recordatorio Mortal", 3000, None, Some("Heridas Graves")),
            ],
        },
        "Kaisa" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3006, "Grebas de Berserker", 1100, None, None),
            core_items: vec![
                item(patch, 3087, "Daga de Statikk", 2900, None, None),
                item(patch, 3124, "Espada de Furia de Guinsoo", 3000, None, None),
                item(patch, 3115, "Diente de Nashor", 3000, None, None),
            ],
            fourth_items: vec![
                item(patch, 3157, "Reloj de Arena de Zhonya", 3250, Some(58.4), Some("Invulnerabilidad")),
                item(patch, 3302, "Terminus", 3000, Some(57.2), Some("Resistencias Hibridas")),
            ],
            fifth_items: vec![
                item(patch, 3089, "Sombrero Mortal de Rabadon", 3600, Some(62.1), Some("Poder Magico")),
                item(patch, 4645, "Llamasombria", 3200, Some(60.5), Some("Critico Magico")),
            ],
            sixth_items: vec![
                item(patch, 3137, "Flor Criptofloreciente", 2850, Some(63.6), Some("Penetracion AP + Cura")),
                item(patch, 3026, "Angel Guardian", 3200, Some(61.8), Some("Revivir")),
            ],
            situational_items: vec![
                item(patch, 3157, "Reloj de Arena de Zhonya", 3250, None, Some("Stasis Defensiva")),
                item(patch, 3102, "Velo del Hada de la Muerte", 3100, None, Some("Antihechizos")),
                item(patch, 3302, "Terminus", 3000, None, Some("Penetracion Dual")),
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Segunda Vida")),
                item(patch, 3135, "Baculo del Vacio", 3000, None, Some("Penetracion Magica")),
            ],
        },
        "MissFortune" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3009, "Botas de Rapidez", 900, None, None),
            core_items: vec![
                item(patch, 6676, "El Coleccionista", 3200, None, None),
                item(patch, 3031, "Filo del Infinito", 3600, None, None),
                item(patch, 3036, "Recuerdos de Lord Dominik", 3000, None, None),
            ],
            fourth_items: vec![
                item(patch, 3072, "La Sanguinaria", 3400, Some(58.2), Some("Robo de Vida")),
                item(patch, 3094, "Cañon de Fuego Rapido", 3000, Some(57.1), Some("Alcance")),
            ],
            fifth_items: vec![
                item(patch, 6697, "Oportunidad", 2700, Some(60.9), Some("Letalidad y Velocidad")),
                item(patch, 3814, "Filo de la Noche", 2800, Some(60.1), Some("Escudo")),
            ],
            sixth_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, Some(63.2), Some("Revivir")),
                item(patch, 6694, "Rencor de Serylda", 3200, Some(61.5), Some("Ralentizacion")),
            ],
            situational_items: vec![
                item(patch, 3814, "Filo de la Noche", 2800, None, Some("Antihechizos")),
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Revivir")),
                item(patch, 3072, "La Sanguinaria", 3400, None, Some("Robo de Vida")),
                item(patch, 3033, "Recordatorio Mortal", 3000, None, Some("Heridas Graves")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Defensa AP")),
            ],
        },
        "Caitlyn" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3006, "Grebas de Berserker", 1100, None, None),
            core_items: vec![
                item(patch, 6676, "El Coleccionista", 3200, None, None),
                item(patch, 3031, "Filo del Infinito", 3600, None, None),
                item(patch, 3094, "Cañon de Fuego Rapido", 3000, None, None),
            ],
            fourth_items: vec![
                item(patch, 3036, "Recuerdos de Lord Dominik", 3000, Some(57.8), Some("Penetracion")),
                item(patch, 3072, "La Sanguinaria", 3400, Some(58.4), Some("Sustento")),
            ],
            fifth_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, Some(61.6), Some("Revivir")),
                item(patch, 6673, "Arcoescudo Inmortal", 3000, Some(60.2), Some("Supervivencia")),
            ],
            sixth_items: vec![
                item(patch, 3139, "Cimitarra Mercurial", 3300, Some(62.9), Some("Limpieza CC")),
                item(patch, 6677, "Flechas Indomitas de Yun Tal", 3000, Some(61.8), Some("Hemorragia Critica")),
            ],
            situational_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Proteccion")),
                item(patch, 3072, "La Sanguinaria", 3400, None, Some("Robo de Vida")),
                item(patch, 3036, "Recuerdos de Lord Dominik", 3000, None, Some("Anti-Tanques")),
                item(patch, 3033, "Recordatorio Mortal", 3000, None, Some("Heridas Graves")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Escudo Magico")),
            ],
        },
        "Ezreal" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3158, "Botas Jonias de la Lucidez", 900, None, None),
            core_items: vec![
                item(patch, 3078, "Fuerza de la Trinidad", 3333, None, None),
                item(patch, 3004, "Manamune", 2900, None, None),
                item(patch, 6694, "Rencor de Serylda", 3200, None, None),
            ],
            fourth_items: vec![
                item(patch, 3161, "Lanza de Shojin", 3300, Some(57.4), Some("CDR de Habilidades")),
                item(patch, 3110, "Corazon de Hielo", 2400, Some(56.8), Some("Armadura y Mana")),
            ],
            fifth_items: vec![
                item(patch, 3072, "La Sanguinaria", 3400, Some(60.5), Some("Sustento")),
                item(patch, 3156, "Fauces de Malmortius", 3100, Some(59.9), Some("Escudo Magico")),
            ],
            sixth_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, Some(63.1), Some("Revivir")),
                item(patch, 6609, "Espada Motosierra Quimopunk", 2800, Some(61.2), Some("Anti-Curacion")),
            ],
            situational_items: vec![
                item(patch, 3110, "Corazon de Hielo", 2400, None, Some("Anti-AD y Mana")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Defensa AP")),
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Revivir")),
                item(patch, 3161, "Lanza de Shojin", 3300, None, Some("Daño Habilidades")),
                item(patch, 6609, "Espada Motosierra Quimopunk", 2800, None, Some("Heridas Graves")),
            ],
        },
        "Aatrox" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3047, "Punteras de Acero", 1100, None, None),
            core_items: vec![
                item(patch, 6692, "Eclipse", 2800, None, None),
                item(patch, 6610, "Cielo Desgarrado", 3100, None, None),
                item(patch, 3071, "Cuchilla Negra", 3000, None, None),
            ],
            fourth_items: vec![
                item(patch, 3053, "Guantelete de Sterak", 3200, Some(56.4), Some("Defensa y Tenacidad")),
                item(patch, 6333, "Danza de la Muerte", 3300, Some(57.8), Some("Anti-Burst")),
            ],
            fifth_items: vec![
                item(patch, 3156, "Fauces de Malmortius", 3100, Some(59.2), Some("Anti-Magico")),
                item(patch, 3026, "Angel Guardian", 3200, Some(60.5), Some("Revivir")),
            ],
            sixth_items: vec![
                item(patch, 6665, "Jak'Sho, el Proteico", 3200, Some(62.1), Some("Resistencias")),
                item(patch, 6694, "Rencor de Serylda", 3200, Some(61.4), Some("Penetracion y Slow")),
            ],
            situational_items: vec![
                item(patch, 6333, "Danza de la Muerte", 3300, None, Some("Anti-Burst AD")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Escudo Magico")),
                item(patch, 3143, "Presagio de Randuin", 2700, None, Some("Anti-Critico")),
                item(patch, 3075, "Malla de Espinas", 2700, None, Some("Anti-Curacion")),
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Segunda Vida")),
            ],
        },
        "Darius" => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3047, "Punteras de Acero", 1100, None, None),
            core_items: vec![
                item(patch, 6631, "Rompeavances", 3300, None, None),
                item(patch, 3078, "Fuerza de la Trinidad", 3333, None, None),
                item(patch, 3742, "Placa del Hombre Muerto", 2900, None, None),
            ],
            fourth_items: vec![
                item(patch, 3053, "Guantelete de Sterak", 3200, Some(57.1), Some("Escudo Furia")),
                item(patch, 4401, "Fuerza de la Naturaleza", 2800, Some(57.8), Some("Velocidad y MR")),
            ],
            fifth_items: vec![
                item(patch, 6333, "Danza de la Muerte", 3300, Some(60.3), Some("Armadura en Combate")),
                item(patch, 3143, "Presagio de Randuin", 2700, Some(61.0), Some("Anti-Tiradores")),
            ],
            sixth_items: vec![
                item(patch, 6665, "Jak'Sho, el Proteico", 3200, Some(62.8), Some("Resistencias Mixtas")),
                item(patch, 3075, "Malla de Espinas", 2700, Some(61.9), Some("Heridas Graves")),
            ],
            situational_items: vec![
                item(patch, 3053, "Guantelete de Sterak", 3200, None, Some("Supervivencia")),
                item(patch, 3742, "Placa del Hombre Muerto", 2900, None, Some("Velocidad de Caza")),
                item(patch, 4401, "Fuerza de la Naturaleza", 2800, None, Some("Velocidad y Resistencia")),
                item(patch, 3075, "Malla de Espinas", 2700, None, Some("Anti-Curacion")),
                item(patch, 3143, "Presagio de Randuin", 2700, None, Some("Anti-Critico")),
            ],
        },
        _ => build_for_archetype(patch, archetype),
    }
}

fn build_for_archetype(patch: &str, archetype: Archetype) -> BuildRecommendation {
    match archetype {
        Archetype::AdBruiser => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3047, "Punteras de Acero", 1100, None, None),
            core_items: vec![
                item(patch, 6692, "Eclipse", 2800, None, None),
                item(patch, 6610, "Cielo Desgarrado", 3100, None, None),
                item(patch, 3071, "Cuchilla Negra", 3000, None, None),
            ],
            fourth_items: vec![
                item(patch, 3053, "Guantelete de Sterak", 3200, Some(56.4), Some("Defensa")),
                item(patch, 6333, "Danza de la Muerte", 3300, Some(57.8), Some("Anti-Burst")),
            ],
            fifth_items: vec![
                item(patch, 3156, "Fauces de Malmortius", 3100, Some(59.2), Some("Anti-Magico")),
                item(patch, 3026, "Angel Guardian", 3200, Some(60.5), Some("Revivir")),
            ],
            sixth_items: vec![
                item(patch, 6665, "Jak'Sho, el Proteico", 3200, Some(62.1), Some("Resistencias")),
                item(patch, 6609, "Espada Motosierra Quimopunk", 2800, Some(61.4), Some("Heridas Graves")),
            ],
            situational_items: vec![
                item(patch, 6333, "Danza de la Muerte", 3300, None, Some("Anti-Burst AD")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Escudo Magico")),
                item(patch, 3143, "Presagio de Randuin", 2700, None, Some("Anti-Critico")),
                item(patch, 3075, "Malla de Espinas", 2700, None, Some("Anti-Curacion")),
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Segunda Vida")),
            ],
        },
        Archetype::ApMage => BuildRecommendation {
            starting_items: vec![
                item(patch, 1056, "Anillo de Doran", 400, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3020, "Botas de Hechicero", 1100, None, None),
            core_items: vec![
                item(patch, 6655, "Compañera de Luden", 2900, None, None),
                item(patch, 4645, "Llamasombria", 3200, None, None),
                item(patch, 4646, "Sobrecarga Tormentosa", 2900, None, None),
            ],
            fourth_items: vec![
                item(patch, 3157, "Reloj de Arena de Zhonya", 3250, Some(57.2), Some("Invulnerabilidad")),
                item(patch, 3135, "Baculo del Vacio", 3000, Some(58.6), Some("Penetracion")),
            ],
            fifth_items: vec![
                item(patch, 3089, "Sombrero Mortal de Rabadon", 3600, Some(61.8), Some("Poder Magico")),
                item(patch, 3137, "Flor Criptofloreciente", 2850, Some(60.4), Some("Penetracion + Cura")),
            ],
            sixth_items: vec![
                item(patch, 3102, "Velo del Hada de la Muerte", 3100, Some(63.5), Some("Antihechizos")),
                item(patch, 4628, "Enfoque al Horizonte", 2700, Some(62.0), Some("Vision + Daño")),
            ],
            situational_items: vec![
                item(patch, 3157, "Reloj de Arena de Zhonya", 3250, None, Some("Defensa Stasis")),
                item(patch, 3102, "Velo del Hada de la Muerte", 3100, None, Some("Antihechizos")),
                item(patch, 3165, "Morellonomicon", 2200, None, Some("Anti-Curacion")),
                item(patch, 3135, "Baculo del Vacio", 3000, None, Some("Penetracion")),
                item(patch, 4629, "Impulso Cosmico", 3000, None, Some("Velocidad")),
            ],
        },
        Archetype::AdCarry => BuildRecommendation {
            starting_items: vec![
                item(patch, 1055, "Espada de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3006, "Grebas de Berserker", 1100, None, None),
            core_items: vec![
                item(patch, 6672, "Verdugo de Krakens", 3100, None, None),
                item(patch, 6676, "El Coleccionista", 3200, None, None),
                item(patch, 3031, "Filo del Infinito", 3600, None, None),
            ],
            fourth_items: vec![
                item(patch, 3036, "Recuerdos de Lord Dominik", 3000, Some(56.9), Some("Penetracion Armor")),
                item(patch, 3094, "Cañon de Fuego Rapido", 3000, Some(57.5), Some("Alcance Extra")),
            ],
            fifth_items: vec![
                item(patch, 3072, "La Sanguinaria", 3400, Some(60.2), Some("Robo de Vida")),
                item(patch, 6673, "Arcoescudo Inmortal", 3000, Some(59.4), Some("Supervivencia")),
            ],
            sixth_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, Some(62.7), Some("Revivir")),
                item(patch, 3139, "Cimitarra Mercurial", 3300, Some(61.3), Some("Purificacion")),
            ],
            situational_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Proteccion Armadura")),
                item(patch, 6673, "Arcoescudo Inmortal", 3000, None, Some("Escudo Salvavidas")),
                item(patch, 3033, "Recordatorio Mortal", 3000, None, Some("Heridas Graves")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Escudo Magico")),
                item(patch, 3153, "Hoja del Rey Arruinado", 3200, None, Some("Destruye Tanques")),
            ],
        },
        Archetype::Tank => BuildRecommendation {
            starting_items: vec![
                item(patch, 1054, "Escudo de Doran", 450, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3047, "Punteras de Acero", 1100, None, None),
            core_items: vec![
                item(patch, 3084, "Corazon de Acero", 3000, None, None),
                item(patch, 3068, "Egida de Fuego Solar", 2700, None, None),
                item(patch, 6662, "Rookern Kaenico", 2900, None, None),
            ],
            fourth_items: vec![
                item(patch, 3075, "Malla de Espinas", 2700, Some(55.8), Some("Heridas Graves")),
                item(patch, 6660, "Desesperacion Interminable", 2800, Some(57.1), Some("Sustento AoE")),
            ],
            fifth_items: vec![
                item(patch, 6665, "Jak'Sho, el Proteico", 3200, Some(60.2), Some("Resistencias Mixtas")),
                item(patch, 3143, "Presagio de Randuin", 2700, Some(59.5), Some("Anti-Critico")),
            ],
            sixth_items: vec![
                item(patch, 3083, "Armadura de Warmog", 3100, Some(62.0), Some("Regeneracion")),
                item(patch, 4401, "Fuerza de la Naturaleza", 2800, Some(61.5), Some("Anti-DoT Magico")),
            ],
            situational_items: vec![
                item(patch, 4401, "Fuerza de la Naturaleza", 2800, None, Some("Resistencia Magica")),
                item(patch, 3143, "Presagio de Randuin", 2700, None, Some("Ralentizacion")),
                item(patch, 3110, "Corazon de Hielo", 2400, None, Some("Aura Anti-Velocidad")),
                item(patch, 8020, "Mascara Abisal", 2500, None, Some("Reduccion MR Enemiga")),
                item(patch, 3083, "Armadura de Warmog", 3100, None, Some("Regeneracion")),
            ],
        },
        Archetype::AdAssassin => BuildRecommendation {
            starting_items: vec![
                item(patch, 1036, "Espada Larga", 350, None, None),
                item(patch, 2031, "Pocion Reutilizable", 150, None, None),
            ],
            boots: item(patch, 3158, "Botas Jonias de la Lucidez", 900, None, None),
            core_items: vec![
                item(patch, 6698, "Hidra Profana", 3300, None, None),
                item(patch, 6697, "Oportunidad", 2700, None, None),
                item(patch, 3814, "Filo de la Noche", 2800, None, None),
            ],
            fourth_items: vec![
                item(patch, 6694, "Rencor de Serylda", 3200, Some(57.3), Some("Penetracion + Slow")),
                item(patch, 3142, "Espada Fantasma de Youmuu", 2700, Some(58.1), Some("Movilidad")),
            ],
            fifth_items: vec![
                item(patch, 6696, "Arco Axiomatico", 3000, Some(60.4), Some("Reinicio Ultimate")),
                item(patch, 6699, "Cicloespada Voltaica", 2900, Some(59.8), Some("Slow Energizado")),
            ],
            sixth_items: vec![
                item(patch, 3026, "Angel Guardian", 3200, Some(63.2), Some("Revivir")),
                item(patch, 3156, "Fauces de Malmortius", 3100, Some(61.7), Some("Anti-Burst Magico")),
            ],
            situational_items: vec![
                item(patch, 3814, "Filo de la Noche", 2800, None, Some("Escudo Antihechizos")),
                item(patch, 6695, "Colmillo de Serpiente", 2500, None, Some("Anti-Escudos")),
                item(patch, 3156, "Fauces de Malmortius", 3100, None, Some("Defensa AP")),
                item(patch, 3026, "Angel Guardian", 3200, None, Some("Supervivencia")),
                item(patch, 6333, "Danza de la Muerte", 3300, None, Some("Tenacidad")),
            ],
        },
        Archetype::ApAssassin => BuildRecommendation {
            starting_items: vec![
                item(patch, 1056, "Anillo de Doran", 400, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3020, "Botas de Hechicero", 1100, None, None),
            core_items: vec![
                item(patch, 3100, "Maldicion del Liche", 3100, None, None),
                item(patch, 4645, "Llamasombria", 3200, None, None),
                item(patch, 3157, "Reloj de Arena de Zhonya", 3250, None, None),
            ],
            fourth_items: vec![
                item(patch, 3135, "Baculo del Vacio", 3000, Some(57.5), Some("Penetracion")),
                item(patch, 4646, "Sobrecarga Tormentosa", 2900, Some(58.2), Some("Burst AoE")),
            ],
            fifth_items: vec![
                item(patch, 3089, "Sombrero Mortal de Rabadon", 3600, Some(62.0), Some("AP Masivo")),
                item(patch, 3102, "Velo del Hada de la Muerte", 3100, Some(60.8), Some("Antihechizos")),
            ],
            sixth_items: vec![
                item(patch, 3041, "Robaalmas de Mejai", 1500, Some(64.5), Some("Bola de Nieve")),
                item(patch, 3137, "Flor Criptofloreciente", 2850, Some(61.9), Some("Cura en Equipo")),
            ],
            situational_items: vec![
                item(patch, 3157, "Reloj de Arena de Zhonya", 3250, None, Some("Invulnerabilidad")),
                item(patch, 3102, "Velo del Hada de la Muerte", 3100, None, Some("Escudo Magico")),
                item(patch, 3165, "Morellonomicon", 2200, None, Some("Anti-Curacion")),
                item(patch, 3135, "Baculo del Vacio", 3000, None, Some("Penetracion")),
                item(patch, 4629, "Impulso Cosmico", 3000, None, Some("Kite y Movilidad")),
            ],
        },
        Archetype::Enchanter => BuildRecommendation {
            starting_items: vec![
                item(patch, 3865, "Atlas Mundial", 400, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3158, "Botas Jonias de la Lucidez", 900, None, None),
            core_items: vec![
                item(patch, 3869, "Creador de Sueños", 400, None, None),
                item(patch, 6617, "Renovador de Piedra Lunar", 2700, None, None),
                item(patch, 6620, "Ecos de Helia", 2200, None, None),
            ],
            fourth_items: vec![
                item(patch, 3504, "Pebetero Ardiente", 2300, Some(56.5), Some("Buff ADC")),
                item(patch, 6616, "Baculo de Agua Fluyente", 2300, Some(57.2), Some("Buff AP y CDR")),
            ],
            fifth_items: vec![
                item(patch, 3107, "Redencion", 2300, Some(59.8), Some("Cura Global")),
                item(patch, 6621, "Nucleo del Alba", 2700, Some(60.5), Some("Amplificacion Curas")),
            ],
            sixth_items: vec![
                item(patch, 3222, "Bendicion de Mikael", 2300, Some(62.1), Some("Purificacion")),
                item(patch, 4638, "Piedra Guardiana Vigilante", 2300, Some(63.0), Some("Control Vision")),
            ],
            situational_items: vec![
                item(patch, 3222, "Bendicion de Mikael", 2300, None, Some("Limpieza de CC")),
                item(patch, 2065, "Cancion de Batalla de Shurelya", 2200, None, Some("Iniciacion / Escape")),
                item(patch, 3107, "Redencion", 2300, None, Some("Sanacion en Area")),
                item(patch, 3190, "Medallon Solari", 2200, None, Some("Escudo Colectivo")),
                item(patch, 3165, "Morellonomicon", 2200, None, Some("Anti-Curacion")),
            ],
        },
        Archetype::SupportTank => BuildRecommendation {
            starting_items: vec![
                item(patch, 3865, "Atlas Mundial", 400, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
                item(patch, 2003, "Pocion de Vida", 50, None, None),
            ],
            boots: item(patch, 3009, "Botas de Rapidez", 900, None, None),
            core_items: vec![
                item(patch, 3870, "Oposicion Celestial", 400, None, None),
                item(patch, 3190, "Medallon de los Solari", 2200, None, None),
                item(patch, 3109, "Promesa del Caballero", 2200, None, None),
            ],
            fourth_items: vec![
                item(patch, 6667, "Pionero", 2500, Some(56.2), Some("Iniciacion Acelerada")),
                item(patch, 3050, "Convergencia de Zeke", 2200, Some(57.0), Some("Slow + Daño")),
            ],
            fifth_items: vec![
                item(patch, 3075, "Malla de Espinas", 2700, Some(58.7), Some("Heridas Graves")),
                item(patch, 6662, "Rookern Kaenico", 2900, Some(59.5), Some("Escudo Magico")),
            ],
            sixth_items: vec![
                item(patch, 4638, "Piedra Guardiana Vigilante", 2300, Some(61.8), Some("Control Vision")),
                item(patch, 3110, "Corazon de Hielo", 2400, Some(60.9), Some("Aura Anti-Velocidad")),
            ],
            situational_items: vec![
                item(patch, 6667, "Pionero", 2500, None, Some("Iniciacion")),
                item(patch, 3050, "Convergencia de Zeke", 2200, None, Some("Ralentizacion")),
                item(patch, 3109, "Promesa del Caballero", 2200, None, Some("Proteger al Carry")),
                item(patch, 3110, "Corazon de Hielo", 2400, None, Some("Anti-Autoataques")),
                item(patch, 6662, "Rookern Kaenico", 2900, None, Some("Resistencia Magica")),
            ],
        },
    }
}

fn runes_for_archetype(archetype: Archetype) -> RuneTree {
    match archetype {
        Archetype::AdBruiser => RuneTree {
            primary_style_id: 8000,
            primary_style_name: "Precision".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7201_Precision.png".to_string(),
            keystone_id: 8010,
            keystone_name: "Conquistador".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png".to_string(),
            primary_runes: vec![
                rune(9111, "Triunfo", "perk-images/Styles/Precision/Triumph.png", 1),
                rune(9105, "Leyenda: Presteza", "perk-images/Styles/Precision/LegendAlacrity/LegendAlacrity.png", 2),
                rune(8299, "Ultimo Esfuerzo", "perk-images/Styles/Precision/LastStand/LastStand.png", 3),
            ],
            secondary_style_id: 8400,
            secondary_style_name: "Valor".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7204_Resolve.png".to_string(),
            secondary_runes: vec![
                rune(8444, "Fuerzas Renovadas", "perk-images/Styles/Resolve/SecondWind/SecondWind.png", 2),
                rune(8451, "Sobrecrecimiento", "perk-images/Styles/Resolve/Overgrowth/Overgrowth.png", 3),
            ],
            shards: vec!["+9 Fuerza Adaptativa".to_string(), "+9 Fuerza Adaptativa".to_string(), "+65 Vida Plana".to_string()],
        },
        Archetype::ApMage => RuneTree {
            primary_style_id: 8200,
            primary_style_name: "Brujeria".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7202_Sorcery.png".to_string(),
            keystone_id: 8229,
            keystone_name: "Cometa Arcano".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/ArcaneComet/ArcaneComet.png".to_string(),
            primary_runes: vec![
                rune(8226, "Banda de Mana", "perk-images/Styles/Sorcery/ManaflowBand/ManaflowBand.png", 1),
                rune(8210, "Trascendencia", "perk-images/Styles/Sorcery/Transcendence/Transcendence.png", 2),
                rune(8237, "Piromancia", "perk-images/Styles/Sorcery/Scorch/Scorch.png", 3),
            ],
            secondary_style_id: 8300,
            secondary_style_name: "Inspiracion".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7203_Whimsy.png".to_string(),
            secondary_runes: vec![
                rune(8304, "Calzado Magico", "perk-images/Styles/Inspiration/MagicalFootwear/MagicalFootwear.png", 1),
                rune(8347, "Perspicacia Cosmica", "perk-images/Styles/Inspiration/CosmicInsight/CosmicInsight.png", 3),
            ],
            shards: vec!["+9 Fuerza Adaptativa".to_string(), "+9 Fuerza Adaptativa".to_string(), "+65 Vida Plana".to_string()],
        },
        Archetype::AdCarry => RuneTree {
            primary_style_id: 8000,
            primary_style_name: "Precision".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7201_Precision.png".to_string(),
            keystone_id: 8005,
            keystone_name: "Ataque Intensificado".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/PressTheAttack/PressTheAttack.png".to_string(),
            primary_runes: vec![
                rune(9101, "Claridad Mental", "perk-images/Styles/Precision/PresenceOfMind/PresenceOfMind.png", 1),
                rune(9103, "Leyenda: Linaje", "perk-images/Styles/Precision/LegendBloodline/LegendBloodline.png", 2),
                rune(8014, "Golpe de Gracia", "perk-images/Styles/Precision/CoupDeGrace/CoupDeGrace.png", 3),
            ],
            secondary_style_id: 8300,
            secondary_style_name: "Inspiracion".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7203_Whimsy.png".to_string(),
            secondary_runes: vec![
                rune(8345, "Entrega de Galletas", "perk-images/Styles/Inspiration/BiscuitDelivery/BiscuitDelivery.png", 2),
                rune(8347, "Perspicacia Cosmica", "perk-images/Styles/Inspiration/CosmicInsight/CosmicInsight.png", 3),
            ],
            shards: vec!["+10% Velocidad de Ataque".to_string(), "+9 Fuerza Adaptativa".to_string(), "+65 Vida Plana".to_string()],
        },
        Archetype::Tank => RuneTree {
            primary_style_id: 8400,
            primary_style_name: "Valor".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7204_Resolve.png".to_string(),
            keystone_id: 8437,
            keystone_name: "Garras del Inmortal".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/GraspOfTheUndying/GraspOfTheUndying.png".to_string(),
            primary_runes: vec![
                rune(8446, "Demoler", "perk-images/Styles/Resolve/Demolish/Demolish.png", 1),
                rune(8429, "Acondicionamiento", "perk-images/Styles/Resolve/Conditioning/Conditioning.png", 2),
                rune(8451, "Sobrecrecimiento", "perk-images/Styles/Resolve/Overgrowth/Overgrowth.png", 3),
            ],
            secondary_style_id: 8000,
            secondary_style_name: "Precision".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7201_Precision.png".to_string(),
            secondary_runes: vec![
                rune(9111, "Triunfo", "perk-images/Styles/Precision/Triumph.png", 1),
                rune(8299, "Ultimo Esfuerzo", "perk-images/Styles/Precision/LastStand/LastStand.png", 3),
            ],
            shards: vec!["+9 Fuerza Adaptativa".to_string(), "+6 Armadura".to_string(), "+65 Vida Plana".to_string()],
        },
        Archetype::AdAssassin => RuneTree {
            primary_style_id: 8100,
            primary_style_name: "Dominacion".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7200_Domination.png".to_string(),
            keystone_id: 8112,
            keystone_name: "Electrocutar".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png".to_string(),
            primary_runes: vec![
                rune(8143, "Impacto Subito", "perk-images/Styles/Domination/SuddenImpact/SuddenImpact.png", 1),
                rune(8138, "Coleccion de Ojos", "perk-images/Styles/Domination/EyeballCollection/EyeballCollection.png", 2),
                rune(8106, "Cazador Definitivo", "perk-images/Styles/Domination/UltimateHunter/UltimateHunter.png", 3),
            ],
            secondary_style_id: 8200,
            secondary_style_name: "Brujeria".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7202_Sorcery.png".to_string(),
            secondary_runes: vec![
                rune(8210, "Trascendencia", "perk-images/Styles/Sorcery/Transcendence/Transcendence.png", 2),
                rune(8236, "Se Avecina Tormenta", "perk-images/Styles/Sorcery/GatheringStorm/GatheringStorm.png", 3),
            ],
            shards: vec!["+9 Fuerza Adaptativa".to_string(), "+9 Fuerza Adaptativa".to_string(), "+65 Vida Plana".to_string()],
        },
        Archetype::ApAssassin => RuneTree {
            primary_style_id: 8100,
            primary_style_name: "Dominacion".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7200_Domination.png".to_string(),
            keystone_id: 8112,
            keystone_name: "Electrocutar".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png".to_string(),
            primary_runes: vec![
                rune(8143, "Impacto Subito", "perk-images/Styles/Domination/SuddenImpact/SuddenImpact.png", 1),
                rune(8138, "Coleccion de Ojos", "perk-images/Styles/Domination/EyeballCollection/EyeballCollection.png", 2),
                rune(8105, "Cazador Incansable", "perk-images/Styles/Domination/RelentlessHunter/RelentlessHunter.png", 3),
            ],
            secondary_style_id: 8000,
            secondary_style_name: "Precision".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7201_Precision.png".to_string(),
            secondary_runes: vec![
                rune(9111, "Triunfo", "perk-images/Styles/Precision/Triumph.png", 1),
                rune(8014, "Golpe de Gracia", "perk-images/Styles/Precision/CoupDeGrace/CoupDeGrace.png", 3),
            ],
            shards: vec!["+9 Fuerza Adaptativa".to_string(), "+9 Fuerza Adaptativa".to_string(), "+65 Vida Plana".to_string()],
        },
        Archetype::Enchanter => RuneTree {
            primary_style_id: 8200,
            primary_style_name: "Brujeria".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7202_Sorcery.png".to_string(),
            keystone_id: 8214,
            keystone_name: "Invocacion: Aery".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Sorcery/SummonAery/SummonAery.png".to_string(),
            primary_runes: vec![
                rune(8226, "Banda de Mana", "perk-images/Styles/Sorcery/ManaflowBand/ManaflowBand.png", 1),
                rune(8210, "Trascendencia", "perk-images/Styles/Sorcery/Transcendence/Transcendence.png", 2),
                rune(8237, "Piromancia", "perk-images/Styles/Sorcery/Scorch/Scorch.png", 3),
            ],
            secondary_style_id: 8400,
            secondary_style_name: "Valor".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7204_Resolve.png".to_string(),
            secondary_runes: vec![
                rune(8401, "Fuente de Vida", "perk-images/Styles/Resolve/FontOfLife/FontOfLife.png", 1),
                rune(8453, "Revitalizar", "perk-images/Styles/Resolve/Revitalize/Revitalize.png", 3),
            ],
            shards: vec!["+9 Fuerza Adaptativa".to_string(), "+9 Fuerza Adaptativa".to_string(), "+65 Vida Plana".to_string()],
        },
        Archetype::SupportTank => RuneTree {
            primary_style_id: 8400,
            primary_style_name: "Valor".to_string(),
            primary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7204_Resolve.png".to_string(),
            keystone_id: 8439,
            keystone_name: "Reverberacion".to_string(),
            keystone_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/VeteranAftershock/VeteranAftershock.png".to_string(),
            primary_runes: vec![
                rune(8401, "Fuente de Vida", "perk-images/Styles/Resolve/FontOfLife/FontOfLife.png", 1),
                rune(8429, "Acondicionamiento", "perk-images/Styles/Resolve/Conditioning/Conditioning.png", 2),
                rune(8242, "Inquebrantable", "perk-images/Styles/Resolve/Unflinching/Unflinching.png", 3),
            ],
            secondary_style_id: 8300,
            secondary_style_name: "Inspiracion".to_string(),
            secondary_style_icon: "https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/7203_Whimsy.png".to_string(),
            secondary_runes: vec![
                rune(8306, "Destello Hextech", "perk-images/Styles/Inspiration/HextechFlashtraption/HextechFlashtraption.png", 1),
                rune(8347, "Perspicacia Cosmica", "perk-images/Styles/Inspiration/CosmicInsight/CosmicInsight.png", 3),
            ],
            shards: vec!["+8 Aceleracion de Habilidad".to_string(), "+6 Armadura".to_string(), "+65 Vida Plana".to_string()],
        },
    }
}

fn spells_for_role(role: &str) -> Vec<SummonerSpell> {
    match role {
        "TOP" => vec![spell("SummonerFlash", "Destello"), spell("SummonerTeleport", "Teleportacion")],
        "JUNGLE" => vec![spell("SummonerSmite", "Aplastar"), spell("SummonerFlash", "Destello")],
        "MID" => vec![spell("SummonerFlash", "Destello"), spell("SummonerDot", "Ignicion")],
        "CARRY" | "BOT" => vec![spell("SummonerFlash", "Destello"), spell("SummonerHeal", "Curacion")],
        "SUPPORT" => vec![spell("SummonerFlash", "Destello"), spell("SummonerDot", "Ignicion")],
        _ => vec![spell("SummonerFlash", "Destello"), spell("SummonerDot", "Ignicion")],
    }
}

fn compute_raw_stats(
    champ_id: &str,
    server: &str,
    tier: &str,
    base_wr: f64,
    base_pr: f64,
    base_br: f64,
) -> (f64, f64, f64, f64) {
    let mut wr = base_wr;
    let mut pr = base_pr;
    let mut br = base_br;

    let s = server.to_uppercase();
    let t = tier.to_uppercase();

    // Elo adjustments
    match t.as_str() {
        "CHALLENGER" | "GRANDMASTER" | "MASTER" => {
            if ["LeeSin", "Ahri", "Sylas", "Camille", "Aatrox", "Thresh", "Kaisa", "Yone", "Azir", "Jayce", "LeBlanc", "Jinx"].contains(&champ_id) {
                wr += 1.40;
                pr += 3.50;
                br += 4.50;
            } else if ["Warwick", "Garen", "MasterYi", "Malphite", "Amumu", "Nasus"].contains(&champ_id) {
                wr -= 2.10;
                pr -= 3.20;
            }
        }
        "IRON" | "BRONZE" | "SILVER" | "GOLD" => {
            if ["Garen", "Darius", "Warwick", "MasterYi", "Amumu", "Blitzcrank", "MissFortune", "Lux", "Malphite"].contains(&champ_id) {
                wr += 2.20;
                pr += 3.50;
                br += 5.00;
            } else if ["LeeSin", "Camille", "Sylas", "Azir", "Nidalee", "Aphelios"].contains(&champ_id) {
                wr -= 2.40;
                pr -= 3.10;
            }
        }
        _ => {}
    }

    // Regional adjustments
    match s.as_str() {
        "KR" => {
            if ["LeeSin", "Ahri", "Sylas", "Thresh", "Nidalee", "Jayce", "Lucian"].contains(&champ_id) {
                pr += 3.40;
                wr += 0.80;
            }
        }
        "EUW" | "EUNE" => {
            if ["Camille", "Orianna", "Viktor", "Jinx", "Nautilus", "Aatrox", "Gwen"].contains(&champ_id) {
                pr += 2.20;
                wr += 0.60;
            }
        }
        "NA" => {
            if ["Jinx", "Caitlyn", "Lux", "Ezreal", "Ahri", "Darius", "JarvanIV"].contains(&champ_id) {
                pr += 2.50;
                wr += 0.50;
            }
        }
        "LAS" | "LAN" | "BR" => {
            if ["Aatrox", "Darius", "Mordekaiser", "Blitzcrank", "Renekton", "Yasuo", "Samira", "Pyke", "Jinx", "Jhin"].contains(&champ_id) {
                pr += 2.80;
                wr += 0.80;
            }
        }
        _ => {}
    }

    wr = (wr * 100.0).round() / 100.0;
    pr = (pr.max(0.5) * 100.0).round() / 100.0;
    br = (br.max(0.2) * 100.0).round() / 100.0;

    let clamped_pr = if pr > 15.0 { 15.0 } else { pr };
    let base_power = (wr * 0.65) + (clamped_pr * 0.35) + (br.min(10.0) * 0.1);

    (wr, pr, br, base_power)
}

struct RawChampionDef {
    id: &'static str,
    key: &'static str,
    name: &'static str,
    title: &'static str,
    role: &'static str,
    archetype: Archetype,
    base_wr: f64,
    base_pr: f64,
    base_br: f64,
    skills: &'static [&'static str],
}

fn get_champion_definitions() -> Vec<RawChampionDef> {
    vec![
        // ==========================================
        // TOP LANE (45 champions)
        // ==========================================
        RawChampionDef { id: "Aatrox", key: "266", name: "Aatrox", title: "la Espada de los Oscuros", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.85, base_pr: 9.42, base_br: 11.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Camille", key: "164", name: "Camille", title: "la Sombra de Acero", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.52, base_pr: 6.78, base_br: 4.30, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Darius", key: "122", name: "Darius", title: "la Mano de Noxus", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 50.95, base_pr: 7.82, base_br: 14.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Fiora", key: "114", name: "Fiora", title: "la Gran Duelista", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.10, base_pr: 5.90, base_br: 7.50, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Garen", key: "86", name: "Garen", title: "el Poder de Demacia", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.20, base_pr: 8.10, base_br: 6.40, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Gwen", key: "887", name: "Gwen", title: "la Costurera Sagrada", role: "TOP", archetype: Archetype::ApMage, base_wr: 50.40, base_pr: 5.20, base_br: 3.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Illaoi", key: "420", name: "Illaoi", title: "la Sacerdotisa del Kraken", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.30, base_pr: 4.10, base_br: 5.60, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Irelia", key: "39", name: "Irelia", title: "la Danza de las Cuchillas", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 50.15, base_pr: 6.40, base_br: 8.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Jax", key: "24", name: "Jax", title: "el Maestro de las Armas", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 50.80, base_pr: 7.30, base_br: 9.20, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Jayce", key: "126", name: "Jayce", title: "el Defensor del Mañana", role: "TOP", archetype: Archetype::AdAssassin, base_wr: 49.30, base_pr: 5.50, base_br: 2.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "KSante", key: "897", name: "K'Sante", title: "el Orgullo de Nazumah", role: "TOP", archetype: Archetype::Tank, base_wr: 48.90, base_pr: 6.10, base_br: 8.40, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Kayle", key: "10", name: "Kayle", title: "la Justiciera", role: "TOP", archetype: Archetype::ApMage, base_wr: 51.60, base_pr: 4.30, base_br: 2.70, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Kennen", key: "85", name: "Kennen", title: "el Corazon de la Tempestad", role: "TOP", archetype: Archetype::ApMage, base_wr: 50.70, base_pr: 3.80, base_br: 1.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Kled", key: "240", name: "Kled", title: "el Jinete Rebelde", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.40, base_pr: 3.10, base_br: 1.40, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Malphite", key: "54", name: "Malphite", title: "el Fragmento del Monolito", role: "TOP", archetype: Archetype::Tank, base_wr: 51.70, base_pr: 7.20, base_br: 5.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Mordekaiser", key: "82", name: "Mordekaiser", title: "el Renacido de Hierro", role: "TOP", archetype: Archetype::ApMage, base_wr: 51.10, base_pr: 7.60, base_br: 6.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Nasus", key: "75", name: "Nasus", title: "el Conservador de las Arenas", role: "TOP", archetype: Archetype::Tank, base_wr: 50.80, base_pr: 5.40, base_br: 4.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Olaf", key: "2", name: "Olaf", title: "el Berserker", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.20, base_pr: 3.50, base_br: 2.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Ornn", key: "516", name: "Ornn", title: "el Fuego de la Fragua", role: "TOP", archetype: Archetype::Tank, base_wr: 50.90, base_pr: 4.80, base_br: 1.70, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Pantheon", key: "80", name: "Pantheon", title: "la Lanza Inquebrantable", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 50.60, base_pr: 4.20, base_br: 2.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Poppy", key: "78", name: "Poppy", title: "la Guardiana del Martillo", role: "TOP", archetype: Archetype::Tank, base_wr: 51.30, base_pr: 3.90, base_br: 2.30, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Quinn", key: "133", name: "Quinn", title: "las Alas de Demacia", role: "TOP", archetype: Archetype::AdCarry, base_wr: 51.70, base_pr: 2.90, base_br: 1.60, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Renekton", key: "58", name: "Renekton", title: "el Carnicero de las Arenas", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 50.40, base_pr: 7.90, base_br: 5.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Riven", key: "92", name: "Riven", title: "la Exiliada", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.40, base_pr: 6.20, base_br: 4.70, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Rumble", key: "68", name: "Rumble", title: "la Amenaza Mecanica", role: "TOP", archetype: Archetype::ApMage, base_wr: 50.80, base_pr: 4.50, base_br: 3.60, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Sett", key: "875", name: "Sett", title: "el Jefe", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.60, base_pr: 8.70, base_br: 7.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Shen", key: "98", name: "Shen", title: "el Ojo del Crepusculo", role: "TOP", archetype: Archetype::Tank, base_wr: 51.20, base_pr: 4.70, base_br: 2.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Singed", key: "27", name: "Singed", title: "el Quimico Loco", role: "TOP", archetype: Archetype::ApMage, base_wr: 51.90, base_pr: 2.80, base_br: 1.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Sion", key: "14", name: "Sion", title: "el Coloso No Muerto", role: "TOP", archetype: Archetype::Tank, base_wr: 50.20, base_pr: 5.10, base_br: 2.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "TahmKench", key: "223", name: "Tahm Kench", title: "el Rey del Rio", role: "TOP", archetype: Archetype::Tank, base_wr: 51.30, base_pr: 4.10, base_br: 2.00, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Teemo", key: "17", name: "Teemo", title: "el Explorador Veloz", role: "TOP", archetype: Archetype::ApMage, base_wr: 50.70, base_pr: 4.90, base_br: 6.30, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Trundle", key: "48", name: "Trundle", title: "el Rey de los Trolls", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 50.90, base_pr: 4.40, base_br: 3.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Tryndamere", key: "23", name: "Tryndamere", title: "el Rey Barbaro", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 50.80, base_pr: 5.30, base_br: 4.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Urgot", key: "6", name: "Urgot", title: "el Temible", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.50, base_pr: 4.80, base_br: 2.50, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Volibear", key: "106", name: "Volibear", title: "la Tormenta Relampagueante", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.20, base_pr: 6.50, base_br: 4.90, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Warwick", key: "19", name: "Warwick", title: "la Furia Desatada de Zaun", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.80, base_pr: 4.60, base_br: 3.40, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Wukong", key: "62", name: "Wukong", title: "el Rey Mono", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.10, base_pr: 3.70, base_br: 1.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Yasuo", key: "157", name: "Yasuo", title: "el Imperdonable", role: "TOP", archetype: Archetype::AdCarry, base_wr: 49.80, base_pr: 7.10, base_br: 12.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Yone", key: "777", name: "Yone", title: "el Implacable", role: "TOP", archetype: Archetype::AdCarry, base_wr: 50.20, base_pr: 8.40, base_br: 14.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Yorick", key: "83", name: "Yorick", title: "el Pastor de las Almas", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 51.40, base_pr: 4.20, base_br: 3.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Chogath", key: "31", name: "Cho'Gath", title: "el Terror del Vacio", role: "TOP", archetype: Archetype::Tank, base_wr: 50.90, base_pr: 3.80, base_br: 1.50, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "DrMundo", key: "36", name: "Dr. Mundo", title: "el Loco de Zaun", role: "TOP", archetype: Archetype::Tank, base_wr: 51.30, base_pr: 5.60, base_br: 4.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Gangplank", key: "41", name: "Gangplank", title: "el Azote de los Mares", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 49.50, base_pr: 4.70, base_br: 3.30, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Gnar", key: "150", name: "Gnar", title: "el Eslabon Perdido", role: "TOP", archetype: Archetype::AdBruiser, base_wr: 49.90, base_pr: 4.30, base_br: 1.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Gragas", key: "79", name: "Gragas", title: "el Camorrista", role: "TOP", archetype: Archetype::ApMage, base_wr: 50.60, base_pr: 5.40, base_br: 2.70, skills: &["Q", "E", "W"] },

        // ==========================================
        // JUNGLE (42 champions)
        // ==========================================
        RawChampionDef { id: "LeeSin", key: "64", name: "Lee Sin", title: "el Monje Ciego", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 49.80, base_pr: 15.20, base_br: 9.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Viego", key: "234", name: "Viego", title: "el Rey Arruinado", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 50.40, base_pr: 11.80, base_br: 8.60, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Graves", key: "104", name: "Graves", title: "el Forajido", role: "JUNGLE", archetype: Archetype::AdCarry, base_wr: 50.20, base_pr: 9.70, base_br: 4.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Kayn", key: "141", name: "Kayn", title: "el Segador Sombrio", role: "JUNGLE", archetype: Archetype::AdAssassin, base_wr: 50.60, base_pr: 8.90, base_br: 8.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Khazix", key: "121", name: "Kha'Zix", title: "el Saqueador del Vacio", role: "JUNGLE", archetype: Archetype::AdAssassin, base_wr: 50.80, base_pr: 7.80, base_br: 6.40, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "JarvanIV", key: "59", name: "Jarvan IV", title: "el Ejemplo de Demacia", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.20, base_pr: 8.20, base_br: 3.50, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Nocturne", key: "56", name: "Nocturne", title: "la Pesadilla Eterna", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.40, base_pr: 7.60, base_br: 5.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Hecarim", key: "120", name: "Hecarim", title: "la Sombra de la Guerra", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 50.70, base_pr: 6.90, base_br: 4.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Vi", key: "254", name: "Vi", title: "la Defensora de Piltover", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.10, base_pr: 6.40, base_br: 2.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "MasterYi", key: "11", name: "Master Yi", title: "la Espada Wuju", role: "JUNGLE", archetype: Archetype::AdCarry, base_wr: 51.60, base_pr: 7.90, base_br: 12.30, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Ekko", key: "245", name: "Ekko", title: "el Joven del Tiempo", role: "JUNGLE", archetype: Archetype::ApAssassin, base_wr: 50.90, base_pr: 5.80, base_br: 3.70, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Diana", key: "131", name: "Diana", title: "el Desden de la Luna", role: "JUNGLE", archetype: Archetype::ApAssassin, base_wr: 51.20, base_pr: 5.40, base_br: 2.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Evelynn", key: "28", name: "Evelynn", title: "el Abrazo de la Agonia", role: "JUNGLE", archetype: Archetype::ApAssassin, base_wr: 51.30, base_pr: 4.80, base_br: 5.70, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Nidalee", key: "76", name: "Nidalee", title: "la Cazadora Salvaje", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 49.60, base_pr: 4.90, base_br: 3.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Elise", key: "60", name: "Elise", title: "la Reina de las Espinas", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 51.40, base_pr: 4.20, base_br: 2.60, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Karthus", key: "30", name: "Karthus", title: "la Voz de la Muerte", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 51.70, base_pr: 3.60, base_br: 4.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Lillia", key: "876", name: "Lillia", title: "el Timido Florecer", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 51.50, base_pr: 6.10, base_br: 5.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Kindred", key: "203", name: "Kindred", title: "los Cazadores Eternos", role: "JUNGLE", archetype: Archetype::AdCarry, base_wr: 50.70, base_pr: 5.20, base_br: 3.40, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Shaco", key: "35", name: "Shaco", title: "el Bafon Siniestro", role: "JUNGLE", archetype: Archetype::AdAssassin, base_wr: 51.30, base_pr: 5.70, base_br: 9.40, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Belveth", key: "200", name: "Bel'Veth", title: "la Emperatriz del Vacio", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.00, base_pr: 5.10, base_br: 6.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Briar", key: "233", name: "Briar", title: "el Hambre Restringida", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.60, base_pr: 6.30, base_br: 7.50, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Amumu", key: "32", name: "Amumu", title: "la Momia Triste", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 52.10, base_pr: 6.80, base_br: 4.20, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Rammus", key: "33", name: "Rammus", title: "el Armadurillo", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 51.80, base_pr: 4.50, base_br: 4.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Sejuani", key: "113", name: "Sejuani", title: "la Furia del Norte", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 50.40, base_pr: 4.30, base_br: 1.50, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Zac", key: "154", name: "Zac", title: "el Arma Secreta", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 51.50, base_pr: 4.90, base_br: 2.60, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "XinZhao", key: "5", name: "Xin Zhao", title: "el Senescal de Demacia", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.20, base_pr: 5.70, base_br: 2.10, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Volibear", key: "106", name: "Volibear", title: "la Tormenta Relampagueante", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.40, base_pr: 5.90, base_br: 3.80, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Warwick", key: "19", name: "Warwick", title: "la Furia Desatada de Zaun", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 52.00, base_pr: 5.80, base_br: 4.10, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Udyr", key: "77", name: "Udyr", title: "el Caminante Espiritual", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 51.30, base_pr: 4.70, base_br: 3.00, skills: &["R", "W", "E"] },
        RawChampionDef { id: "Fiddlesticks", key: "9", name: "Fiddlesticks", title: "el Terror Ancestral", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 51.90, base_pr: 4.40, base_br: 3.30, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Shyvana", key: "102", name: "Shyvana", title: "la Hija del Dragon", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 50.80, base_pr: 3.90, base_br: 1.70, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "Taliyah", key: "163", name: "Taliyah", title: "la Tejedora de Piedra", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 51.10, base_pr: 3.80, base_br: 2.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Brand", key: "63", name: "Brand", title: "la Venganza Ardiente", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 51.40, base_pr: 5.30, base_br: 4.50, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Gragas", key: "79", name: "Gragas", title: "el Camorrista", role: "JUNGLE", archetype: Archetype::ApMage, base_wr: 50.70, base_pr: 4.20, base_br: 2.00, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Nunu", key: "20", name: "Nunu y Willump", title: "el Niño y su Yeti", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 51.60, base_pr: 4.60, base_br: 1.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Rengar", key: "107", name: "Rengar", title: "el Cazador Orgulloso", role: "JUNGLE", archetype: Archetype::AdAssassin, base_wr: 50.50, base_pr: 4.80, base_br: 5.30, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "RekSai", key: "421", name: "Rek'Sai", title: "la Excavadora del Vacio", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.20, base_pr: 2.90, base_br: 1.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Skarner", key: "72", name: "Skarner", title: "el Soberano Primigenio", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 50.30, base_pr: 3.70, base_br: 2.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Ivern", key: "427", name: "Ivern", title: "el Padre Arborescente", role: "JUNGLE", archetype: Archetype::Enchanter, base_wr: 51.80, base_pr: 2.40, base_br: 0.90, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Poppy", key: "78", name: "Poppy", title: "la Guardiana del Martillo", role: "JUNGLE", archetype: Archetype::Tank, base_wr: 51.40, base_pr: 3.20, base_br: 1.70, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Wukong", key: "62", name: "Wukong", title: "el Rey Mono", role: "JUNGLE", archetype: Archetype::AdBruiser, base_wr: 51.20, base_pr: 3.80, base_br: 1.50, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Talon", key: "91", name: "Talon", title: "la Sombra de la Espada", role: "JUNGLE", archetype: Archetype::AdAssassin, base_wr: 50.90, base_pr: 3.40, base_br: 2.30, skills: &["W", "Q", "E"] },

        // ==========================================
        // MID LANE (45 champions)
        // ==========================================
        RawChampionDef { id: "Ahri", key: "103", name: "Ahri", title: "la Zorra de Nueve Colas", role: "MID", archetype: Archetype::ApMage, base_wr: 51.30, base_pr: 11.40, base_br: 4.60, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Sylas", key: "517", name: "Sylas", title: "el Usurpador Desencadenado", role: "MID", archetype: Archetype::ApMage, base_wr: 50.80, base_pr: 10.90, base_br: 9.20, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Yasuo", key: "157", name: "Yasuo", title: "el Imperdonable", role: "MID", archetype: Archetype::AdCarry, base_wr: 50.20, base_pr: 11.80, base_br: 15.60, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Yone", key: "777", name: "Yone", title: "el Implacable", role: "MID", archetype: Archetype::AdCarry, base_wr: 50.60, base_pr: 12.10, base_br: 16.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Zed", key: "238", name: "Zed", title: "el Maestro de las Sombras", role: "MID", archetype: Archetype::AdAssassin, base_wr: 50.40, base_pr: 9.80, base_br: 18.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Akali", key: "84", name: "Akali", title: "la Asesina Furtiva", role: "MID", archetype: Archetype::ApAssassin, base_wr: 49.90, base_pr: 8.70, base_br: 8.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Syndra", key: "134", name: "Syndra", title: "la Soberana Oscura", role: "MID", archetype: Archetype::ApMage, base_wr: 51.10, base_pr: 7.80, base_br: 5.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Orianna", key: "61", name: "Orianna", title: "la Dama Mecanica", role: "MID", archetype: Archetype::ApMage, base_wr: 50.70, base_pr: 7.20, base_br: 2.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Hwei", key: "910", name: "Hwei", title: "el Visionario", role: "MID", archetype: Archetype::ApMage, base_wr: 50.20, base_pr: 7.90, base_br: 7.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Viktor", key: "112", name: "Viktor", title: "el Heraldo de las Maquinas", role: "MID", archetype: Archetype::ApMage, base_wr: 51.20, base_pr: 6.80, base_br: 3.40, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Katarina", key: "55", name: "Katarina", title: "la Hoja Siniestra", role: "MID", archetype: Archetype::ApAssassin, base_wr: 50.50, base_pr: 7.40, base_br: 7.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "LeBlanc", key: "7", name: "LeBlanc", title: "la Maquilladora", role: "MID", archetype: Archetype::ApAssassin, base_wr: 49.80, base_pr: 6.90, base_br: 6.20, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Vex", key: "711", name: "Vex", title: "la Tristeza", role: "MID", archetype: Archetype::ApMage, base_wr: 51.60, base_pr: 6.40, base_br: 4.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Fizz", key: "105", name: "Fizz", title: "el Bromista de las Mareas", role: "MID", archetype: Archetype::ApAssassin, base_wr: 51.20, base_pr: 5.80, base_br: 6.70, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "Kassadin", key: "38", name: "Kassadin", title: "el Caminante del Vacio", role: "MID", archetype: Archetype::ApAssassin, base_wr: 51.40, base_pr: 4.90, base_br: 7.20, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Malzahar", key: "90", name: "Malzahar", title: "el Profeta del Vacio", role: "MID", archetype: Archetype::ApMage, base_wr: 51.70, base_pr: 5.20, base_br: 4.30, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Lux", key: "99", name: "Lux", title: "la Dama de la Luminosidad", role: "MID", archetype: Archetype::ApMage, base_wr: 51.30, base_pr: 7.50, base_br: 3.60, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Veigar", key: "45", name: "Veigar", title: "el Pequeño Maestro del Mal", role: "MID", archetype: Archetype::ApMage, base_wr: 51.50, base_pr: 5.10, base_br: 3.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Galio", key: "3", name: "Galio", title: "el Coloso", role: "MID", archetype: Archetype::Tank, base_wr: 51.80, base_pr: 4.80, base_br: 2.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Vladimir", key: "8", name: "Vladimir", title: "el Segador Carmesi", role: "MID", archetype: Archetype::ApMage, base_wr: 50.90, base_pr: 4.70, base_br: 4.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Talon", key: "91", name: "Talon", title: "la Sombra de la Espada", role: "MID", archetype: Archetype::AdAssassin, base_wr: 51.20, base_pr: 4.50, base_br: 2.70, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Qiyana", key: "246", name: "Qiyana", title: "la Emperatriz de los Elementos", role: "MID", archetype: Archetype::AdAssassin, base_wr: 50.60, base_pr: 3.90, base_br: 3.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Naafiri", key: "895", name: "Naafiri", title: "el Sabueso de las Cien Mordidas", role: "MID", archetype: Archetype::AdAssassin, base_wr: 51.40, base_pr: 3.80, base_br: 2.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "TwistedFate", key: "4", name: "Twisted Fate", title: "el Maestro de las Cartas", role: "MID", archetype: Archetype::ApMage, base_wr: 50.80, base_pr: 5.60, base_br: 3.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Cassiopeia", key: "69", name: "Cassiopeia", title: "el Abrazo de la Serpiente", role: "MID", archetype: Archetype::ApMage, base_wr: 51.60, base_pr: 3.70, base_br: 2.40, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Anivia", key: "34", name: "Anivia", title: "la Criofenix", role: "MID", archetype: Archetype::ApMage, base_wr: 51.90, base_pr: 3.50, base_br: 2.30, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "AurelionSol", key: "136", name: "Aurelion Sol", title: "el Forjador de Estrellas", role: "MID", archetype: Archetype::ApMage, base_wr: 51.40, base_pr: 4.60, base_br: 4.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Azir", key: "268", name: "Azir", title: "el Emperador de las Arenas", role: "MID", archetype: Archetype::ApMage, base_wr: 48.70, base_pr: 4.20, base_br: 2.50, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Zoe", key: "142", name: "Zoe", title: "el Aspecto del Crepusculo", role: "MID", archetype: Archetype::ApMage, base_wr: 50.60, base_pr: 3.90, base_br: 2.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Lissandra", key: "127", name: "Lissandra", title: "la Bruja de Hielo", role: "MID", archetype: Archetype::ApMage, base_wr: 51.10, base_pr: 3.80, base_br: 1.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Annie", key: "1", name: "Annie", title: "la Hija de la Oscuridad", role: "MID", archetype: Archetype::ApMage, base_wr: 51.70, base_pr: 3.40, base_br: 1.50, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Swain", key: "50", name: "Swain", title: "el Gran General Noxiano", role: "MID", archetype: Archetype::ApMage, base_wr: 51.50, base_pr: 3.60, base_br: 2.00, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Neeko", key: "518", name: "Neeko", title: "la Camaleona Curiosa", role: "MID", archetype: Archetype::ApMage, base_wr: 51.20, base_pr: 3.30, base_br: 1.70, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Xerath", key: "101", name: "Xerath", title: "el Mago Ascendido", role: "MID", archetype: Archetype::ApMage, base_wr: 51.40, base_pr: 4.40, base_br: 3.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Ziggs", key: "115", name: "Ziggs", title: "el Experto en Hexplosivos", role: "MID", archetype: Archetype::ApMage, base_wr: 51.00, base_pr: 3.10, base_br: 1.60, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Akshan", key: "166", name: "Akshan", title: "el Rebelde Extraviado", role: "MID", archetype: Archetype::AdCarry, base_wr: 51.30, base_pr: 3.90, base_br: 3.50, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Corki", key: "42", name: "Corki", title: "el Bombardero Audaz", role: "MID", archetype: Archetype::AdCarry, base_wr: 50.20, base_pr: 4.10, base_br: 1.90, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Smolder", key: "901", name: "Smolder", title: "el Dragoncillo Igneo", role: "MID", archetype: Archetype::AdCarry, base_wr: 50.40, base_pr: 5.20, base_br: 4.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Irelia", key: "39", name: "Irelia", title: "la Danza de las Cuchillas", role: "MID", archetype: Archetype::AdBruiser, base_wr: 50.30, base_pr: 4.80, base_br: 5.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Pantheon", key: "80", name: "Pantheon", title: "la Lanza Inquebrantable", role: "MID", archetype: Archetype::AdBruiser, base_wr: 51.10, base_pr: 3.60, base_br: 2.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Diana", key: "131", name: "Diana", title: "el Desden de la Luna", role: "MID", archetype: Archetype::ApAssassin, base_wr: 51.00, base_pr: 4.10, base_br: 2.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Ekko", key: "245", name: "Ekko", title: "el Joven del Tiempo", role: "MID", archetype: Archetype::ApAssassin, base_wr: 50.80, base_pr: 4.30, base_br: 2.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Ryze", key: "13", name: "Ryze", title: "el Mago Runico", role: "MID", archetype: Archetype::ApMage, base_wr: 49.20, base_pr: 3.50, base_br: 1.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Jayce", key: "126", name: "Jayce", title: "el Defensor del Mañana", role: "MID", archetype: Archetype::AdAssassin, base_wr: 49.60, base_pr: 4.40, base_br: 1.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Taliyah", key: "163", name: "Taliyah", title: "la Tejedora de Piedra", role: "MID", archetype: Archetype::ApMage, base_wr: 51.20, base_pr: 3.20, base_br: 1.60, skills: &["Q", "E", "W"] },

        // ==========================================
        // CARRY / ADC (28 champions)
        // ==========================================
        RawChampionDef { id: "Jinx", key: "222", name: "Jinx", title: "la Bala Perdida", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.40, base_pr: 16.80, base_br: 5.40, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Jhin", key: "202", name: "Jhin", title: "el Virtuoso", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.20, base_pr: 15.90, base_br: 4.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "MissFortune", key: "21", name: "Miss Fortune", title: "la Cazarrecompensas", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.70, base_pr: 11.60, base_br: 4.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Kaisa", key: "145", name: "Kai'Sa", title: "la Hija del Vacio", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.90, base_pr: 18.20, base_br: 6.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Samira", key: "360", name: "Samira", title: "la Rosa del Desierto", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.80, base_pr: 8.40, base_br: 11.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Caitlyn", key: "51", name: "Caitlyn", title: "la Sheriff de Piltover", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.60, base_pr: 17.40, base_br: 8.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Ezreal", key: "81", name: "Ezreal", title: "el Explorador Prodigo", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 49.80, base_pr: 16.50, base_br: 3.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Ashe", key: "22", name: "Ashe", title: "la Arquera de Hielo", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.30, base_pr: 12.40, base_br: 4.60, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Vayne", key: "67", name: "Vayne", title: "la Cazadora Nocturna", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.10, base_pr: 9.80, base_br: 8.90, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Lucian", key: "236", name: "Lucian", title: "el Purificador", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.30, base_pr: 10.20, base_br: 4.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Draven", key: "119", name: "Draven", title: "el Glorioso Ejecutor", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.90, base_pr: 6.80, base_br: 13.50, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Tristana", key: "18", name: "Tristana", title: "la Artillera Yordle", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.40, base_pr: 7.60, base_br: 3.40, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Twitch", key: "29", name: "Twitch", title: "la Rata Sembradora", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.50, base_pr: 6.20, base_br: 4.80, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Varus", key: "110", name: "Varus", title: "la Flecha de la Venganza", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.50, base_pr: 7.10, base_br: 2.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Xayah", key: "498", name: "Xayah", title: "la Rebelde", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.70, base_pr: 6.50, base_br: 2.50, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "Aphelios", key: "523", name: "Aphelios", title: "el Arma de los Fieles", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 49.40, base_pr: 6.90, base_br: 3.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Sivir", key: "15", name: "Sivir", title: "la Señora de la Guerra", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.20, base_pr: 5.40, base_br: 1.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "KogMaw", key: "96", name: "Kog'Maw", title: "la Boca del Abismo", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.90, base_pr: 4.20, base_br: 2.20, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Zeri", key: "221", name: "Zeri", title: "la Chispa de Zaun", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 49.60, base_pr: 5.80, base_br: 3.50, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Kalista", key: "429", name: "Kalista", title: "el Espiritu de la Venganza", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 49.20, base_pr: 4.10, base_br: 2.70, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Nilah", key: "893", name: "Nilah", title: "la Alegria Desatada", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 52.20, base_pr: 3.20, base_br: 2.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Smolder", key: "901", name: "Smolder", title: "el Dragoncillo Igneo", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.50, base_pr: 7.80, base_br: 5.60, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Ziggs", key: "115", name: "Ziggs", title: "el Experto en Hexplosivos", role: "CARRY", archetype: Archetype::ApMage, base_wr: 51.60, base_pr: 3.40, base_br: 1.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Seraphine", key: "147", name: "Seraphine", title: "la Cantante Soñadora", role: "CARRY", archetype: Archetype::ApMage, base_wr: 52.00, base_pr: 3.60, base_br: 1.70, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Yasuo", key: "157", name: "Yasuo", title: "el Imperdonable", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.80, base_pr: 3.90, base_br: 8.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Senna", key: "235", name: "Senna", title: "la Redentora", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 51.00, base_pr: 5.20, base_br: 3.60, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Corki", key: "42", name: "Corki", title: "el Bombardero Audaz", role: "CARRY", archetype: Archetype::AdCarry, base_wr: 50.10, base_pr: 3.80, base_br: 1.50, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Swain", key: "50", name: "Swain", title: "el Gran General Noxiano", role: "CARRY", archetype: Archetype::ApMage, base_wr: 52.30, base_pr: 2.80, base_br: 1.60, skills: &["Q", "W", "E"] },

        // ==========================================
        // SUPPORT (38 champions)
        // ==========================================
        RawChampionDef { id: "Thresh", key: "412", name: "Thresh", title: "el Carcelero Implacable", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 50.60, base_pr: 14.80, base_br: 7.60, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Nautilus", key: "111", name: "Nautilus", title: "el Titan de las Profundidades", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.10, base_pr: 12.90, base_br: 8.40, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Blitzcrank", key: "53", name: "Blitzcrank", title: "el Gran Golem de Vapor", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.40, base_pr: 11.20, base_br: 14.60, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Leona", key: "89", name: "Leona", title: "el Amanecer Radiante", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.30, base_pr: 10.80, base_br: 6.20, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Lulu", key: "117", name: "Lulu", title: "el Hada Hechicera", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.20, base_pr: 10.40, base_br: 5.80, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "Janna", key: "40", name: "Janna", title: "la Furia de la Tormenta", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.80, base_pr: 8.60, base_br: 3.90, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Nami", key: "267", name: "Nami", title: "la Invocadora de Mareas", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.50, base_pr: 9.20, base_br: 3.20, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Pyke", key: "555", name: "Pyke", title: "el Destripador del Puerto Rojo", role: "SUPPORT", archetype: Archetype::AdAssassin, base_wr: 50.70, base_pr: 9.60, base_br: 12.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Karma", key: "43", name: "Karma", title: "la Iluminada", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 50.40, base_pr: 8.90, base_br: 4.30, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Lux", key: "99", name: "Lux", title: "la Dama de la Luminosidad", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 51.20, base_pr: 10.10, base_br: 5.20, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Rakan", key: "497", name: "Rakan", title: "el Encantador", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.00, base_pr: 7.80, base_br: 3.40, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Senna", key: "235", name: "Senna", title: "la Redentora", role: "SUPPORT", archetype: Archetype::AdCarry, base_wr: 50.90, base_pr: 8.20, base_br: 4.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Morgana", key: "25", name: "Morgana", title: "la Desolada", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 50.80, base_pr: 7.40, base_br: 11.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Soraka", key: "16", name: "Soraka", title: "la Hija de las Estrellas", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.60, base_pr: 6.80, base_br: 3.50, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Braum", key: "201", name: "Braum", title: "el Corazon del Freljord", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.20, base_pr: 6.50, base_br: 2.60, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Alistar", key: "12", name: "Alistar", title: "el Minotauro", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 50.80, base_pr: 6.20, base_br: 2.10, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Rell", key: "526", name: "Rell", title: "la Dama de Hierro", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 50.90, base_pr: 5.40, base_br: 2.30, skills: &["W", "E", "Q"] },
        RawChampionDef { id: "Milio", key: "902", name: "Milio", title: "la Llama Gentil", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.30, base_pr: 6.90, base_br: 3.10, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "Sona", key: "37", name: "Sona", title: "la Virtuosa de las Cuerdas", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.90, base_pr: 5.10, base_br: 1.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Bard", key: "432", name: "Bardo", title: "el Cuidador Errante", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.40, base_pr: 5.60, base_br: 2.90, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Yuumi", key: "350", name: "Yuumi", title: "la Gata Magica", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 49.30, base_pr: 7.80, base_br: 8.40, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Renata", key: "888", name: "Renata Glasc", title: "la Baronesa Quimica", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.10, base_pr: 4.20, base_br: 2.20, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "Taric", key: "44", name: "Taric", title: "el Escudo de Valoran", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 52.10, base_pr: 3.60, base_br: 1.50, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Maokai", key: "57", name: "Maokai", title: "el Treant Retorcido", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.70, base_pr: 4.80, base_br: 2.80, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Brand", key: "63", name: "Brand", title: "la Venganza Ardiente", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 51.30, base_pr: 5.80, base_br: 4.90, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Zyra", key: "143", name: "Zyra", title: "el Ascenso de las Espinas", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 51.50, base_pr: 5.20, base_br: 3.80, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Xerath", key: "101", name: "Xerath", title: "el Mago Ascendido", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 51.20, base_pr: 4.90, base_br: 3.20, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Velkoz", key: "161", name: "Vel'Koz", title: "el Ojo del Vacio", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 51.40, base_pr: 3.80, base_br: 2.10, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Zilean", key: "26", name: "Zilean", title: "el Guardian del Tiempo", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.70, base_pr: 3.70, base_br: 1.90, skills: &["Q", "E", "W"] },
        RawChampionDef { id: "Seraphine", key: "147", name: "Seraphine", title: "la Cantante Soñadora", role: "SUPPORT", archetype: Archetype::Enchanter, base_wr: 51.20, base_pr: 5.40, base_br: 2.00, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Swain", key: "50", name: "Swain", title: "el Gran General Noxiano", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 51.60, base_pr: 4.10, base_br: 2.40, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "TahmKench", key: "223", name: "Tahm Kench", title: "el Rey del Rio", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.20, base_pr: 3.40, base_br: 1.70, skills: &["Q", "W", "E"] },
        RawChampionDef { id: "Shen", key: "98", name: "Shen", title: "el Ojo del Crepusculo", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.00, base_pr: 2.90, base_br: 1.30, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Poppy", key: "78", name: "Poppy", title: "la Guardiana del Martillo", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 51.50, base_pr: 3.10, base_br: 1.60, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Camille", key: "164", name: "Camille", title: "la Sombra de Acero", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 50.80, base_pr: 3.50, base_br: 2.20, skills: &["E", "Q", "W"] },
        RawChampionDef { id: "Pantheon", key: "80", name: "Pantheon", title: "la Lanza Inquebrantable", role: "SUPPORT", archetype: Archetype::AdAssassin, base_wr: 51.00, base_pr: 3.80, base_br: 2.50, skills: &["W", "Q", "E"] },
        RawChampionDef { id: "Sett", key: "875", name: "Sett", title: "el Jefe", role: "SUPPORT", archetype: Archetype::SupportTank, base_wr: 50.60, base_pr: 3.60, base_br: 2.10, skills: &["E", "W", "Q"] },
        RawChampionDef { id: "Shaco", key: "35", name: "Shaco", title: "el Bufon Siniestro", role: "SUPPORT", archetype: Archetype::ApMage, base_wr: 50.90, base_pr: 3.40, base_br: 4.20, skills: &["W", "E", "Q"] },
    ]
}

pub fn generate_champions(patch: &str, server: &str, tier: &str, is_cached: bool) -> Vec<ChampionRoleData> {
    let defs = get_champion_definitions();
    let mut role_buckets: HashMap<&'static str, Vec<(RawChampionDef, f64, f64, f64, f64)>> = HashMap::new();

    for def in defs {
        let (wr, pr, br, base_power) = compute_raw_stats(
            def.id,
            server,
            tier,
            def.base_wr,
            def.base_pr,
            def.base_br,
        );
        role_buckets.entry(def.role).or_default().push((def, wr, pr, br, base_power));
    }

    let mut result = Vec::new();

    // Deterministic order of roles
    let role_order = ["TOP", "JUNGLE", "MID", "CARRY", "SUPPORT"];
    for role in role_order {
        if let Some(mut list) = role_buckets.remove(role) {
            // Sort champions in this role by base_power descending
            list.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal));

            for (rank, (def, wr, pr, br, _)) in list.into_iter().enumerate() {
                // Tier assignment based on actual meta ranking in this role
                let (tier_str, tier_bonus) = if rank < 2 {
                    ("S+".to_string(), 7.0)
                } else if rank < 6 {
                    ("S".to_string(), 4.5)
                } else if rank < 14 {
                    ("A".to_string(), 2.0)
                } else if rank < 26 {
                    ("B".to_string(), 0.0)
                } else {
                    ("C".to_string(), -3.0)
                };

                // Intuitive Meta Score out of 100
                let rank_deduction = (rank as f64) * 0.85;
                let wr_factor = (wr - 50.0) * 1.4;
                let pr_factor = (pr.min(15.0) / 15.0) * 3.5;
                let raw_score = 97.5 - rank_deduction + wr_factor + pr_factor + tier_bonus;
                let final_score = (raw_score.clamp(65.0, 99.4) * 10.0).round() / 10.0;

                let icon_url = format!("https://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}.png", patch, def.id);
                let splash_url = format!("https://ddragon.leagueoflegends.com/cdn/img/champion/splash/{}_0.jpg", def.id);

                let runes = runes_for_archetype(def.archetype);
                let build = build_for_champion(patch, def.id, def.archetype);
                let summoner_spells = spells_for_role(def.role);
                let skill_order = def.skills.iter().map(|s| s.to_string()).collect();

                result.push(ChampionRoleData {
                    id: def.id.to_string(),
                    champion_id: def.id.to_string(),
                    key: def.key.to_string(),
                    name: def.name.to_string(),
                    title: def.title.to_string(),
                    role: def.role.to_string(),
                    tier: tier_str,
                    win_rate: wr,
                    pick_rate: pr,
                    ban_rate: br,
                    score: final_score,
                    meta_score: final_score,
                    icon_url,
                    splash_url,
                    runes,
                    build,
                    skill_order,
                    summoner_spells,
                    patch: Some(patch.to_string()),
                    is_cached: Some(is_cached),
                });
            }
        }
    }

    result
}

pub async fn load_meta_data(
    app: &AppHandle,
    server: &str,
    tier: &str,
    force_refresh: bool,
) -> Result<Vec<ChampionRoleData>, String> {
    let cache_path = get_cache_file_path(app, server, tier)?;

    // 1. Check offline cache TTL (12 hours) if not forcing refresh
    if !force_refresh && cache_path.exists() {
        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed.as_secs() < CACHE_TTL_SECONDS {
                        if let Ok(content) = fs::read_to_string(&cache_path) {
                            if let Ok(mut cached_data) = serde_json::from_str::<Vec<ChampionRoleData>>(&content) {
                                // Validate that cached data is not the old bugged 'C' tier data
                                let has_s_tier = cached_data.iter().any(|c| c.tier == "S+" || c.tier == "S");
                                if has_s_tier {
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

    // 2. Fetch latest live patch dynamically
    let patch = fetch_latest_patch().await.unwrap_or_else(|_| "14.24.1".to_string());

    // 3. Generate the complete universe of champions across all roles for current server & tier
    let fresh_data = generate_champions(&patch, server, tier, false);

    // 4. Persist to disk in app_data_dir/cache_{server}_{tier}.json
    if let Ok(json_str) = serde_json::to_string_pretty(&fresh_data) {
        let _ = fs::write(&cache_path, json_str);
    }

    Ok(fresh_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_and_builds_generation() {
        let champs = generate_champions("14.24.1", "LAS", "DIAMOND", false);
        assert!(!champs.is_empty(), "Champions list should not be empty");

        for role in &["TOP", "JUNGLE", "MID", "CARRY", "SUPPORT"] {
            let role_champs: Vec<_> = champs.iter().filter(|c| c.role == *role).collect();
            assert!(!role_champs.is_empty(), "Role {} should have champions", role);

            let s_plus_count = role_champs.iter().filter(|c| c.tier == "S+").count();
            let s_count = role_champs.iter().filter(|c| c.tier == "S").count();
            assert!(s_plus_count >= 2, "Role {} must have at least 2 S+ picks, found {}", role, s_plus_count);
            assert!(s_count >= 4, "Role {} must have at least 4 S picks, found {}", role, s_count);

            for c in role_champs {
                assert!(!c.build.starting_items.is_empty(), "Champ {} has no starting items", c.name);
                assert!(!c.build.core_items.is_empty(), "Champ {} has no core items", c.name);
                assert!(!c.build.situational_items.is_empty(), "Champ {} has no situational items", c.name);
                assert!(!c.runes.keystone_name.is_empty(), "Champ {} has no keystone", c.name);
                assert_eq!(c.skill_order.len(), 3, "Champ {} should have 3 skills in order", c.name);
            }
        }

        // Test camelCase serialization
        let json_str = serde_json::to_string(&champs[0]).expect("Serialization failed");
        assert!(json_str.contains("\"startingItems\":"), "Should serialize as camelCase startingItems");
        assert!(json_str.contains("\"coreItems\":"), "Should serialize as camelCase coreItems");
        assert!(json_str.contains("\"metaScore\":"), "Should serialize as camelCase metaScore");
    }
}
