use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::Networks;

#[derive(Debug)]
pub struct NetworkData {
    pub interfaces: Vec<(String, u64, u64, u64, u64)>, // (name, received, transmitted, rx_speed, tx_speed)
    networks: Networks,
    last_update_time: Instant,
    last_received_bytes: HashMap<String, u64>,
    last_transmitted_bytes: HashMap<String, u64>,
}

impl NetworkData {
    pub fn new() -> Result<Self> {
        let networks = Networks::new_with_refreshed_list();
        let mut interfaces = Vec::new();
        let last_received_bytes = HashMap::new();
        let last_transmitted_bytes = HashMap::new();
        Self::update_networks(&networks, &mut interfaces, &HashMap::new(), &HashMap::new(), 0.0);
        Ok(Self { 
            networks, 
            interfaces,
            last_update_time: Instant::now(),
            last_received_bytes,
            last_transmitted_bytes,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        self.interfaces.clear();
        self.networks.refresh();
        
        let now = Instant::now();
        let elapsed_secs = self.last_update_time.elapsed().as_secs_f64();
        
        Self::update_networks(
            &self.networks, 
            &mut self.interfaces, 
            &self.last_received_bytes,
            &self.last_transmitted_bytes,
            elapsed_secs,
        );
        
        // 更新最后一次的字节数
        for (iface_name, data) in self.networks.iter() {
            let received = data.received();
            let transmitted = data.transmitted();
            
            self.last_received_bytes.insert(iface_name.to_string(), received);
            self.last_transmitted_bytes.insert(iface_name.to_string(), transmitted);
        }
        
        self.last_update_time = now;
        Ok(())
    }

    fn update_networks(
        networks: &Networks, 
        interfaces: &mut Vec<(String, u64, u64, u64, u64)>, 
        last_received_bytes: &HashMap<String, u64>,
        last_transmitted_bytes: &HashMap<String, u64>,
        elapsed_secs: f64,
    ) {
        for (iface_name, data) in networks.iter() {
            let received = data.received();
            let transmitted = data.transmitted();
            
            // 计算速度（字节/秒）
            let last_rx = last_received_bytes.get(iface_name).unwrap_or(&0);
            let last_tx = last_transmitted_bytes.get(iface_name).unwrap_or(&0);
            
            // 计算速度（字节/秒），使用最小时间间隔避免除零
            // 对于极短的时间间隔（<1ms），使用1ms作为最小值，避免除零或速度计算过大
            let min_elapsed_secs = elapsed_secs.max(0.001);
            let rx_speed = ((received.saturating_sub(*last_rx)) as f64 / min_elapsed_secs) as u64;
            let tx_speed = ((transmitted.saturating_sub(*last_tx)) as f64 / min_elapsed_secs) as u64;
            
            interfaces.push((iface_name.to_string(), received, transmitted, rx_speed, tx_speed));
        }
    }
}
