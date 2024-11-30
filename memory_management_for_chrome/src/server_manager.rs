use ctrlc;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub struct ServerManager {
    threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
    stop_signal: Arc<Mutex<bool>>,
    child_processes: Arc<Mutex<Vec<Child>>>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            threads: Arc::new(Mutex::new(Vec::new())),
            stop_signal: Arc::new(Mutex::new(false)),
            child_processes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn should_stop(&self) -> bool {
        *self.stop_signal.lock().unwrap()
    }

    pub fn start_python_script(&self, script_name: &str) -> std::io::Result<Child> {
        Command::new("python3").arg(script_name).spawn()
    }
    // clear thread and process call by thread
    fn clean_up_internal(
        threads: &Arc<Mutex<Vec<std::thread::JoinHandle<()>>>>,
        stop_signal: &Arc<Mutex<bool>>,
        child_processes: &Arc<Mutex<Vec<std::process::Child>>>,
    ) {
        {
            let mut stop_signal = stop_signal.lock().unwrap();
            *stop_signal = true;
        }

        // clean thread
        {
            let mut threads = threads.lock().unwrap();
            for thread in threads.iter() {
                let _ = thread.thread().unpark(); // stop the thread
            }

            for thread in threads.drain(..) {
                let _ = thread
                    .join()
                    .unwrap_or_else(|_| eprintln!("Failed to join a thread"));
            }
        }

        // cleanup child processes
        {
            let mut child_processes = child_processes.lock().unwrap();
            for mut child in child_processes.drain(..) {
                let pid = child.id();
                match child.kill() {
                    Ok(_) => println!("Killed child process {}", pid),
                    Err(error) => println!("Failed to kill child process {}, {}", pid, error),
                }
            }
        }
    }

    pub fn clean_up(&self) {
        println!("Cleaning up threads and processes...");
        Self::clean_up_internal(&self.threads, &self.stop_signal, &self.child_processes);
        println!("Cleanup completed.");
    }

    pub fn set_panic_hook(&self) {
        // Clone necessary resources to move into the closure
        let threads = Arc::clone(&self.threads);
        let stop_signal = Arc::clone(&self.stop_signal);
        let child_processes = Arc::clone(&self.child_processes);

        std::panic::set_hook(Box::new(move |info| {
            println!("Panic occurred: {:?}", info);
            ServerManager::clean_up_internal(&threads, &stop_signal, &child_processes);
            println!("Cleanup completed after panic.");
        }));
    }

    pub fn run_server_threads(&self) -> std::io::Result<()> {
        let tab_info_server_thread = {
            let threads = self.threads.clone();
            let child = self.start_python_script("../tab_info_server/server.py")?;
            self.child_processes.lock().unwrap().push(child);
            thread::spawn(move || {
                threads
                    .lock()
                    .unwrap()
                    .retain(|t| t.thread().id() != thread::current().id());
            })
        };

        let grafana_server_thread = {
            let threads = self.threads.clone();
            let child = self.start_python_script("../grafana/server.py")?;
            self.child_processes.lock().unwrap().push(child);
            thread::spawn(move || {
                threads
                    .lock()
                    .unwrap()
                    .retain(|t| t.thread().id() != thread::current().id());
            })
        };

        let mut threads = self.threads.lock().unwrap();
        threads.push(tab_info_server_thread);
        threads.push(grafana_server_thread);
        Ok(())
    }

    pub fn set_signal_hook_handler(&self) {
        let threads = Arc::clone(&self.threads);
        let stop_signal = Arc::clone(&self.stop_signal);
        let child_processes = Arc::clone(&self.child_processes);

        ctrlc::set_handler(move || {
            // Handle cleanup when Ctrl+C is pressed
            println!("Received termination signal. Cleaning up...");
            ServerManager::clean_up_internal(&threads, &stop_signal, &child_processes);
            println!("Cleanup completed after termination signal.");
        })
        .expect("Failed to set Ctrl+C handler");
    }

    pub fn cleanup_thread(&self) {
        let mut threads = self.threads.lock().unwrap();
        for thread in threads.drain(..) {
            thread
                .join()
                .unwrap_or_else(|_| eprintln!("Failed to join a thread"));
        }
    }

    pub fn cleanup_temp_files(&self) {
        let _ = std::fs::remove_file("log.json");
        let _ = std::fs::remove_file("output.json");
    }
}
