use pitwall::SessionInfo;

use crate::audio::SessionMeta;

/// Extract stable session facts for the race engineer from iRacing session YAML.
pub fn build_session_meta(session: &SessionInfo) -> SessionMeta {
    let mut meta = SessionMeta::default();

    let _weekend = &session.weekend_info;

    let Some(driver_info) = &session.driver_info else {
        return meta;
    };

    meta.player_car_idx = driver_info.driver_car_idx.unwrap_or(-1);

    let Some(drivers) = &driver_info.drivers else {
        return meta;
    };

    for driver in drivers {
        if driver.car_idx != meta.player_car_idx {
            continue;
        }
        meta.player_irating = driver.i_rating;
        if let Some(lic) = &driver.lic_string {
            meta.player_licence = Some(lic.clone());
        } else if let Some(level) = driver.lic_level {
            meta.player_licence = Some(level.to_string());
        }
        if let Some(compound) = &driver.car_screen_name {
            meta.tyre_compound = Some(compound.clone());
        }
        break;
    }

    meta
}
