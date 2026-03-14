use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct HistoryData<T> {
    data: VecDeque<T>,
    max_points: usize,
}

impl<T: Clone> HistoryData<T> {
    pub fn new(max_points: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.data.len() >= self.max_points {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    pub fn get_all(&self) -> Vec<T> {
        self.data.iter().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct CpuHistory {
    pub global_usage: HistoryData<f32>,
    pub core_usage: Vec<HistoryData<f32>>,
}

impl CpuHistory {
    pub fn new(core_count: usize, max_points: usize) -> Self {
        let mut core_usage = Vec::with_capacity(core_count);
        for _ in 0..core_count {
            core_usage.push(HistoryData::new(max_points));
        }

        Self {
            global_usage: HistoryData::new(max_points),
            core_usage,
        }
    }

    pub fn update(&mut self, global_usage: f32, core_usages: &[f32]) {
        self.global_usage.push(global_usage);
        for (i, &usage) in core_usages.iter().enumerate() {
            if i < self.core_usage.len() {
                self.core_usage[i].push(usage);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryHistory {
    pub usage_percent: HistoryData<f32>,
    pub used_gb: HistoryData<f32>,
    pub swap_percent: HistoryData<f32>,
}

impl MemoryHistory {
    pub fn new(max_points: usize) -> Self {
        Self {
            usage_percent: HistoryData::new(max_points),
            used_gb: HistoryData::new(max_points),
            swap_percent: HistoryData::new(max_points),
        }
    }

    pub fn update(&mut self, usage_percent: f32, used_gb: f32, swap_percent: f32) {
        self.usage_percent.push(usage_percent);
        self.used_gb.push(used_gb);
        self.swap_percent.push(swap_percent);
    }
}

#[derive(Debug, Clone)]
pub struct NetworkHistory {
    pub receive_speed: HistoryData<f32>,
    pub transmit_speed: HistoryData<f32>,
}

impl NetworkHistory {
    pub fn new(max_points: usize) -> Self {
        Self {
            receive_speed: HistoryData::new(max_points),
            transmit_speed: HistoryData::new(max_points),
        }
    }

    pub fn update(&mut self, receive_speed: f32, transmit_speed: f32) {
        self.receive_speed.push(receive_speed);
        self.transmit_speed.push(transmit_speed);
    }
}

#[derive(Debug, Clone)]
pub struct SystemHistory {
    pub cpu: CpuHistory,
    pub memory: MemoryHistory,
    pub network: NetworkHistory,
    #[allow(dead_code)]
    pub max_points: usize,
}

impl SystemHistory {
    pub fn new(core_count: usize) -> Self {
        let max_points = super::DEFAULT_HISTORY_POINTS;

        Self {
            cpu: CpuHistory::new(core_count, max_points),
            memory: MemoryHistory::new(max_points),
            network: NetworkHistory::new(max_points),
            max_points,
        }
    }
}