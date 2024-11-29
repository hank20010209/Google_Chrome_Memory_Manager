use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use json::JsonValue;
use std::path::Path;
use std::io;

pub struct TabManager {
    pub tab_info_map: HashMap<i32, (String, bool)>, // tab_inner_id tab_name is_active
    pub pid_rss_vector: Vec<(i32, i32)>,
    pub tab_process_info_map: HashMap<String, (i32, i32)>,
    pub tabid_tabname_tabpid_isActive_map: Vec<(i32, String, i32, i32, bool)>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tab_info_map: HashMap::new(),
            pid_rss_vector: Vec::new(),
            tab_process_info_map: HashMap::new(),
            tabid_tabname_tabpid_isActive_map: Vec::new(),
        }
    }

    /// Builds the `tab_info_map` by parsing a JSON log file.
    pub fn build_tab_info_map(&mut self, log_path: &Path) -> std::io::Result<()> {
        let log_file = File::open(log_path).unwrap_or_else(|err| {
            eprintln!("Failed to open file: {}", err);
            panic!("Cannot proceed without the file log.json! file_path: {}", log_path.display());
        });
        let mut reader = BufReader::new(log_file);
        let mut content = String::new();

        println!("Find log.json and open successfully!");
        reader.read_to_string(&mut content)?;
        let parsed = json::parse(&content).expect("Failed to parse JSON");
        self.traverse_json(&parsed);
        Ok(())
    }

    /// Traverses a JSON structure to populate the `tab_info_map`.
    fn traverse_json(&mut self, value: &JsonValue) {
        match value {
            JsonValue::Object(obj) => {
                for (key, value) in obj.iter() {
                    if key == "pid" {
                        let tab_inner_id = value.as_i32().unwrap();
                        let tab_name = obj.get("title").unwrap().as_str().unwrap();
                        let is_active = obj.get("active").unwrap().as_bool().unwrap();
                        self.tab_info_map.insert(tab_inner_id, (tab_name.to_string(), is_active));
                    } else {
                        self.traverse_json(value);
                    }
                }
            }
            JsonValue::Array(arr) => {
                for value in arr.iter() {
                    self.traverse_json(value);
                }
            }
            _ => {}
        }
    }

    /// Populates the `pid_rss_vector` by reading process info from `/proc/chrome_info`.
    /// /proc/chrome_info is crate by kernel module
    pub fn get_pid_from_chrome_info(&mut self, chrome_info_path: &Path) -> std::io::Result<()> {
        let chrome_info_file = File::open(chrome_info_path).unwrap_or_else(|err| {
            eprintln!("Failed to open file: {}", err);
            panic!("Cannot proceed without the file! Please load the kernel module first");
        });

        println!("Find chrome_info and open successfully!");
        
        // todo! Stuck if "/proc/chrome_info" cannot be read
        for line in BufReader::new(chrome_info_file).lines() {
            let line = line?;
            let mut pid: i32 = -1; 
            let mut rss: i32 = 0; 
            // parse to find PID
            if let Some(pos) = line.find("PID: ") {
                let start = pos + 4;
                if start < line.len() {
                    let end_pos = line[start..].find(',').unwrap_or(line.len());
                    pid = line[start..start + end_pos].trim().parse::<i32>().unwrap_or(-1);
                }
            }

            // parse to find RSS
            if let Some(pos) = line.find("RSS: ") {
                let start = pos + 4;
                if start < line.len() {
                    let end_pos = line[start..].find(',').unwrap_or(line.len());
                    rss = line[start..start + end_pos].trim().parse::<i32>().unwrap_or(0);
                }
            }

            // if pid and rss is vaild, push into pid_rss_vector
            if pid != -1 && rss != 0 {
                self.pid_rss_vector.push((pid, rss));
            }
        }

        Ok(())
    }

    /// Builds the `tab_process_info_map` by associating PIDs with tab IDs.
    pub fn build_tab_process_info_map(&mut self) {
        for pid_rss in self.pid_rss_vector.iter() {
            let cmdline_path = format!("/proc/{}/cmdline", pid_rss.0);
            if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
                let key = "--renderer-client-id=";
                if let Some(pos) = cmdline.find(key) {
                    let end_pos = cmdline[pos + key.len()..].find(' ').unwrap_or(cmdline.len());
                    let tab_inner_id = &cmdline[pos + key.len()..pos + key.len() + end_pos];
                    self.tab_process_info_map.insert(tab_inner_id.to_string(), *pid_rss);
                }
            }
        }
    }

    /// Prints the tab process info in a formatted way.
    pub fn print_tab_process_info_map(&self) {
        let is_chinese = |c: char| {
            (c >= '\u{4E00}' && c <= '\u{9FFF}') || // CJK Unified Ideographs
            (c >= '\u{3400}' && c <= '\u{4DBF}') || // CJK Unified Ideographs Extension A
            (c >= '\u{20000}' && c <= '\u{2A6DF}') || // CJK Unified Ideographs Extension B
            (c >= '\u{2A700}' && c <= '\u{2B73F}') || // CJK Unified Ideographs Extension C
            (c >= '\u{2B740}' && c <= '\u{2B81F}') || // CJK Unified Ideographs Extension D
            (c >= '\u{2B820}' && c <= '\u{2CEAF}') || // CJK Unified Ideographs Extension E
            (c >= '\u{F900}' && c <= '\u{FAFF}') ||   // CJK Compatibility Ideographs
            (c >= '\u{2F800}' && c <= '\u{2FA1F}')    // CJK Compatibility Ideographs Supplement
        };
        
        let count_chinese_characters = |input: &str| input.chars().filter(|&c| is_chinese(c)).count();

        println!("print_tab_process_info_map:");
        for (tab_inner_id, tab_name, tab_process_id, tab_rss, is_active) in self.tabid_tabname_tabpid_isActive_map.iter() {
            let chinese_count = count_chinese_characters(tab_name);
            let tab_name_offset = 30 - chinese_count;
            println!(
                "tab_inner_id: {:<5} tab_name: {:<tab_name_offset$} tab_process_id: {:>5} tab_rss: {:>5} is_active: {:>5}", 
                tab_inner_id, 
                tab_name, 
                tab_process_id,
                tab_rss,
                is_active,
                tab_name_offset = tab_name_offset
            );
        }
    }

    pub fn write_tab_process_info_to_file(&self, file_path: &str, pid_inActive_time_counter: &HashMap<i32, i32>) -> io::Result<()> {
        let mut file = File::create(file_path)?;
        let tab_info_map_len = self.tab_info_map.len();
        let mut counter: usize = 0;
    
        writeln!(file, "{{")?;
        writeln!(file, "\t\"tab_info_instance\": [")?;
        for (tab_inner_id, (tab_name, is_active)) in self.tab_info_map.iter() {
            let pid_rss: (i32, i32) = *self
                .tab_process_info_map
                .get(&tab_inner_id.to_string())
                .unwrap_or(&(-1, 0));
            //using json format write
            counter += 1;
            writeln!(file, "\t\t{{")?;
            writeln!(file, "\t\t\t\"tab_id\": {},", tab_inner_id)?;
            writeln!(file, "\t\t\t\"tab_name\": {},", serde_json::json!(tab_name).to_string())?;
            writeln!(file, "\t\t\t\"tab_process_id\": {},", pid_rss.0)?;
            writeln!(file, "\t\t\t\"tab_rss\": {},", pid_rss.1)?;
            writeln!(file, "\t\t\t\"is_active\": {},", is_active)?;
            writeln!(file, "\t\t\t\"inActive_time\": {}", pid_inActive_time_counter.get(&pid_rss.0).unwrap_or(&0))?;
            if counter == tab_info_map_len {
                writeln!(file, "\t\t}}")?;
            } else {
                writeln!(file, "\t\t}},")?;
            }
        }
        writeln!(file, "\t]")?;
        writeln!(file, "}}")?;
        Ok(())
    }

    pub fn build_tabid_tabname_tabpid_isActive_map(&mut self) {
        //contain tab_inner_id tab_name tab_process_id tab_rss
        for (tab_inner_id, tab_name_is_active) in self.tab_info_map.iter() {
            let pid_rss: (i32, i32) = *self
                .tab_process_info_map
                .get(&tab_inner_id.to_string())
                .unwrap_or(&(-1, 0));
            self.tabid_tabname_tabpid_isActive_map.push((*tab_inner_id, tab_name_is_active.0.to_string(), pid_rss.0, pid_rss.1, tab_name_is_active.1));
        }
    }
}
