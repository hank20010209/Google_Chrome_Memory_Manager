use nix::errno::Errno;
use nix::Error;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::collections::HashMap;

pub struct MemoryManager {
    pub rss_limit: i32,
    pub idel_time_limit: i32,
    pub memory_change_rate: f32,
    pub pid_inActive_time_counter: HashMap<i32, i32>,
    pub pid_warmup_time_counter: HashMap<i32, u64>,
    pub memory_total_using: HashMap<i32, u64>,
}

impl MemoryManager {
    pub fn new(rss_limit: i32, idel_time_limit: i32, memory_change_rate: f32) -> Self {
        Self { rss_limit, idel_time_limit, memory_change_rate, pid_inActive_time_counter: HashMap::new(), pid_warmup_time_counter: HashMap::new(), memory_total_using: HashMap::new()}
    }

    /// memory killer can use two strategy to kill the tab render process
    pub fn memory_killer(
        &mut self,
        tabid_tabname_tabpid_isActive_map: &Vec<(i32, String, i32, i32, bool)>,
        reflush_time: u64,
        strategy: &String,
    ) -> nix::Result<()> {
        match strategy.as_str() {
            "idel_time_limit" => self.kill_by_inActive_time(tabid_tabname_tabpid_isActive_map, reflush_time),
            "rss_limit" => self.kill_by_exceed_rss_limit(tabid_tabname_tabpid_isActive_map),
            "memory_change_rate" => self.kill_by_memory_change_rate(tabid_tabname_tabpid_isActive_map, self.memory_change_rate, reflush_time),
            _ => {
                eprintln!("Invalid strategy: {}", strategy);
                return nix::Result::Err(Error::EFAULT);
            }
        }
    }

    fn kill_by_inActive_time(&mut self, tabid_tabname_tabpid_isActive_map: &Vec<(i32, String, i32, i32, bool)>, reflush_time: u64) -> nix::Result<()>{
        for (tab_inner_id, _, tab_process_id, _, is_active) in tabid_tabname_tabpid_isActive_map.iter() {
            if !is_active && tab_process_id != &-1{
                let counter = self.pid_inActive_time_counter.entry(*tab_process_id).or_insert(0);
                *counter += reflush_time as i32;
                if *counter > self.idel_time_limit {
                    kill_process(*tab_process_id).unwrap();
                    println!(
                        "Killing process with PID {} (Tab ID: {}) due to inactivity: {} seconds",
                        tab_process_id, tab_inner_id, counter
                    );
                    self.pid_inActive_time_counter.remove(tab_process_id);
                }
            } else {
                self.pid_inActive_time_counter.remove(tab_process_id);
            }
        }
        Ok(())
    }

    fn kill_by_exceed_rss_limit(&self, tabid_tabname_tabpid_isActive_map: &Vec<(i32, String, i32, i32, bool)>) -> nix::Result<()> {
        let total_rss: i32 = tabid_tabname_tabpid_isActive_map
                .iter()
                .map(|&(_, _, _, rss, _)| rss)
                .sum();
        
            println!(
                "Chrome Total RSS: {} KB (Limit: {} KB)",
                total_rss, self.rss_limit
            );
        
            // out of user input rss limit
            if total_rss > self.rss_limit {
                // find max rss tab
                let mut sorted_tabs = tabid_tabname_tabpid_isActive_map.clone();
                sorted_tabs.sort_by(|a, b| b.3.cmp(&a.3)); // sort by rss
        
                //find the biggest rss tab and is_active is false
                for (tab_inner_id, _, tab_process_id, tab_rss, is_active) in sorted_tabs.iter() {
                    if !is_active {
                        // is_active is false that mean the tab is on the background
                        kill_process(*tab_process_id)?;
                        println!(
                            "Killing process with PID {} (Tab ID: {}) due to high memory usage: {} KB",
                            tab_process_id, tab_inner_id, tab_rss
                        );
                        break; 
                    }
                }
            }
            Ok(())
    }
    
    fn kill_by_memory_change_rate(
        &mut self, 
        tabid_tabname_tabpid_isActive_map: &Vec<(i32, String, i32, i32, bool)>, 
        rate: f32, 
        reflush_time: u64
    ) -> nix::Result<()> {
        for (_, _, tab_process_id, tab_rss, _) in tabid_tabname_tabpid_isActive_map.iter() {
            let counter = self.pid_warmup_time_counter.entry(*tab_process_id).or_insert(0);
            *counter = (*counter + reflush_time as u64).min(u64::MAX);
            //waiting for 30 to denote average memory usage
            //every 30 second, check memory rate, and drop the previous memory usage
            if *counter <= 30 && *tab_process_id != -1 {
                self.memory_total_using
                    .entry(*tab_process_id)
                    .and_modify(|e| *e += *tab_rss as u64)
                    .or_insert(*tab_rss as u64);
            } else if let Some(memory_total) = self.memory_total_using.get_mut(tab_process_id) {
                let memory_average_usage = *memory_total as f32 / (30 / reflush_time) as f32;
                let memory_change_rate = (*tab_rss as f32 - memory_average_usage).abs() / memory_average_usage;
                println!(
                    "Tab ID: {} Memory Change Rate: {} (Average: {})",
                    tab_process_id, memory_change_rate, memory_average_usage
                );
    
                if *tab_process_id > 0 && memory_change_rate < rate {
                    kill_process(*tab_process_id)?;
                    println!(
                        "Killing process with PID {} due to low memory change rate: {}",
                        tab_process_id, memory_change_rate
                    );
                }
                *counter = 0;
                *memory_total = 0;
            }
        }
        Ok(())
    }
}

/// Helper wrapper function to kill a process by PID.
fn kill_process(pid: i32) -> nix::Result<()> {
    if pid == -1 {
        println!("Invalid PID: {}, the process resource is already release", pid);
        return Ok(());
    }
    let pid = Pid::from_raw(pid);
    kill(pid, Signal::SIGKILL).map_err(|err| {
        eprintln!("Failed to kill process with PID {}: {}", pid, Errno::from_raw(err as i32));
        err
    })
}
