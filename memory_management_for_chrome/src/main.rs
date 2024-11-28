mod memory_manager;
mod tab_manager;
mod server_manager;
mod config_manager;

use memory_manager::MemoryManager;
use tab_manager::TabManager;
use server_manager::ServerManager;
use config_manager::ConfigManager;

use std::path::Path;
use std::thread;
use std::time::Duration;


fn main() -> std::io::Result<()> {
    let chrome_info_path = Path::new("/proc/chrome_info");
    let log_path = Path::new("log.json");
    let config_path = Path::new("manager.toml");
    let mut manager = TabManager::new();
    let config_manager = ConfigManager::new(config_path);
    let rss_limit = config_manager.rss_limit;
    let idel_time_limit = config_manager.idel_time_limit;
    let reflush_time = config_manager.reflush_time;
    let strategy = config_manager.strategy;
    let mut memory_manager = MemoryManager::new(rss_limit, idel_time_limit);
    let server_manager = ServerManager::new();
    
    server_manager.set_panic_hook();
    server_manager.set_signal_hook_handler();
    server_manager.run_server_thread();

    println!("\x1b[42mWaiting for servers to start..., using strategy: {}\x1b[0m", strategy);
    thread::sleep(Duration::from_secs(15));

    while !*server_manager.stop_signal.lock().unwrap() {
        manager.build_tab_info_map(log_path)?;
        manager.get_pid_from_chrome_info(chrome_info_path)?;
        manager.build_tab_process_info_map();
        manager.build_tabid_tabname_tabpid_isActive_map();
        manager.print_tab_process_info_map();

        if let Err(err) = memory_manager.memory_killer(&manager.tabid_tabname_tabpid_isActive_map, reflush_time, &strategy) {
            eprintln!("Failed to enforce memory limit: {}", err);
        }

        if let Err(e) = manager.write_tab_process_info_to_file("output.json", &memory_manager.pid_inActive_time_counter) {
            eprintln!("Failed to write to file: {}", e);
        }

        manager.tab_info_map.clear();
        manager.pid_rss_vector.clear();
        manager.tab_process_info_map.clear();
        manager.tabid_tabname_tabpid_isActive_map.clear();

        thread::sleep(Duration::from_secs(reflush_time));
    }

    println!("Shutting down...");
    server_manager.cleanup_thread();
    server_manager.cleanup_temp_files();
    println!("Cleanup completed.");
    Ok(())
}
