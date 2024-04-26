use anyhow::Result;
use esp_idf_svc::{
    espnow::{EspNow, PeerInfo, BROADCAST},
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    wifi::{BlockingWifi, EspWifi},
};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_pass: &'static str,
    #[default(1)]
    espnow_channel: u8,
}

#[derive()]
pub struct EspWireless<'a> {
    _wifi: esp_idf_svc::wifi::BlockingWifi<esp_idf_svc::wifi::EspWifi<'a>>,
}

pub struct EspWirelessNow<'a> {
    espnow: esp_idf_svc::espnow::EspNow<'a>,
}

impl<'a> EspWirelessNow<'a> {
    pub fn new(espnow: esp_idf_svc::espnow::EspNow<'a>) -> Self {
        let config = CONFIG;
        let peer = PeerInfo {
            peer_addr: BROADCAST,
            channel: config.espnow_channel,
            ifidx: 1,
            encrypt: false,
            ..Default::default()
        };
        espnow.add_peer(peer).unwrap();
        EspWirelessNow { espnow }
    }

    pub fn add_peer(&self, addr: [u8; 6]) -> Result<(), EspError> {
        match self.espnow.peer_exists(addr) {
            Ok(true) => Ok(()),
            Ok(false) => self.espnow.add_peer(PeerInfo {
                peer_addr: addr,
                channel: CONFIG.espnow_channel,
                ifidx: 1,
                encrypt: false,
                ..Default::default()
            }),
            Err(e) => Err(e),
        }
    }

    pub fn version(&self) -> Result<u32, EspError> {
        self.espnow.get_version()
    }

    pub fn send(&self, addr: [u8; 6], data: &[u8]) -> Result<(), EspError> {
        self.espnow.send(addr, data)
    }

    pub fn send_broadcast(&self, data: &[u8]) -> Result<(), EspError> {
        self.espnow.send(BROADCAST, data)
    }

    pub fn register_recv_cb<F>(&self, callback: F) -> Result<(), EspError>
    where
        F: FnMut(&[u8], &[u8]) + Send + 'a,
    {
        self.espnow.register_recv_cb(callback)
    }
}

impl<'a> EspWireless<'a> {
    pub fn new(
        _modem: esp_idf_svc::hal::modem::Modem,
        _sysloop: EspSystemEventLoop,
        _nvs: EspDefaultNvsPartition,
    ) -> Result<Self> {
        let esp_wifi = EspWifi::new(_modem, _sysloop.clone(), Some(_nvs))?;
        let mut _wifi = BlockingWifi::wrap(esp_wifi, _sysloop)?;
        _wifi.start()?;
        _wifi.wait_netif_up()?;
        Ok(EspWireless { _wifi })
    }

    pub fn wifi(&self) {
        unimplemented!()
    }

    pub fn espnow(&self) -> EspWirelessNow<'a> {
        let espnow = EspNow::take().unwrap();
        EspWirelessNow::new(espnow)
    }
}
