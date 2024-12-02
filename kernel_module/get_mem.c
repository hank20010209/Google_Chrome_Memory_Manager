#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/sched.h>
#include <linux/sched/signal.h>
#include <linux/mm.h>
#include <linux/sched/mm.h>
#include <linux/proc_fs.h>
#include <linux/uaccess.h>
#include <linux/timer.h>
#include <linux/fs.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("");
MODULE_DESCRIPTION("A kernel module to print Chrome processes and their memory RSS.");
MODULE_VERSION("1.0");

#define BUFFER_SIZE 8192

static struct timer_list chrome_timer;
static char chrome_info[BUFFER_SIZE];
static int chrome_info_size = 0;

void scan_chrome_processes(struct timer_list *t) {
    struct task_struct *task;
    struct mm_struct *mm;

    chrome_info_size = 0; // Reset buffer size for new scan

    chrome_info_size += snprintf(chrome_info + chrome_info_size, BUFFER_SIZE - chrome_info_size,
                                  "Scanning Chrome processes...\n");

    for_each_process(task) {
        // Check if the task name matches "chrome", by field comm
        if (strcmp(task->comm, "chrome") == 0) {
            mm = get_task_mm(task); // Get memory descriptor of the task
            if (mm) {
                unsigned long rss = get_mm_counter(mm, MM_ANONPAGES) * (PAGE_SIZE / 1024); //to get rss
                unsigned long virt_mem = mm->total_vm * PAGE_SIZE / (1024 * 1024); 
                chrome_info_size += snprintf(chrome_info + chrome_info_size, BUFFER_SIZE - chrome_info_size,
                                              "Found Chrome process: PID: %d, Name: %s, RSS: %lu, virt_mem: %lu\n",
                                              task->pid, task->comm, rss, virt_mem);

                mmput(mm);
            }
        }
    }

    // Ensure the buffer is null-terminated
    chrome_info[BUFFER_SIZE - 1] = '\0';

    // Restart the timer for the next scan
    mod_timer(&chrome_timer, jiffies + msecs_to_jiffies(3000));
}

ssize_t chrome_info_read(struct file *file, char __user *buf, size_t count, loff_t *offset) {
    // Return the chrome_info buffer to user space
    return simple_read_from_buffer(buf, count, offset, chrome_info, chrome_info_size);
}

static struct file_operations chrome_info_fops = {
    .owner = THIS_MODULE,
    .read = chrome_info_read,
};

static int __init chrome_proc_init(void) {
    // Create the /proc/chrome_info file, will be read in rust parser
    proc_create("chrome_info", 0666, NULL, (const struct proc_ops *)&chrome_info_fops);

    // Initialize and start the timer
    timer_setup(&chrome_timer, scan_chrome_processes, 0);
    mod_timer(&chrome_timer, jiffies + msecs_to_jiffies(3000));

    printk(KERN_INFO "Chrome process lister module loaded.\n");
    return 0;
}

static void __exit chrome_proc_exit(void) {
    remove_proc_entry("chrome_info", NULL);
    del_timer(&chrome_timer);
    printk(KERN_INFO "Unloading Chrome process lister module.\n");
}

module_init(chrome_proc_init);
module_exit(chrome_proc_exit);
